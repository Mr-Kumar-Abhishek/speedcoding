//! Phase 5: SpeedCo I magnetic tapes (`J..M`), magnetic drums (`P..Q`), and line printer.

use crate::core_types::{Address, DecimalFloat, MachineStop, SpeedError, SpeedFloat, Word72};

pub const DRUM_CAPACITY: usize = 1024;
pub const PRINTER_VALUES_PER_LINE: usize = 5;

/// Magnetic tape unit identifier (`J`, `K`, `L`, `M`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeId {
    J,
    K,
    L,
    M,
}

/// Magnetic drum unit identifier (`P`, `Q`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumId {
    P,
    Q,
}

/// Hardware status latch for each tape unit (manual pages 7-13..7-14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeStatus {
    Read,
    Neutral,
    Write,
}

/// A variable-length unit record stored on magnetic tape together with its checksum and error flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapeRecord {
    pub words: Vec<Word72>,
    pub checksum: u128,
    pub parity_corrupted: bool,
    pub checksum_corrupted: bool,
}

/// Result of `RFTP*` or `RBTP*` tape reading operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapeReadOutcome {
    Ok(Vec<Word72>),
    EndOfFileSkip,
    ChecksumMismatchSkip,
    Stop(MachineStop),
}

/// Result of `SFTP*` or `SBTP*` tape skipping operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeSkipOutcome {
    Ok,
    EndOfFileSkip,
    Stop(MachineStop),
}

/// Result of `WRDRP` / `WRDRQ` drum writing operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumWriteOutcome {
    Ok,
    VerifyDiscrepancySkip,
}

fn compute_checksum(words: &[Word72]) -> u128 {
    words.iter().fold(0u128, |acc, w| acc.wrapping_add(w.raw()))
}

fn check_drum_bounds(start: Address, len: usize) -> Result<usize, SpeedError> {
    let start_idx = usize::from(start.raw());
    if len == 0 {
        return Ok(start_idx);
    }
    if start_idx.saturating_add(len) > DRUM_CAPACITY {
        let end = u32::from(start.raw())
            .saturating_add(u32::try_from(len).unwrap_or(u32::MAX))
            .saturating_sub(1);
        return Err(SpeedError::DrumAddressOutOfRange {
            start: start.raw(),
            end,
        });
    }
    Ok(start_idx)
}

/// Simulated IBM 701 magnetic tape drive supporting SpeedCo I tape semantics.
#[derive(Debug, Clone)]
pub struct TapeUnit {
    id: TapeId,
    status: TapeStatus,
    records: Vec<TapeRecord>,
    position: usize,
    rewound: bool,
}

impl TapeUnit {
    /// Creates a new rewound tape unit in `Neutral` status.
    pub fn new(id: TapeId) -> Self {
        Self {
            id,
            status: TapeStatus::Neutral,
            records: Vec::new(),
            position: 0,
            rewound: true,
        }
    }

    /// Returns the tape unit identifier (`J`, `K`, `L`, or `M`).
    pub fn id(&self) -> TapeId {
        self.id
    }

    /// Returns current read/neutral/write status.
    pub fn status(&self) -> TapeStatus {
        self.status
    }

    /// Returns `true` when the tape is at the beginning-of-file / rewound position.
    pub fn is_rewound(&self) -> bool {
        self.rewound
    }

    /// Returns total number of unit records written in the current file.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Returns the current inter-record gap cursor index (`0` = start of file).
    pub fn position(&self) -> usize {
        self.position
    }

    /// Executes `WRTP*`: writes `words` as one unit record if status permits (`Neutral`/`Write`).
    pub fn write_record(&mut self, words: &[Word72]) -> Result<Option<MachineStop>, SpeedError> {
        if self.status == TapeStatus::Read {
            return Ok(Some(MachineStop::CopyCheck));
        }
        self.records.truncate(self.position);
        self.records.push(TapeRecord {
            words: words.to_vec(),
            checksum: compute_checksum(words),
            parity_corrupted: false,
            checksum_corrupted: false,
        });
        self.position = self.records.len();
        self.status = TapeStatus::Write;
        self.rewound = false;
        Ok(None)
    }

    /// Executes `EFTP*`: marks end-of-file gap (transitions `Write -> Neutral`; ignored in `Read`).
    pub fn end_file(&mut self) -> Result<(), SpeedError> {
        if self.status == TapeStatus::Write {
            self.status = TapeStatus::Neutral;
            self.rewound = false;
        }
        Ok(())
    }

    /// Executes `RWTP*`: rewinds tape and restores `Neutral` status.
    pub fn rewind(&mut self) {
        self.position = 0;
        self.status = TapeStatus::Neutral;
        self.rewound = true;
    }

    /// Executes `RFTP*`: reads forward and returns up to `count` words (`B - A + 1`).
    pub fn read_forward(&mut self, count: usize) -> Result<TapeReadOutcome, SpeedError> {
        if self.status == TapeStatus::Write {
            return Ok(TapeReadOutcome::Stop(MachineStop::CopyCheck));
        }
        self.status = TapeStatus::Read;
        self.rewound = false;
        if self.position >= self.records.len() {
            return Ok(TapeReadOutcome::EndOfFileSkip);
        }
        let idx = self.position;
        self.position += 1;
        let rec = &self.records[idx];
        if count > rec.words.len() {
            return Ok(TapeReadOutcome::Stop(MachineStop::CopyCheck));
        }
        if rec.parity_corrupted {
            return Ok(TapeReadOutcome::Stop(MachineStop::TapeCheck));
        }
        if rec.checksum_corrupted || compute_checksum(&rec.words) != rec.checksum {
            return Ok(TapeReadOutcome::ChecksumMismatchSkip);
        }
        Ok(TapeReadOutcome::Ok(rec.words[..count].to_vec()))
    }

    /// Executes `RBTP*`: backspaces, reads forward, and backspaces again (except on error).
    pub fn read_backward(&mut self, count: usize) -> Result<TapeReadOutcome, SpeedError> {
        if self.status == TapeStatus::Write {
            return Ok(TapeReadOutcome::Stop(MachineStop::CopyCheck));
        }
        self.status = TapeStatus::Read;
        self.rewound = false;
        if self.position == 0 {
            return Ok(TapeReadOutcome::EndOfFileSkip);
        }
        self.position -= 1;
        let outcome = self.read_forward(count)?;
        if matches!(outcome, TapeReadOutcome::Ok(_)) {
            self.position -= 1;
        }
        Ok(outcome)
    }

    /// Executes `SFTP*`: advances over the next unit record without reading.
    pub fn skip_forward(&mut self) -> Result<TapeSkipOutcome, SpeedError> {
        if self.status == TapeStatus::Write {
            return Ok(TapeSkipOutcome::Stop(MachineStop::CopyCheck));
        }
        self.status = TapeStatus::Read;
        self.rewound = false;
        if self.position >= self.records.len() {
            return Ok(TapeSkipOutcome::EndOfFileSkip);
        }
        self.position += 1;
        Ok(TapeSkipOutcome::Ok)
    }

    /// Executes `SBTP*`: backspaces over the preceding unit record without reading.
    pub fn skip_backward(&mut self) -> Result<TapeSkipOutcome, SpeedError> {
        if self.status == TapeStatus::Write {
            return Ok(TapeSkipOutcome::Stop(MachineStop::CopyCheck));
        }
        self.status = TapeStatus::Read;
        self.rewound = false;
        if self.position == 0 {
            return Ok(TapeSkipOutcome::EndOfFileSkip);
        }
        self.position -= 1;
        Ok(TapeSkipOutcome::Ok)
    }

    /// Injects a checksum discrepancy on a recorded unit record for testing.
    pub fn inject_checksum_error(&mut self, record_index: usize) {
        if let Some(rec) = self.records.get_mut(record_index) {
            rec.checksum_corrupted = true;
        }
    }

    /// Injects an even-parity redundancy error on a recorded unit record for testing.
    pub fn inject_parity_error(&mut self, record_index: usize) {
        if let Some(rec) = self.records.get_mut(record_index) {
            rec.parity_corrupted = true;
        }
    }

    /// Computes the 7th-channel odd-parity redundancy bit (`0` or `1`) for a 6-bit group.
    pub fn compute_odd_parity_bit(six_bit_group: u8) -> u8 {
        let ones = (six_bit_group & 0x3F).count_ones();
        u8::from(ones.is_multiple_of(2))
    }
}

/// Simulated 1024-word magnetic drum unit (`P` or `Q`).
#[derive(Debug, Clone)]
pub struct DrumUnit {
    id: DrumId,
    words: Vec<Word72>,
    inject_verify_error: bool,
}

impl DrumUnit {
    /// Creates a new zero-initialized 1024-word drum unit.
    pub fn new(id: DrumId) -> Self {
        Self {
            id,
            words: vec![Word72::default(); DRUM_CAPACITY],
            inject_verify_error: false,
        }
    }

    /// Returns the drum unit identifier (`P` or `Q`).
    pub fn id(&self) -> DrumId {
        self.id
    }

    /// Executes `WRDR*`: writes `words` starting at `start` (`0..=1023`) and verifies against drum.
    pub fn write_block(
        &mut self,
        start: Address,
        words: &[Word72],
    ) -> Result<DrumWriteOutcome, SpeedError> {
        let start_idx = check_drum_bounds(start, words.len())?;
        self.words[start_idx..start_idx + words.len()].copy_from_slice(words);
        if self.inject_verify_error {
            self.inject_verify_error = false;
            return Ok(DrumWriteOutcome::VerifyDiscrepancySkip);
        }
        Ok(DrumWriteOutcome::Ok)
    }

    /// Executes `RFDR*`: reads `count` consecutive words starting at `start` (`0..=1023`).
    pub fn read_block(&self, start: Address, count: usize) -> Result<Vec<Word72>, SpeedError> {
        let start_idx = check_drum_bounds(start, count)?;
        Ok(self.words[start_idx..start_idx + count].to_vec())
    }

    /// Forces the next drum write verification read-back to report a discrepancy.
    pub fn inject_next_write_verify_error(&mut self) {
        self.inject_verify_error = true;
    }
}

/// Formatted printer line with computed `100 * C + n` identification number and up to 5 decimal floats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterLine {
    pub id: u32,
    pub values: Vec<DecimalFloat>,
}

/// Line printer receiving floating decimal blocks from `PRINT` (`OP1 = 580`).
#[derive(Debug, Clone, Default)]
pub struct LinePrinter {
    lines: Vec<PrinterLine>,
}

impl LinePrinter {
    /// Creates an empty printer output log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts `values` to floating decimal (`|E| <= 236`) and prints 5 numbers per line.
    pub fn print_block(
        &mut self,
        values: &[SpeedFloat],
        c_id: u16,
    ) -> Result<Vec<PrinterLine>, SpeedError> {
        let decimals: Vec<DecimalFloat> = values
            .iter()
            .copied()
            .map(DecimalFloat::from_speed_float)
            .collect::<Result<_, _>>()?;
        let printed: Vec<PrinterLine> = decimals
            .chunks(PRINTER_VALUES_PER_LINE)
            .enumerate()
            .map(|(idx, chunk)| PrinterLine {
                id: 100u32 * u32::from(c_id) + (idx as u32) + 1,
                values: chunk.to_vec(),
            })
            .collect();
        self.lines.extend(printed.iter().cloned());
        Ok(printed)
    }

    /// Returns all printed lines accumulated so far.
    pub fn lines(&self) -> &[PrinterLine] {
        &self.lines
    }
}

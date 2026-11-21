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

/// Simulated IBM 701 magnetic tape drive supporting SpeedCo I tape semantics.
#[derive(Debug, Clone)]
pub struct TapeUnit {
    _marker: (),
}

impl TapeUnit {
    /// Creates a new rewound tape unit in `Neutral` status.
    pub fn new(_id: TapeId) -> Self {
        todo!()
    }

    /// Returns current read/neutral/write status.
    pub fn status(&self) -> TapeStatus {
        todo!()
    }

    /// Returns `true` when the tape is at the beginning-of-file / rewound position.
    pub fn is_rewound(&self) -> bool {
        todo!()
    }

    /// Returns total number of unit records written in the current file.
    pub fn record_count(&self) -> usize {
        todo!()
    }

    /// Returns the current inter-record gap cursor index (`0` = start of file).
    pub fn position(&self) -> usize {
        todo!()
    }

    /// Executes `WRTP*`: writes `words` as one unit record if status permits (`Neutral`/`Write`).
    pub fn write_record(&mut self, _words: &[Word72]) -> Result<Option<MachineStop>, SpeedError> {
        todo!()
    }

    /// Executes `EFTP*`: marks end-of-file gap (transitions `Write -> Neutral`; ignored in `Read`).
    pub fn end_file(&mut self) -> Result<(), SpeedError> {
        todo!()
    }

    /// Executes `RWTP*`: rewinds tape and restores `Neutral` status.
    pub fn rewind(&mut self) {
        todo!()
    }

    /// Executes `RFTP*`: reads forward and returns up to `count` words (`B - A + 1`).
    pub fn read_forward(&mut self, _count: usize) -> Result<TapeReadOutcome, SpeedError> {
        todo!()
    }

    /// Executes `RBTP*`: backspaces, reads forward, and backspaces again (except on error).
    pub fn read_backward(&mut self, _count: usize) -> Result<TapeReadOutcome, SpeedError> {
        todo!()
    }

    /// Executes `SFTP*`: advances over the next unit record without reading.
    pub fn skip_forward(&mut self) -> Result<TapeSkipOutcome, SpeedError> {
        todo!()
    }

    /// Executes `SBTP*`: backspaces over the preceding unit record without reading.
    pub fn skip_backward(&mut self) -> Result<TapeSkipOutcome, SpeedError> {
        todo!()
    }

    /// Injects a checksum discrepancy on a recorded unit record for testing.
    pub fn inject_checksum_error(&mut self, _record_index: usize) {
        todo!()
    }

    /// Injects an even-parity redundancy error on a recorded unit record for testing.
    pub fn inject_parity_error(&mut self, _record_index: usize) {
        todo!()
    }

    /// Computes the 7th-channel odd-parity redundancy bit (`0` or `1`) for a 6-bit group.
    pub fn compute_odd_parity_bit(_six_bit_group: u8) -> u8 {
        todo!()
    }
}

/// Simulated 1024-word magnetic drum unit (`P` or `Q`).
#[derive(Debug, Clone)]
pub struct DrumUnit {
    _marker: (),
}

impl DrumUnit {
    /// Creates a new zero-initialized 1024-word drum unit.
    pub fn new(_id: DrumId) -> Self {
        todo!()
    }

    /// Executes `WRDR*`: writes `words` starting at `start` (`0..=1023`) and verifies against drum.
    pub fn write_block(
        &mut self,
        _start: Address,
        _words: &[Word72],
    ) -> Result<DrumWriteOutcome, SpeedError> {
        todo!()
    }

    /// Executes `RFDR*`: reads `count` consecutive words starting at `start` (`0..=1023`).
    pub fn read_block(&self, _start: Address, _count: usize) -> Result<Vec<Word72>, SpeedError> {
        todo!()
    }

    /// Forces the next drum write verification read-back to report a discrepancy.
    pub fn inject_next_write_verify_error(&mut self) {
        todo!()
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
    _marker: (),
}

impl LinePrinter {
    /// Creates an empty printer output log.
    pub fn new() -> Self {
        todo!()
    }

    /// Converts `values` to floating decimal (`|E| <= 236`) and prints 5 numbers per line.
    pub fn print_block(
        &mut self,
        _values: &[SpeedFloat],
        _c_id: u16,
    ) -> Result<Vec<PrinterLine>, SpeedError> {
        todo!()
    }

    /// Returns all printed lines accumulated so far.
    pub fn lines(&self) -> &[PrinterLine] {
        todo!()
    }
}

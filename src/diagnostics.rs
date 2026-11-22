//! Phase 6: SpeedCo I checking loops (`PRCH`/`STCH`/`ECHTR`), selective listing, and timing model.

use crate::core_types::{Address, DecimalFloat, LCode, Op1, Op2, RCode, SpeedError, SpeedFloat, Word72};
use crate::io::PRINTER_VALUES_PER_LINE;

pub const LISTING_BLOCK_SIZE: usize = 10;
pub const LISTING_TIME_AVG_SECS: f64 = 1.325;
pub const LISTING_TIME_MAX_SECS: f64 = 1.370;

/// Active pass within a SpeedCo I checking loop (manual section 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckingPass {
    Inactive,
    Pass1,
    Pass2,
}

/// Result of executing `ECHTR` (`OP2 = 254`) at the end of a checking loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EchTrOutcome {
    RepeatAt(Address),
    VerifiedSkipNext,
    DiscrepancyContinue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum CheckingPhase {
    #[default]
    Unprepared,
    ReadyForPass1,
    InPass1,
    ReadyForPass2,
    InPass2,
}

/// State machine tracking `PRCH`, `STCH`, `ECHTR`, and fixed-point check counters 1 and 2.
#[derive(Debug, Clone, Default)]
pub struct CheckingState {
    phase: CheckingPhase,
    counter_1: u128,
    counter_2: u128,
}

impl CheckingState {
    /// Creates an unprepared checking state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current checking pass state.
    pub fn pass(&self) -> CheckingPass {
        match self.phase {
            CheckingPhase::InPass1 => CheckingPass::Pass1,
            CheckingPhase::InPass2 => CheckingPass::Pass2,
            CheckingPhase::Unprepared
            | CheckingPhase::ReadyForPass1
            | CheckingPhase::ReadyForPass2 => CheckingPass::Inactive,
        }
    }

    /// Executes `PRCH` (`OP2 = 232`), initializing control state before `STCH`.
    pub fn prepare_check(&mut self) {
        self.phase = CheckingPhase::ReadyForPass1;
        self.counter_1 = 0;
        self.counter_2 = 0;
    }

    /// Executes `STCH` (`OP2 = 253`), entering pass 1 or pass 2 and recording `first_result`.
    pub fn start_check(&mut self, first_result: Word72) -> Result<(), SpeedError> {
        match self.phase {
            CheckingPhase::ReadyForPass1 => {
                self.counter_1 = first_result.raw();
                self.counter_2 = 0;
                self.phase = CheckingPhase::InPass1;
                Ok(())
            }
            CheckingPhase::ReadyForPass2 => {
                self.counter_2 = first_result.raw();
                self.phase = CheckingPhase::InPass2;
                Ok(())
            }
            CheckingPhase::Unprepared | CheckingPhase::InPass1 | CheckingPhase::InPass2 => Err(
                SpeedError::CheckingProtocolViolation(
                    "STCH requires prior PRCH or Pass-1 ECHTR".to_string(),
                ),
            ),
        }
    }

    /// Accumulates an intermediate computed result into the active check counter (fixed-point sum).
    pub fn record_computed_result(&mut self, result: Word72) {
        match self.phase {
            CheckingPhase::InPass1 => {
                self.counter_1 = self.counter_1.wrapping_add(result.raw());
            }
            CheckingPhase::InPass2 => {
                self.counter_2 = self.counter_2.wrapping_add(result.raw());
            }
            CheckingPhase::Unprepared
            | CheckingPhase::ReadyForPass1
            | CheckingPhase::ReadyForPass2 => {}
        }
    }

    /// Executes `ECHTR` (`OP2 = 254`): loops to `target` after pass 1 or compares counters after pass 2.
    pub fn end_check_and_transfer(
        &mut self,
        last_result: Word72,
        target: Address,
    ) -> Result<EchTrOutcome, SpeedError> {
        match self.phase {
            CheckingPhase::InPass1 => {
                self.counter_1 = self.counter_1.wrapping_add(last_result.raw());
                self.phase = CheckingPhase::ReadyForPass2;
                Ok(EchTrOutcome::RepeatAt(target))
            }
            CheckingPhase::InPass2 => {
                self.counter_2 = self.counter_2.wrapping_add(last_result.raw());
                self.phase = CheckingPhase::ReadyForPass1;
                if self.counter_1 == self.counter_2 {
                    Ok(EchTrOutcome::VerifiedSkipNext)
                } else {
                    Ok(EchTrOutcome::DiscrepancyContinue)
                }
            }
            CheckingPhase::Unprepared
            | CheckingPhase::ReadyForPass1
            | CheckingPhase::ReadyForPass2 => Err(SpeedError::CheckingProtocolViolation(
                "ECHTR requires active STCH pass".to_string(),
            )),
        }
    }

    /// Returns `true` when executing the second pass of a checking loop.
    pub fn is_second_pass(&self) -> bool {
        self.phase == CheckingPhase::InPass2
    }

    pub(crate) fn is_awaiting_second_pass(&self) -> bool {
        self.phase == CheckingPhase::ReadyForPass2
    }

    /// Returns the accumulated value in check counter 1.
    pub fn counter_1(&self) -> u128 {
        self.counter_1
    }

    /// Returns the accumulated value in check counter 2.
    pub fn counter_2(&self) -> u128 {
        self.counter_2
    }
}

/// Operator panel listing sense switches labeled 1, 2, 3 (`L`-codes 0, 3, 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ListingSwitches {
    pub switch_1_l0: bool,
    pub switch_2_l3: bool,
    pub switch_3_l6: bool,
}

impl ListingSwitches {
    /// Returns `true` if the panel switch corresponding to `l_code` is ON.
    pub fn should_list(self, l_code: LCode) -> bool {
        match l_code.switch_number() {
            1 => self.switch_1_l0,
            2 => self.switch_2_l3,
            3 => self.switch_3_l6,
            other => unreachable!("validated LCode switch number out of range: {other}"),
        }
    }
}

/// Formatted operand value printed in columns 14–16 of a listing line (manual page 11-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListedValue {
    Decimal(DecimalFloat),
    ExponentOverflow { positive: bool },
    Blank,
}

/// Single 16-column SpeedCo I listing trace line (manual page 11-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListingEntry {
    pub location: Address,
    pub op1: Op1,
    pub r_code: RCode,
    pub a: Address,
    pub b: Address,
    pub c: Address,
    pub op2: Op2,
    pub d: Address,
    pub l_code: LCode,
    pub address_counter: i32,
    pub r_a: u16,
    pub r_b: u16,
    pub r_c: u16,
    pub a_val: ListedValue,
    pub b_val: ListedValue,
    pub c_val: ListedValue,
}

fn format_single_listed_value(val: Option<SpeedFloat>) -> ListedValue {
    let Some(sf) = val else {
        return ListedValue::Blank;
    };
    match DecimalFloat::from_speed_float(sf) {
        Ok(dec) => ListedValue::Decimal(dec),
        Err(SpeedError::DecimalExponentOutOfRange(_)) => ListedValue::ExponentOverflow {
            positive: !sf.exp_negative,
        },
        Err(_) => ListedValue::Blank,
    }
}

/// Drum-backed 10-line listing buffer (manual pages 11-1..11-2).
#[derive(Debug, Clone, Default)]
pub struct ListingBuffer {
    pending: Vec<ListingEntry>,
    flushed: Vec<Vec<ListingEntry>>,
}

impl ListingBuffer {
    /// Creates an empty listing buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an entry when enabled by switches and not in checking pass 2, flushing every 10 lines.
    pub fn record(
        &mut self,
        entry: ListingEntry,
        switches: ListingSwitches,
        is_check_pass_2: bool,
    ) -> Option<Vec<ListingEntry>> {
        if is_check_pass_2 || !switches.should_list(entry.l_code) {
            return None;
        }
        self.pending.push(entry);
        if self.pending.len() >= LISTING_BLOCK_SIZE {
            let block = std::mem::take(&mut self.pending);
            self.flushed.push(block.clone());
            Some(block)
        } else {
            None
        }
    }

    /// Returns number of buffered lines currently awaiting a 10-line flush.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Returns all flushed 10-line listing blocks.
    pub fn flushed_blocks(&self) -> &[Vec<ListingEntry>] {
        &self.flushed
    }

    /// Formats `Q(A)`, `Q(B)`, `Q(C)` for listing, applying manual page 11-1/11-2 sign/overflow rules.
    pub fn format_listed_operands(
        op1: Op1,
        qa: Option<SpeedFloat>,
        qb: Option<SpeedFloat>,
        qc: Option<SpeedFloat>,
    ) -> (ListedValue, ListedValue, ListedValue) {
        let (eff_qa, eff_qb) = match op1 {
            Op1::Sub => (qa, qb.map(|v| -v)),
            Op1::AddAb => (qa, qb.map(SpeedFloat::abs)),
            Op1::AbAdd => (qa.map(SpeedFloat::abs), qb.map(SpeedFloat::abs)),
            Op1::SubAb => (qa, qb.map(|v| -v.abs())),
            Op1::AbSub => (qa.map(SpeedFloat::abs), qb.map(|v| -v.abs())),
            Op1::NgMpy | Op1::NgDiv => (qa.map(|v| -v), qb),
            _ => (qa, qb),
        };
        (
            format_single_listed_value(eff_qa),
            format_single_listed_value(eff_qb),
            format_single_listed_value(qc),
        )
    }
}

/// Prior tape operation context determining tape execution times (manual pages 12-1..12-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeCondition {
    Cond1AfterWrite,
    Cond2AfterRewind,
    Cond3AfterReadOrSkipFwd,
    Cond4AfterReadOrSkipBwd,
    Cond5AfterEndFile,
}

/// Average and maximum execution times in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingRange {
    pub avg_ms: f64,
    pub max_ms: f64,
}

/// Parameters controlling execution time lookup for a single SpeedCo I instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimingContext {
    pub word_count: usize,
    pub tape_condition: Option<TapeCondition>,
    pub is_check_pass_2: bool,
    pub op2_condition_met: bool,
}

/// Analytical timing model from manual Section 12 and Appendices 1–2.
pub struct TimingCalculator;

impl TimingCalculator {
    /// Returns average and maximum execution time (ms) for `OP1`.
    pub fn op1_time_ms(op1: Op1, ctx: TimingContext) -> TimingRange {
        let n = ctx.word_count as f64;
        match op1 {
            Op1::Add | Op1::Sub | Op1::AddAb | Op1::AbAdd | Op1::SubAb | Op1::AbSub => {
                TimingRange {
                    avg_ms: 4.200,
                    max_ms: 19.272,
                }
            }
            Op1::Mpy | Op1::NgMpy => TimingRange {
                avg_ms: 3.546,
                max_ms: 3.876,
            },
            Op1::Div | Op1::NgDiv => TimingRange {
                avg_ms: 4.236,
                max_ms: 4.440,
            },
            Op1::NoOp => TimingRange {
                avg_ms: 2.736,
                max_ms: 2.736,
            },
            Op1::Sqrt => TimingRange {
                avg_ms: 8.200,
                max_ms: 9.016,
            },
            Op1::Sine => TimingRange {
                avg_ms: 20.0,
                max_ms: 26.0,
            },
            Op1::Artan => TimingRange {
                avg_ms: 24.0,
                max_ms: 32.0,
            },
            Op1::Exp => TimingRange {
                avg_ms: 18.0,
                max_ms: 24.0,
            },
            Op1::Ln => TimingRange {
                avg_ms: 33.0,
                max_ms: 42.0,
            },
            Op1::WrTpJ | Op1::WrTpK | Op1::WrTpL | Op1::WrTpM => {
                let cond = ctx.tape_condition.unwrap_or(TapeCondition::Cond1AfterWrite);
                let base_avg = match cond {
                    TapeCondition::Cond1AfterWrite => 14.000,
                    TapeCondition::Cond2AfterRewind
                    | TapeCondition::Cond3AfterReadOrSkipFwd
                    | TapeCondition::Cond4AfterReadOrSkipBwd
                    | TapeCondition::Cond5AfterEndFile => 19.000,
                };
                TimingRange {
                    avg_ms: base_avg + 1.600 * n,
                    max_ms: base_avg + 5.000 + 1.600 * n,
                }
            }
            Op1::RfTpJ
            | Op1::RfTpK
            | Op1::RfTpL
            | Op1::RfTpM
            | Op1::SfTpJ
            | Op1::SfTpK
            | Op1::SfTpL
            | Op1::SfTpM
            | Op1::SbTpJ
            | Op1::SbTpK
            | Op1::SbTpL
            | Op1::SbTpM => TimingRange {
                avg_ms: 14.000 + 1.600 * n,
                max_ms: 19.000 + 1.600 * n,
            },
            Op1::RbTpJ | Op1::RbTpK | Op1::RbTpL | Op1::RbTpM => TimingRange {
                avg_ms: 42.000 + 4.800 * n,
                max_ms: 57.000 + 4.800 * n,
            },
            Op1::RwTpJ | Op1::RwTpK | Op1::RwTpL | Op1::RwTpM => TimingRange {
                avg_ms: 3.500,
                max_ms: 3.500,
            },
            Op1::EfTpJ | Op1::EfTpK | Op1::EfTpL | Op1::EfTpM => TimingRange {
                avg_ms: 960.0,
                max_ms: 960.0,
            },
            Op1::WrDrP | Op1::WrDrQ => TimingRange {
                avg_ms: 20.0 + 0.096 * n,
                max_ms: 30.0 + 0.096 * n,
            },
            Op1::RfDrP | Op1::RfDrQ => TimingRange {
                avg_ms: 10.0 + 0.048 * n,
                max_ms: 20.0 + 0.048 * n,
            },
            Op1::Print => {
                if ctx.is_check_pass_2 {
                    TimingRange {
                        avg_ms: 2.896,
                        max_ms: 2.896,
                    }
                } else {
                    let lines = ctx.word_count.div_ceil(PRINTER_VALUES_PER_LINE) as f64;
                    TimingRange {
                        avg_ms: 1663.0 + 400.0 * lines,
                        max_ms: 1703.0 + 400.0 * lines,
                    }
                }
            }
        }
    }

    /// Returns execution time (ms) for `OP2` under the given branch/condition outcome.
    pub fn op2_time_ms(op2: Op2, condition_met: bool) -> f64 {
        match op2 {
            Op2::NoOp => 0.000,
            Op2::Tr => 0.768,
            Op2::TrPl | Op2::TrMn | Op2::TrZ | Op2::SnTrP | Op2::SnTrQ => {
                if condition_met { 0.876 } else { 0.648 }
            }
            Op2::TiA
            | Op2::TiB
            | Op2::TiC
            | Op2::TiAB
            | Op2::TiBC
            | Op2::TiAC
            | Op2::TiABC => 2.256,
            Op2::TdA
            | Op2::TdB
            | Op2::TdC
            | Op2::TdAB
            | Op2::TdBC
            | Op2::TdAC
            | Op2::TdABC => 3.036,
            Op2::SetRA | Op2::SetRB | Op2::SetRC | Op2::PrCh | Op2::StCh => 1.200,
            Op2::SkRA | Op2::SkRB | Op2::SkRC | Op2::Skip | Op2::EchTr => {
                if condition_met { 1.368 } else { 1.260 }
            }
            Op2::RAddA
            | Op2::RAddB
            | Op2::RAddC
            | Op2::RAddD
            | Op2::AddA
            | Op2::AddB
            | Op2::AddC
            | Op2::AddD
            | Op2::SubA
            | Op2::SubB
            | Op2::SubC
            | Op2::SubD
            | Op2::StA
            | Op2::StB
            | Op2::StC
            | Op2::StD => 1.536,
            Op2::Stop => 0.816,
        }
    }

    /// Returns total instruction time (ms) = `op1_time_ms + op2_time_ms`.
    pub fn instruction_time_ms(op1: Op1, op2: Op2, ctx: TimingContext) -> TimingRange {
        let op1_range = Self::op1_time_ms(op1, ctx);
        let op2_ms = Self::op2_time_ms(op2, ctx.op2_condition_met);
        TimingRange {
            avg_ms: op1_range.avg_ms + op2_ms,
            max_ms: op1_range.max_ms + op2_ms,
        }
    }

    /// Computes total physical tape length in inches (manual page 12-3).
    pub fn tape_file_length_inches(record_word_counts: &[usize]) -> f64 {
        let k = record_word_counts.len();
        let data_inches: f64 = record_word_counts
            .iter()
            .map(|&n| (n as f64 + 0.5) * 3.0 / 25.0)
            .sum();
        2.0 * 72.0 + data_inches + (k.saturating_sub(1) as f64)
    }

    /// Computes tape rewind time in seconds (`length_inches / 75.0`).
    pub fn tape_rewind_time_secs(record_word_counts: &[usize]) -> f64 {
        Self::tape_file_length_inches(record_word_counts) / 75.0
    }

    /// Returns time in seconds required to write a 72-inch end-of-file gap (`0.96 s`).
    pub fn tape_eof_gap_time_secs() -> f64 {
        72.0 / 75.0
    }

    /// Computes overlap delay `(1.600 * n - t).max(0.0)` in ms per Note 1 (page 12-2).
    pub fn tape_skip_delay_ms(record_words: usize, elapsed_ms: f64) -> f64 {
        (1.600 * (record_words as f64) - elapsed_ms).max(0.0)
    }

    /// Approximates card loading time `(0.4)(n) + (0.6)(c) + e` in seconds (page 12-5).
    pub fn loading_time_secs(
        total_cards: usize,
        control_cards: usize,
        speedco_on_drums: bool,
    ) -> f64 {
        let e = if speedco_on_drums { 2.0 } else { 55.0 };
        0.4 * (total_cards as f64) + 0.6 * (control_cards as f64) + e
    }
}

//! Phase 6: SpeedCo I checking loops (`PRCH`/`STCH`/`ECHTR`), selective listing, and timing model.

use crate::core_types::{Address, DecimalFloat, LCode, Op1, Op2, RCode, SpeedError, SpeedFloat, Word72};

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

/// State machine tracking `PRCH`, `STCH`, `ECHTR`, and fixed-point check counters 1 and 2.
#[derive(Debug, Clone, Default)]
pub struct CheckingState {
    _marker: (),
}

impl CheckingState {
    /// Creates an unprepared checking state.
    pub fn new() -> Self {
        todo!()
    }

    /// Returns the current checking pass state.
    pub fn pass(&self) -> CheckingPass {
        todo!()
    }

    /// Executes `PRCH` (`OP2 = 232`), initializing control state before `STCH`.
    pub fn prepare_check(&mut self) {
        todo!()
    }

    /// Executes `STCH` (`OP2 = 253`), entering pass 1 or pass 2 and recording `first_result`.
    pub fn start_check(&mut self, _first_result: Word72) -> Result<(), SpeedError> {
        todo!()
    }

    /// Accumulates an intermediate computed result into the active check counter (fixed-point sum).
    pub fn record_computed_result(&mut self, _result: Word72) {
        todo!()
    }

    /// Executes `ECHTR` (`OP2 = 254`): loops to `target` after pass 1 or compares counters after pass 2.
    pub fn end_check_and_transfer(
        &mut self,
        _last_result: Word72,
        _target: Address,
    ) -> Result<EchTrOutcome, SpeedError> {
        todo!()
    }

    /// Returns `true` when executing the second pass of a checking loop.
    pub fn is_second_pass(&self) -> bool {
        todo!()
    }

    /// Returns the accumulated value in check counter 1.
    pub fn counter_1(&self) -> u128 {
        todo!()
    }

    /// Returns the accumulated value in check counter 2.
    pub fn counter_2(&self) -> u128 {
        todo!()
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
    pub fn should_list(self, _l_code: LCode) -> bool {
        todo!()
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

/// Drum-backed 10-line listing buffer (manual pages 11-1..11-2).
#[derive(Debug, Clone, Default)]
pub struct ListingBuffer {
    _marker: (),
}

impl ListingBuffer {
    /// Creates an empty listing buffer.
    pub fn new() -> Self {
        todo!()
    }

    /// Records an entry when enabled by switches and not in checking pass 2, flushing every 10 lines.
    pub fn record(
        &mut self,
        _entry: ListingEntry,
        _switches: ListingSwitches,
        _is_check_pass_2: bool,
    ) -> Option<Vec<ListingEntry>> {
        todo!()
    }

    /// Returns number of buffered lines currently awaiting a 10-line flush.
    pub fn pending_count(&self) -> usize {
        todo!()
    }

    /// Returns all flushed 10-line listing blocks.
    pub fn flushed_blocks(&self) -> &[Vec<ListingEntry>] {
        todo!()
    }

    /// Formats `Q(A)`, `Q(B)`, `Q(C)` for listing, applying manual page 11-1/11-2 sign/overflow rules.
    pub fn format_listed_operands(
        _op1: Op1,
        _qa: Option<SpeedFloat>,
        _qb: Option<SpeedFloat>,
        _qc: Option<SpeedFloat>,
    ) -> (ListedValue, ListedValue, ListedValue) {
        todo!()
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
    pub fn op1_time_ms(_op1: Op1, _ctx: TimingContext) -> TimingRange {
        todo!()
    }

    /// Returns execution time (ms) for `OP2` under the given branch/condition outcome.
    pub fn op2_time_ms(_op2: Op2, _condition_met: bool) -> f64 {
        todo!()
    }

    /// Returns total instruction time (ms) = `op1_time_ms + op2_time_ms`.
    pub fn instruction_time_ms(_op1: Op1, _op2: Op2, _ctx: TimingContext) -> TimingRange {
        todo!()
    }

    /// Computes total physical tape length in inches (manual page 12-3).
    pub fn tape_file_length_inches(_record_word_counts: &[usize]) -> f64 {
        todo!()
    }

    /// Computes tape rewind time in seconds (`length_inches / 75.0`).
    pub fn tape_rewind_time_secs(_record_word_counts: &[usize]) -> f64 {
        todo!()
    }

    /// Returns time in seconds required to write a 72-inch end-of-file gap (`0.96 s`).
    pub fn tape_eof_gap_time_secs() -> f64 {
        todo!()
    }

    /// Computes overlap delay `(1.600 * n - t).max(0.0)` in ms per Note 1 (page 12-2).
    pub fn tape_skip_delay_ms(_record_words: usize, _elapsed_ms: f64) -> f64 {
        todo!()
    }

    /// Approximates card loading time `(0.4)(n) + (0.6)(c) + e` in seconds (page 12-5).
    pub fn loading_time_secs(
        _total_cards: usize,
        _control_cards: usize,
        _speedco_on_drums: bool,
    ) -> f64 {
        todo!()
    }
}

//! Phase 2: SpeedCo I `mpc`-based symbolic program parser and 80-column punched card parser.

use crate::core_types::{Address, DecimalFloat, Instruction, LCode, RCode, SpeedError};

/// Target auxiliary storage unit referenced on loading control cards (column 29).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageUnit {
    TapeJ,
    TapeK,
    TapeL,
    TapeM,
    DrumP,
    DrumQ,
}

/// Symbolic assembly line parsed into an electrostatic storage location and instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInstructionLine {
    pub location: Address,
    pub instruction: Instruction,
    pub remarks: String,
}

/// 80-column SpeedCo I Instruction Card representation (manual pages 7-1..7-3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionCard {
    pub label: String,
    pub location: Address,
    pub alpha_op1: Option<String>,
    pub alpha_op2: Option<String>,
    pub numeric_op1: Option<u16>,
    pub numeric_op2: Option<u16>,
    pub r_code: RCode,
    pub a: Address,
    pub b: Address,
    pub c: Address,
    pub d: Address,
    pub l_code: LCode,
    pub remarks: String,
}

impl InstructionCard {
    /// Converts the card to a machine `Instruction` using numerical codes (rejecting alpha-only cards).
    pub fn to_machine_instruction(&self) -> Result<Instruction, SpeedError> {
        todo!()
    }
}

/// 80-column SpeedCo I Data Card containing 1 to 5 floating decimal numbers (manual pages 7-3..7-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataCard {
    pub identification: String,
    pub first_location: Address,
    pub values: Vec<DecimalFloat>,
    pub word_count: u8,
}

/// 80-column SpeedCo I Loading Control Card (manual page 7-17).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadingControlCard {
    pub first_addr: Address,
    pub last_addr: Address,
    pub drum_addr: Address,
    pub unit: StorageUnit,
}

/// Union of the three SpeedCo I punched card formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PunchedCard {
    Instruction(InstructionCard),
    Data(DataCard),
    LoadingControl(LoadingControlCard),
}

/// Builds an `mpc` combinator parser matching any valid alphabetical `OP1` mnemonic.
pub fn build_op1_mnemonic_parser() -> mpc::MpcParser {
    todo!()
}

/// Builds an `mpc` combinator parser matching any valid alphabetical `OP2` mnemonic.
pub fn build_op2_mnemonic_parser() -> mpc::MpcParser {
    todo!()
}

/// Parses a single symbolic SpeedCo I instruction line using `mpc`.
pub fn parse_instruction_line(_input: &str) -> Result<ParsedInstructionLine, SpeedError> {
    todo!()
}

/// Parses a multi-line symbolic SpeedCo I program.
pub fn parse_program(_source: &str) -> Result<Vec<ParsedInstructionLine>, SpeedError> {
    todo!()
}

/// Parses an 80-column SpeedCo I Instruction Card record.
pub fn parse_instruction_card(_card_80cols: &str) -> Result<InstructionCard, SpeedError> {
    todo!()
}

/// Simulates sort-and-gang-punch translation of alphabetical `OP1`/`OP2` into numerical columns 46–51.
pub fn gang_punch_instruction_card(_card: &InstructionCard) -> Result<InstructionCard, SpeedError> {
    todo!()
}

/// Parses an 80-column SpeedCo I Data Card record.
pub fn parse_data_card(_card_80cols: &str) -> Result<DataCard, SpeedError> {
    todo!()
}

/// Parses an 80-column SpeedCo I Loading Control Card record.
pub fn parse_loading_control_card(_card_80cols: &str) -> Result<LoadingControlCard, SpeedError> {
    todo!()
}

/// Dispatches and parses any 80-column SpeedCo I card based on columns 8–9.
pub fn parse_card(_card_80cols: &str) -> Result<PunchedCard, SpeedError> {
    todo!()
}

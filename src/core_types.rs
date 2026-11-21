//! Phase 1: Core SpeedCo I data representations, 72-bit words, operation codes,
//! instruction encoding, and electrostatic memory.

pub const WORD_BITS: usize = 72;
pub const TOTAL_MEM_WORDS: usize = 1024;
pub const USER_MEM_START: u16 = 300;
pub const USER_MEM_END: u16 = 1013;
pub const USER_MEM_LEN: usize = 714;
pub const INITIAL_PC: u16 = 300;
pub const SPECIAL_R_CONTROL_ADDR: u16 = 25;
pub const MAX_DECIMAL_EXPONENT: i16 = 236;
pub const MAX_BINARY_EXPONENT: u32 = 131_071;
pub const FRACTION_BITS: u32 = 35;

/// Errors raised during decoding, validation, assembly, or execution of SpeedCo I state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeedError {
    AddressOutOfBounds(u16),
    ControlRegionAccess(u16),
    DecimalExponentOutOfRange(i32),
    BinaryExponentOutOfRange(i32),
    ExponentDifferenceTooLarge { diff: u32 },
    InvalidWordBits(u128),
    InvalidRCode(u8),
    InvalidLCode(u8),
    InvalidOp1Code(u16),
    InvalidOp2Code(u16),
    InvalidMnemonic(String),
    InvalidCardFormat(String),
    ForbiddenOp2WithSkipOp1 { op1: Op1, op2: Op2 },
    DrumAddressOutOfRange { start: u16, end: u32 },
    CheckingProtocolViolation(String),
    ParseError(String),
}

/// Hardware or programmed stop conditions defined by the SpeedCo I manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineStop {
    DivideCheck,
    CopyCheck,
    TapeCheck,
    RQuantityUnderflow,
    ProgramStop { restart_addr: Address },
    DecimalConversionError,
}

/// 10-bit electrostatic or drum address (`0..=1023`), with user storage in `300..=1013`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    _raw: u16,
}

impl Address {
    /// Creates an address within the 1024-word address space (`0..=1023`).
    pub fn new(_raw: u16) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Creates a user-accessible electrostatic storage address (`300..=1013`).
    pub fn user(_raw: u16) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Returns the raw integer address value (`0..=1023`).
    pub fn raw(self) -> u16 {
        todo!()
    }

    /// Returns `true` when the address lies inside user storage (`300..=1013`).
    pub fn is_user_accessible(self) -> bool {
        todo!()
    }
}

/// 72-bit IBM 701 word stored right-aligned in the lower 72 bits of `u128`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Word72 {
    _bits: u128,
}

impl Word72 {
    /// Validates that only bits `0..72` are populated and wraps them in `Word72`.
    pub fn new(_bits: u128) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Returns the raw 72-bit value right-aligned in `u128`.
    pub fn raw(self) -> u128 {
        todo!()
    }

    /// Extracts a single bit using IBM 701 1-based MSB numbering (`1..=72`).
    pub fn bit(self, _pos_1based: u8) -> bool {
        todo!()
    }

    /// Extracts an inclusive bit slice using IBM 701 1-based MSB numbering (`1..=72`).
    pub fn bits(self, _start_1based: u8, _end_1based: u8) -> u64 {
        todo!()
    }

    /// Splits the 72-bit word into 12 six-bit tape groups (group 1 = bits `1..=6` .. group 12 = bits `67..=72`).
    pub fn to_six_bit_groups(self) -> [u8; 12] {
        todo!()
    }

    /// Reassembles a 72-bit word from 12 six-bit tape groups.
    pub fn from_six_bit_groups(_groups: [u8; 12]) -> Self {
        todo!()
    }
}

/// Internal SpeedCo I floating binary representation (`F * 2^E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeedFloat {
    pub negative: bool,
    pub fraction: u64,
    pub exp_negative: bool,
    pub exponent: u32,
}

impl SpeedFloat {
    /// Returns floating-point zero (`F = 0`, `E = 0`).
    pub fn zero() -> Self {
        todo!()
    }

    /// Converts an IEEE-754 `f64` into normalized SpeedCo I 35-bit floating binary.
    pub fn from_f64(_value: f64) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Converts this SpeedCo I floating binary value to `f64`.
    pub fn to_f64(self) -> f64 {
        todo!()
    }

    /// Returns the signed binary exponent (`-131_071..=131_071`).
    pub fn signed_exponent(self) -> i32 {
        todo!()
    }

    /// Returns `true` when the fractional part is zero.
    pub fn is_zero_fraction(self) -> bool {
        todo!()
    }

    /// Returns `true` if `fraction == 0` or `0.5 <= |F| < 1` (`fraction >= 2^34`).
    pub fn is_normalized(self) -> bool {
        todo!()
    }

    /// Returns `|self|`.
    pub fn abs(self) -> Self {
        todo!()
    }

    /// Returns `-self`.
    pub fn neg(self) -> Self {
        todo!()
    }

    /// Encodes into a 72-bit word (bit 1 sign, 2..=36 frac, 37 exp sign, 38..=54 exp, 55..=72 zero).
    pub fn to_word(self) -> Word72 {
        todo!()
    }

    /// Decodes a 72-bit word into `SpeedFloat`.
    pub fn from_word(_word: Word72) -> Self {
        todo!()
    }
}

/// External SpeedCo I floating decimal representation (`F * 10^E`, 10-digit fraction, `|E| <= 236`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalFloat {
    pub negative: bool,
    pub digits: u64,
    pub exponent: i16,
}

impl DecimalFloat {
    /// Constructs and validates a floating decimal value (`|exponent| <= 236`).
    pub fn new(_negative: bool, _digits: u64, _exponent: i16) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Converts external decimal floating point to internal binary `SpeedFloat`.
    pub fn to_speed_float(self) -> Result<SpeedFloat, SpeedError> {
        todo!()
    }

    /// Converts internal binary `SpeedFloat` to external decimal `DecimalFloat`, rejecting `|E| > 236`.
    pub fn from_speed_float(_val: SpeedFloat) -> Result<Self, SpeedError> {
        todo!()
    }
}

/// 3-bit address modification selector (`0..=7`) controlling `R_A`, `R_B`, and `R_C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RCode {
    _raw: u8,
}

impl RCode {
    /// Validates `val <= 7`.
    pub fn new(_val: u8) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Returns raw R-code value (`0..=7`).
    pub fn raw(self) -> u8 {
        todo!()
    }

    /// Returns `true` when bit 4 is set (`R` in `{4, 5, 6, 7}`), adding `R_A` to `A`.
    pub fn modifies_a(self) -> bool {
        todo!()
    }

    /// Returns `true` when bit 2 is set (`R` in `{2, 3, 6, 7}`), adding `R_B` to `B`.
    pub fn modifies_b(self) -> bool {
        todo!()
    }

    /// Returns `true` when bit 1 is set (`R` in `{1, 3, 5, 7}`), adding `R_C` to `C`.
    pub fn modifies_c(self) -> bool {
        todo!()
    }
}

/// Selective listing code (`0`, `3`, or `6`), corresponding to panel switches 1, 2, or 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LCode {
    _raw: u8,
}

impl LCode {
    /// Validates `val` is one of `0`, `3`, or `6`.
    pub fn new(_val: u8) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Returns raw L-code (`0`, `3`, or `6`).
    pub fn raw(self) -> u8 {
        todo!()
    }

    /// Maps `0 -> 1`, `3 -> 2`, `6 -> 3`.
    pub fn switch_number(self) -> u8 {
        todo!()
    }
}

/// All 49 SpeedCo I `OP1` operations (manual pages 5-1 and 5-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op1 {
    Add,
    Sub,
    AddAb,
    AbAdd,
    SubAb,
    AbSub,
    Mpy,
    NgMpy,
    Div,
    NgDiv,
    Sqrt,
    Sine,
    Artan,
    Exp,
    Ln,
    WrTpJ,
    WrTpK,
    WrTpL,
    WrTpM,
    RfTpJ,
    RfTpK,
    RfTpL,
    RfTpM,
    RbTpJ,
    RbTpK,
    RbTpL,
    RbTpM,
    SfTpJ,
    SfTpK,
    SfTpL,
    SfTpM,
    SbTpJ,
    SbTpK,
    SbTpL,
    SbTpM,
    RwTpJ,
    RwTpK,
    RwTpL,
    RwTpM,
    EfTpJ,
    EfTpK,
    EfTpL,
    EfTpM,
    WrDrP,
    WrDrQ,
    RfDrP,
    RfDrQ,
    Print,
    NoOp,
}

impl Op1 {
    /// Returns the 3-digit numerical operation code for `OP1`.
    pub fn code(self) -> u16 {
        todo!()
    }

    /// Returns the alphabetical mnemonic for `OP1`.
    pub fn mnemonic(self) -> &'static str {
        todo!()
    }

    /// Decodes a 3-digit numerical `OP1` code.
    pub fn from_code(_code: u16) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Decodes an alphabetical `OP1` mnemonic.
    pub fn from_mnemonic(_s: &str) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Returns `true` for `OP1` operations with automatic skips (`RFTP*`, `RBTP*`, `SFTP*`, `SBTP*`, `WRDR*`).
    pub fn forbids_op2(self) -> bool {
        todo!()
    }
}

/// All 48 SpeedCo I `OP2` operations (manual pages 5-3 through 5-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op2 {
    Tr,
    TrPl,
    TrMn,
    TrZ,
    SnTrP,
    SnTrQ,
    TiA,
    TiB,
    TiC,
    TiAB,
    TiBC,
    TiAC,
    TiABC,
    TdA,
    TdB,
    TdC,
    TdAB,
    TdBC,
    TdAC,
    TdABC,
    SetRA,
    SetRB,
    SetRC,
    SkRA,
    SkRB,
    SkRC,
    RAddA,
    RAddB,
    RAddC,
    RAddD,
    AddA,
    AddB,
    AddC,
    AddD,
    SubA,
    SubB,
    SubC,
    SubD,
    StA,
    StB,
    StC,
    StD,
    Skip,
    PrCh,
    StCh,
    EchTr,
    Stop,
    NoOp,
}

impl Op2 {
    /// Returns the 3-digit numerical operation code for `OP2`.
    pub fn code(self) -> u16 {
        todo!()
    }

    /// Returns the alphabetical mnemonic for `OP2` (`""` for `NoOp`).
    pub fn mnemonic(self) -> &'static str {
        todo!()
    }

    /// Decodes a 3-digit numerical `OP2` code (`0` -> `Op2::NoOp`).
    pub fn from_code(_code: u16) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Decodes an alphabetical `OP2` mnemonic (`""` or `"-"` -> `Op2::NoOp`).
    pub fn from_mnemonic(_s: &str) -> Result<Self, SpeedError> {
        todo!()
    }
}

/// Decoded SpeedCo I instruction occupying one 72-bit word (manual page 4-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub r_code: RCode,
    pub op1: Op1,
    pub op2: Op2,
    pub d: Address,
    pub a: Address,
    pub b: Address,
    pub c: Address,
    pub l_code: LCode,
}

impl Instruction {
    /// Encodes the instruction into its 72-bit memory layout (page 4-4).
    pub fn to_word(self) -> Word72 {
        todo!()
    }

    /// Decodes a 72-bit instruction word and validates field constraints.
    pub fn from_word(_word: Word72) -> Result<Self, SpeedError> {
        todo!()
    }
}

/// 1024-word electrostatic cathode-ray-tube memory with user-accessible cells `300..=1013`.
#[derive(Debug, Clone, Default)]
pub struct ElectrostaticMemory {
    _marker: (),
}

impl ElectrostaticMemory {
    /// Creates zero-initialized electrostatic memory.
    pub fn new() -> Self {
        todo!()
    }

    /// Reads a raw 72-bit word from user storage (`300..=1013`), rejecting control cells.
    pub fn read(&self, _addr: Address) -> Result<Word72, SpeedError> {
        todo!()
    }

    /// Writes a raw 72-bit word into user storage (`300..=1013`), rejecting control cells.
    pub fn write(&mut self, _addr: Address, _word: Word72) -> Result<(), SpeedError> {
        todo!()
    }

    /// Reads and decodes a `SpeedFloat` from user storage (`300..=1013`).
    pub fn read_float(&self, _addr: Address) -> Result<SpeedFloat, SpeedError> {
        todo!()
    }

    /// Encodes and writes a `SpeedFloat` into user storage (`300..=1013`).
    pub fn write_float(&mut self, _addr: Address, _val: SpeedFloat) -> Result<(), SpeedError> {
        todo!()
    }

    /// Reads and decodes an `Instruction` from user storage (`300..=1013`).
    pub fn read_instruction(&self, _addr: Address) -> Result<Instruction, SpeedError> {
        todo!()
    }

    /// Encodes and writes an `Instruction` into user storage (`300..=1013`).
    pub fn write_instruction(
        &mut self,
        _addr: Address,
        _instr: Instruction,
    ) -> Result<(), SpeedError> {
        todo!()
    }
}

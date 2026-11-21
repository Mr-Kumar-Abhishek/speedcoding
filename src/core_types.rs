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

const WORD72_MASK: u128 = (1u128 << WORD_BITS) - 1;
const FRACTION_MASK: u64 = (1u64 << FRACTION_BITS) - 1;
const MIN_NORM_FRACTION: u64 = 1u64 << (FRACTION_BITS - 1);
const TEN_DIGITS_LIMIT: u64 = 10_000_000_000;

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
    raw: u16,
}

impl Address {
    /// Creates an address within the 1024-word address space (`0..=1023`).
    pub fn new(raw: u16) -> Result<Self, SpeedError> {
        if raw < TOTAL_MEM_WORDS as u16 {
            Ok(Self { raw })
        } else {
            Err(SpeedError::AddressOutOfBounds(raw))
        }
    }

    /// Creates a user-accessible electrostatic storage address (`300..=1013`).
    pub fn user(raw: u16) -> Result<Self, SpeedError> {
        let addr = Self::new(raw)?;
        if addr.is_user_accessible() {
            Ok(addr)
        } else {
            Err(SpeedError::ControlRegionAccess(raw))
        }
    }

    /// Returns the raw integer address value (`0..=1023`).
    pub fn raw(self) -> u16 {
        self.raw
    }

    /// Returns `true` when the address lies inside user storage (`300..=1013`).
    pub fn is_user_accessible(self) -> bool {
        (USER_MEM_START..=USER_MEM_END).contains(&self.raw)
    }
}

/// 72-bit IBM 701 word stored right-aligned in the lower 72 bits of `u128`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Word72 {
    bits: u128,
}

impl Word72 {
    /// Validates that only bits `0..72` are populated and wraps them in `Word72`.
    pub fn new(bits: u128) -> Result<Self, SpeedError> {
        if (bits & !WORD72_MASK) != 0 {
            Err(SpeedError::InvalidWordBits(bits))
        } else {
            Ok(Self { bits })
        }
    }

    /// Returns the raw 72-bit value right-aligned in `u128`.
    pub fn raw(self) -> u128 {
        self.bits
    }

    /// Extracts a single bit using IBM 701 1-based MSB numbering (`1..=72`).
    pub fn bit(self, pos_1based: u8) -> bool {
        assert!(
            (1..=WORD_BITS as u8).contains(&pos_1based),
            "bit position {pos_1based} out of 1..=72"
        );
        ((self.bits >> (WORD_BITS as u8 - pos_1based)) & 1) != 0
    }

    /// Extracts an inclusive bit slice using IBM 701 1-based MSB numbering (`1..=72`).
    pub fn bits(self, start_1based: u8, end_1based: u8) -> u64 {
        assert!(
            (1..=WORD_BITS as u8).contains(&start_1based)
                && (start_1based..=WORD_BITS as u8).contains(&end_1based),
            "invalid 1-based bit slice {start_1based}..={end_1based}"
        );
        let width = (end_1based - start_1based + 1) as u32;
        assert!(width <= 64, "bit slice width {width} exceeds u64");
        let mask = if width == 64 {
            u64::MAX as u128
        } else {
            (1u128 << width) - 1
        };
        ((self.bits >> (WORD_BITS as u8 - end_1based)) & mask) as u64
    }

    /// Splits the 72-bit word into 12 six-bit tape groups (group 1 = bits `1..=6` .. group 12 = bits `67..=72`).
    pub fn to_six_bit_groups(self) -> [u8; 12] {
        let mut groups = [0u8; 12];
        for (idx, slot) in groups.iter_mut().enumerate() {
            *slot = ((self.bits >> (66 - 6 * idx)) & 0x3F) as u8;
        }
        groups
    }

    /// Reassembles a 72-bit word from 12 six-bit tape groups.
    pub fn from_six_bit_groups(groups: [u8; 12]) -> Self {
        let mut bits = 0u128;
        for (idx, &group) in groups.iter().enumerate() {
            bits |= ((group & 0x3F) as u128) << (66 - 6 * idx);
        }
        Self { bits }
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
        Self {
            negative: false,
            fraction: 0,
            exp_negative: false,
            exponent: 0,
        }
    }

    /// Converts an IEEE-754 `f64` into normalized SpeedCo I 35-bit floating binary.
    pub fn from_f64(value: f64) -> Result<Self, SpeedError> {
        if !value.is_finite() {
            return Err(SpeedError::BinaryExponentOutOfRange(i32::MAX));
        }
        if value == 0.0 {
            return Ok(Self::zero());
        }

        let bits = value.to_bits();
        let negative = (bits >> 63) != 0;
        let raw_exp = ((bits >> 52) & 0x7FF) as i32;
        let raw_mant = bits & ((1u64 << 52) - 1);

        let (sig53, mut exp) = if raw_exp == 0 {
            // Normalize subnormal mantissa so MSB sits at bit 52 (53-bit significand).
            let shift = raw_mant.leading_zeros() - 11;
            (raw_mant << shift, (1 - 1022) - (shift as i32))
        } else {
            ((1u64 << 52) | raw_mant, raw_exp - 1022)
        };

        // Round-to-nearest-even when truncating 53-bit significand down to 35 fraction bits.
        const DROP_BITS: u32 = 53 - FRACTION_BITS;
        let dropped = sig53 & ((1u64 << DROP_BITS) - 1);
        let halfway = 1u64 << (DROP_BITS - 1);
        let mut fraction = sig53 >> DROP_BITS;
        if dropped > halfway || (dropped == halfway && (fraction & 1) != 0) {
            fraction += 1;
            if fraction == (1u64 << FRACTION_BITS) {
                fraction >>= 1;
                exp += 1;
            }
        }

        if exp.unsigned_abs() > MAX_BINARY_EXPONENT {
            return Err(SpeedError::BinaryExponentOutOfRange(exp));
        }

        Ok(Self {
            negative,
            fraction,
            exp_negative: exp < 0,
            exponent: exp.unsigned_abs(),
        })
    }

    /// Converts this SpeedCo I floating binary value to `f64`.
    pub fn to_f64(self) -> f64 {
        if self.fraction == 0 {
            return 0.0;
        }
        let frac = (self.fraction as f64) / ((1u64 << FRACTION_BITS) as f64);
        let mag = frac * 2.0_f64.powi(self.signed_exponent());
        if self.negative { -mag } else { mag }
    }

    /// Returns the signed binary exponent (`-131_071..=131_071`).
    pub fn signed_exponent(self) -> i32 {
        if self.exp_negative {
            -(self.exponent as i32)
        } else {
            self.exponent as i32
        }
    }

    /// Returns `true` when the fractional part is zero.
    pub fn is_zero_fraction(self) -> bool {
        self.fraction == 0
    }

    /// Returns `true` if `fraction == 0` or `0.5 <= |F| < 1` (`fraction >= 2^34`).
    pub fn is_normalized(self) -> bool {
        self.fraction == 0 || (MIN_NORM_FRACTION..=FRACTION_MASK).contains(&self.fraction)
    }

    /// Returns `|self|`.
    pub fn abs(self) -> Self {
        Self {
            negative: false,
            ..self
        }
    }

    /// Returns `-self`.
    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Self {
        Self {
            negative: !self.negative,
            ..self
        }
    }

    /// Encodes into a 72-bit word (bit 1 sign, 2..=36 frac, 37 exp sign, 38..=54 exp, 55..=72 zero).
    pub fn to_word(self) -> Word72 {
        let mut bits = 0u128;
        if self.negative {
            bits |= 1u128 << 71;
        }
        bits |= ((self.fraction & FRACTION_MASK) as u128) << 36;
        if self.exp_negative {
            bits |= 1u128 << 35;
        }
        bits |= ((self.exponent & MAX_BINARY_EXPONENT) as u128) << 18;
        Word72 { bits }
    }

    /// Decodes a 72-bit word into `SpeedFloat`.
    pub fn from_word(word: Word72) -> Self {
        Self {
            negative: word.bit(1),
            fraction: word.bits(2, 36),
            exp_negative: word.bit(37),
            exponent: word.bits(38, 54) as u32,
        }
    }
}

impl std::ops::Neg for SpeedFloat {
    type Output = Self;

    fn neg(self) -> Self::Output {
        SpeedFloat::neg(self)
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
    pub fn new(negative: bool, digits: u64, exponent: i16) -> Result<Self, SpeedError> {
        if !(-MAX_DECIMAL_EXPONENT..=MAX_DECIMAL_EXPONENT).contains(&exponent) {
            return Err(SpeedError::DecimalExponentOutOfRange(exponent as i32));
        }
        if digits >= TEN_DIGITS_LIMIT {
            return Err(SpeedError::InvalidCardFormat(format!(
                "decimal fraction exceeds 10 digits: {digits}"
            )));
        }
        Ok(Self {
            negative,
            digits,
            exponent,
        })
    }

    /// Converts external decimal floating point to internal binary `SpeedFloat`.
    pub fn to_speed_float(self) -> Result<SpeedFloat, SpeedError> {
        if self.digits == 0 {
            return Ok(SpeedFloat::zero());
        }
        let sign = if self.negative { "-" } else { "" };
        let repr = format!("{sign}0.{:010}e{}", self.digits, self.exponent);
        let val: f64 = repr.parse().map_err(|err| {
            SpeedError::InvalidCardFormat(format!("invalid decimal float '{repr}': {err}"))
        })?;
        SpeedFloat::from_f64(val)
    }

    /// Converts internal binary `SpeedFloat` to external decimal `DecimalFloat`, rejecting `|E| > 236`.
    pub fn from_speed_float(val: SpeedFloat) -> Result<Self, SpeedError> {
        if val.is_zero_fraction() {
            return Ok(Self {
                negative: false,
                digits: 0,
                exponent: 0,
            });
        }

        // Reject out-of-range decimal exponents before calling to_f64() so large binary exponents do not overflow f64.
        let frac = (val.fraction as f64) / ((1u64 << FRACTION_BITS) as f64);
        let log10_val = frac.log10() + (val.signed_exponent() as f64) * std::f64::consts::LOG10_2;
        let est_dec_exp = log10_val.floor() as i32 + 1;
        let dec_limit = MAX_DECIMAL_EXPONENT as i32;
        if est_dec_exp < -dec_limit - 1 || est_dec_exp > dec_limit + 1 {
            return Err(SpeedError::DecimalExponentOutOfRange(est_dec_exp));
        }

        let formatted = format!("{:.9e}", val.to_f64().abs());
        let (mant_str, exp_str) = formatted.split_once('e').ok_or_else(|| {
            SpeedError::ParseError(format!("unexpected scientific format: {formatted}"))
        })?;
        let digits_str: String = mant_str.chars().filter(|&ch| ch != '.').collect();
        let digits: u64 = digits_str.parse().map_err(|err| {
            SpeedError::ParseError(format!("invalid decimal digits '{digits_str}': {err}"))
        })?;
        let sci_exp: i32 = exp_str.parse().map_err(|err| {
            SpeedError::ParseError(format!("invalid decimal exponent '{exp_str}': {err}"))
        })?;
        let dec_exp = sci_exp + 1;
        if !(-dec_limit..=dec_limit).contains(&dec_exp) {
            return Err(SpeedError::DecimalExponentOutOfRange(dec_exp));
        }

        Ok(Self {
            negative: val.negative,
            digits,
            exponent: dec_exp as i16,
        })
    }
}

/// 3-bit address modification selector (`0..=7`) controlling `R_A`, `R_B`, and `R_C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RCode {
    raw: u8,
}

impl RCode {
    /// Validates `val <= 7`.
    pub fn new(val: u8) -> Result<Self, SpeedError> {
        if val <= 7 {
            Ok(Self { raw: val })
        } else {
            Err(SpeedError::InvalidRCode(val))
        }
    }

    /// Returns raw R-code value (`0..=7`).
    pub fn raw(self) -> u8 {
        self.raw
    }

    /// Returns `true` when bit 4 is set (`R` in `{4, 5, 6, 7}`), adding `R_A` to `A`.
    pub fn modifies_a(self) -> bool {
        (self.raw & 0b100) != 0
    }

    /// Returns `true` when bit 2 is set (`R` in `{2, 3, 6, 7}`), adding `R_B` to `B`.
    pub fn modifies_b(self) -> bool {
        (self.raw & 0b010) != 0
    }

    /// Returns `true` when bit 1 is set (`R` in `{1, 3, 5, 7}`), adding `R_C` to `C`.
    pub fn modifies_c(self) -> bool {
        (self.raw & 0b001) != 0
    }
}

/// Selective listing code (`0`, `3`, or `6`), corresponding to panel switches 1, 2, or 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LCode {
    raw: u8,
}

impl LCode {
    /// Validates `val` is one of `0`, `3`, or `6`.
    pub fn new(val: u8) -> Result<Self, SpeedError> {
        match val {
            0 | 3 | 6 => Ok(Self { raw: val }),
            _ => Err(SpeedError::InvalidLCode(val)),
        }
    }

    /// Returns raw L-code (`0`, `3`, or `6`).
    pub fn raw(self) -> u8 {
        self.raw
    }

    /// Maps `0 -> 1`, `3 -> 2`, `6 -> 3`.
    pub fn switch_number(self) -> u8 {
        match self.raw {
            0 => 1,
            3 => 2,
            6 => 3,
            other => unreachable!("validated LCode invariant broken: {other}"),
        }
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

const OP1_TABLE: &[(Op1, &str, u16, bool)] = &[
    (Op1::Add, "ADD", 658, false),
    (Op1::Sub, "SUB", 696, false),
    (Op1::AddAb, "ADDAB", 699, false),
    (Op1::AbAdd, "ABADD", 703, false),
    (Op1::SubAb, "SUBAB", 707, false),
    (Op1::AbSub, "ABSUB", 711, false),
    (Op1::Mpy, "MPY", 715, false),
    (Op1::NgMpy, "NGMPY", 731, false),
    (Op1::Div, "DIV", 734, false),
    (Op1::NgDiv, "NGDIV", 748, false),
    (Op1::Sqrt, "SQRT", 782, false),
    (Op1::Sine, "SINE", 780, false),
    (Op1::Artan, "ARTAN", 781, false),
    (Op1::Exp, "EXP", 783, false),
    (Op1::Ln, "LN", 784, false),
    (Op1::WrTpJ, "WRTPJ", 532, false),
    (Op1::WrTpK, "WRTPK", 533, false),
    (Op1::WrTpL, "WRTPL", 534, false),
    (Op1::WrTpM, "WRTPM", 535, false),
    (Op1::RfTpJ, "RFTPJ", 435, true),
    (Op1::RfTpK, "RFTPK", 437, true),
    (Op1::RfTpL, "RFTPL", 439, true),
    (Op1::RfTpM, "RFTPM", 441, true),
    (Op1::RbTpJ, "RBTPJ", 416, true),
    (Op1::RbTpK, "RBTPK", 417, true),
    (Op1::RbTpL, "RBTPL", 418, true),
    (Op1::RbTpM, "RBTPM", 419, true),
    (Op1::SfTpJ, "SFTPJ", 556, true),
    (Op1::SfTpK, "SFTPK", 557, true),
    (Op1::SfTpL, "SFTPL", 558, true),
    (Op1::SfTpM, "SFTPM", 559, true),
    (Op1::SbTpJ, "SBTPJ", 546, true),
    (Op1::SbTpK, "SBTPK", 547, true),
    (Op1::SbTpL, "SBTPL", 548, true),
    (Op1::SbTpM, "SBTPM", 549, true),
    (Op1::RwTpJ, "RWTPJ", 572, false),
    (Op1::RwTpK, "RWTPK", 574, false),
    (Op1::RwTpL, "RWTPL", 576, false),
    (Op1::RwTpM, "RWTPM", 578, false),
    (Op1::EfTpJ, "EFTPJ", 564, false),
    (Op1::EfTpK, "EFTPK", 566, false),
    (Op1::EfTpL, "EFTPL", 568, false),
    (Op1::EfTpM, "EFTPM", 570, false),
    (Op1::WrDrP, "WRDRP", 497, true),
    (Op1::WrDrQ, "WRDRQ", 498, true),
    (Op1::RfDrP, "RFDRP", 526, false),
    (Op1::RfDrQ, "RFDRQ", 528, false),
    (Op1::Print, "PRINT", 580, false),
    (Op1::NoOp, "NOOP", 751, false),
];

impl Op1 {
    /// Returns the 3-digit numerical operation code for `OP1`.
    pub fn code(self) -> u16 {
        OP1_TABLE
            .iter()
            .find(|&&(op, _, _, _)| op == self)
            .expect("every Op1 variant exists in OP1_TABLE")
            .2
    }

    /// Returns the alphabetical mnemonic for `OP1`.
    pub fn mnemonic(self) -> &'static str {
        OP1_TABLE
            .iter()
            .find(|&&(op, _, _, _)| op == self)
            .expect("every Op1 variant exists in OP1_TABLE")
            .1
    }

    /// Decodes a 3-digit numerical `OP1` code.
    pub fn from_code(code: u16) -> Result<Self, SpeedError> {
        OP1_TABLE
            .iter()
            .find(|&&(_, _, c, _)| c == code)
            .map(|&(op, _, _, _)| op)
            .ok_or(SpeedError::InvalidOp1Code(code))
    }

    /// Decodes an alphabetical `OP1` mnemonic.
    pub fn from_mnemonic(s: &str) -> Result<Self, SpeedError> {
        let trimmed = s.trim();
        OP1_TABLE
            .iter()
            .find(|&&(_, mnem, _, _)| mnem == trimmed)
            .map(|&(op, _, _, _)| op)
            .ok_or_else(|| SpeedError::InvalidMnemonic(s.to_string()))
    }

    /// Returns `true` for `OP1` operations with automatic skips (`RFTP*`, `RBTP*`, `SFTP*`, `SBTP*`, `WRDR*`).
    pub fn forbids_op2(self) -> bool {
        OP1_TABLE
            .iter()
            .find(|&&(op, _, _, _)| op == self)
            .expect("every Op1 variant exists in OP1_TABLE")
            .3
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

const OP2_TABLE: &[(Op2, &str, u16)] = &[
    (Op2::Tr, "TR", 104),
    (Op2::TrPl, "TRPL", 109),
    (Op2::TrMn, "TRMN", 115),
    (Op2::TrZ, "TRZ", 112),
    (Op2::SnTrP, "SNTRP", 117),
    (Op2::SnTrQ, "SNTRQ", 120),
    (Op2::TiA, "TIA", 128),
    (Op2::TiB, "TIB", 126),
    (Op2::TiC, "TIC", 125),
    (Op2::TiAB, "TIAB", 130),
    (Op2::TiBC, "TIBC", 127),
    (Op2::TiAC, "TIAC", 129),
    (Op2::TiABC, "TIABC", 131),
    (Op2::TdA, "TDA", 135),
    (Op2::TdB, "TDB", 133),
    (Op2::TdC, "TDC", 132),
    (Op2::TdAB, "TDAB", 137),
    (Op2::TdBC, "TDBC", 134),
    (Op2::TdAC, "TDAC", 136),
    (Op2::TdABC, "TDABC", 138),
    (Op2::SetRA, "SETRA", 139),
    (Op2::SetRB, "SETRB", 250),
    (Op2::SetRC, "SETRC", 145),
    (Op2::SkRA, "SKRA", 152),
    (Op2::SkRB, "SKRB", 159),
    (Op2::SkRC, "SKRC", 162),
    (Op2::RAddA, "RADDA", 199),
    (Op2::RAddB, "RADDB", 202),
    (Op2::RAddC, "RADDC", 205),
    (Op2::RAddD, "RADDD", 208),
    (Op2::AddA, "ADDA", 177),
    (Op2::AddB, "ADDB", 184),
    (Op2::AddC, "ADDC", 190),
    (Op2::AddD, "ADDD", 193),
    (Op2::SubA, "SUBA", 211),
    (Op2::SubB, "SUBB", 216),
    (Op2::SubC, "SUBC", 221),
    (Op2::SubD, "SUBD", 226),
    (Op2::StA, "STA", 251),
    (Op2::StB, "STB", 252),
    (Op2::StC, "STC", 235),
    (Op2::StD, "STD", 244),
    (Op2::Skip, "SKIP", 165),
    (Op2::PrCh, "PRCH", 232),
    (Op2::StCh, "STCH", 253),
    (Op2::EchTr, "ECHTR", 254),
    (Op2::Stop, "STOP", 123),
    (Op2::NoOp, "", 0),
];

impl Op2 {
    /// Returns the 3-digit numerical operation code for `OP2`.
    pub fn code(self) -> u16 {
        OP2_TABLE
            .iter()
            .find(|&&(op, _, _)| op == self)
            .expect("every Op2 variant exists in OP2_TABLE")
            .2
    }

    /// Returns the alphabetical mnemonic for `OP2` (`""` for `NoOp`).
    pub fn mnemonic(self) -> &'static str {
        OP2_TABLE
            .iter()
            .find(|&&(op, _, _)| op == self)
            .expect("every Op2 variant exists in OP2_TABLE")
            .1
    }

    /// Decodes a 3-digit numerical `OP2` code (`0` -> `Op2::NoOp`).
    pub fn from_code(code: u16) -> Result<Self, SpeedError> {
        OP2_TABLE
            .iter()
            .find(|&&(_, _, c)| c == code)
            .map(|&(op, _, _)| op)
            .ok_or(SpeedError::InvalidOp2Code(code))
    }

    /// Decodes an alphabetical `OP2` mnemonic (`""` or `"-"` -> `Op2::NoOp`).
    pub fn from_mnemonic(s: &str) -> Result<Self, SpeedError> {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed == "-" {
            return Ok(Op2::NoOp);
        }
        OP2_TABLE
            .iter()
            .find(|&&(_, mnem, _)| mnem == trimmed)
            .map(|&(op, _, _)| op)
            .ok_or_else(|| SpeedError::InvalidMnemonic(s.to_string()))
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
        let bits = ((self.r_code.raw() as u128) << 67)
            | ((self.op1.code() as u128) << 54)
            | ((self.op2.code() as u128) << 46)
            | ((self.d.raw() as u128) << 36)
            | ((self.a.raw() as u128) << 25)
            | ((self.b.raw() as u128) << 15)
            | ((self.l_code.raw() as u128) << 10)
            | (self.c.raw() as u128);
        Word72 { bits }
    }

    /// Decodes a 72-bit instruction word and validates field constraints.
    pub fn from_word(word: Word72) -> Result<Self, SpeedError> {
        if word.bit(2) || word.bit(6) || word.bit(37) || word.bits(58, 59) != 0 {
            return Err(SpeedError::InvalidWordBits(word.raw()));
        }
        let r_code = RCode::new(word.bits(3, 5) as u8)?;
        let op1 = Op1::from_code(word.bits(7, 18) as u16)?;
        let op2 = Op2::from_code(word.bits(19, 26) as u16)?;
        if op1.forbids_op2() && op2 != Op2::NoOp {
            return Err(SpeedError::ForbiddenOp2WithSkipOp1 { op1, op2 });
        }
        let d = Address::new(word.bits(27, 36) as u16)?;
        let a = Address::new(word.bits(38, 47) as u16)?;
        let b = Address::new(word.bits(48, 57) as u16)?;
        let l_code = LCode::new(word.bits(60, 62) as u8)?;
        let c = Address::new(word.bits(63, 72) as u16)?;

        Ok(Self {
            r_code,
            op1,
            op2,
            d,
            a,
            b,
            c,
            l_code,
        })
    }
}

/// 1024-word electrostatic cathode-ray-tube memory with user-accessible cells `300..=1013`.
#[derive(Debug, Clone)]
pub struct ElectrostaticMemory {
    words: Vec<Word72>,
}

impl Default for ElectrostaticMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl ElectrostaticMemory {
    /// Creates zero-initialized electrostatic memory.
    pub fn new() -> Self {
        Self {
            words: vec![Word72::default(); TOTAL_MEM_WORDS],
        }
    }

    /// Reads a raw 72-bit word from user storage (`300..=1013`), rejecting control cells.
    pub fn read(&self, addr: Address) -> Result<Word72, SpeedError> {
        if !addr.is_user_accessible() {
            return Err(SpeedError::ControlRegionAccess(addr.raw()));
        }
        Ok(self.words[addr.raw() as usize])
    }

    /// Writes a raw 72-bit word into user storage (`300..=1013`), rejecting control cells.
    pub fn write(&mut self, addr: Address, word: Word72) -> Result<(), SpeedError> {
        if !addr.is_user_accessible() {
            return Err(SpeedError::ControlRegionAccess(addr.raw()));
        }
        self.words[addr.raw() as usize] = word;
        Ok(())
    }

    /// Reads and decodes a `SpeedFloat` from user storage (`300..=1013`).
    pub fn read_float(&self, addr: Address) -> Result<SpeedFloat, SpeedError> {
        Ok(SpeedFloat::from_word(self.read(addr)?))
    }

    /// Encodes and writes a `SpeedFloat` into user storage (`300..=1013`).
    pub fn write_float(&mut self, addr: Address, val: SpeedFloat) -> Result<(), SpeedError> {
        self.write(addr, val.to_word())
    }

    /// Reads and decodes an `Instruction` from user storage (`300..=1013`).
    pub fn read_instruction(&self, addr: Address) -> Result<Instruction, SpeedError> {
        Instruction::from_word(self.read(addr)?)
    }

    /// Encodes and writes an `Instruction` into user storage (`300..=1013`).
    pub fn write_instruction(&mut self, addr: Address, instr: Instruction) -> Result<(), SpeedError> {
        self.write(addr, instr.to_word())
    }
}

//! Phase 2: SpeedCo I `mpc`-based symbolic program parser and 80-column punched card parser.

use mpc::{
    MpcAst, MpcParser, MpcResult, mpc_anchor, mpc_and, mpc_blank, mpc_char, mpc_digit, mpc_eoi,
    mpc_many, mpc_many1, mpc_maybe, mpc_noneof, mpc_or, mpc_parse, mpc_pass, mpc_string, mpca_tag,
    mpcf_fold_ast, mpcf_fst, mpcf_null, mpcf_strfold,
};

use crate::core_types::{
    Address, DecimalFloat, Instruction, LCode, Op1, Op2, RCode, SpeedError,
};

/// Ordered from longest mnemonic to shortest so `mpc_or` never greedily matches a shorter prefix.
const OP1_MNEMONICS_DESC: &[&str] = &[
    "ADDAB", "ABADD", "SUBAB", "ABSUB", "NGMPY", "NGDIV", "ARTAN", "WRTPJ", "WRTPK", "WRTPL",
    "WRTPM", "RFTPJ", "RFTPK", "RFTPL", "RFTPM", "RBTPJ", "RBTPK", "RBTPL", "RBTPM", "SFTPJ",
    "SFTPK", "SFTPL", "SFTPM", "SBTPJ", "SBTPK", "SBTPL", "SBTPM", "RWTPJ", "RWTPK", "RWTPL",
    "RWTPM", "EFTPJ", "EFTPK", "EFTPL", "EFTPM", "WRDRP", "WRDRQ", "RFDRP", "RFDRQ", "PRINT",
    "SQRT", "SINE", "NOOP", "ADD", "SUB", "MPY", "DIV", "EXP", "LN",
];

/// Ordered from longest mnemonic to shortest for unambiguous `mpc_or` matching.
const OP2_MNEMONICS_DESC: &[&str] = &[
    "SNTRP", "SNTRQ", "TIABC", "TDABC", "SETRA", "SETRB", "SETRC", "RADDA", "RADDB", "RADDC",
    "RADDD", "ECHTR", "TRPL", "TRMN", "TIAB", "TIBC", "TIAC", "TDAB", "TDBC", "TDAC", "SKRA",
    "SKRB", "SKRC", "ADDA", "ADDB", "ADDC", "ADDD", "SUBA", "SUBB", "SUBC", "SUBD", "SKIP",
    "PRCH", "STCH", "STOP", "TRZ", "TIA", "TIB", "TIC", "TDA", "TDB", "TDC", "STA", "STB", "STC",
    "STD", "TR",
];

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
        let code1 = self.numeric_op1.ok_or_else(|| {
            SpeedError::InvalidCardFormat(
                "alphabetical instruction card must be gang-punched into cols 46-48 before loading"
                    .to_string(),
            )
        })?;
        let code2 = match (self.numeric_op2, &self.alpha_op2) {
            (Some(c), _) => c,
            (None, Some(_)) => {
                return Err(SpeedError::InvalidCardFormat(
                    "alphabetical OP2 must be gang-punched into cols 49-51 before loading"
                        .to_string(),
                ));
            }
            (None, None) => 0,
        };

        let op1 = Op1::from_code(code1)?;
        let op2 = Op2::from_code(code2)?;
        if op1.forbids_op2() && op2 != Op2::NoOp {
            return Err(SpeedError::ForbiddenOp2WithSkipOp1 { op1, op2 });
        }

        Ok(Instruction {
            r_code: self.r_code,
            op1,
            op2,
            d: self.d,
            a: self.a,
            b: self.b,
            c: self.c,
            l_code: self.l_code,
        })
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

fn build_mnemonic_parser(mnemonics_desc: &[&str]) -> MpcParser {
    let alts = mnemonics_desc.iter().map(|m| mpc_string(m)).collect();
    let word_boundary = mpc_anchor(|_, next| !next.is_ascii_alphanumeric());
    mpc_and(vec![mpc_or(alts), word_boundary], mpcf_fst)
}

/// Builds an `mpc` combinator parser matching any valid alphabetical `OP1` mnemonic.
pub fn build_op1_mnemonic_parser() -> MpcParser {
    build_mnemonic_parser(OP1_MNEMONICS_DESC)
}

/// Builds an `mpc` combinator parser matching any valid alphabetical `OP2` mnemonic.
pub fn build_op2_mnemonic_parser() -> MpcParser {
    build_mnemonic_parser(OP2_MNEMONICS_DESC)
}

fn build_symbolic_line_parser() -> MpcParser {
    let ws1 = mpc_many1(mpc_blank(), mpcf_null);
    let ws0 = mpc_many(mpc_blank(), mpcf_null);
    let digits = mpc_many1(mpc_digit(), mpcf_strfold);

    let loc = mpca_tag(digits.clone(), "loc");
    let op1 = mpca_tag(build_op1_mnemonic_parser(), "op1");
    let r = mpca_tag(digits.clone(), "r");
    let a = mpca_tag(digits.clone(), "a");
    let b = mpca_tag(digits.clone(), "b");
    let c = mpca_tag(digits.clone(), "c");

    let op2_tok = mpca_tag(mpc_or(vec![build_op2_mnemonic_parser(), mpc_string("-")]), "op2");
    let d_tok = mpca_tag(mpc_or(vec![digits.clone(), mpc_string("-")]), "d");
    let l_tok = mpca_tag(mpc_or(vec![digits, mpc_string("-")]), "l");

    let opt_l = mpc_maybe(mpc_and(vec![ws1.clone(), l_tok], mpcf_fold_ast));
    let opt_d = mpc_maybe(mpc_and(vec![ws1.clone(), d_tok, opt_l], mpcf_fold_ast));
    let opt_op2 = mpc_maybe(mpc_and(vec![ws1.clone(), op2_tok, opt_d], mpcf_fold_ast));

    let opt_remarks = mpc_maybe(mpc_and(
        vec![
            ws0.clone(),
            mpc_char(';'),
            mpca_tag(mpc_many(mpc_noneof("\r\n"), mpcf_strfold), "remarks"),
        ],
        mpcf_fold_ast,
    ));

    mpc_and(
        vec![
            ws0.clone(),
            loc,
            ws1.clone(),
            op1,
            ws1.clone(),
            r,
            ws1.clone(),
            a,
            ws1.clone(),
            b,
            ws1.clone(),
            c,
            opt_op2,
            opt_remarks,
            ws0,
            mpc_or(vec![mpc_eoi(), mpc_pass()]),
            mpc_eoi(),
        ],
        mpcf_fold_ast,
    )
}

fn collect_ast_tags<'a>(ast: &'a MpcAst, out: &mut Vec<(&'a str, &'a str)>) {
    if !ast.tag.is_empty() {
        out.push((ast.tag.as_str(), ast.contents.as_str()));
    }
    for child in &ast.children {
        collect_ast_tags(child, out);
    }
}

fn parse_u16_digits(s: &str, field: &str) -> Result<u16, SpeedError> {
    let trimmed = s.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(SpeedError::InvalidCardFormat(format!(
            "expected digits in {field}, got '{s}'"
        )));
    }
    trimmed
        .parse::<u16>()
        .map_err(|err| SpeedError::InvalidCardFormat(format!("invalid {field} '{s}': {err}")))
}

fn parse_u8_digits(s: &str, field: &str) -> Result<u8, SpeedError> {
    let trimmed = s.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(SpeedError::InvalidCardFormat(format!(
            "expected digits in {field}, got '{s}'"
        )));
    }
    trimmed
        .parse::<u8>()
        .map_err(|err| SpeedError::InvalidCardFormat(format!("invalid {field} '{s}': {err}")))
}

/// Parses a single symbolic SpeedCo I instruction line using `mpc`.
pub fn parse_instruction_line(input: &str) -> Result<ParsedInstructionLine, SpeedError> {
    let line = input.trim_end_matches(&['\r', '\n'][..]);
    let parser = build_symbolic_line_parser();
    let val = match mpc_parse("speedcoding", line, &parser) {
        MpcResult::Ok(v) => v,
        MpcResult::Err(err) => return Err(SpeedError::ParseError(err.to_string())),
    };
    let ast = val
        .downcast_ref::<MpcAst>()
        .ok_or_else(|| SpeedError::ParseError("missing parsed instruction AST".to_string()))?;

    let mut tags = Vec::new();
    collect_ast_tags(ast, &mut tags);

    let find_tag = |name: &str| -> Option<&str> {
        tags.iter()
            .find(|&&(t, _)| t == name || t.ends_with(&format!("|{name}")))
            .map(|&(_, c)| c)
    };

    let loc_str = find_tag("loc")
        .ok_or_else(|| SpeedError::ParseError("missing instruction location".to_string()))?;
    let op1_str = find_tag("op1")
        .ok_or_else(|| SpeedError::ParseError("missing OP1 mnemonic".to_string()))?;
    let r_str =
        find_tag("r").ok_or_else(|| SpeedError::ParseError("missing R-code".to_string()))?;
    let a_str =
        find_tag("a").ok_or_else(|| SpeedError::ParseError("missing A address".to_string()))?;
    let b_str =
        find_tag("b").ok_or_else(|| SpeedError::ParseError("missing B address".to_string()))?;
    let c_str =
        find_tag("c").ok_or_else(|| SpeedError::ParseError("missing C address".to_string()))?;

    let location = Address::new(parse_u16_digits(loc_str, "location")?)?;
    let op1 = Op1::from_mnemonic(op1_str)?;
    let r_code = RCode::new(parse_u8_digits(r_str, "R-code")?)?;
    let a = Address::new(parse_u16_digits(a_str, "A address")?)?;
    let b = Address::new(parse_u16_digits(b_str, "B address")?)?;
    let c = Address::new(parse_u16_digits(c_str, "C address")?)?;

    let op2 = match find_tag("op2") {
        Some(s) => Op2::from_mnemonic(s)?,
        None => Op2::NoOp,
    };
    if op1.forbids_op2() && op2 != Op2::NoOp {
        return Err(SpeedError::ForbiddenOp2WithSkipOp1 { op1, op2 });
    }

    let d_raw = match find_tag("d") {
        None | Some("-") => 0,
        Some(s) => parse_u16_digits(s, "D address")?,
    };
    let d = Address::new(d_raw)?;

    let l_raw = match find_tag("l") {
        None | Some("-") => 0,
        Some(s) => parse_u8_digits(s, "L-code")?,
    };
    let l_code = LCode::new(l_raw)?;

    let remarks = find_tag("remarks")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    Ok(ParsedInstructionLine {
        location,
        instruction: Instruction {
            r_code,
            op1,
            op2,
            d,
            a,
            b,
            c,
            l_code,
        },
        remarks,
    })
}

/// Parses a multi-line symbolic SpeedCo I program.
pub fn parse_program(source: &str) -> Result<Vec<ParsedInstructionLine>, SpeedError> {
    let mut lines = Vec::new();
    for raw_line in source.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        lines.push(parse_instruction_line(trimmed)?);
    }
    Ok(lines)
}

fn card_chars_80(card_80cols: &str) -> Result<Vec<char>, SpeedError> {
    let chars: Vec<char> = card_80cols.chars().collect();
    if chars.len() != 80 {
        return Err(SpeedError::InvalidCardFormat(format!(
            "expected exactly 80 card columns, got {}",
            chars.len()
        )));
    }
    Ok(chars)
}

fn col_slice(chars: &[char], start_1based: usize, end_1based: usize) -> String {
    chars[start_1based - 1..end_1based].iter().collect()
}

fn is_data_col8_double_punch(ch: char) -> bool {
    matches!(ch, '*' | '+' | '&' | 'Z')
}

/// Parses an 80-column SpeedCo I Instruction Card record.
pub fn parse_instruction_card(card_80cols: &str) -> Result<InstructionCard, SpeedError> {
    let chars = card_chars_80(card_80cols)?;
    if is_data_col8_double_punch(chars[7]) {
        return Err(SpeedError::InvalidCardFormat(format!(
            "col 8 indicates data double-punch ('{}'), not instruction card",
            chars[7]
        )));
    }
    if chars[8] != ' ' {
        return Err(SpeedError::InvalidCardFormat(format!(
            "instruction card col 9 must be blank, got '{}'",
            chars[8]
        )));
    }
    if chars[13] != '0' {
        return Err(SpeedError::InvalidCardFormat(format!(
            "instruction card col 14 must be '0', got '{}'",
            chars[13]
        )));
    }
    let zeros_33_34 = col_slice(&chars, 33, 34);
    if zeros_33_34 != "00" {
        return Err(SpeedError::InvalidCardFormat(format!(
            "instruction card cols 33-34 must be '00', got '{zeros_33_34}'"
        )));
    }
    let zeros_52_55 = col_slice(&chars, 52, 55);
    if zeros_52_55 != "0000" {
        return Err(SpeedError::InvalidCardFormat(format!(
            "instruction card cols 52-55 must be '0000', got '{zeros_52_55}'"
        )));
    }

    let label = col_slice(&chars, 1, 7).trim_end().to_string();
    let location = Address::new(parse_u16_digits(&col_slice(&chars, 10, 13), "cols 10-13 location")?)?;

    let alpha1_raw = col_slice(&chars, 15, 19);
    let alpha1_trim = alpha1_raw.trim();
    let alpha_op1 = if alpha1_trim.is_empty() {
        None
    } else {
        Op1::from_mnemonic(alpha1_trim)?;
        Some(alpha1_trim.to_string())
    };

    let r_raw = match chars[19] {
        ' ' => 0,
        ch if ch.is_ascii_digit() => (ch as u8) - b'0',
        other => {
            return Err(SpeedError::InvalidCardFormat(format!(
                "invalid col 20 R-code '{other}'"
            )));
        }
    };
    let r_code = RCode::new(r_raw)?;

    let a = Address::new(parse_u16_digits(&col_slice(&chars, 21, 24), "cols 21-24 A")?)?;
    let b = Address::new(parse_u16_digits(&col_slice(&chars, 25, 28), "cols 25-28 B")?)?;
    let c = Address::new(parse_u16_digits(&col_slice(&chars, 29, 32), "cols 29-32 C")?)?;

    let alpha2_raw = col_slice(&chars, 35, 39);
    let alpha2_trim = alpha2_raw.trim();
    let alpha_op2 = if alpha2_trim.is_empty() || alpha2_trim == "-" {
        None
    } else {
        Op2::from_mnemonic(alpha2_trim)?;
        Some(alpha2_trim.to_string())
    };

    let d_raw_str = col_slice(&chars, 40, 43);
    let d_val = if d_raw_str.trim().is_empty() {
        0
    } else {
        parse_u16_digits(&d_raw_str, "cols 40-43 D")?
    };
    let d = Address::new(d_val)?;

    let l_raw = match chars[43] {
        ' ' => 0,
        ch if ch.is_ascii_digit() => (ch as u8) - b'0',
        other => {
            return Err(SpeedError::InvalidCardFormat(format!(
                "invalid col 44 L-code '{other}'"
            )));
        }
    };
    let l_code = LCode::new(l_raw)?;

    let num1_str = col_slice(&chars, 46, 48);
    let numeric_op1 = if num1_str.trim().is_empty() {
        None
    } else {
        let code = parse_u16_digits(&num1_str, "cols 46-48 num OP1")?;
        Op1::from_code(code)?;
        Some(code)
    };

    let num2_str = col_slice(&chars, 49, 51);
    let numeric_op2 = if num2_str.trim().is_empty() {
        None
    } else {
        let code = parse_u16_digits(&num2_str, "cols 49-51 num OP2")?;
        Op2::from_code(code)?;
        Some(code)
    };

    if let (Some(c1), Some(c2)) = (numeric_op1, numeric_op2) {
        let op1 = Op1::from_code(c1)?;
        let op2 = Op2::from_code(c2)?;
        if op1.forbids_op2() && op2 != Op2::NoOp {
            return Err(SpeedError::ForbiddenOp2WithSkipOp1 { op1, op2 });
        }
    }

    let remarks = col_slice(&chars, 56, 80).trim_end().to_string();

    Ok(InstructionCard {
        label,
        location,
        alpha_op1,
        alpha_op2,
        numeric_op1,
        numeric_op2,
        r_code,
        a,
        b,
        c,
        d,
        l_code,
        remarks,
    })
}

/// Simulates sort-and-gang-punch translation of alphabetical `OP1`/`OP2` into numerical columns 46–51.
pub fn gang_punch_instruction_card(card: &InstructionCard) -> Result<InstructionCard, SpeedError> {
    let mut punched = card.clone();

    let code1 = match (&card.alpha_op1, card.numeric_op1) {
        (Some(mnem), _) => Op1::from_mnemonic(mnem)?.code(),
        (None, Some(code)) => {
            Op1::from_code(code)?;
            code
        }
        (None, None) => {
            return Err(SpeedError::InvalidCardFormat(
                "cannot gang-punch card with neither alpha nor numeric OP1".to_string(),
            ));
        }
    };
    punched.numeric_op1 = Some(code1);

    let code2 = match (&card.alpha_op2, card.numeric_op2) {
        (Some(mnem), _) => Op2::from_mnemonic(mnem)?.code(),
        (None, Some(code)) => {
            Op2::from_code(code)?;
            code
        }
        (None, None) => 0,
    };
    punched.numeric_op2 = Some(code2);

    let op1 = Op1::from_code(code1)?;
    let op2 = Op2::from_code(code2)?;
    if op1.forbids_op2() && op2 != Op2::NoOp {
        return Err(SpeedError::ForbiddenOp2WithSkipOp1 { op1, op2 });
    }

    Ok(punched)
}

/// Decodes the trailing IBM 026 overpunched sign+digit character (`12`-zone = positive, `11`-zone = negative).
fn decode_overpunch_digit(ch: char) -> Result<(bool, u8), SpeedError> {
    match ch {
        '0'..='9' => Ok((false, (ch as u8) - b'0')),
        '{' | '+' | '&' => Ok((false, 0)),
        'A'..='I' => Ok((false, (ch as u8) - b'A' + 1)),
        '}' | '-' => Ok((true, 0)),
        'J'..='R' => Ok((true, (ch as u8) - b'J' + 1)),
        other => Err(SpeedError::InvalidCardFormat(format!(
            "invalid IBM overpunch digit '{other}'"
        ))),
    }
}

fn decode_overpunched_uint(raw: &str, field: &str) -> Result<(bool, u64), SpeedError> {
    let chars: Vec<char> = raw.chars().collect();
    if chars.is_empty() {
        return Err(SpeedError::InvalidCardFormat(format!(
            "empty field {field}"
        )));
    }
    let last_idx = chars.len() - 1;
    let mut acc: u64 = 0;
    for &ch in &chars[..last_idx] {
        if !ch.is_ascii_digit() {
            return Err(SpeedError::InvalidCardFormat(format!(
                "expected ASCII digit in {field}, got '{ch}'"
            )));
        }
        acc = acc * 10 + u64::from((ch as u8) - b'0');
    }
    let (neg, last_digit) = decode_overpunch_digit(chars[last_idx])?;
    acc = acc * 10 + u64::from(last_digit);
    Ok((neg, acc))
}

/// Parses an 80-column SpeedCo I Data Card record.
pub fn parse_data_card(card_80cols: &str) -> Result<DataCard, SpeedError> {
    let chars = card_chars_80(card_80cols)?;
    if !is_data_col8_double_punch(chars[7]) {
        return Err(SpeedError::InvalidCardFormat(format!(
            "data card col 8 must carry double-punch 0+1 marker, got '{}'",
            chars[7]
        )));
    }
    if chars[8] != '8' {
        return Err(SpeedError::InvalidCardFormat(format!(
            "data card col 9 must be '8', got '{}'",
            chars[8]
        )));
    }

    let identification = col_slice(&chars, 1, 7).trim_end().to_string();
    let first_location =
        Address::new(parse_u16_digits(&col_slice(&chars, 10, 13), "cols 10-13 location")?)?;

    let word_count = match chars[79] {
        ch @ '1'..='5' => (ch as u8) - b'0',
        other => {
            return Err(SpeedError::InvalidCardFormat(format!(
                "data card col 80 word count must be in 1..=5, got '{other}'"
            )));
        }
    };

    // Slot column ranges (1-based inclusive); slot 3 skips col 45 (manual page 7-4).
    let slot_ranges = [
        (14, 23, None, 24, 26),
        (27, 36, None, 37, 39),
        (40, 44, Some((46, 50)), 51, 53),
        (54, 63, None, 64, 66),
        (67, 76, None, 77, 79),
    ];

    let mut values = Vec::with_capacity(usize::from(word_count));
    for &(f_start, f_end, f_tail, e_start, e_end) in &slot_ranges[..usize::from(word_count)] {
        let mut frac_str = col_slice(&chars, f_start, f_end);
        if let Some((t_start, t_end)) = f_tail {
            frac_str.push_str(&col_slice(&chars, t_start, t_end));
        }
        let exp_str = col_slice(&chars, e_start, e_end);

        let (frac_neg, digits) = decode_overpunched_uint(&frac_str, "data fraction")?;
        let (exp_neg, exp_mag) = decode_overpunched_uint(&exp_str, "data exponent")?;
        let exp_i32 = i32::try_from(exp_mag)
            .map(|v| if exp_neg { -v } else { v })
            .map_err(|_| SpeedError::DecimalExponentOutOfRange(i32::MAX))?;
        let exponent = i16::try_from(exp_i32)
            .map_err(|_| SpeedError::DecimalExponentOutOfRange(exp_i32))?;

        values.push(DecimalFloat::new(frac_neg, digits, exponent)?);
    }

    Ok(DataCard {
        identification,
        first_location,
        values,
        word_count,
    })
}

/// Parses an 80-column SpeedCo I Loading Control Card record.
pub fn parse_loading_control_card(card_80cols: &str) -> Result<LoadingControlCard, SpeedError> {
    let chars = card_chars_80(card_80cols)?;
    if is_data_col8_double_punch(chars[7]) {
        return Err(SpeedError::InvalidCardFormat(format!(
            "loading control card col 8 cannot contain data double-punch marker '{}'",
            chars[7]
        )));
    }
    if chars[8] != '9' {
        return Err(SpeedError::InvalidCardFormat(format!(
            "loading control card col 9 must be '9', got '{}'",
            chars[8]
        )));
    }
    if chars[9] != '0' || chars[14] != '0' || chars[19] != '0' {
        return Err(SpeedError::InvalidCardFormat(
            "loading control card cols 10, 15, and 20 must be '0'".to_string(),
        ));
    }
    let zeros_25_28 = col_slice(&chars, 25, 28);
    if zeros_25_28 != "0000" {
        return Err(SpeedError::InvalidCardFormat(format!(
            "loading control card cols 25-28 must be '0000', got '{zeros_25_28}'"
        )));
    }

    let first_addr =
        Address::new(parse_u16_digits(&col_slice(&chars, 11, 14), "cols 11-14 first")?)?;
    let last_addr =
        Address::new(parse_u16_digits(&col_slice(&chars, 16, 19), "cols 16-19 last")?)?;
    let drum_addr =
        Address::new(parse_u16_digits(&col_slice(&chars, 21, 24), "cols 21-24 drum")?)?;

    let unit = match chars[28] {
        'J' => StorageUnit::TapeJ,
        'K' => StorageUnit::TapeK,
        'L' => StorageUnit::TapeL,
        'M' => StorageUnit::TapeM,
        'P' => StorageUnit::DrumP,
        'Q' => StorageUnit::DrumQ,
        other => {
            return Err(SpeedError::InvalidCardFormat(format!(
                "invalid loading control card unit col 29 '{other}'"
            )));
        }
    };

    if matches!(
        unit,
        StorageUnit::TapeJ | StorageUnit::TapeK | StorageUnit::TapeL | StorageUnit::TapeM
    ) && drum_addr.raw() != 0
    {
        return Err(SpeedError::InvalidCardFormat(format!(
            "tape loading control card cols 21-24 must be '0000', got '{}'",
            drum_addr.raw()
        )));
    }

    Ok(LoadingControlCard {
        first_addr,
        last_addr,
        drum_addr,
        unit,
    })
}

/// Dispatches and parses any 80-column SpeedCo I card based on columns 8–9.
pub fn parse_card(card_80cols: &str) -> Result<PunchedCard, SpeedError> {
    let chars = card_chars_80(card_80cols)?;
    match chars[8] {
        ' ' => Ok(PunchedCard::Instruction(parse_instruction_card(card_80cols)?)),
        '8' => Ok(PunchedCard::Data(parse_data_card(card_80cols)?)),
        '9' => Ok(PunchedCard::LoadingControl(parse_loading_control_card(card_80cols)?)),
        other => Err(SpeedError::InvalidCardFormat(format!(
            "unrecognized card type in col 9: '{other}'"
        ))),
    }
}

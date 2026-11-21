//! Phase 1 TDD tests: 72-bit words, addresses, floating point, operation codes,
//! instruction layout, and electrostatic memory (SpeedCo I manual sections 1-5).

use speedcoding::core_types::{
    Address, DecimalFloat, ElectrostaticMemory, FRACTION_BITS, INITIAL_PC, Instruction, LCode,
    MAX_BINARY_EXPONENT, MAX_DECIMAL_EXPONENT, Op1, Op2, RCode, SPECIAL_R_CONTROL_ADDR,
    SpeedError, SpeedFloat, TOTAL_MEM_WORDS, USER_MEM_END, USER_MEM_LEN, USER_MEM_START,
    WORD_BITS, Word72,
};

fn addr(raw: u16) -> Address {
    Address::new(raw).expect("valid address")
}

#[test]
fn word72_bounds_bits_and_six_bit_groups() {
    assert_eq!(WORD_BITS, 72);
    assert!(Word72::new(0).is_ok());
    assert!(Word72::new((1u128 << 72) - 1).is_ok());
    assert!(matches!(
        Word72::new(1u128 << 72),
        Err(SpeedError::InvalidWordBits(_))
    ));

    // Bit 1 is MSB (2^71), bit 72 is LSB (2^0).
    let msb_set = Word72::new(1u128 << 71).unwrap();
    assert!(msb_set.bit(1));
    assert!(!msb_set.bit(72));
    assert_eq!(msb_set.bits(1, 6), 0b10_0000);

    let lsb_set = Word72::new(1u128).unwrap();
    assert!(!lsb_set.bit(1));
    assert!(lsb_set.bit(72));
    assert_eq!(lsb_set.bits(67, 72), 0b00_0001);

    let groups: [u8; 12] = [0x2A, 0x15, 0x3F, 0x00, 0x01, 0x20, 0x1F, 0x07, 0x30, 0x0C, 0x12, 0x29];
    let packed = Word72::from_six_bit_groups(groups);
    assert_eq!(packed.to_six_bit_groups(), groups);
}

#[test]
fn address_and_user_storage_bounds() {
    assert_eq!(TOTAL_MEM_WORDS, 1024);
    assert_eq!(USER_MEM_START, 300);
    assert_eq!(USER_MEM_END, 1013);
    assert_eq!(USER_MEM_LEN, 714);
    assert_eq!(INITIAL_PC, 300);
    assert_eq!(SPECIAL_R_CONTROL_ADDR, 25);

    assert_eq!(Address::new(0).unwrap().raw(), 0);
    assert_eq!(Address::new(1023).unwrap().raw(), 1023);
    assert!(matches!(Address::new(1024), Err(SpeedError::AddressOutOfBounds(1024))));

    assert!(Address::user(300).unwrap().is_user_accessible());
    assert!(Address::user(1013).unwrap().is_user_accessible());
    assert!(!Address::new(299).unwrap().is_user_accessible());
    assert!(!Address::new(1014).unwrap().is_user_accessible());

    assert!(matches!(Address::user(299), Err(SpeedError::ControlRegionAccess(299))));
    assert!(matches!(Address::user(1014), Err(SpeedError::ControlRegionAccess(1014))));
    assert!(matches!(Address::user(1024), Err(SpeedError::AddressOutOfBounds(1024))));
}

#[test]
fn speed_float_72bit_layout_and_normalization() {
    assert_eq!(FRACTION_BITS, 35);
    let zero = SpeedFloat::zero();
    assert!(zero.is_zero_fraction());
    assert!(zero.is_normalized());
    assert_eq!(zero.to_word().raw(), 0);

    // 1.5 = 0.75 * 2^1 -> fraction = 3 * 2^33 = 0x6_0000_0000, exp = +1.
    let val = SpeedFloat::from_f64(-1.5).unwrap();
    assert!(val.negative);
    assert!(!val.exp_negative);
    assert_eq!(val.signed_exponent(), 1);
    assert!(val.is_normalized());
    assert!(val.fraction >= (1u64 << 34));
    assert_eq!(val.fraction, 3u64 << 33);

    // Verify manual page 3-2 bit positions:
    // Bit 1: sign, Bits 2-36: 35-bit fraction, Bit 37: exp sign, Bits 38-54: exp, Bits 55-72: 0
    let word = val.to_word();
    assert!(word.bit(1));
    assert_eq!(word.bits(2, 36), val.fraction);
    assert!(!word.bit(37));
    assert_eq!(word.bits(38, 54), 1);
    assert_eq!(word.bits(55, 72), 0);
    assert_eq!(SpeedFloat::from_word(word), val);

    assert!(!val.abs().negative);
    assert!(!val.neg().negative);
    assert!((val.abs().to_f64() - 1.5).abs() < 1e-9);
}

#[test]
fn speed_float_binary_exponent_bounds() {
    assert_eq!(MAX_BINARY_EXPONENT, 131_071);

    let max_pos = SpeedFloat {
        negative: false,
        fraction: 1u64 << 34,
        exp_negative: false,
        exponent: MAX_BINARY_EXPONENT,
    };
    let roundtrip_pos = SpeedFloat::from_word(max_pos.to_word());
    assert_eq!(roundtrip_pos.signed_exponent(), 131_071);

    let max_neg = SpeedFloat {
        negative: true,
        fraction: 1u64 << 34,
        exp_negative: true,
        exponent: MAX_BINARY_EXPONENT,
    };
    let roundtrip_neg = SpeedFloat::from_word(max_neg.to_word());
    assert_eq!(roundtrip_neg.signed_exponent(), -131_071);
    assert!(roundtrip_neg.to_word().bit(37));
}

#[test]
#[allow(clippy::approx_constant)]
fn decimal_float_validation_and_roundtrip() {
    assert_eq!(MAX_DECIMAL_EXPONENT, 236);
    assert!(DecimalFloat::new(false, 5_000_000_000, 236).is_ok());
    assert!(DecimalFloat::new(true, 1_234_567_890, -236).is_ok());
    assert!(matches!(
        DecimalFloat::new(false, 5_000_000_000, 237),
        Err(SpeedError::DecimalExponentOutOfRange(237))
    ));
    assert!(matches!(
        DecimalFloat::new(false, 5_000_000_000, -237),
        Err(SpeedError::DecimalExponentOutOfRange(-237))
    ));

    let dec = DecimalFloat::new(false, 3_141_592_650, 1).unwrap(); // 3.141592650
    let sf = dec.to_speed_float().unwrap();
    assert!((sf.to_f64() - 3.14159265).abs() < 1e-8);
    let back = DecimalFloat::from_speed_float(sf).unwrap();
    assert_eq!(back.exponent, 1);
    assert!((back.digits as i64 - 3_141_592_650).abs() <= 2);

    // Binary exponent exceeding decimal +-236 range must fail decimal conversion.
    let huge_binary = SpeedFloat {
        negative: false,
        fraction: 1u64 << 34,
        exp_negative: false,
        exponent: 1000,
    };
    assert!(matches!(
        DecimalFloat::from_speed_float(huge_binary),
        Err(SpeedError::DecimalExponentOutOfRange(_))
    ));
}

#[test]
fn r_code_flags_and_bounds() {
    for raw in 0..=7u8 {
        let r = RCode::new(raw).unwrap();
        assert_eq!(r.raw(), raw);
        assert_eq!(r.modifies_a(), (raw & 0b100) != 0);
        assert_eq!(r.modifies_b(), (raw & 0b010) != 0);
        assert_eq!(r.modifies_c(), (raw & 0b001) != 0);
    }
    assert!(matches!(RCode::new(8), Err(SpeedError::InvalidRCode(8))));
}

#[test]
fn l_code_panel_switches_and_validation() {
    let l0 = LCode::new(0).unwrap();
    let l3 = LCode::new(3).unwrap();
    let l6 = LCode::new(6).unwrap();
    assert_eq!((l0.raw(), l0.switch_number()), (0, 1));
    assert_eq!((l3.raw(), l3.switch_number()), (3, 2));
    assert_eq!((l6.raw(), l6.switch_number()), (6, 3));

    for invalid in [1u8, 2, 4, 5, 7, 9] {
        assert!(matches!(LCode::new(invalid), Err(SpeedError::InvalidLCode(v)) if v == invalid));
    }
}

#[test]
fn op1_all_49_operations_and_skip_restrictions() {
    let expected: &[(Op1, &str, u16, bool)] = &[
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
    assert_eq!(expected.len(), 49);

    for &(op, mnem, code, forbids_op2) in expected {
        assert_eq!(op.mnemonic(), mnem);
        assert_eq!(op.code(), code);
        assert_eq!(op.forbids_op2(), forbids_op2);
        assert_eq!(Op1::from_code(code).unwrap(), op);
        assert_eq!(Op1::from_mnemonic(mnem).unwrap(), op);
    }

    assert!(matches!(Op1::from_code(999), Err(SpeedError::InvalidOp1Code(999))));
    assert!(matches!(Op1::from_mnemonic("BOGUS"), Err(SpeedError::InvalidMnemonic(_))));
}

#[test]
fn op2_all_48_operations_and_noop() {
    let expected: &[(Op2, &str, u16)] = &[
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
    assert_eq!(expected.len(), 48);

    for &(op, mnem, code) in expected {
        assert_eq!(op.mnemonic(), mnem);
        assert_eq!(op.code(), code);
        assert_eq!(Op2::from_code(code).unwrap(), op);
        assert_eq!(Op2::from_mnemonic(mnem).unwrap(), op);
    }
    assert_eq!(Op2::from_mnemonic("-").unwrap(), Op2::NoOp);
    assert!(matches!(Op2::from_code(255), Err(SpeedError::InvalidOp2Code(255))));
    assert!(matches!(Op2::from_mnemonic("NOPE"), Err(SpeedError::InvalidMnemonic(_))));
}

#[test]
fn instruction_72bit_layout_roundtrip() {
    // Manual page 4-4 layout:
    // 1: sign, 2: 0, 3-5: R, 6: 0, 7-18: OP1, 19-26: OP2, 27-36: D,
    // 37: 0, 38-47: A, 48-57: B, 58-59: 0, 60-62: L, 63-72: C
    let instr = Instruction {
        r_code: RCode::new(6).unwrap(),
        op1: Op1::Mpy,
        op2: Op2::SkRB,
        d: addr(20),
        a: addr(500),
        b: addr(900),
        c: addr(1001),
        l_code: LCode::new(3).unwrap(),
    };

    let word = instr.to_word();
    assert!(!word.bit(2));
    assert_eq!(word.bits(3, 5), 6);
    assert!(!word.bit(6));
    assert_eq!(word.bits(7, 18), Op1::Mpy.code() as u64);
    assert_eq!(word.bits(19, 26), Op2::SkRB.code() as u64);
    assert_eq!(word.bits(27, 36), 20);
    assert!(!word.bit(37));
    assert_eq!(word.bits(38, 47), 500);
    assert_eq!(word.bits(48, 57), 900);
    assert_eq!(word.bits(58, 59), 0);
    assert_eq!(word.bits(60, 62), 3);
    assert_eq!(word.bits(63, 72), 1001);

    assert_eq!(Instruction::from_word(word).unwrap(), instr);
}

#[test]
fn electrostatic_memory_user_access_and_control_protection() {
    let mut mem = ElectrostaticMemory::new();

    let cell = Address::user(300).unwrap();
    let val = SpeedFloat::from_f64(42.25).unwrap();
    mem.write_float(cell, val).unwrap();
    assert_eq!(mem.read_float(cell).unwrap(), val);

    let instr_cell = Address::user(1013).unwrap();
    let instr = Instruction {
        r_code: RCode::new(0).unwrap(),
        op1: Op1::Add,
        op2: Op2::Tr,
        d: addr(300),
        a: addr(400),
        b: addr(401),
        c: addr(402),
        l_code: LCode::new(0).unwrap(),
    };
    mem.write_instruction(instr_cell, instr).unwrap();
    assert_eq!(mem.read_instruction(instr_cell).unwrap(), instr);

    // Control regions 0..=299 and 1014..=1023 must not be writable or readable as user data.
    for protected in [0u16, 25, 299, 1014, 1023] {
        let p = addr(protected);
        assert!(matches!(
            mem.read(p),
            Err(SpeedError::ControlRegionAccess(v)) if v == protected
        ));
        assert!(matches!(
            mem.write(p, Word72::default()),
            Err(SpeedError::ControlRegionAccess(v)) if v == protected
        ));
    }
}

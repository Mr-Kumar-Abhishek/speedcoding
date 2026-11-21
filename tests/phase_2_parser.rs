//! Phase 2 TDD tests: `mpc`-based symbolic program parser and 80-column card formats
//! (SpeedCo I manual pages 7-1..7-4, 7-10..7-17, 9-5, 9-6, 10-2).

use mpc::{MpcResult, mpc_parse};
use speedcoding::core_types::{Op1, Op2, SpeedError};
use speedcoding::parser::{
    PunchedCard, StorageUnit, build_op1_mnemonic_parser, build_op2_mnemonic_parser,
    gang_punch_instruction_card, parse_card, parse_data_card, parse_instruction_card,
    parse_instruction_line, parse_loading_control_card, parse_program,
};

fn make_80col_card(segments: &[(usize, &str)]) -> String {
    let mut cols = vec![' '; 80];
    for &(col1, text) in segments {
        for (offset, ch) in text.chars().enumerate() {
            let idx = col1 - 1 + offset;
            if idx < 80 {
                cols[idx] = ch;
            }
        }
    }
    cols.into_iter().collect()
}

#[test]
fn mpc_op1_and_op2_mnemonic_parsers() {
    let op1_p = build_op1_mnemonic_parser();
    for mnem in ["ADD", "ABSUB", "NGMPY", "WRTPK", "RFTPK", "WRDRQ", "PRINT", "NOOP"] {
        assert!(
            matches!(mpc_parse("op1", mnem, &op1_p), MpcResult::Ok(_)),
            "expected OP1 parser to match {mnem}"
        );
    }
    assert!(matches!(mpc_parse("op1", "BADOP", &op1_p), MpcResult::Err(_)));

    let op2_p = build_op2_mnemonic_parser();
    for mnem in ["TR", "TRPL", "SNTRP", "TIAB", "TDABC", "SETRA", "SETRB", "RADDA", "ECHTR", "STOP"] {
        assert!(
            matches!(mpc_parse("op2", mnem, &op2_p), MpcResult::Ok(_)),
            "expected OP2 parser to match {mnem}"
        );
    }
    assert!(matches!(mpc_parse("op2", "INVALID", &op2_p), MpcResult::Err(_)));
}

#[test]
fn symbolic_instruction_line_examples() {
    // Manual page 9-5 line 0416
    let line = parse_instruction_line("0416 ADD 0 1013 1013 0945 SETRA 0000").unwrap();
    assert_eq!(line.location.raw(), 416);
    assert_eq!(line.instruction.op1, Op1::Add);
    assert_eq!(line.instruction.r_code.raw(), 0);
    assert_eq!(line.instruction.a.raw(), 1013);
    assert_eq!(line.instruction.b.raw(), 1013);
    assert_eq!(line.instruction.c.raw(), 945);
    assert_eq!(line.instruction.op2, Op2::SetRA);
    assert_eq!(line.instruction.d.raw(), 0);

    // Manual page 9-6 line 0354 with '-' for blank OP2/D and optional L-code
    let blank_op2 = parse_instruction_line("0354 ADD 0 1013 1013 1000 - - ; reset accum").unwrap();
    assert_eq!(blank_op2.location.raw(), 354);
    assert_eq!(blank_op2.instruction.op1, Op1::Add);
    assert_eq!(blank_op2.instruction.op2, Op2::NoOp);
    assert_eq!(blank_op2.instruction.l_code.raw(), 0);
    assert_eq!(blank_op2.remarks, "reset accum");
}

#[test]
fn multi_line_program_parsing_sum_and_check_loops() {
    // Manual page 9-5 sum of 34 numbers
    let prog = "\
        0416 ADD  0 1013 1013 0945 SETRA 0000\n\
        0417 ADD  4 0688 0945 0945 SKRA  0033\n\
        0418 NOOP 0 0000 0000 0000 TIA   0417\n";
    let parsed = parse_program(prog).unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[1].instruction.r_code.raw(), 4);
    assert_eq!(parsed[1].instruction.op2, Op2::SkRA);
    assert_eq!(parsed[2].instruction.op1, Op1::NoOp);
    assert_eq!(parsed[2].instruction.op2, Op2::TiA);

    // Manual page 10-2 checking loop
    let check_prog = "\
        0379 MPY  0 0500 0500 0400 STCH  0000\n\
        0380 MPY  0 0501 0501 0401 -     0000\n\
        0381 ADD  0 0400 0401 0402 -     0000\n\
        0382 SQRT 0 0402 0000 0403 ECHTR 0379\n\
        0383 NOOP 0 0000 0000 0000 STOP  0379\n";
    let check_parsed = parse_program(check_prog).unwrap();
    assert_eq!(check_parsed.len(), 5);
    assert_eq!(check_parsed[0].instruction.op2, Op2::StCh);
    assert_eq!(check_parsed[3].instruction.op1, Op1::Sqrt);
    assert_eq!(check_parsed[3].instruction.op2, Op2::EchTr);
    assert_eq!(check_parsed[4].instruction.op2, Op2::Stop);
}

#[test]
fn instruction_card_80col_fields_and_fixed_zeros() {
    // Manual page 7-2 instruction card columns:
    // 8: not double-punched 0+1, 9: blank, 10-13: location, 14: 0, 15-19: alpha OP1,
    // 20: R, 21-24: A, 25-28: B, 29-32: C, 33-34: 00, 35-39: alpha OP2, 40-43: D,
    // 44: L, 45: unused, 46-48: num OP1, 49-51: num OP2, 52-55: 0000, 56-80: remarks
    let card_str = make_80col_card(&[
        (1, "MAIN01 "),
        (8, "0"),
        (9, " "),
        (10, "0519"),
        (14, "0"),
        (15, "ABSUB"),
        (20, "0"),
        (21, "0684"),
        (25, "0832"),
        (29, "0712"),
        (33, "00"),
        (35, "TRPL "),
        (40, "0498"),
        (44, "3"),
        (45, "0"),
        (46, "711"),
        (49, "109"),
        (52, "0000"),
        (56, "TEST BRANCH"),
    ]);

    let card = parse_instruction_card(&card_str).unwrap();
    assert_eq!(card.location.raw(), 519);
    assert_eq!(card.alpha_op1.as_deref(), Some("ABSUB"));
    assert_eq!(card.alpha_op2.as_deref(), Some("TRPL"));
    assert_eq!(card.numeric_op1, Some(711));
    assert_eq!(card.numeric_op2, Some(109));
    assert_eq!(card.l_code.raw(), 3);
    assert_eq!(card.remarks.trim(), "TEST BRANCH");

    // Non-blank column 9 or missing mandatory zeros in col 14/33-34/52-55 must fail.
    let mut bad_col9 = card_str.clone();
    bad_col9.replace_range(8..9, "8");
    assert!(matches!(parse_instruction_card(&bad_col9), Err(SpeedError::InvalidCardFormat(_))));

    let mut bad_col14 = card_str;
    bad_col14.replace_range(13..14, "1");
    assert!(matches!(parse_instruction_card(&bad_col14), Err(SpeedError::InvalidCardFormat(_))));
}

#[test]
fn alpha_only_instruction_card_requires_gang_punch() {
    // Manual page 7-3: 701 cannot execute alphabetical-only cards before gang-punching.
    let alpha_only = make_80col_card(&[
        (8, "0"),
        (9, " "),
        (10, "0487"),
        (14, "0"),
        (15, "NOOP "),
        (20, "0"),
        (21, "0000"),
        (25, "0000"),
        (29, "0000"),
        (33, "00"),
        (35, "TR   "),
        (40, "0811"),
        (44, "0"),
        (45, "0"),
        (46, "   "),
        (49, "   "),
        (52, "0000"),
    ]);

    let card = parse_instruction_card(&alpha_only).unwrap();
    assert_eq!(card.numeric_op1, None);
    assert!(matches!(card.to_machine_instruction(), Err(SpeedError::InvalidCardFormat(_))));

    let punched = gang_punch_instruction_card(&card).unwrap();
    assert_eq!(punched.numeric_op1, Some(751));
    assert_eq!(punched.numeric_op2, Some(104));
    let instr = punched.to_machine_instruction().unwrap();
    assert_eq!(instr.op1, Op1::NoOp);
    assert_eq!(instr.op2, Op2::Tr);
    assert_eq!(instr.d.raw(), 811);
}

#[test]
fn numeric_only_instruction_card_loads_directly() {
    // Manual page 7-3: alphabetical cols 15-19 and 35-39 blank, numerical cols 46-51 populated.
    let numeric_only = make_80col_card(&[
        (8, "0"),
        (9, " "),
        (10, "0300"),
        (14, "0"),
        (15, "     "),
        (20, "0"),
        (21, "0600"),
        (25, "0601"),
        (29, "0602"),
        (33, "00"),
        (35, "     "),
        (40, "0000"),
        (44, "0"),
        (45, "0"),
        (46, "658"),
        (49, "000"),
        (52, "0000"),
    ]);

    let card = parse_instruction_card(&numeric_only).unwrap();
    let instr = card.to_machine_instruction().unwrap();
    assert_eq!(instr.op1, Op1::Add);
    assert_eq!(instr.op2, Op2::NoOp);
    assert_eq!((instr.a.raw(), instr.b.raw(), instr.c.raw()), (600, 601, 602));
}

#[test]
fn data_card_80col_five_values_col45_skip_and_overpunch_signs() {
    // Manual page 7-4:
    // Col 8: double-punched '0'+'1' (represented with '+'/'*' marker or punch code, e.g. 'Z' or '&'),
    // Col 9: '8', Col 10-13: Location,
    // Col 14-23 N1, 24-26 n1, 27-36 N2, 37-39 n2,
    // Col 40-44 + 46-50 N3 (col 45 skipped!), 51-53 n3,
    // Col 54-63 N4, 64-66 n4, 67-76 N5, 77-79 n5, Col 80: word count 1..=5.
    // Standard IBM 026 overpunch on trailing digit: 'A'..'I' / '{' (+0..+9, 12-zone) and 'J'..'R' / '}' (-1..-9/-0, 11-zone).
    let card_str = make_80col_card(&[
        (1, "DATA001"),
        (8, "*"), // double-punch 0+1 marker for ASCII card representation
        (9, "8"),
        (10, "0688"),
        (14, "500000000A"), // +0.5000000001 -> '+' overpunch 1 ('A')
        (24, "00A"),        // +1
        (27, "250000000J"), // -0.2500000001 -> '-' overpunch 1 ('J')
        (37, "00K"),        // -2
        (40, "12345"),      // cols 40-44 (first half of N3)
        (45, "X"),          // col 45 ignored!
        (46, "6789{"),      // cols 46-50 (second half of N3: +1234567890)
        (51, "00{"),        // +0
        (54, "000000000{"),
        (64, "00{"),
        (67, "000000000{"),
        (77, "00{"),
        (80, "3"),
    ]);

    let card = parse_data_card(&card_str).unwrap();
    assert_eq!(card.first_location.raw(), 688);
    assert_eq!(card.word_count, 3);
    assert_eq!(card.values.len(), 3);
    assert_eq!((card.values[0].negative, card.values[0].digits, card.values[0].exponent), (false, 5_000_000_001, 1));
    assert_eq!((card.values[1].negative, card.values[1].digits, card.values[1].exponent), (true, 2_500_000_001, -2));
    assert_eq!((card.values[2].negative, card.values[2].digits, card.values[2].exponent), (false, 1_234_567_890, 0));

    assert!(matches!(parse_card(&card_str).unwrap(), PunchedCard::Data(_)));
}

#[test]
fn data_card_validation_rejects_bad_exponent_and_word_count() {
    let bad_exp = make_80col_card(&[
        (8, "*"),
        (9, "8"),
        (10, "0500"),
        (14, "500000000{"),
        (24, "23G"), // +237 -> exceeds 236!
        (27, "00000000000000000000000000000000000000000000000000000"),
        (80, "1"),
    ]);
    assert!(matches!(parse_data_card(&bad_exp), Err(SpeedError::DecimalExponentOutOfRange(237))));

    let bad_wc = make_80col_card(&[
        (8, "*"),
        (9, "8"),
        (10, "0500"),
        (14, "500000000{"),
        (24, "00{"),
        (27, "00000000000000000000000000000000000000000000000000000"),
        (80, "6"),
    ]);
    assert!(matches!(parse_data_card(&bad_wc), Err(SpeedError::InvalidCardFormat(_))));
}

#[test]
fn loading_control_card_tape_and_drum_formats() {
    // Manual page 7-17: Col 9 = '9', 10 = '0', 11-14 = first, 15 = '0',
    // 16-19 = last, 20 = '0', 21-24 = drum addr (0000 for tape), 25-28 = '0000', 29 = unit
    let tape_ctrl = make_80col_card(&[
        (8, "0"),
        (9, "9"),
        (10, "0"),
        (11, "0300"),
        (15, "0"),
        (16, "0450"),
        (20, "0"),
        (21, "0000"),
        (25, "0000"),
        (29, "K"),
    ]);
    let parsed_tape = parse_loading_control_card(&tape_ctrl).unwrap();
    assert_eq!(parsed_tape.first_addr.raw(), 300);
    assert_eq!(parsed_tape.last_addr.raw(), 450);
    assert_eq!(parsed_tape.drum_addr.raw(), 0);
    assert_eq!(parsed_tape.unit, StorageUnit::TapeK);
    assert!(matches!(parse_card(&tape_ctrl).unwrap(), PunchedCard::LoadingControl(_)));

    let drum_ctrl = make_80col_card(&[
        (8, "0"),
        (9, "9"),
        (10, "0"),
        (11, "0947"),
        (15, "0"),
        (16, "1013"),
        (20, "0"),
        (21, "0032"),
        (25, "0000"),
        (29, "Q"),
    ]);
    let parsed_drum = parse_loading_control_card(&drum_ctrl).unwrap();
    assert_eq!(parsed_drum.drum_addr.raw(), 32);
    assert_eq!(parsed_drum.unit, StorageUnit::DrumQ);
}

#[test]
fn forbids_op2_with_skip_capable_op1() {
    // Manual pages 7-10, 7-11, 7-12, 7-15 prohibit non-blank OP2 on RFTP*, RBTP*, SFTP*, SBTP*, WRDR*.
    for line in [
        "0362 RFTPJ 0 0900 0919 0000 TR 0355",
        "0365 RBTPJ 0 0900 0919 0000 STOP 0362",
        "0400 SFTPK 0 0000 0000 0000 TIA 0401",
        "0401 SBTPM 0 0000 0000 0000 SETRA 0001",
        "0402 WRDRP 0 0300 0350 0010 TRPL 0500",
        "0403 WRDRQ 0 0300 0350 0010 SKIP 0005",
    ] {
        assert!(
            matches!(
                parse_instruction_line(line),
                Err(SpeedError::ForbiddenOp2WithSkipOp1 { .. })
            ),
            "expected ForbiddenOp2WithSkipOp1 for `{line}`"
        );
    }

    // RFDRP / RFDRQ do NOT have skip features and may pair with OP2.
    assert!(parse_instruction_line("0404 RFDRQ 0 0811 0820 0041 TR 0350").is_ok());
}

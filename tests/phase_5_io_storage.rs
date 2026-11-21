//! Phase 5 TDD tests: Magnetic tapes (J..M), magnetic drums (P..Q), line printer,
//! and card reader deck loading (SpeedCo I manual section 7).

use speedcoding::control::SpeedCoMachine;
use speedcoding::core_types::{Address, MachineStop, SpeedError, SpeedFloat, Word72};
use speedcoding::io::{
    DRUM_CAPACITY, DrumId, DrumUnit, DrumWriteOutcome, LinePrinter, PRINTER_VALUES_PER_LINE,
    TapeId, TapeReadOutcome, TapeSkipOutcome, TapeStatus, TapeUnit,
};
use speedcoding::parser::{PunchedCard, parse_card};

fn words(vals: &[u128]) -> Vec<Word72> {
    vals.iter().map(|&v| Word72::new(v).unwrap()).collect()
}

fn addr(raw: u16) -> Address {
    Address::new(raw).unwrap()
}

#[test]
fn tape_odd_parity_redundancy_channel() {
    // Manual page 7-5: 7th channel ensures an odd number of 1 bits across all 7 channels.
    for group in 0..64u8 {
        let p = TapeUnit::compute_odd_parity_bit(group);
        assert!(p <= 1);
        let total_ones = group.count_ones() + u32::from(p);
        assert_eq!(total_ones % 2, 1, "group={group:#08b} parity={p}");
    }
}

#[test]
fn tape_status_transition_matrix_pages_7_13_and_7_14() {
    let rec = words(&[1, 2, 3]);

    // Neutral + Write -> Write; Write + Read/Skip -> CopyCheck stop (remains Write).
    let mut t = TapeUnit::new(TapeId::J);
    assert_eq!(t.status(), TapeStatus::Neutral);
    assert_eq!(t.write_record(&rec).unwrap(), None);
    assert_eq!(t.status(), TapeStatus::Write);
    assert_eq!(t.read_forward(3).unwrap(), TapeReadOutcome::Stop(MachineStop::CopyCheck));
    assert_eq!(t.read_backward(3).unwrap(), TapeReadOutcome::Stop(MachineStop::CopyCheck));
    assert_eq!(t.skip_forward().unwrap(), TapeSkipOutcome::Stop(MachineStop::CopyCheck));
    assert_eq!(t.skip_backward().unwrap(), TapeSkipOutcome::Stop(MachineStop::CopyCheck));
    assert_eq!(t.status(), TapeStatus::Write);

    // Write + EndFile -> Neutral.
    t.end_file().unwrap();
    assert_eq!(t.status(), TapeStatus::Neutral);

    // Rewind -> Read backward/forward allowed; Read + Write -> CopyCheck stop (remains Read).
    t.rewind();
    assert_eq!(t.status(), TapeStatus::Neutral);
    assert!(matches!(t.read_forward(3).unwrap(), TapeReadOutcome::Ok(_)));
    assert_eq!(t.status(), TapeStatus::Read);
    assert_eq!(t.write_record(&rec).unwrap(), Some(MachineStop::CopyCheck));
    assert_eq!(t.status(), TapeStatus::Read);

    // Read + EndFile -> ignored without stop, stays Read (page 7-14).
    t.end_file().unwrap();
    assert_eq!(t.status(), TapeStatus::Read);

    // Read + Rewind -> Neutral.
    t.rewind();
    assert_eq!(t.status(), TapeStatus::Neutral);
    assert!(t.is_rewound());
}

#[test]
fn tape_write_end_file_rewind_and_full_or_partial_read() {
    let mut t = TapeUnit::new(TapeId::K);
    let r1 = words(&[10, 20, 30, 40, 50]);
    let r2 = words(&[100, 200]);

    assert_eq!(t.write_record(&r1).unwrap(), None);
    assert_eq!(t.write_record(&r2).unwrap(), None);
    assert_eq!(t.record_count(), 2);
    t.end_file().unwrap();
    t.rewind();
    assert_eq!(t.position(), 0);

    // Partial read (3 words of 5-word record) still advances tape to end-of-record gap (page 7-9).
    assert_eq!(t.read_forward(3).unwrap(), TapeReadOutcome::Ok(r1[..3].to_vec()));
    assert_eq!(t.position(), 1);
    assert_eq!(t.read_forward(2).unwrap(), TapeReadOutcome::Ok(r2.clone()));
    assert_eq!(t.position(), 2);
}

#[test]
fn tape_oversized_read_triggers_copy_check_stop() {
    let mut t = TapeUnit::new(TapeId::L);
    t.write_record(&words(&[1, 2, 3])).unwrap();
    t.end_file().unwrap();
    t.rewind();

    // Requesting B - A + 1 = 4 words from a 3-word record causes CopyCheck (page 7-9).
    assert_eq!(t.read_forward(4).unwrap(), TapeReadOutcome::Stop(MachineStop::CopyCheck));
}

#[test]
fn tape_eof_skip_checksum_skip_and_parity_stop() {
    let mut t = TapeUnit::new(TapeId::M);
    t.write_record(&words(&[1, 2])).unwrap();
    t.write_record(&words(&[3, 4])).unwrap();
    t.write_record(&words(&[5, 6])).unwrap();
    t.end_file().unwrap();
    t.rewind();

    t.inject_checksum_error(0);
    t.inject_parity_error(1);

    // Record 0 has checksum discrepancy -> skips next 2 instructions.
    assert_eq!(t.read_forward(2).unwrap(), TapeReadOutcome::ChecksumMismatchSkip);
    // Record 1 has parity discrepancy -> TapeCheck machine stop.
    assert_eq!(t.read_forward(2).unwrap(), TapeReadOutcome::Stop(MachineStop::TapeCheck));
    // Record 2 succeeds, then reading at end-of-file yields 1-instruction EndOfFileSkip.
    assert_eq!(t.read_forward(2).unwrap(), TapeReadOutcome::Ok(words(&[5, 6])));
    assert_eq!(t.read_forward(2).unwrap(), TapeReadOutcome::EndOfFileSkip);
}

#[test]
fn tape_backward_read_and_skip_worked_example_page_7_12() {
    // Manual page 7-12: 5 records on tape K; after reading 5th backward, tape is between 4th & 5th.
    // Two SBTPK calls move to gap between 2nd & 3rd; then RBTPK reads 2nd record and stops between 1st & 2nd.
    let mut t = TapeUnit::new(TapeId::K);
    for i in 1..=5u128 {
        t.write_record(&words(&[i * 100])).unwrap();
    }
    t.end_file().unwrap();
    assert_eq!(t.position(), 5);

    assert_eq!(t.read_backward(1).unwrap(), TapeReadOutcome::Ok(words(&[500])));
    assert_eq!(t.position(), 4);

    assert_eq!(t.skip_backward().unwrap(), TapeSkipOutcome::Ok);
    assert_eq!(t.skip_backward().unwrap(), TapeSkipOutcome::Ok);
    assert_eq!(t.position(), 2);

    assert_eq!(t.read_backward(1).unwrap(), TapeReadOutcome::Ok(words(&[200])));
    assert_eq!(t.position(), 1);

    assert_eq!(t.skip_forward().unwrap(), TapeSkipOutcome::Ok);
    assert_eq!(t.position(), 2);
}

#[test]
fn tape_read_backward_error_does_not_final_backspace() {
    // Manual page 7-11: on parity, checksum, or copy-check error during RBTP's read-forward phase,
    // final backspace does NOT happen — tape stops at the gap following the record read.
    let mut t = TapeUnit::new(TapeId::J);
    t.write_record(&words(&[11, 22])).unwrap();
    t.write_record(&words(&[33, 44])).unwrap();
    t.end_file().unwrap();
    t.inject_checksum_error(1);

    // Currently at position 2; RBTP backs up to 1, reads forward (advances to 2), sees checksum error, stays at 2.
    assert_eq!(t.read_backward(2).unwrap(), TapeReadOutcome::ChecksumMismatchSkip);
    assert_eq!(t.position(), 2);
}

#[test]
fn drum_write_read_worked_example_pages_7_15_and_7_16() {
    assert_eq!(DRUM_CAPACITY, 1024);
    let mut drum_q = DrumUnit::new(DrumId::Q);

    // Page 7-15: 68 words (cells 947..=1014) written starting at drum address 32 -> 32..=99.
    let block: Vec<Word72> = (1..=68u128).map(|v| Word72::new(v).unwrap()).collect();
    assert_eq!(drum_q.write_block(addr(32), &block).unwrap(), DrumWriteOutcome::Ok);

    // Page 7-16: read 10 words starting at drum address 41 -> 10th..=19th words (10..=19).
    let slice = drum_q.read_block(addr(41), 10).unwrap();
    assert_eq!(slice.len(), 10);
    assert_eq!(slice[0].raw(), 10);
    assert_eq!(slice[9].raw(), 19);
}

#[test]
fn drum_single_word_verify_error_skip_and_bounds_no_wrap() {
    let mut drum_p = DrumUnit::new(DrumId::P);
    drum_p.inject_next_write_verify_error();
    assert_eq!(
        drum_p.write_block(addr(500), &words(&[77])).unwrap(),
        DrumWriteOutcome::VerifyDiscrepancySkip
    );

    // Page 7-16: cannot wrap past drum address 1023 (e.g., 7 words starting at 1022).
    assert!(matches!(
        drum_p.write_block(addr(1022), &words(&[1, 2, 3, 4, 5, 6, 7])),
        Err(SpeedError::DrumAddressOutOfRange { start: 1022, end: 1028 })
    ));
    assert!(matches!(
        drum_p.read_block(addr(1022), 7),
        Err(SpeedError::DrumAddressOutOfRange { start: 1022, end: 1028 })
    ));
}

#[test]
fn line_printer_page_7_19_worked_example_and_exponent_limit() {
    assert_eq!(PRINTER_VALUES_PER_LINE, 5);
    let mut printer = LinePrinter::new();

    // Page 7-19: cells 461..=473 (13 values), C = 0407 -> lines 40701 (5), 40702 (5), 40703 (3).
    let vals: Vec<SpeedFloat> = (1..=13).map(|i| SpeedFloat::from_f64(i as f64).unwrap()).collect();
    let lines = printer.print_block(&vals, 407).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!((lines[0].id, lines[0].values.len()), (40701, 5));
    assert_eq!((lines[1].id, lines[1].values.len()), (40702, 5));
    assert_eq!((lines[2].id, lines[2].values.len()), (40703, 3));
    assert_eq!(printer.lines().len(), 3);

    // Manual page 7-19: |decimal exponent| > 236 cannot be printed.
    let overflow = SpeedFloat {
        negative: false,
        fraction: 1u64 << 34,
        exp_negative: false,
        exponent: 1000,
    };
    assert!(matches!(
        printer.print_block(&[overflow], 1),
        Err(SpeedError::DecimalExponentOutOfRange(_))
    ));
}

#[test]
fn card_reader_load_deck_rewinds_tapes_and_offloads_blocks() {
    // Manual page 7-18: loading rewinds all 4 tapes; control cards offload blocks to tape/drum
    // leaving tape-written units in Write status, and execution starts at PC = 300.
    let mut machine = SpeedCoMachine::new();
    machine.tape_mut(TapeId::L).write_record(&words(&[999])).unwrap();

    let mut data_card = vec![' '; 80];
    data_card[7] = '*';
    data_card[8] = '8';
    for (i, ch) in "0350500000000{00{".chars().enumerate() {
        data_card[9 + i] = ch;
    }
    for col in &mut data_card[26..79] {
        *col = '0';
    }
    data_card[79] = '1';
    let data_str: String = data_card.into_iter().collect();

    let mut ctrl_card = vec![' '; 80];
    ctrl_card[7] = '0';
    ctrl_card[8] = '9';
    for (i, ch) in "00350003500000000000J".chars().enumerate() {
        ctrl_card[9 + i] = ch;
    }
    let ctrl_str: String = ctrl_card.into_iter().collect();

    let mut instr_card = vec![' '; 80];
    for (i, ch) in "003000NOOP 000000000000000STOP 0300007511230000".chars().enumerate() {
        instr_card[7 + i] = ch;
    }
    let instr_str: String = instr_card.into_iter().collect();

    let cards: Vec<PunchedCard> = vec![
        parse_card(&data_str).unwrap(),
        parse_card(&ctrl_str).unwrap(),
        parse_card(&instr_str).unwrap(),
    ];

    machine.load_deck(&cards).unwrap();
    assert_eq!(machine.pc().raw(), 300);
    assert!(machine.tape(TapeId::L).is_rewound());
    assert_eq!(machine.tape(TapeId::L).status(), TapeStatus::Neutral);
    assert_eq!(machine.tape(TapeId::J).status(), TapeStatus::Write);
    assert_eq!(machine.tape(TapeId::J).record_count(), 1);
}

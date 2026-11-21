//! Phase 6 TDD tests: Checking loops (`PRCH`/`STCH`/`ECHTR`), selective listing (`L`-code),
//! and analytical execution/tape/loading timing (SpeedCo I manual sections 10-12).

use speedcoding::control::SpeedCoMachine;
use speedcoding::core_types::{Address, LCode, MachineStop, Op1, Op2, RCode, SpeedError, SpeedFloat, Word72};
use speedcoding::diagnostics::{
    CheckingPass, CheckingState, EchTrOutcome, LISTING_BLOCK_SIZE, LISTING_TIME_AVG_SECS,
    LISTING_TIME_MAX_SECS, ListedValue, ListingBuffer, ListingEntry, ListingSwitches,
    TapeCondition, TimingCalculator, TimingContext,
};
use speedcoding::parser::parse_program;

fn addr(raw: u16) -> Address {
    Address::new(raw).unwrap()
}

fn uaddr(raw: u16) -> Address {
    Address::user(raw).unwrap()
}

fn sf(val: f64) -> SpeedFloat {
    SpeedFloat::from_f64(val).unwrap()
}

fn dummy_listing_entry(loc: u16, l: u8) -> ListingEntry {
    ListingEntry {
        location: uaddr(loc),
        op1: Op1::Add,
        r_code: RCode::new(0).unwrap(),
        a: addr(500),
        b: addr(501),
        c: addr(502),
        op2: Op2::NoOp,
        d: addr(0),
        l_code: LCode::new(l).unwrap(),
        address_counter: 0,
        r_a: 0,
        r_b: 0,
        r_c: 0,
        a_val: ListedValue::Blank,
        b_val: ListedValue::Blank,
        c_val: ListedValue::Blank,
    }
}

#[test]
fn checking_protocol_state_machine_rules_page_10_4() {
    let mut chk = CheckingState::new();
    let w1 = Word72::new(100).unwrap();
    let w2 = Word72::new(200).unwrap();

    // Rule 2: STCH without prior PRCH or ECHTR is invalid.
    assert!(matches!(
        chk.start_check(w1),
        Err(SpeedError::CheckingProtocolViolation(_))
    ));

    // Rule 1: ECHTR without prior STCH is invalid.
    chk.prepare_check();
    assert!(matches!(
        chk.end_check_and_transfer(w1, uaddr(300)),
        Err(SpeedError::CheckingProtocolViolation(_))
    ));

    // Valid pass 1 -> RepeatAt -> pass 2 -> VerifiedSkipNext.
    chk.start_check(w1).unwrap();
    assert_eq!(chk.pass(), CheckingPass::Pass1);
    chk.record_computed_result(w2);
    assert_eq!(
        chk.end_check_and_transfer(w1, uaddr(379)).unwrap(),
        EchTrOutcome::RepeatAt(uaddr(379))
    );

    chk.start_check(w1).unwrap();
    assert!(chk.is_second_pass());
    chk.record_computed_result(w2);
    assert_eq!(
        chk.end_check_and_transfer(w1, uaddr(379)).unwrap(),
        EchTrOutcome::VerifiedSkipNext
    );
    assert_eq!(chk.counter_1(), chk.counter_2());
    assert_eq!(chk.pass(), CheckingPass::Inactive);
}

#[test]
fn checking_loop_pass_1_and_pass_2_worked_example_page_10_2() {
    // Manual page 10-2: PRCH at 0378, loop 0379..=0382 computes sqrt(x^2 + y^2) twice,
    // skips 0383 STOP, and halts at 0384 STOP.
    let mut machine = SpeedCoMachine::new();
    machine.memory_mut().write_float(uaddr(500), sf(3.0)).unwrap();
    machine.memory_mut().write_float(uaddr(501), sf(4.0)).unwrap();

    for line in parse_program(
        "\
        0378 NOOP 0 0000 0000 0000 PRCH  0000\n\
        0379 MPY  0 0500 0500 0400 STCH  0000\n\
        0380 MPY  0 0501 0501 0401 -     0000\n\
        0381 ADD  0 0400 0401 0402 -     0000\n\
        0382 SQRT 0 0402 0000 0403 ECHTR 0379\n\
        0383 NOOP 0 0000 0000 0000 STOP  0379\n\
        0384 NOOP 0 0000 0000 0000 STOP  0384\n",
    )
    .unwrap()
    {
        machine
            .memory_mut()
            .write_instruction(line.location, line.instruction)
            .unwrap();
    }

    machine.set_pc(uaddr(378)).unwrap();
    let stop = machine.run_until_stop(30).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(384) });
    assert!((machine.memory().read_float(uaddr(403)).unwrap().to_f64() - 5.0).abs() < 1e-8);
}

#[test]
fn checking_loop_discrepancy_does_not_skip_and_print_suppressed_on_pass2() {
    let mut chk = CheckingState::new();
    chk.prepare_check();
    chk.start_check(Word72::new(10).unwrap()).unwrap();
    assert_eq!(
        chk.end_check_and_transfer(Word72::new(20).unwrap(), uaddr(300)).unwrap(),
        EchTrOutcome::RepeatAt(uaddr(300))
    );

    // Second pass computes a different result -> ECHTR does not skip next instruction.
    chk.start_check(Word72::new(10).unwrap()).unwrap();
    assert_eq!(
        chk.end_check_and_transfer(Word72::new(21).unwrap(), uaddr(300)).unwrap(),
        EchTrOutcome::DiscrepancyContinue
    );

    // Page 10-2 & 12-2: PRINT inside checking loop executes in ~2.896 ms without printing on pass 2.
    let mut machine = SpeedCoMachine::new();
    machine.memory_mut().write_float(uaddr(461), sf(7.0)).unwrap();
    for line in parse_program(
        "\
        0300 NOOP  0 0000 0000 0000 PRCH  0000\n\
        0301 PRINT 0 0461 0461 0100 STCH  0000\n\
        0302 ADD   0 0461 0461 0462 ECHTR 0301\n\
        0303 NOOP  0 0000 0000 0000 STOP  0999\n\
        0304 NOOP  0 0000 0000 0000 STOP  0304\n",
    )
    .unwrap()
    {
        machine
            .memory_mut()
            .write_instruction(line.location, line.instruction)
            .unwrap();
    }
    let stop = machine.run_until_stop(20).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(304) });
    assert_eq!(machine.printer().lines().len(), 1); // Printed only once (pass 1), suppressed on pass 2.
}

#[test]
fn checking_loop_inter_pass_initialization_not_summed_page_10_3() {
    // Manual page 10-4: calculations performed after ECHTR and before STCH do NOT enter check counters.
    let mut chk = CheckingState::new();
    chk.prepare_check();
    chk.start_check(Word72::new(42).unwrap()).unwrap();
    assert_eq!(
        chk.end_check_and_transfer(Word72::new(8).unwrap(), uaddr(483)).unwrap(),
        EchTrOutcome::RepeatAt(uaddr(483))
    );

    // Inter-pass computation outside checking loop must be ignored.
    chk.record_computed_result(Word72::new(999_999).unwrap());

    chk.start_check(Word72::new(42).unwrap()).unwrap();
    assert_eq!(
        chk.end_check_and_transfer(Word72::new(8).unwrap(), uaddr(483)).unwrap(),
        EchTrOutcome::VerifiedSkipNext
    );
}

#[test]
fn selective_listing_switches_10_line_drum_flush_and_pass2_suppression() {
    assert_eq!(LISTING_BLOCK_SIZE, 10);

    let switches = ListingSwitches {
        switch_1_l0: true,
        switch_2_l3: false,
        switch_3_l6: true,
    };
    assert!(switches.should_list(LCode::new(0).unwrap()));
    assert!(!switches.should_list(LCode::new(3).unwrap()));
    assert!(switches.should_list(LCode::new(6).unwrap()));

    let mut buf = ListingBuffer::new();
    // L=3 filtered out and pass 2 suppressed.
    assert!(buf.record(dummy_listing_entry(300, 3), switches, false).is_none());
    assert!(buf.record(dummy_listing_entry(301, 0), switches, true).is_none());
    assert_eq!(buf.pending_count(), 0);

    // Accumulates 9 lines, flushes on 10th.
    for i in 0..9u16 {
        assert!(buf.record(dummy_listing_entry(310 + i, 0), switches, false).is_none());
        assert_eq!(buf.pending_count(), usize::from(i + 1));
    }
    let flushed = buf.record(dummy_listing_entry(319, 6), switches, false).unwrap();
    assert_eq!(flushed.len(), 10);
    assert_eq!(buf.pending_count(), 0);
    assert_eq!(buf.flushed_blocks().len(), 1);
}

#[test]
fn listing_operand_sign_exceptions_and_exponent_overflow_page_11_2() {
    let a = Some(sf(10.0));
    let neg_a = Some(sf(-10.0));
    let b = Some(sf(-3.0));
    let c = Some(sf(13.0));

    // SUB lists -Q(B) instead of Q(B) (page 11-1).
    let (la, lb, _lc) = ListingBuffer::format_listed_operands(Op1::Sub, a, b, c);
    assert!(matches!(la, ListedValue::Decimal(d) if !d.negative));
    assert!(matches!(lb, ListedValue::Decimal(d) if !d.negative));

    // ABSUB lists |Q(A)| and -|Q(B)|.
    let (la2, lb2, _) = ListingBuffer::format_listed_operands(Op1::AbSub, neg_a, b, c);
    assert!(matches!(la2, ListedValue::Decimal(d) if !d.negative));
    assert!(matches!(lb2, ListedValue::Decimal(d) if d.negative));

    // NGMPY lists -Q(A).
    let (la3, _, _) = ListingBuffer::format_listed_operands(Op1::NgMpy, a, b, c);
    assert!(matches!(la3, ListedValue::Decimal(d) if d.negative));

    // Page 11-2: |decimal_exp| > 236 lists fraction 0 and +-999 exponent.
    let huge_pos = SpeedFloat {
        negative: false,
        fraction: 1u64 << 34,
        exp_negative: false,
        exponent: 1000,
    };
    let huge_neg = SpeedFloat {
        negative: false,
        fraction: 1u64 << 34,
        exp_negative: true,
        exponent: 1000,
    };
    let (ov_a, ov_b, _) =
        ListingBuffer::format_listed_operands(Op1::Add, Some(huge_pos), Some(huge_neg), None);
    assert_eq!(ov_a, ListedValue::ExponentOverflow { positive: true });
    assert_eq!(ov_b, ListedValue::ExponentOverflow { positive: false });
}

#[test]
fn listing_captures_state_after_op1_before_op2() {
    // Manual page 11-1: listing line shows state AFTER OP1 and BEFORE OP2.
    let mut machine = SpeedCoMachine::new();
    machine.switches_mut().listing.switch_1_l0 = true;
    machine.memory_mut().write_float(uaddr(500), sf(2.0)).unwrap();
    machine.memory_mut().write_float(uaddr(501), sf(5.0)).unwrap();
    machine.r_quantities_mut().r_a = 4;

    let mut lines = Vec::new();
    for i in 0..10u16 {
        if i == 0 {
            lines.push("0300 ADD 0 0500 0501 0502 TIA 0301".to_string());
        } else if i < 9 {
            lines.push(format!("0{} NOOP 0 0000 0000 0000 - 0000", 300 + i));
        } else {
            lines.push("0309 NOOP 0 0000 0000 0000 STOP 0309".to_string());
        }
    }
    for parsed in parse_program(&lines.join("\n")).unwrap() {
        machine
            .memory_mut()
            .write_instruction(parsed.location, parsed.instruction)
            .unwrap();
    }

    machine.run_until_stop(20).unwrap();
    let block = &machine.listing().flushed_blocks()[0];
    // At instruction 0300, C already holds 7.0 (post-OP1), while R_A is still 4 (pre-TIA!).
    assert_eq!(block[0].r_a, 4);
    assert!(matches!(block[0].c_val, ListedValue::Decimal(d) if !d.negative && d.exponent == 1));
    assert_eq!(machine.r_quantities().r_a, 5);
}

#[test]
fn timing_op1_arithmetic_math_io_and_print_pass2() {
    let default_ctx = TimingContext::default();
    assert_eq!(
        (
            TimingCalculator::op1_time_ms(Op1::Add, default_ctx).avg_ms,
            TimingCalculator::op1_time_ms(Op1::Add, default_ctx).max_ms
        ),
        (4.200, 19.272)
    );
    assert_eq!(
        TimingCalculator::op1_time_ms(Op1::Mpy, default_ctx).avg_ms,
        3.546
    );
    assert_eq!(
        TimingCalculator::op1_time_ms(Op1::NoOp, default_ctx).avg_ms,
        2.736
    );
    assert_eq!(
        TimingCalculator::op1_time_ms(Op1::Sqrt, default_ctx).max_ms,
        9.016
    );
    assert_eq!(
        TimingCalculator::op1_time_ms(Op1::Ln, default_ctx).avg_ms,
        33.0
    );

    let wrtp_ctx = TimingContext {
        word_count: 10,
        tape_condition: Some(TapeCondition::Cond1AfterWrite),
        ..Default::default()
    };
    let wrtp = TimingCalculator::op1_time_ms(Op1::WrTpK, wrtp_ctx);
    assert!((wrtp.avg_ms - 30.000).abs() < 1e-6);
    assert!((wrtp.max_ms - 35.000).abs() < 1e-6);

    // PRINT: 13 words -> ceil(13/5) = 3 lines -> avg 1663 + 400*3 = 2863 ms; pass 2 = 2.896 ms.
    let print_p1 = TimingCalculator::op1_time_ms(
        Op1::Print,
        TimingContext {
            word_count: 13,
            is_check_pass_2: false,
            ..Default::default()
        },
    );
    assert!((print_p1.avg_ms - 2863.0).abs() < 1e-6);

    let print_p2 = TimingCalculator::op1_time_ms(
        Op1::Print,
        TimingContext {
            word_count: 13,
            is_check_pass_2: true,
            ..Default::default()
        },
    );
    assert_eq!((print_p2.avg_ms, print_p2.max_ms), (2.896, 2.896));
}

#[test]
fn timing_op2_and_additive_instruction_time() {
    assert_eq!(TimingCalculator::op2_time_ms(Op2::NoOp, false), 0.000);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::Tr, true), 0.768);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::TrPl, true), 0.876);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::TrPl, false), 0.648);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::TiA, true), 2.256);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::TdB, true), 3.036);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::EchTr, true), 1.368);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::EchTr, false), 1.260);
    assert_eq!(TimingCalculator::op2_time_ms(Op2::Stop, true), 0.816);

    let combined = TimingCalculator::instruction_time_ms(
        Op1::Add,
        Op2::TiA,
        TimingContext::default(),
    );
    assert!((combined.avg_ms - (4.200 + 2.256)).abs() < 1e-9);
    assert!((combined.max_ms - (19.272 + 2.256)).abs() < 1e-9);
}

#[test]
fn timing_tape_length_rewind_worked_example_eof_and_skip_delay() {
    // Manual page 12-3 worked example:
    // 100 unit records of 10 words each: 2*72 + 100*(10 + 0.5)*3/25 + 99*1 = 369 inches;
    // rewind time = 369 / 75 = 4.920 seconds.
    let records = vec![10usize; 100];
    assert!((TimingCalculator::tape_file_length_inches(&records) - 369.0).abs() < 1e-9);
    assert!((TimingCalculator::tape_rewind_time_secs(&records) - 4.920).abs() < 1e-9);
    assert!((TimingCalculator::tape_eof_gap_time_secs() - 0.96).abs() < 1e-9);

    // Note 1 (page 12-2): (1.600 * n) - t when positive, else 0.
    assert!((TimingCalculator::tape_skip_delay_ms(10, 5.0) - 11.0).abs() < 1e-9);
    assert_eq!(TimingCalculator::tape_skip_delay_ms(10, 20.0), 0.0);
}

#[test]
fn timing_listing_constants_and_card_loading_formula_page_12_5() {
    assert_eq!(LISTING_TIME_AVG_SECS, 1.325);
    assert_eq!(LISTING_TIME_MAX_SECS, 1.370);

    // Manual page 12-5: (0.4)(n) + (0.6)(c) + e, where e = 2 (on drums) or 55 (not on drums).
    assert!((TimingCalculator::loading_time_secs(100, 5, true) - 45.0).abs() < 1e-9);
    assert!((TimingCalculator::loading_time_secs(100, 5, false) - 98.0).abs() < 1e-9);
}

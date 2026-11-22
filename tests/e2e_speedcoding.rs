//! End-to-end IBM 701 Speedcoding System (SpeedCo I) integration scenarios.

use speedcoding::control::SpeedCoMachine;
use speedcoding::core_types::{Address, LCode, MachineStop, Op1, Op2, SpeedFloat};
use speedcoding::diagnostics::{TimingCalculator, TimingContext};
use speedcoding::io::DrumId;
use speedcoding::parser::{PunchedCard, gang_punch_instruction_card, parse_card, parse_program};

fn addr(raw: u16) -> Address {
    Address::new(raw).unwrap()
}

fn uaddr(raw: u16) -> Address {
    Address::user(raw).unwrap()
}

fn sf(val: f64) -> SpeedFloat {
    SpeedFloat::from_f64(val).unwrap()
}

struct AlphaCardSpec<'a> {
    loc: u16,
    op1: &'a str,
    r: u8,
    a: u16,
    b: u16,
    c: u16,
    op2: &'a str,
    d: u16,
}

fn encode_alpha_instruction_card(spec: AlphaCardSpec<'_>) -> PunchedCard {
    let mut card = vec![' '; 80];
    let body = format!(
        "0 {:04}0{:<5}{}{:04}{:04}{:04}00{:<5}{:04}0       0000",
        spec.loc, spec.op1, spec.r, spec.a, spec.b, spec.c, spec.op2, spec.d
    );
    for (i, ch) in body.chars().enumerate() {
        card[7 + i] = ch;
    }
    let raw: String = card.into_iter().collect();
    let parsed = match parse_card(&raw).unwrap() {
        PunchedCard::Instruction(ic) => ic,
        other => panic!("expected instruction card, got {other:?}"),
    };
    PunchedCard::Instruction(gang_punch_instruction_card(&parsed).unwrap())
}

#[test]
fn e2e_page_9_6_matrix_multiplication_worked_example() {
    // Multiply A (2x3 row-major at 500..=505) * B (3x2 col-major at 600..=605) -> C (2x2 at 700..=703).
    // Combines R-quantity indexing (SETRA/B/C, TIAB, TIC, SKRB, SKRC) with AC address modification
    // (RADDA, ADDA, STA, SUBA, SKIP) inspired by SpeedCo I manual page 9-6.
    let mut machine = SpeedCoMachine::new();
    let mem = machine.memory_mut();

    for (offset, val) in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0].into_iter().enumerate() {
        mem.write_float(uaddr(500 + offset as u16), sf(val)).unwrap();
    }
    for (offset, val) in [7.0, 9.0, 11.0, 8.0, 10.0, 12.0].into_iter().enumerate() {
        mem.write_float(uaddr(600 + offset as u16), sf(val)).unwrap();
    }
    mem.write_float(uaddr(400), sf(0.0)).unwrap();

    let program = "\
        0402 NOOP 0 0003 0000 0000 -     0000\n\
        0403 NOOP 0 0506 0000 0000 -     0000\n\
        0300 NOOP 0 0000 0000 0000 SETRC 0000\n\
        0301 NOOP 0 0000 0000 0000 SETRB 0000\n\
        0302 ADD  0 0400 0400 0404 SETRA 0000\n\
        0303 MPY  6 0500 0600 0401 -     0000\n\
        0304 ADD  0 0404 0401 0404 -     0000\n\
        0305 NOOP 0 0000 0000 0000 SKRA  0002\n\
        0306 NOOP 0 0000 0000 0000 TIAB  0303\n\
        0307 ADD  1 0400 0404 0700 TIC   0308\n\
        0308 NOOP 0 0000 0000 0000 SKRB  0005\n\
        0309 NOOP 0 0000 0000 0000 TIB   0302\n\
        0310 NOOP 0 0000 0000 0000 RADDA 0303\n\
        0311 NOOP 0 0000 0000 0000 ADDA  0402\n\
        0312 NOOP 0 0000 0000 0000 STA   0303\n\
        0313 NOOP 0 0000 0000 0000 SUBA  0403\n\
        0314 NOOP 0 0000 0000 0000 SKIP  0000\n\
        0315 NOOP 0 0000 0000 0000 TR    0301\n\
        0316 NOOP 0 0000 0000 0000 STOP  0317\n";

    for line in parse_program(program).unwrap() {
        machine
            .memory_mut()
            .write_instruction(line.location, line.instruction)
            .unwrap();
    }

    let stop = machine.run_until_stop(200).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(317) });

    let expected = [(700, 58.0), (701, 64.0), (702, 139.0), (703, 154.0)];
    for (cell, want) in expected {
        let got = machine.memory().read_float(uaddr(cell)).unwrap().to_f64();
        assert!((got - want).abs() < 1e-8, "C[{cell}] got {got}, expected {want}");
    }
}

#[test]
fn e2e_punched_card_deck_checking_loop_and_line_printer() {
    let mut machine = SpeedCoMachine::new();

    // Data card seeding 3.0 and 4.0 into cells 0500..=0501.
    let mut data_card = vec![' '; 80];
    data_card[7] = '*';
    data_card[8] = '8';
    for (i, ch) in "0500300000000{00A400000000{00A".chars().enumerate() {
        data_card[9 + i] = ch;
    }
    data_card[39..79].fill('0');
    data_card[79] = '2';
    let data_str: String = data_card.into_iter().collect();

    // Loading control card offloading 0500..=0501 onto Drum P starting at drum address 0010.
    let mut ctrl_card = vec![' '; 80];
    ctrl_card[7] = '0';
    ctrl_card[8] = '9';
    for (i, ch) in "0050000501000100000P".chars().enumerate() {
        ctrl_card[9 + i] = ch;
    }
    let ctrl_str: String = ctrl_card.into_iter().collect();

    let cards = vec![
        parse_card(&data_str).unwrap(),
        parse_card(&ctrl_str).unwrap(),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 300, op1: "NOOP", r: 0, a: 0, b: 0, c: 0, op2: "PRCH", d: 0 }),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 301, op1: "MPY", r: 0, a: 500, b: 500, c: 400, op2: "STCH", d: 0 }),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 302, op1: "MPY", r: 0, a: 501, b: 501, c: 401, op2: "-", d: 0 }),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 303, op1: "ADD", r: 0, a: 400, b: 401, c: 402, op2: "-", d: 0 }),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 304, op1: "SQRT", r: 0, a: 402, b: 0, c: 403, op2: "-", d: 0 }),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 305, op1: "PRINT", r: 0, a: 403, b: 403, c: 205, op2: "ECHTR", d: 301 }),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 306, op1: "NOOP", r: 0, a: 0, b: 0, c: 0, op2: "STOP", d: 999 }),
        encode_alpha_instruction_card(AlphaCardSpec { loc: 307, op1: "NOOP", r: 0, a: 0, b: 0, c: 0, op2: "STOP", d: 307 }),
    ];

    machine.load_deck(&cards).unwrap();
    let drum_copy = machine.drum(DrumId::P).read_block(addr(10), 2).unwrap();
    assert_eq!(drum_copy, vec![sf(3.0).to_word(), sf(4.0).to_word()]);

    let stop = machine.run_until_stop(40).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(307) });
    assert!((machine.memory().read_float(uaddr(403)).unwrap().to_f64() - 5.0).abs() < 1e-8);
    assert_eq!(machine.printer().lines().len(), 1);
    assert_eq!(machine.printer().lines()[0].id, 20501);
    assert_eq!(machine.restart_after_stop().unwrap(), uaddr(307));
}

#[test]
fn e2e_tape_and_drum_machine_instructions_roundtrip() {
    let mut machine = SpeedCoMachine::new();
    for (idx, val) in [11.25, -22.5, 33.75].into_iter().enumerate() {
        machine
            .memory_mut()
            .write_float(uaddr(500 + idx as u16), sf(val))
            .unwrap();
    }

    let program = "\
        0300 WRTPJ 0 0500 0502 0000 -     0000\n\
        0301 EFTPJ 0 0000 0000 0000 -     0000\n\
        0302 RWTPJ 0 0000 0000 0000 -     0000\n\
        0303 RFTPJ 0 0600 0602 0000 -     0000\n\
        0304 RFTPJ 0 0610 0612 0000 -     0000\n\
        0305 NOOP  0 0000 0000 0000 STOP  0999\n\
        0306 WRDRQ 0 0600 0602 0100 -     0000\n\
        0307 RFDRQ 0 0700 0702 0100 STOP  0307\n";

    for line in parse_program(program).unwrap() {
        machine
            .memory_mut()
            .write_instruction(line.location, line.instruction)
            .unwrap();
    }

    let stop = machine.run_until_stop(30).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(307) });

    for idx in 0..3u16 {
        let src = machine.memory().read(uaddr(500 + idx)).unwrap();
        let dst = machine.memory().read(uaddr(700 + idx)).unwrap();
        assert_eq!(dst, src);
    }
}

#[test]
fn e2e_timing_and_selective_listing_full_diagnostic_profile() {
    let mut machine = SpeedCoMachine::new();
    machine.switches_mut().listing.switch_1_l0 = true;
    machine.memory_mut().write_float(uaddr(500), sf(9.0)).unwrap();
    machine.memory_mut().write_float(uaddr(501), sf(16.0)).unwrap();

    let mut lines = vec![
        "0300 ADD  0 0500 0501 0502 -    0000".to_string(),
        "0301 SQRT 0 0502 0000 0503 -    0000".to_string(),
    ];
    for loc in 302..=308u16 {
        lines.push(format!("0{loc} NOOP 0 0000 0000 0000 - 0000"));
    }
    lines.push("0309 NOOP 0 0000 0000 0000 STOP 0309".to_string());

    for parsed in parse_program(&lines.join("\n")).unwrap() {
        machine
            .memory_mut()
            .write_instruction(parsed.location, parsed.instruction)
            .unwrap();
    }

    let stop = machine.run_until_stop(20).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(309) });
    assert_eq!(machine.listing().flushed_blocks().len(), 1);
    let block = &machine.listing().flushed_blocks()[0];
    assert_eq!(block.len(), 10);
    assert_eq!(block[0].l_code, LCode::new(0).unwrap());

    let add_time = TimingCalculator::instruction_time_ms(Op1::Add, Op2::NoOp, TimingContext::default());
    let sqrt_time =
        TimingCalculator::instruction_time_ms(Op1::Sqrt, Op2::NoOp, TimingContext::default());
    assert!((add_time.avg_ms - 4.200).abs() < 1e-9);
    assert!((sqrt_time.avg_ms - 8.200).abs() < 1e-9);
    assert!((TimingCalculator::loading_time_secs(12, 1, true) - 7.4).abs() < 1e-9);
}

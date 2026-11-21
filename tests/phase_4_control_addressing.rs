//! Phase 4 TDD tests: Program counter, OP2 transfers, R-quantities, address counter,
//! location 0025 manipulation, and SpeedCo I worked examples (manual sections 8-9).

use speedcoding::control::{AC_MAX, AC_MIN, AddressCounter, RQuantities, SpeedCoMachine, StepOutcome};
use speedcoding::core_types::{
    Address, Instruction, LCode, MachineStop, Op1, Op2, RCode, SPECIAL_R_CONTROL_ADDR, SpeedError,
    SpeedFloat,
};
use speedcoding::parser::parse_program;

fn addr(raw: u16) -> Address {
    Address::new(raw).unwrap()
}

fn uaddr(raw: u16) -> Address {
    Address::user(raw).unwrap()
}

fn load_symbolic(machine: &mut SpeedCoMachine, source: &str) {
    for line in parse_program(source).unwrap() {
        machine
            .memory_mut()
            .write_instruction(line.location, line.instruction)
            .unwrap();
    }
}

fn sf(v: f64) -> SpeedFloat {
    SpeedFloat::from_f64(v).unwrap()
}

#[test]
fn initial_pc_is_300_and_op1_precedes_op2() {
    let mut machine = SpeedCoMachine::new();
    assert_eq!(machine.pc().raw(), 300);

    machine.memory_mut().write_float(uaddr(500), sf(2.0)).unwrap();
    machine.memory_mut().write_float(uaddr(501), sf(3.0)).unwrap();

    // OP1 computes +5.0 at C=502, then OP2 (TRPL) sees Q(502) > 0 and branches to 400.
    load_symbolic(
        &mut machine,
        "0300 ADD 0 0500 0501 0502 TRPL 0400",
    );

    let outcome = machine.step().unwrap();
    assert_eq!(outcome, StepOutcome::Continued { next_pc: uaddr(400) });
    assert_eq!(machine.pc().raw(), 400);
    assert!((machine.memory().read_float(uaddr(502)).unwrap().to_f64() - 5.0).abs() < 1e-9);
}

#[test]
fn unconditional_tr_and_conditional_trpl_trmn_trz() {
    let mut machine = SpeedCoMachine::new();
    machine.memory_mut().write_float(uaddr(600), sf(-4.0)).unwrap();
    machine.memory_mut().write_float(uaddr(601), sf(1.0)).unwrap();
    machine.memory_mut().write_float(uaddr(602), sf(2.0)).unwrap();

    load_symbolic(
        &mut machine,
        "\
        0300 ADD 0 0600 0601 0610 TRPL 0310\n\
        0301 ADD 0 0600 0601 0610 TRMN 0305\n\
        0305 SUB 0 0602 0602 0611 TRZ  0320\n\
        0320 NOOP 0 0000 0000 0000 TR  0350\n",
    );

    // 0300: -3.0 is negative -> TRPL does not branch -> PC=301
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(301) });
    // 0301: -3.0 is negative -> TRMN branches -> PC=305
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(305) });
    // 0305: 2 - 2 has zero fractional part -> TRZ branches -> PC=320
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(320) });
    // 0320: TR -> PC=350
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(350) });
}

#[test]
fn two_instruction_ge_zero_transfer_idiom() {
    // Manual page 8-2/8-3: TRZ followed by TRPL transfers to g iff x - y >= 0.
    for (x, y, expected_next) in [(5.0, 5.0, 400), (8.0, 3.0, 400), (2.0, 7.0, 302)] {
        let mut machine = SpeedCoMachine::new();
        machine.memory_mut().write_float(uaddr(500), sf(x)).unwrap();
        machine.memory_mut().write_float(uaddr(501), sf(y)).unwrap();
        load_symbolic(
            &mut machine,
            "\
            0300 SUB 0 0500 0501 0502 TRZ  0400\n\
            0301 SUB 0 0500 0501 0502 TRPL 0400\n",
        );

        let s1 = machine.step().unwrap();
        let final_pc = match s1 {
            StepOutcome::Continued { next_pc } if next_pc.raw() == 400 => next_pc.raw(),
            _ => match machine.step().unwrap() {
                StepOutcome::Continued { next_pc } => next_pc.raw(),
                other => panic!("unexpected outcome: {other:?}"),
            },
        };
        assert_eq!(final_pc, expected_next);
    }
}

#[test]
fn sense_and_transfer_p_q_switches() {
    let mut machine = SpeedCoMachine::new();
    load_symbolic(
        &mut machine,
        "\
        0300 NOOP 0 0000 0000 0000 SNTRP 0310\n\
        0301 NOOP 0 0000 0000 0000 SNTRQ 0320\n",
    );

    // Switches off -> no transfers.
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(301) });
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(302) });

    // Switches on -> transfers occur.
    machine.set_pc(uaddr(300)).unwrap();
    machine.switches_mut().p_switch_4 = true;
    machine.switches_mut().q_switch_5 = true;
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(310) });
    machine.set_pc(uaddr(301)).unwrap();
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(320) });
}

#[test]
fn stop_and_transfer_halts_and_restarts_at_d() {
    let mut machine = SpeedCoMachine::new();
    load_symbolic(
        &mut machine,
        "\
        0300 NOOP 0 0000 0000 0000 STOP 0362\n\
        0362 NOOP 0 0000 0000 0000 STOP 0400\n",
    );

    let stop1 = machine.step().unwrap();
    assert_eq!(stop1, StepOutcome::Stopped(MachineStop::ProgramStop { restart_addr: uaddr(362) }));
    let resumed_pc = machine.restart_after_stop().unwrap();
    assert_eq!(resumed_pc.raw(), 362);
    assert_eq!(machine.pc().raw(), 362);

    let stop2 = machine.step().unwrap();
    assert_eq!(stop2, StepOutcome::Stopped(MachineStop::ProgramStop { restart_addr: uaddr(400) }));
}

#[test]
fn r_code_non_destructive_effective_addresses_page_9_2() {
    // Manual page 9-2 table: ADD 0410 0563 0741 with R_A=7, R_B=19, R_C=104
    let r = RQuantities { r_a: 7, r_b: 19, r_c: 104 };
    let expected = [
        (0, (410, 563, 741)),
        (1, (410, 563, 845)),
        (2, (410, 582, 741)),
        (3, (410, 582, 845)),
        (4, (417, 563, 741)),
        (5, (417, 563, 845)),
        (6, (417, 582, 741)),
        (7, (417, 582, 845)),
    ];
    for (r_code, (ea, eb, ec)) in expected {
        let (a, b, c) = r
            .effective_addresses(RCode::new(r_code).unwrap(), addr(410), addr(563), addr(741))
            .unwrap();
        assert_eq!((a.raw(), b.raw(), c.raw()), (ea, eb, ec));
    }

    // Stored instruction in electrostatic memory must remain unmodified after R-code execution.
    let mut machine = SpeedCoMachine::new();
    machine.memory_mut().write_float(uaddr(417), sf(10.0)).unwrap();
    machine.memory_mut().write_float(uaddr(582), sf(20.0)).unwrap();
    *machine.r_quantities_mut() = r;
    load_symbolic(&mut machine, "0300 ADD 7 0410 0563 0741 - 0000");

    let stored_before = machine.memory().read_instruction(uaddr(300)).unwrap();
    machine.step().unwrap();
    let stored_after = machine.memory().read_instruction(uaddr(300)).unwrap();
    assert_eq!(stored_before, stored_after);
    assert!((machine.memory().read_float(uaddr(845)).unwrap().to_f64() - 30.0).abs() < 1e-9);
}

#[test]
fn ti_td_setr_skr_and_underflow_stop() {
    let mut machine = SpeedCoMachine::new();
    load_symbolic(
        &mut machine,
        "\
        0300 NOOP 0 0000 0000 0000 SETRA 0005\n\
        0301 NOOP 0 0000 0000 0000 SETRB 0010\n\
        0302 NOOP 0 0000 0000 0000 SETRC 0001\n\
        0303 NOOP 0 0000 0000 0000 TIABC 0304\n\
        0304 NOOP 0 0000 0000 0000 SKRA  0006\n\
        0305 NOOP 0 0000 0000 0000 STOP  0999\n\
        0306 NOOP 0 0000 0000 0000 TDABC 0307\n\
        0307 NOOP 0 0000 0000 0000 TDC   0308\n\
        0308 NOOP 0 0000 0000 0000 TDC   0309\n",
    );

    for _ in 0..4 {
        machine.step().unwrap();
    }
    assert_eq!(machine.r_quantities(), RQuantities { r_a: 6, r_b: 11, r_c: 2 });

    // 0304: SKRA 0006 skips 0305 and lands at 0306.
    assert_eq!(machine.step().unwrap(), StepOutcome::Continued { next_pc: uaddr(306) });
    // 0306: TDABC -> (5, 10, 1)
    machine.step().unwrap();
    assert_eq!(machine.r_quantities(), RQuantities { r_a: 5, r_b: 10, r_c: 1 });
    // 0307: TDC -> (5, 10, 0)
    machine.step().unwrap();
    assert_eq!(machine.r_quantities().r_c, 0);
    // 0308: TDC when R_C == 0 -> automatic error stop (page 9-4).
    assert_eq!(machine.step().unwrap(), StepOutcome::Stopped(MachineStop::RQuantityUnderflow));
}

#[test]
fn worked_example_sum_34_numbers_page_9_5() {
    let mut machine = SpeedCoMachine::new();
    machine.memory_mut().write_float(uaddr(1013), SpeedFloat::zero()).unwrap();
    for i in 0..34u16 {
        machine
            .memory_mut()
            .write_float(uaddr(688 + i), sf((i + 1) as f64))
            .unwrap();
    }

    load_symbolic(
        &mut machine,
        "\
        0416 ADD  0 1013 1013 0945 SETRA 0000\n\
        0417 ADD  4 0688 0945 0945 SKRA  0033\n\
        0418 NOOP 0 0000 0000 0000 TIA   0417\n\
        0419 NOOP 0 0000 0000 0000 STOP  0419\n",
    );

    machine.set_pc(uaddr(416)).unwrap();
    let stop = machine.run_until_stop(200).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(419) });

    // Sum 1..=34 = 34 * 35 / 2 = 595
    let sum = machine.memory().read_float(uaddr(945)).unwrap().to_f64();
    assert!((sum - 595.0).abs() < 1e-7);
    assert_eq!(machine.r_quantities().r_a, 33);
}

#[test]
fn address_counter_modulo_1024_table_page_9_8() {
    assert_eq!(AC_MIN, -131_071);
    assert_eq!(AC_MAX, 131_071);

    // Manual page 9-8 table:
    for (ac_val, expected_stored) in [
        (308, 308u16),
        (-1003, 1003),
        (1348, 324),
        (-1888, 864),
        (2348, 300),
    ] {
        let ac = AddressCounter::new(ac_val).unwrap();
        assert_eq!(ac.value(), ac_val);
        assert_eq!(ac.stored_address().raw(), expected_stored);
    }

    assert!(matches!(
        AddressCounter::new(131_072),
        Err(SpeedError::BinaryExponentOutOfRange(_))
    ));
}

#[test]
fn address_counter_radd_add_sub_st_skip_and_page_9_10_example() {
    // Manual page 9-10: modify D fields of instructions 0355 and 0358 using 0428..=0431.
    let mut machine = SpeedCoMachine::new();
    load_symbolic(
        &mut machine,
        "\
        0355 MPY  6 0500 0900 1001 SKRB 0020\n\
        0358 ADD  0 1013 1013 1000 SKRC 0017\n\
        0428 NOOP 0 0017 0006 0000 RADDA 0428\n\
        0429 NOOP 0 0000 0000 0000 STD   0355\n\
        0430 NOOP 0 0000 0000 0000 SUBB  0428\n\
        0431 NOOP 0 0000 0000 0000 STD   0358\n\
        0432 NOOP 0 0000 0000 0000 SKIP  0011\n\
        0433 NOOP 0 0000 0000 0000 STOP  0999\n\
        0434 NOOP 0 0000 0000 0000 STOP  0434\n",
    );

    machine.set_pc(uaddr(428)).unwrap();
    let stop = machine.run_until_stop(20).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(434) });
    assert_eq!(machine.address_counter().value(), 11);
    assert_eq!(machine.memory().read_instruction(uaddr(355)).unwrap().d.raw(), 17);
    assert_eq!(machine.memory().read_instruction(uaddr(358)).unwrap().d.raw(), 11);
}

#[test]
fn combined_ac_and_r_quantities_via_addr_0025_page_9_12() {
    // Manual pages 9-11 & 9-12: RADDA 0025 saves R_A=800 into AC, STD 0600 patches SETRA 0000,
    // SETRA 0000 resets R_A=0, then executing 0600 restores R_A=800.
    let mut machine = SpeedCoMachine::new();
    machine.r_quantities_mut().r_a = 800;

    let radd_25 = Instruction {
        r_code: RCode::new(0).unwrap(),
        op1: Op1::NoOp,
        op2: Op2::RAddA,
        d: addr(SPECIAL_R_CONTROL_ADDR),
        a: addr(0),
        b: addr(0),
        c: addr(0),
        l_code: LCode::new(0).unwrap(),
    };
    machine.memory_mut().write_instruction(uaddr(500), radd_25).unwrap();

    load_symbolic(
        &mut machine,
        "\
        0501 NOOP 0 0000 0000 0000 STD   0600\n\
        0502 NOOP 0 0000 0000 0000 SETRA 0000\n\
        0503 NOOP 0 0000 0000 0000 TR    0600\n\
        0600 NOOP 0 0000 0000 0000 SETRA 0000\n\
        0601 NOOP 0 0000 0000 0000 STOP  0601\n",
    );

    machine.set_pc(uaddr(500)).unwrap();
    machine.step().unwrap(); // 0500: RADDA 0025 -> AC = 800
    assert_eq!(machine.address_counter().value(), 800);
    machine.step().unwrap(); // 0501: STD 0600
    assert_eq!(machine.memory().read_instruction(uaddr(600)).unwrap().d.raw(), 800);
    machine.step().unwrap(); // 0502: SETRA 0000 -> R_A = 0
    assert_eq!(machine.r_quantities().r_a, 0);

    let stop = machine.run_until_stop(10).unwrap();
    assert_eq!(stop, MachineStop::ProgramStop { restart_addr: uaddr(601) });
    assert_eq!(machine.r_quantities().r_a, 800);
}

#[test]
fn control_region_transfer_and_effective_address_rejected() {
    // Manual page 4-2 & 9-10: transferring to 0..=299 or 1014..=1023 is always an error.
    let mut machine = SpeedCoMachine::new();
    load_symbolic(&mut machine, "0300 NOOP 0 0000 0000 0000 TR 0250");
    assert!(matches!(machine.step(), Err(SpeedError::ControlRegionAccess(250))));

    // R-code pushing effective address above 1013 must also error.
    let r = RQuantities { r_a: 20, r_b: 0, r_c: 0 };
    assert!(matches!(
        r.effective_addresses(RCode::new(4).unwrap(), addr(1000), addr(300), addr(300)),
        Err(SpeedError::ControlRegionAccess(1020))
    ));
}

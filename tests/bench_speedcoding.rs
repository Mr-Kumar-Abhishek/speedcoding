//! SpeedCo I + `mpc-rs` parsing performance benchmark suite.

use mpc::{
    MpcAst, MpcInput, MpcParser, MpcResult, mpc_and, mpc_char, mpc_count, mpc_digit, mpc_eoi,
    mpc_many, mpc_many1, mpc_newline, mpc_noneof, mpc_oneof, mpc_or, mpc_parse, mpc_range,
    mpc_string, mpca_root, mpca_tag, mpcf_fold_ast, mpcf_fst, mpcf_null, mpcf_strfold,
};
use speedcoding::core_types::USER_MEM_LEN;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

// Longer prefixes must precede shorter prefixes in ordered `mpc_or` choices.
const OP1_CYCLE: [&str; 11] = [
    "ADDAB", "ABSUB", "WRTPK", "PRINT", "SINE", "SQRT", "NOOP", "ADD", "SUB", "MPY", "DIV",
];

const OP2_CYCLE: [&str; 11] = [
    "ECHTR", "SETRA", "RADDA", "TIAB", "TRPL", "TRMN", "SKRA", "TIA", "STD", "TR", "-",
];

fn report_bench(label: &str, items: usize, unit: &str, bytes: usize, elapsed: Duration) {
    let secs = elapsed.as_secs_f64();
    let items_per_sec = if secs > 0.0 {
        items as f64 / secs
    } else {
        f64::INFINITY
    };
    let mb_per_sec = if secs > 0.0 {
        (bytes as f64 / (1024.0 * 1024.0)) / secs
    } else {
        f64::INFINITY
    };
    println!(
        "[bench] {label}: {items} {unit} ({bytes} bytes) in {ms:.2} ms ({items_per_sec:.0} {unit}/s, {mb_per_sec:.2} MB/s)",
        ms = secs * 1000.0
    );
}

fn gen_symbolic_lines(count: usize) -> String {
    let mut out = String::with_capacity(count * 44);
    for i in 0..count {
        let loc = 300 + (i % USER_MEM_LEN);
        let op1 = OP1_CYCLE[i % OP1_CYCLE.len()];
        let op2 = OP2_CYCLE[i % OP2_CYCLE.len()];
        let r = i % 8;
        let a = 300 + ((i * 3) % USER_MEM_LEN);
        let b = 300 + ((i * 7) % USER_MEM_LEN);
        let c = 300 + ((i * 11) % USER_MEM_LEN);
        let d = 300 + ((i * 5) % USER_MEM_LEN);
        let _ = writeln!(out, "{loc:04} {op1} {r} {a:04} {b:04} {c:04} {op2} {d:04}");
    }
    out
}

fn build_symbolic_program_parser() -> MpcParser {
    let sp = mpc_many1(mpc_char(' '), mpcf_null);
    let addr4 = mpc_count(4, mpc_digit(), mpcf_null);
    let op1 = mpc_or(OP1_CYCLE.iter().map(|m| mpc_string(m)).collect());
    let op2 = mpc_or(OP2_CYCLE.iter().map(|m| mpc_string(m)).collect());
    let line = mpc_and(
        vec![
            addr4.clone(),
            sp.clone(),
            op1,
            sp.clone(),
            mpc_range('0', '7'),
            sp.clone(),
            addr4.clone(),
            sp.clone(),
            addr4.clone(),
            sp.clone(),
            addr4.clone(),
            sp.clone(),
            op2,
            sp,
            addr4,
            mpc_newline(),
        ],
        mpcf_null,
    );
    mpc_and(vec![mpc_many(line, mpcf_null), mpc_eoi()], mpcf_null)
}

fn build_symbolic_ast_parser() -> MpcParser {
    let sp = mpc_many1(mpc_char(' '), mpcf_null);
    let addr4 = mpc_count(4, mpc_digit(), mpcf_strfold);
    let op1 = mpca_tag(
        mpc_or(OP1_CYCLE.iter().map(|m| mpc_string(m)).collect()),
        "op1",
    );
    let op2 = mpca_tag(
        mpc_or(OP2_CYCLE.iter().map(|m| mpc_string(m)).collect()),
        "op2",
    );
    let instr = mpca_tag(
        mpc_and(
            vec![
                mpca_tag(addr4.clone(), "loc"),
                sp.clone(),
                op1,
                sp.clone(),
                mpca_tag(mpc_range('0', '7'), "r"),
                sp.clone(),
                mpca_tag(addr4.clone(), "a"),
                sp.clone(),
                mpca_tag(addr4.clone(), "b"),
                sp.clone(),
                mpca_tag(addr4.clone(), "c"),
                sp.clone(),
                op2,
                sp,
                mpca_tag(addr4, "d"),
                mpc_newline(),
            ],
            mpcf_fold_ast,
        ),
        "instr",
    );
    mpca_root(mpca_tag(
        mpc_and(vec![mpc_many(instr, mpcf_fold_ast), mpc_eoi()], mpcf_fst),
        "speedco_program",
    ))
}

#[test]
fn bench_parse_714_line_full_memory_program() {
    let data = gen_symbolic_lines(USER_MEM_LEN);
    let parser = build_symbolic_program_parser();

    let mut input = MpcInput::new("full_mem_714.sc", &data);
    let start = Instant::now();
    let res = parser.parse(&mut input);
    let elapsed = start.elapsed();

    assert!(matches!(res, MpcResult::Ok(_)), "expected 714-line parse to succeed");
    assert_eq!(input.state.row, USER_MEM_LEN as i64);
    assert_eq!(input.pos, data.len());
    report_bench("speedco_714_full_memory", USER_MEM_LEN, "lines", data.len(), elapsed);

    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_secs(1),
        "release 714-line parse exceeded 1s: {elapsed:?}"
    );
}

#[test]
fn bench_parse_2000_line_speedco_program_ast() {
    let data = gen_symbolic_lines(2000);
    let parser = build_symbolic_ast_parser();

    let mut input = MpcInput::new("program_2000.sc", &data);
    let start = Instant::now();
    let res = parser.parse(&mut input);
    let elapsed = start.elapsed();

    match res {
        MpcResult::Ok(val) => {
            let ast = val.downcast_ref::<MpcAst>().expect("expected MpcAst");
            assert_eq!(ast.tag, ">|speedco_program");
            assert_eq!(ast.children_num, 2000);
        }
        MpcResult::Err(err) => panic!("unexpected err: {err}"),
    }
    assert_eq!(input.state.row, 2000);
    assert_eq!(input.pos, data.len());
    report_bench("speedco_2000_ast", 2000, "lines", data.len(), elapsed);

    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_secs(1),
        "release 2000-line AST parse exceeded 1s: {elapsed:?}"
    );
}

#[test]
fn bench_parse_2000_instruction_cards_80col() {
    let mut data = String::with_capacity(2000 * 81);
    for i in 0..2000 {
        let loc = 300 + (i % USER_MEM_LEN);
        let alpha_op1 = match i % 4 {
            0 => "ADD  ",
            1 => "MPY  ",
            2 => "ABSUB",
            _ => "NOOP ",
        };
        let (num_op1, alpha_op2, num_op2) = match i % 4 {
            0 => (658, "TRPL ", 109),
            1 => (715, "TIA  ", 128),
            2 => (711, "SETRA", 139),
            _ => (751, "     ", 0),
        };
        let r = i % 8;
        let l = match i % 3 {
            0 => 0,
            1 => 3,
            _ => 6,
        };
        let a = 300 + ((i * 3) % USER_MEM_LEN);
        let b = 300 + ((i * 5) % USER_MEM_LEN);
        let c = 300 + ((i * 7) % USER_MEM_LEN);
        let d = 300 + ((i * 11) % USER_MEM_LEN);
        let _ = writeln!(
            data,
            "PRG{i:04}0 {loc:04}0{alpha_op1}{r}{a:04}{b:04}{c:04}00{alpha_op2}{d:04}{l}0{num_op1:03}{num_op2:03}0000SPEEDCO INSTRUCTION CARD "
        );
    }

    let parser = mpc_and(
        vec![
            mpc_many(
                mpc_and(
                    vec![
                        mpc_count(7, mpc_noneof("\r\n"), mpcf_null),
                        mpc_char('0'),
                        mpc_char(' '),
                        mpc_count(4, mpc_digit(), mpcf_null),
                        mpc_char('0'),
                        mpc_count(5, mpc_noneof("\r\n"), mpcf_null),
                        mpc_range('0', '7'),
                        mpc_count(12, mpc_digit(), mpcf_null),
                        mpc_string("00"),
                        mpc_count(5, mpc_noneof("\r\n"), mpcf_null),
                        mpc_count(4, mpc_digit(), mpcf_null),
                        mpc_oneof("036"),
                        mpc_digit(),
                        mpc_count(6, mpc_digit(), mpcf_null),
                        mpc_string("0000"),
                        mpc_count(25, mpc_noneof("\r\n"), mpcf_null),
                        mpc_newline(),
                    ],
                    mpcf_null,
                ),
                mpcf_null,
            ),
            mpc_eoi(),
        ],
        mpcf_null,
    );

    let mut input = MpcInput::new("instr_cards_2000.deck", &data);
    let start = Instant::now();
    let res = parser.parse(&mut input);
    let elapsed = start.elapsed();

    assert!(matches!(res, MpcResult::Ok(_)), "expected 80-col instruction card parse to succeed");
    assert_eq!(input.state.row, 2000);
    assert_eq!(input.pos, data.len());
    report_bench("speedco_2000_cards_80col", 2000, "cards", data.len(), elapsed);

    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_secs(1),
        "release 2000 instruction card parse exceeded 1s: {elapsed:?}"
    );
}

#[test]
fn bench_parse_2000_data_cards_10000_floats() {
    let mut data = String::with_capacity(2000 * 81);
    for i in 0..2000 {
        let loc = 300 + (i % USER_MEM_LEN);
        let _ = writeln!(
            data,
            "DAT{i:04}*8{loc:04}500000000A00A250000000J00K12345X6789{{00{{314159265D00A100000000{{01}}5"
        );
    }

    let overpunch = mpc_oneof("ABCDEFGHI{JKLMNOPQR}");
    let frac10 = mpc_and(
        vec![mpc_count(9, mpc_digit(), mpcf_null), overpunch.clone()],
        mpcf_null,
    );
    let exp3 = mpc_and(
        vec![mpc_count(2, mpc_digit(), mpcf_null), overpunch.clone()],
        mpcf_null,
    );
    let frac3_col45 = mpc_and(
        vec![
            mpc_count(5, mpc_digit(), mpcf_null),
            mpc_noneof("\r\n"),
            mpc_count(4, mpc_digit(), mpcf_null),
            overpunch,
        ],
        mpcf_null,
    );

    let data_card = mpc_and(
        vec![
            mpc_count(7, mpc_noneof("\r\n"), mpcf_null),
            mpc_char('*'),
            mpc_char('8'),
            mpc_count(4, mpc_digit(), mpcf_null),
            frac10.clone(),
            exp3.clone(),
            frac10.clone(),
            exp3.clone(),
            frac3_col45,
            exp3.clone(),
            frac10.clone(),
            exp3.clone(),
            frac10,
            exp3,
            mpc_oneof("12345"),
            mpc_newline(),
        ],
        mpcf_null,
    );
    let parser = mpc_and(vec![mpc_many(data_card, mpcf_null), mpc_eoi()], mpcf_null);

    let mut input = MpcInput::new("data_cards_2000.deck", &data);
    let start = Instant::now();
    let res = parser.parse(&mut input);
    let elapsed = start.elapsed();

    assert!(matches!(res, MpcResult::Ok(_)), "expected 80-col data card parse to succeed");
    assert_eq!(input.state.row, 2000);
    assert_eq!(input.pos, data.len());
    report_bench("speedco_2000_data_cards", 2000, "cards", data.len(), elapsed);
    report_bench("speedco_10000_data_floats", 10_000, "numbers", data.len(), elapsed);

    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_secs(1),
        "release 2000 data card parse exceeded 1s: {elapsed:?}"
    );
}

#[test]
fn bench_linear_scaling_500_to_2000_lines() {
    let data_500 = gen_symbolic_lines(500);
    let data_2000 = gen_symbolic_lines(2000);
    let parser = build_symbolic_program_parser();

    let runs = 3u32;
    let start_500 = Instant::now();
    for _ in 0..runs {
        let res = mpc_parse("scale_500.sc", &data_500, &parser);
        assert!(matches!(res, MpcResult::Ok(_)));
    }
    let avg_500 = start_500.elapsed() / runs;

    let start_2000 = Instant::now();
    for _ in 0..runs {
        let res = mpc_parse("scale_2000.sc", &data_2000, &parser);
        assert!(matches!(res, MpcResult::Ok(_)));
    }
    let avg_2000 = start_2000.elapsed() / runs;

    let ratio = avg_2000.as_secs_f64() / avg_500.as_secs_f64().max(1e-6);
    println!(
        "[bench] scaling_500_to_2000: 500={:.2} ms, 2000={:.2} ms, ratio={:.2}x (4x input)",
        avg_500.as_secs_f64() * 1000.0,
        avg_2000.as_secs_f64() * 1000.0,
        ratio
    );
    assert!(
        ratio < 12.0,
        "expected near-linear scaling (< 12.0x for 4x input), got {ratio:.2}x"
    );
}

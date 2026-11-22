use std::env;
use std::fs;
use std::process::ExitCode;

use speedcoding::control::{SpeedCoMachine, StepOutcome};
use speedcoding::core_types::{
    Address, DecimalFloat, FRACTION_BITS, INITIAL_PC, MAX_BINARY_EXPONENT, MAX_DECIMAL_EXPONENT,
    MachineStop, SPECIAL_R_CONTROL_ADDR, SpeedFloat, TOTAL_MEM_WORDS, USER_MEM_END, USER_MEM_LEN,
    USER_MEM_START, WORD_BITS,
};
use speedcoding::diagnostics::{
    LISTING_BLOCK_SIZE, TimingCalculator, TimingContext, TimingRange,
};
use speedcoding::io::{DRUM_CAPACITY, PRINTER_VALUES_PER_LINE};
use speedcoding::parser::parse_program;

const DEFAULT_MAX_STEPS: usize = 10_000;

#[derive(Debug, Clone, PartialEq)]
enum CliAction {
    Help,
    Version,
    Info,
    Run {
        path: String,
        data: Vec<(u16, f64)>,
        max_steps: usize,
        trace: bool,
    },
}

#[derive(Debug, Clone)]
struct RunReport {
    stop: MachineStop,
    steps: usize,
    printer_lines: usize,
    listing_lines: usize,
    estimated_avg_ms: f64,
    estimated_max_ms: f64,
}

fn print_help() {
    println!("speedcoding {}", env!("CARGO_PKG_VERSION"));
    println!("Rust implementation of the 1953 IBM 701 Speedcoding System (SpeedCo I)");
    println!();
    println!(
        "Status: v0.1.0 stable release (Phases 1-6 + E2E & CLI complete; 79/79 tests passing)"
    );
    println!();
    println!("USAGE:");
    println!("    speedcoding [COMMAND | OPTION]");
    println!();
    println!("COMMANDS:");
    println!("    info                                Display SpeedCo I architecture constants");
    println!(
        "    run <program.sco> [OPTIONS]         Execute a symbolic SpeedCo I program until STOP"
    );
    println!(
        "    trace <program.sco> [OPTIONS]       Execute with all 3 listing switches enabled"
    );
    println!();
    println!("RUN / TRACE OPTIONS:");
    println!("    --data <ADDR>=<VALUE>               Seed user memory cell (300..=1013) before execution");
    println!("    --max-steps <N>                     Maximum instruction steps (default: 10000)");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help                          Print help information");
    println!("    -V, --version                       Print version information");
}

fn print_info() {
    println!("IBM 701 Speedcoding System (SpeedCo I) Parameters:");
    println!("  Word size:                {WORD_BITS} bits");
    println!("  Fraction precision:       {FRACTION_BITS} binary bits");
    println!("  Total electrostatic mem:  {TOTAL_MEM_WORDS} words (0..=1023)");
    println!("  User electrostatic mem:   {USER_MEM_START}..={USER_MEM_END} ({USER_MEM_LEN} words)");
    println!("  Initial program counter:  {INITIAL_PC}");
    println!("  R-control special cell:   {SPECIAL_R_CONTROL_ADDR:04}");
    println!("  Decimal exponent limit:   +/-{MAX_DECIMAL_EXPONENT}");
    println!("  Binary exponent limit:    +/-{MAX_BINARY_EXPONENT}");
    println!("  Magnetic drums (P, Q):    {DRUM_CAPACITY} words each");
    println!("  Line printer width:       {PRINTER_VALUES_PER_LINE} values per line");
    println!("  Listing drum block size:  {LISTING_BLOCK_SIZE} lines");
}

fn parse_data_binding(spec: &str) -> Result<(u16, f64), String> {
    let (addr_str, val_str) = spec
        .split_once('=')
        .ok_or_else(|| format!("expected --data <ADDR>=<VALUE>, got '{spec}'"))?;
    let addr: u16 = addr_str
        .trim()
        .parse()
        .map_err(|err| format!("invalid --data address '{addr_str}': {err}"))?;
    let val: f64 = val_str
        .trim()
        .parse()
        .map_err(|err| format!("invalid --data value '{val_str}': {err}"))?;
    Ok((addr, val))
}

fn parse_cli_args<I: IntoIterator<Item = String>>(args: I) -> Result<CliAction, String> {
    let mut iter = args.into_iter();
    let Some(first) = iter.next() else {
        return Ok(CliAction::Help);
    };
    match first.as_str() {
        "-h" | "--help" => Ok(CliAction::Help),
        "-V" | "--version" => Ok(CliAction::Version),
        "info" => {
            if let Some(extra) = iter.next() {
                return Err(format!("unexpected argument '{extra}' after `info`"));
            }
            Ok(CliAction::Info)
        }
        "run" | "trace" => {
            let trace = first == "trace";
            let mut path: Option<String> = None;
            let mut data = Vec::new();
            let mut max_steps = DEFAULT_MAX_STEPS;

            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--data" => {
                        let spec = iter
                            .next()
                            .ok_or_else(|| "missing value after `--data`".to_string())?;
                        data.push(parse_data_binding(&spec)?);
                    }
                    "--max-steps" => {
                        let raw = iter
                            .next()
                            .ok_or_else(|| "missing integer after `--max-steps`".to_string())?;
                        max_steps = raw
                            .parse::<usize>()
                            .map_err(|err| format!("invalid `--max-steps` value '{raw}': {err}"))?;
                    }
                    other if other.starts_with('-') => {
                        return Err(format!("unknown option '{other}'"));
                    }
                    file_path => {
                        if path.is_some() {
                            return Err(format!("unexpected extra positional argument '{file_path}'"));
                        }
                        path = Some(file_path.to_string());
                    }
                }
            }

            let path = path.ok_or_else(|| format!("missing `<program.sco>` argument for `{first}`"))?;
            Ok(CliAction::Run {
                path,
                data,
                max_steps,
                trace,
            })
        }
        other => Err(format!(
            "unexpected argument '{other}'. Run `speedcoding --help` for usage."
        )),
    }
}

fn execute_source(
    source: &str,
    data: &[(u16, f64)],
    trace: bool,
    max_steps: usize,
) -> Result<(RunReport, SpeedCoMachine), String> {
    let program = parse_program(source).map_err(|err| format!("parse error: {err:?}"))?;
    let mut machine = SpeedCoMachine::new();

    for &(raw_addr, val) in data {
        let addr = Address::user(raw_addr)
            .map_err(|err| format!("invalid data address {raw_addr}: {err:?}"))?;
        let sf = SpeedFloat::from_f64(val)
            .map_err(|err| format!("invalid floating value {val}: {err:?}"))?;
        machine
            .memory_mut()
            .write_float(addr, sf)
            .map_err(|err| format!("memory write failed at {raw_addr}: {err:?}"))?;
    }

    for line in program {
        machine
            .memory_mut()
            .write_instruction(line.location, line.instruction)
            .map_err(|err| format!("instruction write failed at {}: {err:?}", line.location.raw()))?;
    }

    if trace {
        let switches = machine.switches_mut();
        switches.listing.switch_1_l0 = true;
        switches.listing.switch_2_l3 = true;
        switches.listing.switch_3_l6 = true;
    }

    let mut total_avg_ms = 0.0_f64;
    let mut total_max_ms = 0.0_f64;

    for steps in 1..=max_steps {
        let pc = machine.pc();
        let instr = machine
            .memory()
            .read_instruction(pc)
            .map_err(|err| format!("instruction fetch error at {}: {err:?}", pc.raw()))?;
        let outcome = machine
            .step()
            .map_err(|err| format!("execution error at {}: {err:?}", pc.raw()))?;

        let TimingRange { avg_ms, max_ms } =
            TimingCalculator::instruction_time_ms(instr.op1, instr.op2, TimingContext::default());
        total_avg_ms += avg_ms;
        total_max_ms += max_ms;

        if let StepOutcome::Stopped(stop) = outcome {
            let flushed: usize = machine
                .listing()
                .flushed_blocks()
                .iter()
                .map(Vec::len)
                .sum();
            let listing_lines = flushed + machine.listing().pending_count();
            let report = RunReport {
                stop,
                steps,
                printer_lines: machine.printer().lines().len(),
                listing_lines,
                estimated_avg_ms: total_avg_ms,
                estimated_max_ms: total_max_ms,
            };
            return Ok((report, machine));
        }
    }

    Err(format!("machine did not stop within {max_steps} steps"))
}

fn format_decimal(dec: DecimalFloat) -> String {
    let sign = if dec.negative { "-" } else { "+" };
    format!("{sign}0.{:010}e{:+}", dec.digits, dec.exponent)
}

fn format_run_report(report: &RunReport, machine: &SpeedCoMachine, trace: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!("Stop condition: {:?}\n", report.stop));
    out.push_str(&format!("Executed steps: {}\n", report.steps));
    out.push_str(&format!(
        "Estimated time: {:.3} ms (avg) / {:.3} ms (max)\n",
        report.estimated_avg_ms, report.estimated_max_ms
    ));
    out.push_str(&format!("Printer lines:  {}\n", report.printer_lines));
    for line in machine.printer().lines() {
        let vals: Vec<String> = line.values.iter().copied().map(format_decimal).collect();
        out.push_str(&format!("  [{:05}] {}\n", line.id, vals.join(", ")));
    }
    if trace {
        out.push_str(&format!(
            "Listing lines:  {} ({} flushed blocks, {} pending)\n",
            report.listing_lines,
            machine.listing().flushed_blocks().len(),
            machine.listing().pending_count()
        ));
    }
    out
}

fn main() -> ExitCode {
    match parse_cli_args(env::args().skip(1)) {
        Ok(CliAction::Help) => {
            print_help();
            ExitCode::SUCCESS
        }
        Ok(CliAction::Version) => {
            println!("speedcoding {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(CliAction::Info) => {
            print_info();
            ExitCode::SUCCESS
        }
        Ok(CliAction::Run {
            path,
            data,
            max_steps,
            trace,
        }) => match fs::read_to_string(&path) {
            Ok(source) => match execute_source(&source, &data, trace, max_steps) {
                Ok((report, machine)) => {
                    print!("{}", format_run_report(&report, &machine, trace));
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("error: {err}");
                    ExitCode::FAILURE
                }
            },
            Err(err) => {
                eprintln!("error: failed to read '{path}': {err}");
                ExitCode::FAILURE
            }
        },
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_help_version_info_run_and_trace_flags() {
        assert_eq!(parse_cli_args(Vec::<String>::new()), Ok(CliAction::Help));
        assert_eq!(
            parse_cli_args(["--help".to_string()]),
            Ok(CliAction::Help)
        );
        assert_eq!(
            parse_cli_args(["-V".to_string()]),
            Ok(CliAction::Version)
        );
        assert_eq!(parse_cli_args(["info".to_string()]), Ok(CliAction::Info));
        assert_eq!(
            parse_cli_args([
                "run".to_string(),
                "sample.sco".to_string(),
                "--data".to_string(),
                "500=3.5".to_string(),
                "--max-steps".to_string(),
                "250".to_string(),
            ]),
            Ok(CliAction::Run {
                path: "sample.sco".to_string(),
                data: vec![(500, 3.5)],
                max_steps: 250,
                trace: false,
            })
        );
        assert_eq!(
            parse_cli_args(["trace".to_string(), "demo.sco".to_string()]),
            Ok(CliAction::Run {
                path: "demo.sco".to_string(),
                data: Vec::new(),
                max_steps: DEFAULT_MAX_STEPS,
                trace: true,
            })
        );
    }

    #[test]
    fn cli_rejects_unknown_command_or_missing_file_arg() {
        assert!(parse_cli_args(["unknown".to_string()]).is_err());
        assert!(parse_cli_args(["run".to_string()]).is_err());
        assert!(parse_cli_args(["info".to_string(), "extra".to_string()]).is_err());
        assert!(
            parse_cli_args([
                "run".to_string(),
                "prog.sco".to_string(),
                "--data".to_string(),
                "bad".to_string()
            ])
            .is_err()
        );
    }

    #[test]
    fn cli_execute_source_runs_sum_program_and_prints_output() {
        let src = "\
            0300 ADD   0 0500 0501 0502 -    0000\n\
            0301 PRINT 0 0502 0502 0101 STOP 0301\n";
        let (report, machine) =
            execute_source(src, &[(500, 12.5), (501, 29.5)], false, 50).unwrap();
        assert_eq!(
            report.stop,
            MachineStop::ProgramStop {
                restart_addr: Address::user(301).unwrap()
            }
        );
        assert_eq!(report.steps, 2);
        assert_eq!(report.printer_lines, 1);
        let res = machine
            .memory()
            .read_float(Address::user(502).unwrap())
            .unwrap()
            .to_f64();
        assert!((res - 42.0).abs() < 1e-8);
    }

    #[test]
    fn cli_trace_mode_enables_listing_and_computes_timing() {
        let mut lines = Vec::new();
        for i in 0..9u16 {
            lines.push(format!("0{} NOOP 0 0000 0000 0000 - 0000", 300 + i));
        }
        lines.push("0309 NOOP 0 0000 0000 0000 STOP 0309".to_string());
        let (report, machine) = execute_source(&lines.join("\n"), &[], true, 50).unwrap();
        assert_eq!(report.steps, 10);
        assert_eq!(report.listing_lines, 10);
        assert_eq!(machine.listing().flushed_blocks().len(), 1);
        assert!(report.estimated_avg_ms > 20.0);
        assert!(report.estimated_max_ms >= report.estimated_avg_ms);
        let summary = format_run_report(&report, &machine, true);
        assert!(summary.contains("Listing lines:  10"));
    }
}

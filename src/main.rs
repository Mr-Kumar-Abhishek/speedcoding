use std::env;
use std::process::ExitCode;

use speedcoding::core_types::{
    FRACTION_BITS, INITIAL_PC, MAX_BINARY_EXPONENT, MAX_DECIMAL_EXPONENT, SPECIAL_R_CONTROL_ADDR,
    TOTAL_MEM_WORDS, USER_MEM_END, USER_MEM_LEN, USER_MEM_START, WORD_BITS,
};
use speedcoding::diagnostics::LISTING_BLOCK_SIZE;
use speedcoding::io::{DRUM_CAPACITY, PRINTER_VALUES_PER_LINE};

fn print_help() {
    println!("speedcoding {}", env!("CARGO_PKG_VERSION"));
    println!("Rust implementation of the 1953 IBM 701 Speedcoding System (SpeedCo I)");
    println!();
    println!("Status: Sprint 4 pre-release (Phases 1-4 core_types, parser, arithmetic & control implemented; 44/66 TDD + 5/5 bench passing)");
    println!();
    println!("USAGE:");
    println!("    speedcoding [COMMAND | OPTION]");
    println!();
    println!("COMMANDS:");
    println!("    info           Display SpeedCo I architecture constants");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help     Print help information");
    println!("    -V, --version  Print version information");
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

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("-h") | Some("--help") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("-V") | Some("--version") => {
            println!("speedcoding {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("info") if args.next().is_none() => {
            print_info();
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("error: unexpected argument '{other}'. Run `speedcoding --help` for usage.");
            ExitCode::FAILURE
        }
    }
}

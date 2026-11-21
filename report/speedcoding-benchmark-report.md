# SpeedCo I Parsing Benchmark Report (`speedcoding`)

## 1. Executive Summary & Historical Comparison

- **Benchmark Scope:** Measures SpeedCo I (`mpc-rs`-backed) parsing throughput and asymptotic scaling across realistic 1953 IBM 701 Speedcoding workloads: full 714-word electrostatic memory programs, 2,000-line symbolic AST construction, 2,000 fixed-width 80-column Instruction Cards, and 2,000 80-column Data Cards (`10,000` signed floating-decimal numbers).
- **Release Latency (`opt-level = 3`):** All 2,000-line / 2,000-card benchmarks complete in **`8.74–11.67 ms`** (`~85x–114x` faster than the `< 1.00 s` CI gate), while a full 714-word electrostatic memory program parses in **`1.88 ms`** (`380,347 lines/s`).
- **Debug Latency (`opt-level = 0`):** Even without compiler optimizations, every benchmark finishes in **`6.04–37.32 ms`**.
- **Linear `O(n)` Scaling:** Increasing input size `4x` (from `500` to `2,000` symbolic lines) yields a **`2.90x`** ratio in Debug mode and **`3.40x`** in Release mode (well below the `< 12.0x` threshold).
- **1953 Hardware Comparison:** Manual page 7-5 specifies an IBM 701 card-reader input rate of **150 cards/min** (`2.5 cards/s`) and **750 data numbers/min** (`12.5 numbers/s`). In Release mode, `tests/bench_speedcoding.rs` parses 80-column instruction cards at **`191,889 cards/s`** (**`~76,756x`** faster) and 5-value data cards at **`228,893 cards/s`** / **`1,144,463 numbers/s`** (**`~91,557x`** faster).

---

## 2. Test Environment & Execution Metadata

| Property | Value |
|---|---|
| **Repository / Commit** | `Mr-Kumar-Abhishek/speedcoding` (`239bf342d05c6e9c573b108fa31a7f1a20d5f884`) |
| **Benchmark Suite** | [`tests/bench_speedcoding.rs`](../tests/bench_speedcoding.rs) |
| **CI Workflow / Run ID** | [`.github/workflows/rust.yml`](../.github/workflows/rust.yml) — Run [`#44938969997`](https://github.com/Mr-Kumar-Abhishek/speedcoding/actions/runs/44938969997) (`success`) |
| **Runner OS** | `ubuntu-latest` (GitHub Actions) |
| **Rust Edition / Version** | Rust `2024` (`speedcoding v0.1.0-alpha.1` + `mpc v0.1.0`) |
| **Execution Commands** | `cargo test --test bench_speedcoding --verbose -- --nocapture` and `cargo test --release --test bench_speedcoding --verbose -- --nocapture` |

---

## 3. Benchmark Results (Debug vs. Release)

### 3.1 Latency & Throughput Comparison

| Test Case | Input Size | Debug Latency | Debug Throughput | Release Latency | Release Throughput | `< 1s` Release Gate |
|---|---:|---:|---:|---:|---:|:---:|
| `bench_parse_714_line_full_memory_program` | 714 lines (`26,162 B`) | `6.04 ms` | `118,151 lines/s` (`4.13 MB/s`) | `1.88 ms` | `380,347 lines/s` (`13.29 MB/s`) | Pass |
| `bench_parse_2000_line_speedco_program_ast` | 2,000 lines (`73,279 B`) | `37.32 ms` | `53,588 lines/s` (`1.87 MB/s`) | `11.67 ms` | `171,328 lines/s` (`5.99 MB/s`) | Pass |
| `bench_parse_2000_instruction_cards_80col` | 2,000 cards (`162,000 B`) | `35.47 ms` | `56,385 cards/s` (`4.36 MB/s`) | `10.42 ms` | `191,889 cards/s` (`14.82 MB/s`) | Pass |
| `bench_parse_2000_data_cards_10000_floats` | 2,000 cards / 10,000 nums (`162,000 B`) | `36.96 ms` | `54,111 cards/s` / `270,555 nums/s` (`4.18 MB/s`) | `8.74 ms` | `228,893 cards/s` / `1,144,463 nums/s` (`17.68 MB/s`) | Pass |

### 3.2 Workload Descriptions

1. **`bench_parse_714_line_full_memory_program`**: Generates a full 714-line (`USER_MEM_LEN`) SpeedCo I symbolic program cycling through `OP1` (`ADDAB`, `ABSUB`, `WRTPK`, `PRINT`, `SINE`, `SQRT`, `NOOP`, `ADD`, `SUB`, `MPY`, `DIV`) and `OP2` (`ECHTR`, `SETRA`, `RADDA`, `TIAB`, `TRPL`, `TRMN`, `SKRA`, `TIA`, `STD`, `TR`, `-`) mnemonics, verifying full cursor consumption (`row == 714`).
2. **`bench_parse_2000_line_speedco_program_ast`**: Parses 2,000 symbolic SpeedCo I lines into a full recursive `MpcAst` (`>|speedco_program` root with `2,000` tagged `instr` child nodes) using `mpca_root`, `mpca_tag`, and `mpcf_fold_ast`.
3. **`bench_parse_2000_instruction_cards_80col`**: Parses 2,000 fixed-width 80-column SpeedCo I Instruction Cards (`81` bytes/line including `\n`), validating column 8 (`0`), column 9 (blank), fixed zero fields (cols `14`, `33–34`, `52–55`), `R` (`0..=7`), `L` (`0/3/6`), numerical codes, and 25-column remarks.
4. **`bench_parse_2000_data_cards_10000_floats`**: Parses 2,000 fixed-width 80-column SpeedCo I Data Cards (`10,000` 10-digit signed floating-decimal numbers), validating double-punch col 8 (`*`), col 9 (`8`), col 45 skip inside `N3`, IBM 12/11 overpunched sign characters, and col 80 word count.

---

## 4. Linear Scaling Check (500 vs. 2,000 Lines)

Test `bench_linear_scaling_500_to_2000_lines` averages 3 runs of the symbolic SpeedCo I program parser at 500 lines vs. 2,000 lines (`4x` input growth):

| Profile | 500 Lines (Avg) | 2,000 Lines (Avg) | Measured Ratio (`4x` Input) | Assertion (`< 12.0x`) |
|---|---:|---:|---:|:---:|
| **Debug** | `5.13 ms` | `14.87 ms` | `2.90x` | Pass |
| **Release** | `1.30 ms` | `4.42 ms` | `3.40x` | Pass |

---

## 5. CI Test & Compilation Summary (`Run #44938969997`)

| Suite / Profile | Passed | Failed | Ignored | Reported Suite Time | Notes |
|---|---:|---:|---:|---:|---|
| Unit tests (`src/lib.rs`) — Debug | 1 | 0 | 0 | `0.00s` | `mpc_library_inclusion` |
| Binary unit tests (`src/main.rs`) — Debug | 0 | 0 | 0 | `0.00s` | Compiles cleanly (`0` warnings) |
| Benchmark suite (`tests/bench_speedcoding.rs`) — Debug | 5 | 0 | 0 | `0.06s` | All 5 benchmarks printed `[bench]` metrics |
| Phase 1–6 TDD suites (`tests/phase_1..6`) — Debug | — | — | — | Compiled (`--no-run`) | 65 Red-phase tests compile cleanly (`0` warnings) |
| Benchmark suite (`tests/bench_speedcoding.rs`) — Release | 5 | 0 | 0 | `0.02s` | All `< 1s` release gates passed |

---

## 6. How to Reproduce

```bash
# Debug profile (unoptimized + debuginfo)
cargo test --test bench_speedcoding --verbose -- --nocapture

# Final Release profile (opt-level = 3, enforces < 1s assertions)
cargo test --release --test bench_speedcoding --verbose -- --nocapture
```

*(Note: On Windows hosts where an Application Control policy blocks executing local binaries under `target\debug\deps\` or `target\release\deps\` with `os error 4551`, compile locally with `--no-run` and inspect runtime metrics via `.github/workflows/rust.yml` on GitHub Actions.)*

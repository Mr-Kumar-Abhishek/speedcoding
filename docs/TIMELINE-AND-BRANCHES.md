# Development Timeline & Branching Strategy

`speedcoding` follows a Gitflow + milestone-branch model aligned with 2-week **Agile Sprints** and **Red -> Green -> Refactor** Test-Driven Development (TDD) on <https://github.com/Mr-Kumar-Abhishek/speedcoding>. For Sprint ceremonies, Fibonacci estimates, and the full `US-01..US-14` backlog, see [AGILE-SPRINTS.md](AGILE-SPRINTS.md).

---

## Branch Topology

```mermaid
flowchart LR
    M["master (Stable Releases)"]
    D["develop (Integration & TDD Baseline)"]
    P1["phase-1/core-types (Sprint 1)"]
    P2["phase-2/parser (Sprint 2)"]
    P3["phase-3/arithmetic-math (Sprint 3)"]
    P4["phase-4/control-addressing (Sprint 4)"]
    P5["phase-5/io-storage (Sprint 5)"]
    P6["phase-6/checking-listing-timing (Sprint 6)"]

    M -->|initial baseline| D
    D --> P1
    D --> P2
    D --> P3
    D --> P4
    D --> P5
    D --> P6
    P1 -->|Green + Refactor| D
    P2 -->|Green + Refactor| D
    P3 -->|Green + Refactor| D
    P4 -->|Green + Refactor| D
    P5 -->|Green + Refactor| D
    P6 -->|Green + Refactor| D
    D -->|Sprint 7 Release Gate| M
```

| Branch | Role | Current Status |
|---|---|---|
| `master` | Stable baseline and verified releases (`v0.1.0`) | Pre-stub crate + licensing setup (`d03f111`) |
| `develop` | Primary integration branch hosting shared API stubs, CLI, docs, CI, and all phase test suites | Sprint 0 baseline (`v0.1.0-alpha.1`, 66 compiling tests) |
| `phase-1/core-types` | Sprint 1 implementation branch | Synced to `develop` (`v0.1.0-alpha.1` baseline) |
| `phase-2/parser` | Sprint 2 implementation branch | Synced to `develop` (`v0.1.0-alpha.1` baseline) |
| `phase-3/arithmetic-math` | Sprint 3 implementation branch | Synced to `develop` (`v0.1.0-alpha.1` baseline) |
| `phase-4/control-addressing` | Sprint 4 implementation branch | Synced to `develop` (`v0.1.0-alpha.1` baseline) |
| `phase-5/io-storage` | Sprint 5 implementation branch | Synced to `develop` (`v0.1.0-alpha.1` baseline) |
| `phase-6/checking-listing-timing` | Sprint 6 implementation branch | Synced to `develop` (`v0.1.0-alpha.1` baseline) |

---

## Sprint & Phase Timeline

| Sprint | Phase / Branch | Target Tag | Implementation Module | Test Suite | Tests | SpeedCo I Manual Sections |
|---|---|---|---|---|---:|---|
| **Sprint 0** | Bootstrap / `develop` | `v0.1.0-alpha.1` *(Done)* | [`src/lib.rs`](../src/lib.rs), [`src/main.rs`](../src/main.rs), `docs/`, `.github/` | `mpc_library_inclusion` + 6 phase skeletons | 1 + 65 | Repository setup, licensing, `mpc-rs`, docs, CI, pre-release binary |
| **Sprint 1** | Phase 1 (`phase-1/core-types`) | `v0.1.0-alpha.2` | [`src/core_types.rs`](../src/core_types.rs) | [`tests/phase_1_core_types.rs`](../tests/phase_1_core_types.rs) | 11 | Sec. 1–2 (Storage), Sec. 3-2 (Number Word), Sec. 4-4 (Instruction Layout), Sec. 5 (All 49 `OP1` + 48 `OP2` codes) |
| **Sprint 2** | Phase 2 (`phase-2/parser`) | `v0.1.0-alpha.3` | [`src/parser.rs`](../src/parser.rs) | [`tests/phase_2_parser.rs`](../tests/phase_2_parser.rs) | 10 | Sec. 4-2..4-3, Sec. 7-1..7-4 (Instruction & Data Cards), Sec. 7-17 (Loading Control Cards), Skip-`OP1` constraints |
| **Sprint 3** | Phase 3 (`phase-3/arithmetic-math`) | `v0.1.0-alpha.4` | [`src/arithmetic.rs`](../src/arithmetic.rs) | [`tests/phase_3_arithmetic_math.rs`](../tests/phase_3_arithmetic_math.rs) | 10 | Sec. 3-1..3-4 (Add/Sub/Mpy/Div), Sec. 6, Appendices 1–2 (Set 1 `SQRT`, `SINE`, `ARTAN`, `EXP`, `LN`) |
| **Sprint 4** | Phase 4 (`phase-4/control-addressing`) | `v0.1.0-alpha.5` | [`src/control.rs`](../src/control.rs) | [`tests/phase_4_control_addressing.rs`](../tests/phase_4_control_addressing.rs) | 12 | Sec. 4-1 (`PC = 300`), Sec. 8 (Transfers & Sense Switches), Sec. 9 (`R`-quantities, Address Counter, `0025`, Worked Examples) |
| **Sprint 5** | Phase 5 (`phase-5/io-storage`) | `v0.1.0-alpha.6` | [`src/io.rs`](../src/io.rs) + `load_deck` | [`tests/phase_5_io_storage.rs`](../tests/phase_5_io_storage.rs) | 11 | Sec. 7-4..7-19 (7-ch Tape Parity, Status Matrix, `RFTP`/`RBTP`/`SFTP`/`SBTP`, Drums `P`/`Q`, Line Printer, Card Reader) |
| **Sprint 6** | Phase 6 (`phase-6/checking-listing-timing`) | `v0.1.0-beta.1` | [`src/diagnostics.rs`](../src/diagnostics.rs) | [`tests/phase_6_checking_listing_timing.rs`](../tests/phase_6_checking_listing_timing.rs) | 11 | Sec. 10 (`PRCH`/`STCH`/`ECHTR`), Sec. 11 (Selective Listing & 10-line Drum Buffer), Sec. 12 & Appendices (Execution/Tape/Load Timing) |
| **Sprint 7** | Release (`develop` -> `master`) | `v0.1.0` | End-to-end CLI & Matrix Example | All 66 tests + E2E sample tests | 66+ | Page 9-6 matrix multiplication E2E validation, stable release artifacts |

---

## TDD Workflow & Definition of Done

```mermaid
flowchart LR
    RED["1. RED: Tests compile with 0 warnings against todo!() stubs"] --> GREEN["2. GREEN: Implement minimal safe Rust to pass target phase tests"]
    GREEN --> REFACTOR["3. REFACTOR: Clean helpers, docstrings, zero warnings"]
    REFACTOR --> VERIFY["4. VERIFY: Debug + Release tests, cargo doc, cargo package"]
```

A `phase-N/*` Sprint branch is ready to merge into `develop` only when:
1. All `tests/phase_N_*.rs` tests (and all prior phase tests + `src/lib.rs` unit tests) compile and pass in both debug (`cargo test`) and release (`cargo test --release`) profiles with **0 warnings**.
2. Public items carry concise rustdoc comments and `cargo doc --no-deps` builds cleanly.
3. `cargo package --list --allow-dirty` completes with 0 warnings and keeps `102632810-05-01-acc.pdf` excluded.
4. Pull Request checks pass in [`.github/workflows/rust.yml`](../.github/workflows/rust.yml) and [AGILE-SPRINTS.md](AGILE-SPRINTS.md) backlog statuses are updated before syncing `phase-*` branches and tagging the sprint pre-release.

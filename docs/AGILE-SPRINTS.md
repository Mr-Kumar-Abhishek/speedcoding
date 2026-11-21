# Agile Sprint Workflow & Backlog (`speedcoding`)

`speedcoding` combines **2-week Agile Sprints** with **Gitflow milestone branches** and **Red-Green-Refactor Test-Driven Development (TDD)** to build a faithful Rust implementation of the 1953 IBM 701 Speedcoding System (SpeedCo I).

---

## 1. Sprint Lifecycle & Ceremonies

```mermaid
flowchart LR
    BKL["Product Backlog (US-01..US-14)"] --> PLN["Sprint Planning (DoR Check)"]
    PLN --> BR["Phase / Story Branch (phase-N/*)"]
    BR --> TDD["TDD Cycle (Red -> Green -> Refactor)"]
    TDD --> PR["Sprint PR & CI (.github/workflows/rust.yml)"]
    PR --> MRG["Merge to develop"]
    MRG --> MST["Merge develop to master & Tag"]
    MST --> RET["Sprint Review & Retrospective"]
    RET -->|Next Sprint| PLN
```

### Cadence & Ceremonies (2-Week Sprints)
1. **Sprint Planning (Day 1):**
   - Select the target `phase-N/*` milestone (or subset of User Stories `US-XX`) that meets the **Definition of Ready (DoR)**.
   - Confirm test coverage in [`tests/phase_N_*.rs`](TDD-TEST-SPECIFICATION.md) and open/assign issues using [`.github/ISSUE_TEMPLATE/user-story.md`](../.github/ISSUE_TEMPLATE/user-story.md).
2. **Daily / Async TDD Execution (Days 2–12):**
   - Work in focused commits on `phase-N/*` (or short-lived `sprint-N/us-XX-slug` branches off `phase-N/*`).
   - Follow **Red -> Green -> Refactor**:
     - **Red:** Verify the target test case fails (or panics on `todo!()`).
     - **Green:** Implement the minimal safe Rust code in the corresponding `src/*.rs` module to pass.
     - **Refactor:** Remove duplication, verify zero compiler/rustdoc warnings, and keep prior phase suites green.
3. **Sprint Review, Integration & Promotion to `master` (Day 13):**
   - Open a Pull Request into `develop` using [`.github/PULL_REQUEST_TEMPLATE.md`](../.github/PULL_REQUEST_TEMPLATE.md).
   - Confirm GitHub Actions CI ([`.github/workflows/rust.yml`](../.github/workflows/rust.yml)) passes debug build, unit/phase/benchmark tests, release build, and `cargo doc --no-deps`.
   - Merge `phase-N/*` into `develop`, fast-forward merge `develop` into `master`, sync downstream `phase-*` branches, and publish the sprint release tag (e.g., `v0.1.0-alpha.2`).
4. **Sprint Retrospective (Day 14):**
   - Audit edge cases from `102632810-05-01-acc.pdf`, refine story point estimates, and update `docs/` before starting the next sprint.

---

## 2. Sprint Roadmap (Sprints 0–7)

| Sprint | Status | Target Flow | Target Tag | Primary Deliverable | Test Milestone |
|---|---|---|---|---|---|
| **Sprint 0** | **Done** | `develop` -> `master` & `phase-1..6` | `v0.1.0-alpha.1` | Crate setup, `mpc-rs`, `MIT OR Apache-2.0` + `NOTICE`, Red-phase API stubs (`src/*.rs`), 66 TDD tests, 5 benchmarks + report, `docs/`, GitHub Pages, thin CLI (`src/main.rs`), pre-release (`8523081`..`6abe1c8`) | 1/66 TDD + 5/5 bench passing; 71/71 compiling (0 warnings) |
| **Sprint 1** | **Done** | `phase-1/core-types` -> `develop` -> `master` | `v0.1.0-alpha.2` | Implement `src/core_types.rs` (`Word72`, `Address`, `SpeedFloat`, `DecimalFloat`, `RCode`, `LCode`, 49 `Op1` + 48 `Op2`, `Instruction`, `ElectrostaticMemory`) | 12/66 green (`phase_1_core_types`: 11/11 in Debug & Release, CI Run `#44944881890`) |
| **Sprint 2** | Planned | `phase-2/parser` -> `develop` -> `master` | `v0.1.0-alpha.3` | Implement `src/parser.rs` (`mpc` mnemonic/line/program parsers, 80-col instruction/data/control cards, gang-punching, skip-`OP1` rules) | 22/66 green (`phase_2_parser`: 10/10) |
| **Sprint 3** | Planned | `phase-3/arithmetic-math` -> `develop` -> `master` | `v0.1.0-alpha.4` | Implement `src/arithmetic.rs` (6 add/sub ops, cancellation & `>76` diff rules, `MPY`/`NGMPY`, `DIV`/`NGDIV`, Set 1 math accuracy & quirks) | 32/66 green (`phase_3_arithmetic_math`: 10/10) |
| **Sprint 4** | Planned | `phase-4/control-addressing` -> `develop` -> `master` | `v0.1.0-alpha.5` | Implement `src/control.rs` (`PC=300`, `OP1`-before-`OP2`, `OP2` transfers, `RQuantities`, `AddressCounter`, location `0025`, `SpeedCoMachine` loop) | 44/66 green (`phase_4_control_addressing`: 12/12) |
| **Sprint 5** | Planned | `phase-5/io-storage` -> `develop` -> `master` | `v0.1.0-alpha.6` | Implement `src/io.rs` & `load_deck` (`TapeUnit` J..M parity/status/skips, `DrumUnit` P..Q, `LinePrinter`, Card Reader offloading) | 55/66 green (`phase_5_io_storage`: 11/11) |
| **Sprint 6** | Planned | `phase-6/checking-listing-timing` -> `develop` -> `master` | `v0.1.0-beta.1` | Implement `src/diagnostics.rs` (`PRCH`/`STCH`/`ECHTR` checking loops, 10-line selective `ListingBuffer`, Section 12 `TimingCalculator`) | 66/66 green (`phase_6_checking_listing_timing`: 11/11) |
| **Sprint 7** | Planned | `develop` -> `master` | `v0.1.0` | End-to-end IBM 701 sample programs (Page 9-6 matrix multiplication), full CLI run/trace commands, release verification, stable `v0.1.0` release on `master` | 66/66 TDD + 5/5 bench + E2E sample integration tests green |

---

## 3. Product Backlog (`US-01` through `US-14`)

| Story ID | Sprint | Branch | User Story | Priority | Pts | Status | Acceptance Criteria |
|---|---|---|---|---|---:|---|---|
| **US-01** | Sprint 0 | `master` / `develop` | As a maintainer, I need a dual-licensed Rust crate with `mpc-rs` parser combinator support and proper attribution for the 1953 manual. | High | 3 | **Done** | `Cargo.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, PDF excluded from `cargo package`, `mpc_library_inclusion` compiles. |
| **US-02** | Sprint 0 | `develop` -> `master` | As a team, we need a public GitHub repo with Gitflow + milestone branches and complete Red-phase API stubs + TDD test suites. | High | 5 | **Done** | Public repo live, 8 branches pushed, 6 `src/*.rs` stub modules + 65 integration tests compile with 0 warnings. |
| **US-03** | Sprint 0 | `develop` -> `master` | As a contributor, I need project, timeline, architecture, TDD, and GitHub Pages documentation in `docs/` and `report/`. | Medium | 3 | **Done** | `docs/README.md`, `index.html`, `TIMELINE-AND-BRANCHES.md`, `TDD-TEST-SPECIFICATION.md`, `ARCHITECTURE-AND-SPEC.md`, `report/speedcoding-benchmark-report.md` verified. |
| **US-04** | Sprint 0 | `develop` -> `master` | As an evaluator, I need a `--release` CLI pre-release (`v0.1.0-alpha.1`) with binary/library assets and SHA-256 checksums, plus Agile sprint templates & CI. | Medium | 5 | **Done** | `src/main.rs` release build, tag `v0.1.0-alpha.1`, GitHub pre-release assets + `SHA256SUMS.txt`, `AGILE-SPRINTS.md`, `.github/` CI, Pages & templates. |
| **US-05** | Sprint 1 | `phase-1/core-types` | As an emulator core, I need 72-bit words, floating-binary/decimal conversions, and 1024-word protected electrostatic memory. | High | 8 | **Done** | `tests/phase_1_core_types.rs` word, address, float, and memory tests pass (`11/11` overall in Debug & Release, CI Run `#44944881890`). |
| **US-06** | Sprint 1 | `phase-1/core-types` | As an assembler/decoder, I need `RCode`, `LCode`, all 49 `Op1` + 48 `Op2` codes, and 72-bit `Instruction` packing/unpacking. | High | 5 | **Done** | All 11 tests in `tests/phase_1_core_types.rs` pass (`11/11` green in Debug & Release) and phase merges to `develop` & `master` (`v0.1.0-alpha.2`). |
| **US-07** | Sprint 2 | `phase-2/parser` | As a SpeedCo I programmer, I need an `mpc`-powered parser for symbolic instructions and multi-line programs enforcing skip-`OP1` rules. | High | 5 | Planned | `mpc_op1_and_op2_mnemonic_parsers`, line/program, and `forbids_op2_with_skip_capable_op1` tests pass. |
| **US-08** | Sprint 2 | `phase-2/parser` | As a card reader loader, I need 80-column instruction, data (col 45 skip + 12/11 signs), and loading-control card parsers + gang-punching. | High | 8 | Planned | All 10 tests in `tests/phase_2_parser.rs` pass (`10/10` green) and phase merges to `develop` & `master`. |
| **US-09** | Sprint 3 | `phase-3/arithmetic-math` | As an arithmetic unit, I need all 6 add/sub ops, `MPY`/`NGMPY`, and `DIV`/`NGDIV` with SpeedCo I cancellation, truncation, and `DivideCheck`. | High | 5 | Planned | First 6 tests in `tests/phase_3_arithmetic_math.rs` pass. |
| **US-10** | Sprint 3 | `phase-3/arithmetic-math` | As a scientific programmer, I need Set 1 `SQRT`, `SINE`, `ARTAN`, `EXP`, and `LN` matching SpeedCo I precision and Appendix 1–2 edge quirks. | Medium | 8 | Planned | All 10 tests in `tests/phase_3_arithmetic_math.rs` pass (`10/10` green) and phase merges to `develop` & `master`. |
| **US-11** | Sprint 4 | `phase-4/control-addressing` | As an execution engine, I need `PC = 300` sequencing, conditional/switch `OP2` transfers, `STOP`/restart, and `R`-quantity index modification. | High | 8 | Planned | Tests 1–8 in `tests/phase_4_control_addressing.rs` (including Page 9-5 sum loop) pass. |
| **US-12** | Sprint 4 | `phase-4/control-addressing` | As a self-modifying program, I need the 18-bit `AddressCounter`, `0025` `R`-control tricks, and control-region boundary enforcement. | High | 5 | Planned | All 12 tests in `tests/phase_4_control_addressing.rs` pass (`12/12` green) and phase merges to `develop` & `master`. |
| **US-13** | Sprint 5 | `phase-5/io-storage` | As an I/O subsystem, I need 7-channel parity `TapeUnit`s (`J..M`), 1024-word `DrumUnit`s (`P..Q`), `LinePrinter`, and `load_deck`. | High | 8 | Planned | All 11 tests in `tests/phase_5_io_storage.rs` pass (`11/11` green) and phase merges to `develop` & `master`. |
| **US-14** | Sprint 6 | `phase-6/checking-listing-timing` | As a diagnostic & profiling tool, I need `PRCH`/`STCH`/`ECHTR` checking loops, 10-line `ListingBuffer`, and Section 12 `TimingCalculator`. | Medium | 8 | Planned | All 11 tests in `tests/phase_6_checking_listing_timing.rs` pass (`66/66` green) and phase merges to `develop` & `master`. |

---

## 4. Branching, Commit & CI Conventions

- **Long-lived & Milestone Branches:**
  - `master`: Receives fast-forward / verified merges from `develop` upon completion of each phase (`Sprint 0`, `Phase 1`, ..., `Phase 6`) and hosts release tags (`v0.1.0-alpha.*`, `v0.1.0-beta.1`, `v0.1.0`).
  - `develop`: Active integration branch for completed sprint work before promotion to `master`.
  - `phase-1/core-types` .. `phase-6/checking-listing-timing`: Milestone branches mapped 1-to-1 to Sprints 1–6.
  - Optional sub-story branches: `sprint-<N>/us-<XX>-<short-slug>` branched from and merged back into the active `phase-N/*` branch.
- **Commit Message Format:** Imperative, single-purpose summary tied to the story/phase when applicable (e.g., `Sprint 1 (US-05): Implement 72-bit Word72 and SpeedFloat normalization`).
- **GitHub Automation & Templates:**
  - CI pipeline: [`.github/workflows/rust.yml`](../.github/workflows/rust.yml) (runs on `push` and `pull_request` for `master`, `develop`, and `phase-*/*`).
  - User story template: [`.github/ISSUE_TEMPLATE/user-story.md`](../.github/ISSUE_TEMPLATE/user-story.md).
  - Sprint PR checklist: [`.github/PULL_REQUEST_TEMPLATE.md`](../.github/PULL_REQUEST_TEMPLATE.md).

---

## 5. Definition of Ready (DoR) & Definition of Done (DoD)

### Definition of Ready (Before Starting a Sprint / Story)
1. Story references exact SpeedCo I manual section(s)/page(s) in `102632810-05-01-acc.pdf` and [ARCHITECTURE-AND-SPEC.md](ARCHITECTURE-AND-SPEC.md).
2. Red-phase TDD test cases exist and compile cleanly under `tests/phase_*.rs`.
3. Story points (Fibonacci `1, 2, 3, 5, 8`) and dependencies on earlier phases are clear.

### Definition of Done (Before Completing a Phase & Merging to `master`)
1. Target phase tests pass in both debug (`cargo test`) and release (`cargo test --release`) profiles without relaxing assertions, and all earlier phases remain green. *(On hosts where Windows Application Control blocks local test `.exe` execution with `os error 4551`, verification is completed via CI in `.github/workflows/rust.yml` plus local `cargo test --no-run` / `cargo test --release --no-run`.)*
2. `cargo build`, `cargo build --release`, and `cargo doc --no-deps` complete with **0 compiler or rustdoc warnings**.
3. `cargo package --list --allow-dirty` produces **0 warnings** and excludes `102632810-05-01-acc.pdf`.
4. Relevant docs (`docs/AGILE-SPRINTS.md`, `docs/TIMELINE-AND-BRANCHES.md`, `docs/TDD-TEST-SPECIFICATION.md`) reflect updated story status and test counts.
5. Sprint PR is merged into `develop`, CI passes, `develop` is merged into `master`, and remaining downstream `phase-*` branches are synced with `develop`.

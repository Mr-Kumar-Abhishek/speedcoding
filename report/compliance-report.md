# Compliance Report — `speedcoding` (`v0.1.0-alpha.5`)

## 1. Executive Summary & Audit Metadata

| Audit Attribute | Observed Value |
|---|---|
| **Crate / Version** | `speedcoding` `v0.1.0-alpha.5` (Rust `edition = "2024"`) |
| **Repository** | `https://github.com/Mr-Kumar-Abhishek/speedcoding` |
| **Sprint 4 Code Commit (`phase-4/control-addressing`)** | `5e9aa3355c6a31bbe589c3f91387715fbe2e4f50` (`5e9aa33`, CI Run `#44980132581`) |
| **Sprint 4 Phase Branch Target** | `phase-4/control-addressing` -> `develop` -> `master`, synced to `phase-1..3` & `phase-5/io-storage`..`phase-6/checking-listing-timing` |
| **Toolchain (Local)** | `cargo` 1.95.0 / `rustc` 1.95.0 (`x86_64-pc-windows-msvc`) |
| **Compliance Status** | **Pass with Open Development Gaps** (Sprint 4 / Phases 1–4 Green: 44/66 TDD + 5/5 benchmarks + strict `--all-targets` Clippy passing; Phases 5–6 pending Sprints 5–7) |

This report audits the `speedcoding` crate across legal/IP licensing, dependency hygiene, historical manual traceability against the 1953 *IBM 701 Speedcoding System* specification, Rust safety and release-artifact integrity (`v0.1.0-alpha.1`–`v0.1.0-alpha.5`), Agile/CI/Pages governance, and remaining open gaps.

---

## 2. License, Dependency & IP Compliance

### 2.1 Crate & Third-Party Dependency Licensing
Output from `cargo tree -e normal,dev,build -f "{p} ({l})"`:

```text
speedcoding v0.1.0-alpha.5 (D:\big-projects\speedcoding) (MIT OR Apache-2.0)
└── mpc v0.1.0 (https://github.com/Mr-Kumar-Abhishek/mpc-rs.git#011d12ec) (MIT)
```

| Component | Version / Source | License | Transitive Dependencies | Compliance Status |
|---|---|---|---|---|
| `speedcoding` | `v0.1.0-alpha.5` (workspace crate) | `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`) | 1 direct (`mpc`) | Compliant |
| `mpc` | `v0.1.0` (`mpc-rs.git#011d12ec`, `Copyright (c) 2025 Abhishek Kumar`) | `MIT` | 0 transitive dependencies | Compatible with `MIT OR Apache-2.0` |

### 2.2 Historical Reference PDF Attribution & Exclusion (`102632810-05-01-acc.pdf`)
- **Archival Document:** `102632810-05-01-acc.pdf` (7,718,762 bytes, 67 pages) — scanned copy of the 1953 *IBM 701 Speedcoding System* manual (Introduction dated June 19, 1953 by Cuthbert Hurd, crediting John Backus et al.; Sections 1–12 and Appendices 1–2; XMP metadata `Collection of the Computer History Museum, 102632810`, iTextSharp 2018-02-12).
- **License Separation (`NOTICE`):** `NOTICE` (`1639` B) explicitly carves `102632810-05-01-acc.pdf` out of the crate's `MIT OR Apache-2.0` grant and documents historical provenance and independent implementation boundaries.
- **Cargo Packaging Exclusion:** `Cargo.toml` specifies `exclude = ["102632810-05-01-acc.pdf"]`, verified absent in `cargo package --list --allow-dirty` (`38` packaged files, `0` PDF matches).
- **Distribution Exclusions:** Excluded from `speedcoding-v0.1.0-alpha.5-x86_64-pc-windows-msvc.zip` release assets and excluded from GitHub Pages deployment (`.github/workflows/gh-pages.yml` deploys only `docs/index.html`, `docs/assets/*`, `docs/*.md`, `report/index.html`, `report/*.md`, and Rustdoc).

---

## 3. 1953 SpeedCo I Manual Traceability Matrix

Every phase module and test suite maps directly to Sections 1–12 and Appendices 1–2 of the 1953 *IBM 701 Speedcoding System* manual (71 total Rust tests: 1 library unit test, 5 benchmark tests, and 65 phase TDD tests; **49 passing tests verified in Linux CI** = 1 unit + 5 benchmarks + 11 Phase 1 + 10 Phase 2 + 10 Phase 3 + 12 Phase 4 TDD tests):

| Module / Target | Phase / Scope | Manual Sections Covered | Test File & Count | Current Execution Status |
|---|---|---|---|---|
| `src/lib.rs` | Crate root & parser library check | — (`mpc` integration) | `src/lib.rs` (`mpc_library_inclusion`, **1** test) | **Passing** in Linux CI (`0.00s`, Run `#44980132581`) |
| `src/main.rs` | Thin CLI binary (`speedcoding`) | — | `0` binary unit tests | Compiles cleanly in Debug & Release |
| `src/parser.rs` (bench target) | Speedcoding parser throughput | Sec. 5, 7 | `tests/bench_speedcoding.rs` (**5** tests) | **Passing** in Linux CI (`5 passed; 0 failed`; Debug `0.12s`, Release `0.03s`, Run `#44980132581`) |
| `src/core_types.rs` | Phase 1: 72-bit word, float/half-word, opcodes, accumulators | Sec. 1-1, 2-1, 3-1..3-2, 4-2, 4-4, 5-1..5-5, 9-11, 11-2 | `tests/phase_1_core_types.rs` (**11** tests) | **Passing** in Linux CI (`11 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44980132581`) |
| `src/parser.rs` | Phase 2: Card/assembly text parsing (`mpc`) | Sec. 5, 7-1..7-4, 7-10..7-17, 9-5..9-6, 10-2 | `tests/phase_2_parser.rs` (**10** tests) | **Passing** in Linux CI (`10 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44980132581`) |
| `src/arithmetic.rs` | Phase 3: Floating/fixed ops, scaling, math subroutines | Sec. 3-3..3-4, 6, Appendix 1-2 | `tests/phase_3_arithmetic_math.rs` (**10** tests) | **Passing** in Linux CI (`10 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44980132581`) |
| `src/control.rs` | Phase 4: Transfers, index registers, address modification | Sec. 4-1..4-3, 5-5, 8-1..8-3, 9-1..9-12 | `tests/phase_4_control_addressing.rs` (**12** tests) | **Passing** in Linux CI (**`12 passed; 0 failed`** in Debug `0.00s` & Release `0.00s`, Run `#44980132581`) |
| `src/io.rs` | Phase 5: Cards, printer, magnetic drum, tape storage | Sec. 7-4..7-19 | `tests/phase_5_io_storage.rs` (**11** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/diagnostics.rs` | Phase 6: Checking, listing, timing analysis | Sec. 10-1..10-4, 11-1..11-2, 12-1..12-5, Appendix 1-2 | `tests/phase_6_checking_listing_timing.rs` (**11** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |

---

## 4. Rust Safety, Packaging & Release Integrity

### 4.1 Code Safety & Build Hygiene
- **Unsafe Code Audit:** `Select-String -Path "src\*.rs","tests\*.rs" -Pattern "\bunsafe\b"` returned `unsafe_matches=0` (zero `unsafe` blocks across all source and test files after Sprint 4 `src/control.rs` implementation).
- **Clippy & Compiler Warnings:** Clean compilation (`0` warnings) across `cargo clippy --all-targets -- -D warnings` (Linux CI Run `#44980132581`), `cargo build`, `cargo test --no-run`, `cargo build --release`, `cargo test --release --no-run`, `cargo doc --no-deps`, and `cargo package --list --allow-dirty`.
- **Local Windows Application Control Constraint:** Executing locally compiled `.exe` binaries (`cargo test` runtime, `target\release\speedcoding.exe`, and local `cargo clippy`) is blocked on the Windows development host by Application Control policy (`os error 4551`). Execution and Clippy verification run in GitHub Actions `ubuntu-latest`.

### 4.2 Sprint 4 Pre-Release Asset Verification (`v0.1.0-alpha.5`)
- **Tag / Target:** Annotated tag `v0.1.0-alpha.5` (`isPrerelease: true`), built from Sprint 4 Release profile (`cargo build --release`).
- **Release URL:** `https://github.com/Mr-Kumar-Abhishek/speedcoding/releases/tag/v0.1.0-alpha.5`

| Artifact | Size (Bytes) | Verified SHA-256 Digest | Compliance Notes |
|---|---|---|---|
| `speedcoding-v0.1.0-alpha.5-x86_64-pc-windows-msvc.zip` | `384447` | `50d70046689b55507e0e1383bdf062c8d14bdaafca161926161867b1de93fdca` | Sprint 4 binary archive (excludes `102632810-05-01-acc.pdf`) |
| `speedcoding.exe` *(inside zip)* | `138240` | `6a766e6e95cd74ab66c4f5991243f236ecd3f07f8231578aba6842cbbad27458` | Sprint 4 Release CLI binary |
| `libspeedcoding.rlib` *(inside zip)* | `870712` | `000c2695951671a626ac878df87b5ad76c52332fd76178333e22235b663316d2` | Sprint 4 Release Rust library (`core_types` + `parser` + `arithmetic` + `control`) |
| `LICENSE-MIT` *(inside zip)* | `1102` | — | MIT license notice bundled |
| `LICENSE-APACHE` *(inside zip)* | `10280` | — | Apache-2.0 license notice bundled |
| `NOTICE` *(inside zip)* | `1639` | — | Historical reference carve-out bundled |
| `SHA256SUMS.txt` | `288` | — | Standalone checksum manifest on release |

*(Prior pre-releases `v0.1.0-alpha.1`, `v0.1.0-alpha.2`, `v0.1.0-alpha.3`, and `v0.1.0-alpha.4` remain archived unchanged.)*

---

## 5. Agile Governance, Branch Alignment & CI Verification

### 5.1 Documentation & Templates
- **Agile & Architecture Documentation (`docs/`):** `docs/README.md`, `docs/index.html` + `docs/assets/*`, `docs/AGILE-SPRINTS.md`, `docs/TIMELINE-AND-BRANCHES.md`, `docs/TDD-TEST-SPECIFICATION.md`, `docs/ARCHITECTURE-AND-SPEC.md`, and `report/index.html` + `report/*.md`.
- **GitHub Repository Templates:** `.github/ISSUE_TEMPLATE/user-story.md` and `.github/PULL_REQUEST_TEMPLATE.md`.

### 5.2 CI/CD & Pages Workflows
- **`.github/workflows/rust.yml`:** Triggers on `master`, `develop`, `"phase-*/*"`; executes `cargo build --verbose`, `cargo clippy --all-targets -- -D warnings`, `cargo test --lib --bins --verbose`, `cargo test --test bench_speedcoding --verbose -- --nocapture`, `cargo test --test phase_1_core_types --verbose`, `cargo test --test phase_2_parser --verbose`, `cargo test --test phase_3_arithmetic_math --verbose`, `cargo test --test phase_4_control_addressing --verbose`, `cargo test --tests --no-run --verbose`, `cargo build --release --verbose`, `cargo test --release --test bench_speedcoding --verbose -- --nocapture`, `cargo test --release --test phase_1_core_types --verbose`, `cargo test --release --test phase_2_parser --verbose`, `cargo test --release --test phase_3_arithmetic_math --verbose`, `cargo test --release --test phase_4_control_addressing --verbose`, `cargo doc --no-deps`, and `cargo package --list --allow-dirty`.
- **`.github/workflows/gh-pages.yml`:** Publishes `docs/index.html`, `docs/assets/*`, `docs/*.md`, `report/index.html`, `report/*.md`, and Rustdoc output to `gh-pages` under concurrency group `"gh-pages"`.
- **Verified Workflow Runs:**
  - `Rust Sprint CI` Run `#44980132581` (`phase-4/control-addressing`, `5e9aa33`, `25s`): `completed` / `success` (`12/12` `phase_4_control_addressing` + `10/10` `phase_3_arithmetic_math` + `10/10` `phase_2_parser` + `11/11` `phase_1_core_types` in Debug & Release + `5/5` benchmarks + `1/1` unit test + `cargo clippy --all-targets -- -D warnings`).
  - `Rust Sprint CI` Run `#44966484440` & `Deploy GitHub Pages` Run `#44966484424` (`master`, `7cf6e22`): `completed` / `success`.

---

## 6. Open Gaps & Remediation Plan

| # | Open Compliance / Engineering Gap | Severity | Planned Remediation / Resolution | Target Milestone |
|---|---|---|---|---|
| 1 | **Red-phase `todo!()` stubs remain for Phases 5–6 (`22` phase TDD tests)** | Expected (Sprint 4 complete) | Implement `io` and `diagnostics` sequentially via TDD Green/Refactor cycles and enable each phase in CI as completed | Sprints 5–7 (`phase-5` .. `phase-6`) |
| 2 | **`mpc` dependency pinned via Git URL (`mpc-rs.git`) rather than crates.io version** | Medium (blocks `cargo publish`) | Publish or vendor/pin a crates.io release of `mpc` in `Cargo.toml` before cutting the first crates.io release | Pre-`v1.0.0` release prep |
| 3 | **Local `cargo clippy --all-targets` blocked by Windows Application Control (`os error 4551`)** | **Resolved (Sprint 3)** | `cargo clippy --all-targets -- -D warnings` enabled in `.github/workflows/rust.yml` on `ubuntu-latest` and passing (`0` warnings, Run `#44980132581`) | Closed in Sprint 3 (verified Sprint 4) |

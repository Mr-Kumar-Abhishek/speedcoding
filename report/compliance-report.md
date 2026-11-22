# Compliance Report — `speedcoding` (`v0.1.0-beta.1`)

## 1. Executive Summary & Audit Metadata

| Audit Attribute | Observed Value |
|---|---|
| **Crate / Version** | `speedcoding` `v0.1.0-beta.1` (Rust `edition = "2024"`) |
| **Repository** | `https://github.com/Mr-Kumar-Abhishek/speedcoding` |
| **Sprint 6 Code Commit (`phase-6/checking-listing-timing`)** | `a06e3c02f7bf1d418e26a5e11390403993e9fb9c` (`a06e3c0`, CI Run `#44984107348`) |
| **Sprint 6 Phase Branch Target** | `phase-6/checking-listing-timing` -> `develop` -> `master`, synced to `phase-1..5` |
| **Toolchain (Local)** | `cargo` 1.95.0 / `rustc` 1.95.0 (`x86_64-pc-windows-msvc`) |
| **Compliance Status** | **Pass — All Phases 1–6 Complete (`71/71` Tests Green)** (`66/66` TDD + `5/5` benchmarks + strict `--all-targets` Clippy passing; `0` remaining `todo!()` stubs; `1` non-blocking pre-`v1.0.0` `mpc` publish gap) |

This report audits the `speedcoding` crate across legal/IP licensing, dependency hygiene, historical manual traceability against the 1953 *IBM 701 Speedcoding System* specification, Rust safety and release-artifact integrity (`v0.1.0-alpha.1`–`v0.1.0-beta.1`), Agile/CI/Pages governance, and remaining open gaps.

---

## 2. License, Dependency & IP Compliance

### 2.1 Crate & Third-Party Dependency Licensing
Output from `cargo tree -e normal,dev,build -f "{p} ({l})"`:

```text
speedcoding v0.1.0-beta.1 (D:\big-projects\speedcoding) (MIT OR Apache-2.0)
└── mpc v0.1.0 (https://github.com/Mr-Kumar-Abhishek/mpc-rs.git#011d12ec) (MIT)
```

| Component | Version / Source | License | Transitive Dependencies | Compliance Status |
|---|---|---|---|---|
| `speedcoding` | `v0.1.0-beta.1` (workspace crate) | `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`) | 1 direct (`mpc`) | Compliant |
| `mpc` | `v0.1.0` (`mpc-rs.git#011d12ec`, `Copyright (c) 2025 Abhishek Kumar`) | `MIT` | 0 transitive dependencies | Compatible with `MIT OR Apache-2.0` |

### 2.2 Historical Reference PDF Attribution & Exclusion (`102632810-05-01-acc.pdf`)
- **Archival Document:** `102632810-05-01-acc.pdf` (7,718,762 bytes, 67 pages) — scanned copy of the 1953 *IBM 701 Speedcoding System* manual (Introduction dated June 19, 1953 by Cuthbert Hurd, crediting John Backus et al.; Sections 1–12 and Appendices 1–2; XMP metadata `Collection of the Computer History Museum, 102632810`, iTextSharp 2018-02-12).
- **License Separation (`NOTICE`):** `NOTICE` (`1639` B) explicitly carves `102632810-05-01-acc.pdf` out of the crate's `MIT OR Apache-2.0` grant and documents historical provenance and independent implementation boundaries.
- **Cargo Packaging Exclusion:** `Cargo.toml` specifies `exclude = ["102632810-05-01-acc.pdf"]`, verified absent in `cargo package --list --allow-dirty` (`38` packaged files, `0` PDF matches); `.gitignore` ignores `102632810-05-01-acc*`.
- **Distribution Exclusions:** Excluded from `speedcoding-v0.1.0-beta.1-x86_64-pc-windows-msvc.zip` release assets and excluded from GitHub Pages deployment (`.github/workflows/gh-pages.yml` deploys only `docs/index.html`, `docs/assets/*`, `docs/*.md`, `report/index.html`, `report/*.md`, and Rustdoc).

---

## 3. 1953 SpeedCo I Manual Traceability Matrix

Every phase module and test suite maps directly to Sections 1–12 and Appendices 1–2 of the 1953 *IBM 701 Speedcoding System* manual (all **71** Rust tests — `1` library unit test, `5` benchmark tests, and `65` phase TDD tests across Phases 1–6 — **verified passing in both Debug & Release Linux CI**):

| Module / Target | Phase / Scope | Manual Sections Covered | Test File & Count | Current Execution Status |
|---|---|---|---|---|
| `src/lib.rs` | Crate root & parser library check | — (`mpc` integration) | `src/lib.rs` (`mpc_library_inclusion`, **1** test) | **Passing** in Linux CI (`0.00s`, Run `#44984107348`) |
| `src/main.rs` | Thin CLI binary (`speedcoding`) | — | `0` binary unit tests | Compiles cleanly in Debug & Release |
| `src/parser.rs` (bench target) | Speedcoding parser throughput | Sec. 5, 7 | `tests/bench_speedcoding.rs` (**5** tests) | **Passing** in Linux CI (`5 passed; 0 failed`; Debug `0.15s`, Release `0.03s`, Run `#44984107348`) |
| `src/core_types.rs` | Phase 1: 72-bit word, float/half-word, opcodes, accumulators | Sec. 1-1, 2-1, 3-1..3-2, 4-2, 4-4, 5-1..5-5, 9-11, 11-2 | `tests/phase_1_core_types.rs` (**11** tests) | **Passing** in Linux CI (`11 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44984107348`) |
| `src/parser.rs` | Phase 2: Card/assembly text parsing (`mpc`) | Sec. 5, 7-1..7-4, 7-10..7-17, 9-5..9-6, 10-2 | `tests/phase_2_parser.rs` (**10** tests) | **Passing** in Linux CI (`10 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44984107348`) |
| `src/arithmetic.rs` | Phase 3: Floating/fixed ops, scaling, math subroutines | Sec. 3-3..3-4, 6, Appendix 1-2 | `tests/phase_3_arithmetic_math.rs` (**10** tests) | **Passing** in Linux CI (`10 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44984107348`) |
| `src/control.rs` | Phases 4–6: Transfers, `R`-quantities, AC, `load_deck`, I/O `OP1`, checking loops & selective listing | Sec. 4-1..4-3, 5-5, 7-6..7-19, 8-1..8-3, 9-1..9-12, 10–11 | `tests/phase_4_control_addressing.rs` (**12** tests) + Phases 5–6 integration | **Passing** in Linux CI (`12 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44984107348`) |
| `src/io.rs` | Phase 5: Cards, printer, magnetic drum, tape storage | Sec. 7-4..7-19 | `tests/phase_5_io_storage.rs` (**11** tests) | **Passing** in Linux CI (`11 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44984107348`) |
| `src/diagnostics.rs` | Phase 6: Checking loops, selective listing, Section 12 timing | Sec. 10-1..10-4, 11-1..11-2, 12-1..12-5, Appendix 1-2 | `tests/phase_6_checking_listing_timing.rs` (**11** tests) | **Passing** in Linux CI (**`11 passed; 0 failed`** in Debug `0.00s` & Release `0.00s`, Run `#44984107348`) |

---

## 4. Rust Safety, Packaging & Release Integrity

### 4.1 Code Safety & Build Hygiene
- **Unsafe & Stub Audit:** `Select-String -Path "src\*.rs","tests\*.rs" -Pattern "\bunsafe\b|todo!\(|unimplemented!\("` returned `unsafe_or_todo_hits=0` (zero `unsafe` blocks and zero `todo!()`/`unimplemented!()` stubs remaining across all source and test files after Sprint 6).
- **Clippy & Compiler Warnings:** Clean compilation (`0` warnings) across `cargo clippy --all-targets -- -D warnings` (Linux CI Run `#44984107348`), `cargo build`, `cargo test --no-run`, `cargo build --release`, `cargo test --release --no-run`, `cargo doc --no-deps`, and `cargo package --list --allow-dirty`.
- **Prior Test Fixes Transparency:** Across Sprints 3 and 5, two Red-phase test defects were corrected against the 1953 manual (`tests/phase_3_arithmetic_math.rs` `odd * odd` MPY example & `tests/phase_5_io_storage.rs` two 1-char 80-col card typos); all `11` tests in `tests/phase_6_checking_listing_timing.rs` passed unmodified on the first run.
- **Local Windows Application Control Constraint:** Executing locally compiled `.exe` binaries (`cargo test` runtime, `target\release\speedcoding.exe`, and local `cargo clippy`) is blocked on the Windows development host by Application Control policy (`os error 4551`). Execution and Clippy verification run in GitHub Actions `ubuntu-latest`.

### 4.2 Sprint 6 Beta Pre-Release Asset Verification (`v0.1.0-beta.1`)
- **Tag / Target:** Annotated tag `v0.1.0-beta.1` (`isPrerelease: true`), built from Sprint 6 Release profile (`cargo build --release`).
- **Release URL:** `https://github.com/Mr-Kumar-Abhishek/speedcoding/releases/tag/v0.1.0-beta.1`

| Artifact | Size (Bytes) | Verified SHA-256 Digest | Compliance Notes |
|---|---|---|---|
| `speedcoding-v0.1.0-beta.1-x86_64-pc-windows-msvc.zip` | `442966` | `aac0390dbdd24edee267ae6d364acab4069c38855e3555a44f66fe97e369835e` | Sprint 6 binary archive (excludes `102632810-05-01-acc.pdf`) |
| `speedcoding.exe` *(inside zip)* | `138240` | `261caa2e9d99b5554daeed8f2bf577a2f446e5a1cc3da30411fd0f7efad41f1e` | Sprint 6 Release CLI binary |
| `libspeedcoding.rlib` *(inside zip)* | `1015220` | `50e44d7f658c569d0b2203beb0d04a5a33cc39e9a0f1d055ce41a2e11efc7067` | Sprint 6 Complete Release Rust library (Phases 1–6) |
| `README.md` *(inside zip)* | `5352` | — | Sprint 6 crate overview bundled |
| `CHANGELOG-v0.1.0-beta.1.md` *(inside zip)* | `2621` | — | Sprint 6 release notes bundled |
| `LICENSE-MIT` *(inside zip)* | `1102` | — | MIT license notice bundled |
| `LICENSE-APACHE` *(inside zip)* | `10280` | — | Apache-2.0 license notice bundled |
| `NOTICE` *(inside zip)* | `1639` | — | Historical reference carve-out bundled |
| `SHA256SUMS.txt` | `290` | — | Standalone checksum manifest on release |

*(Prior pre-releases `v0.1.0-alpha.1` through `v0.1.0-alpha.6` remain archived unchanged.)*

---

## 5. Agile Governance, Branch Alignment & CI Verification

### 5.1 Documentation & Templates
- **Agile & Architecture Documentation (`docs/`):** `docs/README.md`, `docs/index.html` + `docs/assets/*`, `docs/AGILE-SPRINTS.md`, `docs/TIMELINE-AND-BRANCHES.md`, `docs/TDD-TEST-SPECIFICATION.md`, `docs/ARCHITECTURE-AND-SPEC.md`, and `report/index.html` + `report/*.md`.
- **GitHub Repository Templates:** `.github/ISSUE_TEMPLATE/user-story.md` and `.github/PULL_REQUEST_TEMPLATE.md`.

### 5.2 CI/CD & Pages Workflows
- **`.github/workflows/rust.yml`:** Triggers on `master`, `develop`, `"phase-*/*"`; executes `cargo build --verbose`, `cargo clippy --all-targets -- -D warnings`, `cargo test --lib --bins --verbose`, `cargo test --test bench_speedcoding --verbose -- --nocapture`, all 6 phase TDD suites (`phase_1_core_types`..`phase_6_checking_listing_timing`) in both Debug and Release profiles, `cargo test --tests --no-run --verbose`, `cargo doc --no-deps`, and `cargo package --list --allow-dirty`.
- **`.github/workflows/gh-pages.yml`:** Publishes `docs/index.html`, `docs/assets/*`, `docs/*.md`, `report/index.html`, `report/*.md`, and Rustdoc output to `gh-pages` under concurrency group `"gh-pages"`.
- **Verified Workflow Runs:**
  - `Rust Sprint CI` Run `#44984107348` (`phase-6/checking-listing-timing`, `a06e3c0`, `21s`): `completed` / `success` (all **71/71** tests in Debug & Release: `11/11` Phase 6 + `11/11` Phase 5 + `12/12` Phase 4 + `10/10` Phase 3 + `10/10` Phase 2 + `11/11` Phase 1 + `5/5` benchmarks + `1/1` unit test + `cargo clippy --all-targets -- -D warnings`).
  - `Rust Sprint CI` Run `#44982982667` & `Deploy GitHub Pages` Run `#44982982635` (`master`, `.gitignore` sync `f6589e3`): `completed` / `success`.

---

## 6. Open Gaps & Remediation Plan

| # | Open Compliance / Engineering Gap | Severity | Planned Remediation / Resolution | Target Milestone |
|---|---|---|---|---|
| 1 | **Red-phase `todo!()` stubs across Phases 1–6 (`66` TDD tests)** | **Resolved (Sprint 6)** | Implemented all 6 modules (`core_types`, `parser`, `arithmetic`, `control`, `io`, `diagnostics`); `0` `todo!()` stubs remain and `71/71` tests pass in Debug & Release (Run `#44984107348`) | Closed in Sprint 6 (`v0.1.0-beta.1`) |
| 2 | **`mpc` dependency pinned via Git URL (`mpc-rs.git`) rather than crates.io version** | Medium (blocks `cargo publish`) | Publish or vendor/pin a crates.io release of `mpc` in `Cargo.toml` before cutting the first crates.io release | Sprint 7 / pre-`v1.0.0` release prep |
| 3 | **Local `cargo clippy --all-targets` blocked by Windows Application Control (`os error 4551`)** | **Resolved (Sprint 3)** | `cargo clippy --all-targets -- -D warnings` enabled in `.github/workflows/rust.yml` on `ubuntu-latest` and passing (`0` warnings, Run `#44984107348`) | Closed in Sprint 3 (verified Sprint 6) |

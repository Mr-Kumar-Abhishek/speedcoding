# Compliance Report — `speedcoding` (`v0.1.0-alpha.3`)

## 1. Executive Summary & Audit Metadata

| Audit Attribute | Observed Value |
|---|---|
| **Crate / Version** | `speedcoding` `v0.1.0-alpha.3` (Rust `edition = "2024"`) |
| **Repository** | `https://github.com/Mr-Kumar-Abhishek/speedcoding` |
| **Sprint 2 Code Commit (`phase-2/parser`)** | `bab514ba5427f1b05dcb6489e47c45b037a43512` (`bab514b`, CI Run `#44946046507`) |
| **Sprint 2 Phase Branch Target** | `phase-2/parser` -> `develop` -> `master`, synced to `phase-1` & `phase-3/arithmetic-math`..`phase-6/checking-listing-timing` |
| **Toolchain (Local)** | `cargo` 1.95.0 / `rustc` 1.95.0 (`x86_64-pc-windows-msvc`) |
| **Compliance Status** | **Pass with Open Development Gaps** (Sprint 2 / Phases 1–2 Green: 22/66 TDD + 5/5 benchmarks passing; Phases 3–6 pending Sprints 3–7) |

This report audits the `speedcoding` crate across legal/IP licensing, dependency hygiene, historical manual traceability against the 1953 *IBM 701 Speedcoding System* specification, Rust safety and release-artifact integrity (`v0.1.0-alpha.1`–`v0.1.0-alpha.3`), Agile/CI/Pages governance, and remaining open gaps.

---

## 2. License, Dependency & IP Compliance

### 2.1 Crate & Third-Party Dependency Licensing
Output from `cargo tree -e normal,dev,build -f "{p} ({l})"`:

```text
speedcoding v0.1.0-alpha.3 (D:\big-projects\speedcoding) (MIT OR Apache-2.0)
└── mpc v0.1.0 (https://github.com/Mr-Kumar-Abhishek/mpc-rs.git#011d12ec) (MIT)
```

| Component | Version / Source | License | Transitive Dependencies | Compliance Status |
|---|---|---|---|---|
| `speedcoding` | `v0.1.0-alpha.3` (workspace crate) | `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`) | 1 direct (`mpc`) | Compliant |
| `mpc` | `v0.1.0` (`mpc-rs.git#011d12ec`, `Copyright (c) 2025 Abhishek Kumar`) | `MIT` | 0 transitive dependencies | Compatible with `MIT OR Apache-2.0` |

### 2.2 Historical Reference PDF Attribution & Exclusion (`102632810-05-01-acc.pdf`)
- **Archival Document:** `102632810-05-01-acc.pdf` (7,718,762 bytes, 67 pages) — scanned copy of the 1953 *IBM 701 Speedcoding System* manual (Introduction dated June 19, 1953 by Cuthbert Hurd, crediting John Backus et al.; Sections 1–12 and Appendices 1–2; XMP metadata `Collection of the Computer History Museum, 102632810`, iTextSharp 2018-02-12).
- **License Separation (`NOTICE`):** `NOTICE` (`1639` B) explicitly carves `102632810-05-01-acc.pdf` out of the crate's `MIT OR Apache-2.0` grant and documents historical provenance and independent implementation boundaries.
- **Cargo Packaging Exclusion:** `Cargo.toml` specifies `exclude = ["102632810-05-01-acc.pdf"]`, verified absent in `cargo package --list --allow-dirty`.
- **Distribution Exclusions:** Excluded from `speedcoding-v0.1.0-alpha.3-x86_64-pc-windows-msvc.zip` release assets and excluded from GitHub Pages deployment (`.github/workflows/gh-pages.yml` deploys only `docs/index.html`, `docs/assets/*`, `docs/*.md`, `report/index.html`, `report/*.md`, and Rustdoc).

---

## 3. 1953 SpeedCo I Manual Traceability Matrix

Every phase module and test suite maps directly to Sections 1–12 and Appendices 1–2 of the 1953 *IBM 701 Speedcoding System* manual (71 total Rust tests: 1 library unit test, 5 benchmark tests, and 65 phase TDD tests; **27 passing tests verified in Linux CI** = 1 unit + 5 benchmarks + 11 Phase 1 + 10 Phase 2 TDD tests):

| Module / Target | Phase / Scope | Manual Sections Covered | Test File & Count | Current Execution Status |
|---|---|---|---|---|
| `src/lib.rs` | Crate root & parser library check | — (`mpc` integration) | `src/lib.rs` (`mpc_library_inclusion`, **1** test) | **Passing** in Linux CI (`0.00s`, Run `#44946046507`) |
| `src/main.rs` | Thin CLI binary (`speedcoding`) | — | `0` binary unit tests | Compiles cleanly in Debug & Release |
| `src/parser.rs` (bench target) | Speedcoding parser throughput | Sec. 5, 7 | `tests/bench_speedcoding.rs` (**5** tests) | **Passing** in Linux CI (`5 passed; 0 failed`; Debug `0.13s`, Release `0.04s`, Run `#44946046507`) |
| `src/core_types.rs` | Phase 1: 72-bit word, float/half-word, opcodes, accumulators | Sec. 1-1, 2-1, 3-1..3-2, 4-2, 4-4, 5-1..5-5, 9-11, 11-2 | `tests/phase_1_core_types.rs` (**11** tests) | **Passing** in Linux CI (`11 passed; 0 failed` in Debug `0.00s` & Release `0.00s`, Run `#44946046507`) |
| `src/parser.rs` | Phase 2: Card/assembly text parsing (`mpc`) | Sec. 5, 7-1..7-4, 7-10..7-17, 9-5..9-6, 10-2 | `tests/phase_2_parser.rs` (**10** tests) | **Passing** in Linux CI (**`10 passed; 0 failed`** in Debug `0.00s` & Release `0.00s`, Run `#44946046507`) |
| `src/arithmetic.rs` | Phase 3: Floating/fixed ops, scaling, math subroutines | Sec. 3-3..3-4, 6, Appendix 1-2 | `tests/phase_3_arithmetic_math.rs` (**10** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/control.rs` | Phase 4: Transfers, index registers, address modification | Sec. 4-1..4-3, 5-5, 8-1..8-3, 9-1..9-12 | `tests/phase_4_control_addressing.rs` (**12** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/io.rs` | Phase 5: Cards, printer, magnetic drum, tape storage | Sec. 7-4..7-19 | `tests/phase_5_io_storage.rs` (**11** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/diagnostics.rs` | Phase 6: Checking, listing, timing analysis | Sec. 10-1..10-4, 11-1..11-2, 12-1..12-5, Appendix 1-2 | `tests/phase_6_checking_listing_timing.rs` (**11** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |

---

## 4. Rust Safety, Packaging & Release Integrity

### 4.1 Code Safety & Build Hygiene
- **Unsafe Code Audit:** `Select-String -Path "src\*.rs","tests\*.rs" -Pattern "\bunsafe\b"` returned `unsafe_matches=0` (zero `unsafe` blocks across all source and test files after Sprint 2 `src/parser.rs` implementation).
- **Compiler Warnings:** Clean compilation (`0` warnings) across `cargo build`, `cargo test --no-run`, `cargo build --release`, `cargo test --release --no-run`, and `cargo doc --no-deps`.
- **Local Windows Application Control Constraint:** Executing locally compiled `.exe` binaries (`cargo test` runtime, `target\release\speedcoding.exe`, and `cargo clippy --all-targets`) is blocked on the Windows development host by Application Control policy (`os error 4551`). Execution verification runs on GitHub Actions `ubuntu-latest`.

### 4.2 Sprint 2 Pre-Release Asset Verification (`v0.1.0-alpha.3`)
- **Tag / Target:** Annotated tag `v0.1.0-alpha.3` (`isPrerelease: true`), built from Sprint 2 Release profile (`cargo build --release`).
- **Release URL:** `https://github.com/Mr-Kumar-Abhishek/speedcoding/releases/tag/v0.1.0-alpha.3`

| Artifact | Size (Bytes) | Verified SHA-256 Digest | Compliance Notes |
|---|---|---|---|
| `speedcoding-v0.1.0-alpha.3-x86_64-pc-windows-msvc.zip` | `357990` | `ecd6e7cf70c8781b3844b4597ca54360681ec0b0876da54bc4e7721122d5edc8` | Sprint 2 binary archive (excludes `102632810-05-01-acc.pdf`) |
| `speedcoding.exe` *(inside zip)* | `138240` | `0605abd8bebca4afcafd7044a166c898ea3b40ce101552319bf09b7394194751` | Sprint 2 Release CLI binary |
| `libspeedcoding.rlib` *(inside zip)* | `811424` | `175c66e46211b418718947cbea8f49df551c926d06d1adc03c907bb9a6cad701` | Sprint 2 Release Rust library (`core_types` + `parser`) |
| `LICENSE-MIT` *(inside zip)* | `1102` | — | MIT license notice bundled |
| `LICENSE-APACHE` *(inside zip)* | `10280` | — | Apache-2.0 license notice bundled |
| `NOTICE` *(inside zip)* | `1639` | — | Historical reference carve-out bundled |
| `SHA256SUMS.txt` | `291` | — | Standalone checksum manifest on release |

*(Prior pre-releases `v0.1.0-alpha.1` and `v0.1.0-alpha.2` remain archived unchanged.)*

---

## 5. Agile Governance, Branch Alignment & CI Verification

### 5.1 Documentation & Templates
- **Agile & Architecture Documentation (`docs/`):** `docs/README.md`, `docs/index.html` + `docs/assets/*`, `docs/AGILE-SPRINTS.md`, `docs/TIMELINE-AND-BRANCHES.md`, `docs/TDD-TEST-SPECIFICATION.md`, `docs/ARCHITECTURE-AND-SPEC.md`, and `report/index.html` + `report/*.md`.
- **GitHub Repository Templates:** `.github/ISSUE_TEMPLATE/user-story.md` and `.github/PULL_REQUEST_TEMPLATE.md`.

### 5.2 CI/CD & Pages Workflows
- **`.github/workflows/rust.yml`:** Triggers on `master`, `develop`, `"phase-*/*"`; executes `cargo build --verbose`, `cargo test --lib --bins --verbose`, `cargo test --test bench_speedcoding --verbose -- --nocapture`, `cargo test --test phase_1_core_types --verbose`, `cargo test --test phase_2_parser --verbose`, `cargo test --tests --no-run --verbose`, `cargo build --release --verbose`, `cargo test --release --test bench_speedcoding --verbose -- --nocapture`, `cargo test --release --test phase_1_core_types --verbose`, `cargo test --release --test phase_2_parser --verbose`, `cargo doc --no-deps`, and `cargo package --list --allow-dirty`.
- **`.github/workflows/gh-pages.yml`:** Publishes `docs/index.html`, `docs/assets/*`, `docs/*.md`, `report/index.html`, `report/*.md`, and Rustdoc output to `gh-pages` under concurrency group `"gh-pages"`.
- **Verified Workflow Runs:**
  - `Rust Sprint CI` Run `#44946046507` (`phase-2/parser`, `bab514b`, `18s`): `completed` / `success` (`10/10` `phase_2_parser` + `11/11` `phase_1_core_types` in Debug & Release + `5/5` benchmarks + `1/1` unit test).
  - `Rust Sprint CI` Run `#44945311533` & `Deploy GitHub Pages` Run `#44945311481` (`master`, `831b957`): `completed` / `success`.

---

## 6. Open Gaps & Remediation Plan

| # | Open Compliance / Engineering Gap | Severity | Planned Remediation | Target Milestone |
|---|---|---|---|---|
| 1 | **Red-phase `todo!()` stubs remain for Phases 3–6 (`44` phase TDD tests)** | Expected (Sprint 2 complete) | Implement `arithmetic`, `control`, `io`, and `diagnostics` sequentially via TDD Green/Refactor cycles and enable each phase in CI as completed | Sprints 3–7 (`phase-3` .. `phase-6`) |
| 2 | **`mpc` dependency pinned via Git URL (`mpc-rs.git`) rather than crates.io version** | Medium (blocks `cargo publish`) | Publish or vendor/pin a crates.io release of `mpc` in `Cargo.toml` before cutting the first crates.io release | Pre-`v1.0.0` release prep |
| 3 | **Local `cargo clippy --all-targets` blocked by Windows Application Control (`os error 4551`)** | Low (host environment) | Add `cargo clippy --all-targets -- -D warnings` to `.github/workflows/rust.yml` on `ubuntu-latest` so lint checks run reliably in CI | Sprint 3 CI hardening |

# Compliance Report — `speedcoding` (`v0.1.0-alpha.1`)

## 1. Executive Summary & Audit Metadata

| Audit Attribute | Observed Value |
|---|---|
| **Crate / Version** | `speedcoding` `v0.1.0-alpha.1` (Rust `edition = "2024"`) |
| **Repository** | `https://github.com/Mr-Kumar-Abhishek/speedcoding` |
| **Audited Commit (`develop` / `master`)** | `9afc854077fec53a766c126e498b2718d019210b` (`9afc854`) |
| **Synced Phase Branches** | `phase-1/core-types`, `phase-2/parser`, `phase-3/arithmetic-math`, `phase-4/control-addressing`, `phase-5/io-storage`, `phase-6/checking-listing-timing` |
| **Pages Branch (`gh-pages`)** | `3a97fa4806b05b4935495a95a5c07d867c8d0b0b` |
| **Toolchain (Local)** | `cargo` 1.95.0 / `rustc` 1.95.0 (`x86_64-pc-windows-msvc`) |
| **Compliance Status** | **Pass with Open Development Gaps** (Sprint 0 Red-phase baseline compliant; Phases 1–6 implementations pending Sprints 1–7) |

This report audits the `speedcoding` crate across legal/IP licensing, dependency hygiene, historical manual traceability against the 1953 *IBM 701 Speedcoding System* specification, Rust safety and release-artifact integrity, Agile/CI/Pages governance, and remaining open gaps.

---

## 2. License, Dependency & IP Compliance

### 2.1 Crate & Third-Party Dependency Licensing
Output from `cargo tree -e normal,dev,build -f "{p} ({l})"`:

```text
speedcoding v0.1.0-alpha.1 (D:\big-projects\speedcoding) (MIT OR Apache-2.0)
└── mpc v0.1.0 (https://github.com/Mr-Kumar-Abhishek/mpc-rs.git#011d12ec) (MIT)
```

| Component | Version / Source | License | Transitive Dependencies | Compliance Status |
|---|---|---|---|---|
| `speedcoding` | `v0.1.0-alpha.1` (workspace crate) | `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`) | 1 direct (`mpc`) | Compliant |
| `mpc` | `v0.1.0` (`mpc-rs.git#011d12ec`, `Copyright (c) 2025 Abhishek Kumar`) | `MIT` | 0 transitive dependencies | Compatible with `MIT OR Apache-2.0` |

### 2.2 Historical Reference PDF Attribution & Exclusion (`102632810-05-01-acc.pdf`)
- **Archival Document:** `102632810-05-01-acc.pdf` (7,718,762 bytes, 67 pages) — scanned copy of the 1953 *IBM 701 Speedcoding System* manual (Introduction dated June 19, 1953 by Cuthbert Hurd, crediting John Backus et al.; Sections 1–12 and Appendices 1–2; XMP metadata `Collection of the Computer History Museum, 102632810`, iTextSharp 2018-02-12).
- **License Separation (`NOTICE`):** `NOTICE` (`1639` B) explicitly carves `102632810-05-01-acc.pdf` out of the crate's `MIT OR Apache-2.0` grant and documents historical provenance and independent implementation boundaries.
- **Cargo Packaging Exclusion:** `Cargo.toml` specifies `exclude = ["102632810-05-01-acc.pdf"]`, verified absent in `cargo package --list --allow-dirty`.
- **Distribution Exclusions:** Excluded from `speedcoding-v0.1.0-alpha.1-x86_64-pc-windows-msvc.zip` release assets and excluded from GitHub Pages deployment (`.github/workflows/gh-pages.yml` deploys only `docs/index.html`, `docs/*.md`, `report/*.md`, and Rustdoc).

---

## 3. 1953 SpeedCo I Manual Traceability Matrix

Every phase module and test suite maps directly to Sections 1–12 and Appendices 1–2 of the 1953 *IBM 701 Speedcoding System* manual (71 total Rust tests: 1 library unit test, 5 benchmark tests, and 65 phase TDD tests):

| Module / Target | Phase / Scope | Manual Sections Covered | Test File & Count | Current Execution Status |
|---|---|---|---|---|
| `src/lib.rs` | Crate root & parser library check | — (`mpc` integration) | `src/lib.rs` (`mpc_library_inclusion`, **1** test) | **Passing** in Linux CI (`0.00s`, Run `#44938969997`) |
| `src/main.rs` | Thin CLI binary (`speedcoding`) | — | `0` binary unit tests | Compiles cleanly in Debug & Release |
| `src/parser.rs` (bench target) | Speedcoding parser throughput | Sec. 5, 7 | `tests/bench_speedcoding.rs` (**5** tests) | **Passing** in Linux CI (`5 passed; 0 failed`; Debug `0.06s`, Release `0.02s`, Run `#44938969997`) |
| `src/core_types.rs` | Phase 1: 72-bit word, float/half-word, opcodes, accumulators | Sec. 1-1, 2-1, 3-1..3-2, 4-2, 4-4, 5-1..5-5, 9-11, 11-2 | `tests/phase_1_core_types.rs` (**11** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/parser.rs` | Phase 2: Card/assembly text parsing (`mpc`) | Sec. 5, 7-1..7-4, 7-10..7-17, 9-5..9-6, 10-2 | `tests/phase_2_parser.rs` (**10** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/arithmetic.rs` | Phase 3: Floating/fixed ops, scaling, math subroutines | Sec. 3-3..3-4, 6, Appendix 1-2 | `tests/phase_3_arithmetic_math.rs` (**10** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/control.rs` | Phase 4: Transfers, index registers, address modification | Sec. 4-1..4-3, 5-5, 8-1..8-3, 9-1..9-12 | `tests/phase_4_control_addressing.rs` (**12** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/io.rs` | Phase 5: Cards, printer, magnetic drum, tape storage | Sec. 7-4..7-19 | `tests/phase_5_io_storage.rs` (**11** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |
| `src/diagnostics.rs` | Phase 6: Checking, listing, timing analysis | Sec. 10-1..10-4, 11-1..11-2, 12-1..12-5, Appendix 1-2 | `tests/phase_6_checking_listing_timing.rs` (**11** tests) | Compiles `0` warnings; Red-phase `todo!()` stubs |

---

## 4. Rust Safety, Packaging & Release Integrity

### 4.1 Code Safety & Build Hygiene
- **Unsafe Code Audit:** `Select-String -Path "src\*.rs","tests\*.rs" -Pattern "\bunsafe\b"` returned `unsafe_matches=0` (zero `unsafe` blocks across all source and test files).
- **Compiler Warnings:** Clean compilation (`0` warnings) across `cargo build`, `cargo test --no-run`, and `cargo doc --no-deps`.
- **Local Windows Application Control Constraint:** Executing locally compiled `.exe` binaries (`cargo test` runtime, `target\release\speedcoding.exe`, and `cargo clippy --all-targets`) is blocked on the Windows development host by Application Control policy (`os error 4551`). Execution verification is therefore performed on GitHub Actions `ubuntu-latest`.

### 4.2 Pre-Release Asset Verification (`v0.1.0-alpha.1`)
- **Tag / Target:** Annotated tag `v0.1.0-alpha.1` (`7832185231d681163fa81e65046f29850ab63cff` -> `c7689af`), marked `isPrerelease: true`.
- **Release URL:** `https://github.com/Mr-Kumar-Abhishek/speedcoding/releases/tag/v0.1.0-alpha.1`

| Artifact | Size (Bytes) | Verified SHA-256 Digest | Compliance Notes |
|---|---|---|---|
| `speedcoding-v0.1.0-alpha.1-x86_64-pc-windows-msvc.zip` | `262679` | `34bdbd1f00fd023f0b8fd46d39c1769bf799bbcd0146cc93c7c9ad90aef387ee` | Published binary archive (excludes `102632810-05-01-acc.pdf`) |
| `speedcoding.exe` *(inside zip)* | `138240` | `b41325859765f4f86e6b9f55102b0a471a57f14e835d232e7a4f3f29ca81fc79` | Release binary |
| `libspeedcoding.rlib` *(inside zip)* | `501252` | `1cdc196940b093f005e272c114ea4e1d6ae2d4c2090313f623bf056842dda6fd` | Release Rust library artifact |
| `LICENSE-MIT` *(inside zip)* | `1102` | — | MIT license notice bundled |
| `LICENSE-APACHE` *(inside zip)* | `10280` | — | Apache-2.0 license notice bundled |
| `NOTICE` *(inside zip)* | `1639` | — | Historical reference carve-out bundled |
| `SHA256SUMS.txt` | `288` | — | Standalone checksum manifest on release |

---

## 5. Agile Governance, Branch Alignment & CI Verification

### 5.1 Documentation & Templates
- **Agile & Architecture Documentation (`docs/`):**
  - `docs/README.md` (documentation index)
  - `docs/index.html` (interactive GitHub Pages portal)
  - `docs/AGILE-SPRINTS.md` (2-week Scrum cadence, Sprints 0–7, backlog `US-01..US-14`, DoR/DoD)
  - `docs/TIMELINE-AND-BRANCHES.md` (Gitflow + 6 phase branches + release schedule)
  - `docs/TDD-TEST-SPECIFICATION.md` (complete specification of Red-phase TDD test cases)
  - `docs/ARCHITECTURE-AND-SPEC.md` (SpeedCo I 72-bit architectural reference)
- **GitHub Repository Templates:** `.github/ISSUE_TEMPLATE/user-story.md` and `.github/PULL_REQUEST_TEMPLATE.md`.

### 5.2 CI/CD & Pages Workflows (Baseline Verification at `9afc854`)
- **`.github/workflows/rust.yml`:** Triggers on `master`, `develop`, `"phase-*/*"`; executes `cargo build --verbose`, `cargo test --lib --bins --verbose`, `cargo test --test bench_speedcoding --verbose -- --nocapture`, `cargo test --tests --no-run --verbose`, `cargo build --release --verbose`, `cargo test --release --test bench_speedcoding --verbose -- --nocapture`, `cargo doc --no-deps`, and `cargo package --list --allow-dirty`.
- **`.github/workflows/gh-pages.yml`:** Publishes `docs/index.html`, `docs/*.md`, `report/*.md`, and Rustdoc output to `gh-pages`.
- **Verified Workflow Runs (Baseline Prior to Report Commit):**
  - `Rust Sprint CI` Run `#44939829185` (`master`, `9afc854`): `completed` / `success` (`15s`)
  - `Deploy GitHub Pages` Run `#44939829111` (`master`, `9afc854`): `completed` / `success` (`21s`)
  - `Rust Sprint CI` Run `#44938969997` (`develop`, `239bf34`): `completed` / `success` (benchmark capture run)

---

## 6. Open Gaps & Remediation Plan

| # | Open Compliance / Engineering Gap | Severity | Planned Remediation | Target Milestone |
|---|---|---|---|---|
| 1 | **Red-phase `todo!()` implementations in `src/` (`65` phase TDD tests cannot pass yet)** | Expected (Sprint 0) | Implement `core_types`, `parser`, `arithmetic`, `control`, `io`, and `diagnostics` sequentially via TDD Green/Refactor cycles and enable full `cargo test` in CI as each phase completes | Sprints 1–7 (`phase-1` .. `phase-6`) |
| 2 | **`mpc` dependency pinned via Git URL (`mpc-rs.git`) rather than crates.io version** | Medium (blocks `cargo publish`) | Publish or vendor/pin a crates.io release of `mpc` in `Cargo.toml` before cutting the first crates.io release | Pre-`v1.0.0` release prep |
| 3 | **Local `cargo clippy --all-targets` blocked by Windows Application Control (`os error 4551`)** | Low (host environment) | Add `cargo clippy --all-targets -- -D warnings` to `.github/workflows/rust.yml` on `ubuntu-latest` so lint checks run reliably in CI | Sprint 1 CI hardening |

# `speedcoding` Documentation

Rust library and CLI implementation of the **1953 IBM 701 Speedcoding System (SpeedCo I)**, built with [`mpc-rs`](https://github.com/Mr-Kumar-Abhishek/mpc-rs) for parser combinators and organized around a 2-week Agile Sprint + 6-phase Red-Green-Refactor TDD roadmap.

- **Repository:** <https://github.com/Mr-Kumar-Abhishek/speedcoding>
- **GitHub Pages Portal & Rustdoc:** <https://mr-kumar-abhishek.github.io/speedcoding/>
- **GitHub Pages Reports Portal:** <https://mr-kumar-abhishek.github.io/speedcoding/report/>
- **Crate Version:** `0.1.0-alpha.6` (`edition = "2024"`, Sprint 5 pre-release tag [`v0.1.0-alpha.6`](https://github.com/Mr-Kumar-Abhishek/speedcoding/releases/tag/v0.1.0-alpha.6))
- **Primary Reference Manual:** *IBM 701 Speedcoding System* (IBM Scientific Computing Service, 1953), archived in `102632810-05-01-acc.pdf` (Computer History Museum Catalog ID `102632810`).

---

## Documentation Index

| Document | Description |
|---|---|
| [index.html](index.html) | Interactive GitHub Pages documentation portal (live specs, opcode filter, Mermaid diagrams, reports, and Rustdoc entrypoint) |
| [AGILE-SPRINTS.md](AGILE-SPRINTS.md) | 2-week Agile Sprint workflow, ceremonies, Sprints 0–7 roadmap, Product Backlog (`US-01..US-14`), DoR & DoD |
| [TIMELINE-AND-BRANCHES.md](TIMELINE-AND-BRANCHES.md) | Gitflow + 6-phase branch topology, TDD workflow, milestone schedule, and release targets |
| [TDD-TEST-SPECIFICATION.md](TDD-TEST-SPECIFICATION.md) | Complete specification of all 66 Red-phase TDD test cases across `src/lib.rs` and `tests/phase_1..6` |
| [ARCHITECTURE-AND-SPEC.md](ARCHITECTURE-AND-SPEC.md) | Technical reference distilled from the 1953 SpeedCo I manual: 72-bit layouts, opcodes, cards, I/O, checking, listing, and timing |
| [report/index.html](../report/index.html) | Standalone GitHub Pages HTML viewer for the Benchmark and Compliance reports |
| [speedcoding-benchmark-report.md](../report/speedcoding-benchmark-report.md) | Debug & Release parsing benchmark results (`tests/bench_speedcoding.rs`, CI Run `#44938969997`) |
| [compliance-report.md](../report/compliance-report.md) | License/IP, SpeedCo I manual traceability, safety/SHA-256, and Agile/CI compliance audit |

---

## Crate Module Map & Repository Tooling

```mermaid
flowchart TD
    MPC["mpc (git: mpc-rs)"] --> PAR["parser (Phase 2)"]
    CT["core_types (Phase 1)"] --> PAR
    CT --> ARITH["arithmetic (Phase 3)"]
    CT --> IO["io (Phase 5)"]
    CT --> DIAG["diagnostics (Phase 6)"]
    CT --> CTRL["control (Phase 4 & Machine)"]
    PAR --> CTRL
    ARITH --> CTRL
    IO --> CTRL
    DIAG --> CTRL
    CTRL --> CLI["src/main.rs (speedcoding CLI)"]
```

| Module / Asset | File | Sprint / Phase | Responsibility |
|---|---|---|---|
| `core_types` | [`src/core_types.rs`](../src/core_types.rs) | Sprint 1 / Phase 1 | `Word72`, `Address`, `SpeedFloat`, `DecimalFloat`, `RCode`, `LCode`, 49 `Op1` + 48 `Op2` opcodes, `Instruction`, `ElectrostaticMemory` |
| `parser` | [`src/parser.rs`](../src/parser.rs) | Sprint 2 / Phase 2 | `mpc`-backed mnemonic/line/program parsers and 80-column instruction, data, and loading-control card parsers |
| `arithmetic` | [`src/arithmetic.rs`](../src/arithmetic.rs) | Sprint 3 / Phase 3 | Floating-binary `OP1` arithmetic (`ADD`..`NGDIV`) and Set 1 elementary functions (`SQRT`, `SINE`, `ARTAN`, `EXP`, `LN`) |
| `control` | [`src/control.rs`](../src/control.rs) | Sprint 4–5 / Phases 4–5 | `SpeedCoMachine`, `PC` execution loop, `OP2` transfers, `RQuantities`, `AddressCounter`, location `0025` control tricks, `load_deck` & I/O `OP1` dispatch |
| `io` | [`src/io.rs`](../src/io.rs) | Sprint 5 / Phase 5 | `TapeUnit` (`J..M`) with parity/checksum/status, `DrumUnit` (`P..Q`), `LinePrinter` |
| `diagnostics` | [`src/diagnostics.rs`](../src/diagnostics.rs) | Sprint 6 / Phase 6 | `CheckingState` (`PRCH`/`STCH`/`ECHTR`), selective 10-line `ListingBuffer`, and Section 12 `TimingCalculator` |
| `speedcoding` CLI | [`src/main.rs`](../src/main.rs) | Sprint 0 & 7 | Thin CLI entrypoint (`--help`, `--version`, `info`) |
| CI, Pages & Templates | [`.github/workflows/rust.yml`](../.github/workflows/rust.yml) | Sprint 0 | Sprint CI, [GitHub Pages deploy](../.github/workflows/gh-pages.yml), [User Story template](../.github/ISSUE_TEMPLATE/user-story.md), and [PR template](../.github/PULL_REQUEST_TEMPLATE.md) |

---

## Licensing & Historical Attribution

- **Rust Crate License:** Dual-licensed under the **MIT License** ([`LICENSE-MIT`](../LICENSE-MIT)) or **Apache License, Version 2.0** ([`LICENSE-APACHE`](../LICENSE-APACHE)), declared in [`Cargo.toml`](../Cargo.toml) as `license = "MIT OR Apache-2.0"`.
- **Parser Dependency:** `mpc` (`https://github.com/Mr-Kumar-Abhishek/mpc-rs.git`, v0.1.0), licensed under the MIT License.
- **Historical Documentation Scan:** `102632810-05-01-acc.pdf` (Collection of the Computer History Museum, ID `102632810`) is included solely as historical and educational reference material, is **excluded** from Cargo crate packages (`exclude = ["102632810-05-01-acc.pdf"]`), and is governed by the Computer History Museum Terms of Use (non-commercial, personal, educational, and research fair use). See [`NOTICE`](../NOTICE) for full attribution details.

## Sprint & Story Context
- **Sprint:** `Sprint <N>`
- **Story ID(s):** `US-XX`
- **Branch Flow:** `phase-<N>/<name>` -> `develop` -> `master` (upon phase completion)
- **Target Release Tag (if closing Sprint):** `v0.1.0-alpha.<N>`

## Summary of Changes
<!-- Describe the minimal Green implementation and refactoring completed in this sprint -->

## Red -> Green -> Refactor TDD Checklist
- [ ] Confirmed failing / `todo!()` Red baseline before implementing
- [ ] Implemented minimal safe Rust in `src/` to pass all target tests without relaxing assertions
- [ ] Refactored duplicate logic and verified all prior phase suites remain green

## Verification Gate (Definition of Done)
- [ ] `cargo build --verbose` (0 warnings)
- [ ] `cargo test` for completed phase suite(s) + `--lib --bins` + `bench_speedcoding` (0 failures)
- [ ] `cargo build --release --verbose` & `cargo test --release` (for implemented phases + benchmarks)
- [ ] `cargo doc --no-deps` (0 warnings)
- [ ] `cargo package --list --allow-dirty` (0 warnings; `102632810-05-01-acc.pdf` excluded)
- [ ] Updated `docs/AGILE-SPRINTS.md` backlog status & test counts
- [ ] Merged `develop` -> `master` after CI passes and synced remaining `phase-*` branches

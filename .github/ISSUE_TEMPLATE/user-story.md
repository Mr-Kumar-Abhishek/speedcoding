---
name: Agile User Story
about: Define or track a sprint backlog item for the speedcoding crate
title: "[US-XX] <short story title>"
labels: user-story
assignees: ""
---

## Story Metadata
- **Story ID:** `US-XX`
- **Sprint:** `Sprint <N>`
- **Target Branch:** `phase-<N>/<branch-name>` (or `sprint-<N>/us-XX-<slug>`)
- **Story Points (Fibonacci):** `1 | 2 | 3 | 5 | 8`
- **Manual Reference:** `102632810-05-01-acc.pdf` Section `<X-Y>`

## User Story
**As a** `<role / subsystem>`,
**I want** `<capability>`,
**So that** `<SpeedCo I behavior or value>`.

## TDD Targets (`tests/phase_*.rs`)
- [ ] Red-phase test cases exist and compile cleanly
- [ ] Target tests listed:
  - `test_name_1`
  - `test_name_2`

## Acceptance Criteria & Definition of Done
- [ ] Implementation compiles in debug and `--release` with **0 warnings**
- [ ] Target phase tests (and all prior phase suites) pass under `cargo test` and `cargo test --release`
- [ ] Public types/functions have concise rustdoc comments and `cargo doc --no-deps` passes cleanly
- [ ] `docs/AGILE-SPRINTS.md` and `docs/TDD-TEST-SPECIFICATION.md` updated if needed

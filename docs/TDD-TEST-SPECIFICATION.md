# TDD Test Specification (66 Cases)

This document catalogs every TDD test case currently defined in the repository (**1** unit test in `src/lib.rs` + **65** phase integration tests across `tests/phase_1_core_types.rs`..`tests/phase_6_checking_listing_timing.rs`).

> [!NOTE]
> **Current TDD Stage (Sprint 3 / Phases 1–3 Green):** `src/lib.rs` (`1/1`), Phase 1 [`tests/phase_1_core_types.rs`](../tests/phase_1_core_types.rs) (`11/11`), Phase 2 [`tests/phase_2_parser.rs`](../tests/phase_2_parser.rs) (`10/10`), and Phase 3 [`tests/phase_3_arithmetic_math.rs`](../tests/phase_3_arithmetic_math.rs) (`10/10`) now pass in both Debug and Release builds in GitHub Actions Linux CI (Run [`#44948301907`](https://github.com/Mr-Kumar-Abhishek/speedcoding/actions/runs/44948301907), alongside `cargo clippy --all-targets -- -D warnings`; `32/66` TDD tests Green alongside `5/5` benchmarks). Phases 4–6 (`34` tests) compile cleanly with 0 warnings against `todo!()` stubs in `src/*.rs`. Running compiled `.exe` binaries under `target\debug\deps\` on this Windows host is blocked by an Application Control policy (`os error 4551`), so runtime test execution is verified via `.github/workflows/rust.yml`.

---

## Smoke Unit Test — `src/lib.rs` (1 Test)

| Test Function | Specification / Purpose |
|---|---|
| `mpc_library_inclusion` | Verifies `mpc::{MpcResult, mpc_parse, mpc_string}` dependency integration by matching `"speedcoding"`. |

---

## Phase 1 — [`tests/phase_1_core_types.rs`](../tests/phase_1_core_types.rs) (11 Tests)

| # | Test Function | Manual Ref | Specification Verified |
|---:|---|---|---|
| 1 | `word72_bounds_bits_and_six_bit_groups` | Sec. 1-1, 4-4, 7-5 | Enforces `WORD_BITS = 72`, rejects bit `>= 2^72`, checks 1-based MSB bit numbering (`1..=72`), and roundtrips 12 six-bit tape groups. |
| 2 | `address_and_user_storage_bounds` | Sec. 2-1, 3-2, 4-2, 9-11 | Validates 1024-word space (`0..=1023`), 714 user cells (`300..=1013`), `INITIAL_PC = 300`, `SPECIAL_R_CONTROL_ADDR = 25`, and `ControlRegionAccess` for `0..=299` & `1014..=1023`. |
| 3 | `speed_float_72bit_layout_and_normalization` | Sec. 3-1, 3-2 | Checks zero representation, normalization (`0.5 <= \|F\| < 1` or `F = 0`), bit layout (1: sign, 2–36: 35-bit fraction, 37: exp sign, 38–54: exp, 55–72: 0), `abs`, and `neg`. |
| 4 | `speed_float_binary_exponent_bounds` | Sec. 3-1 | Verifies binary exponent range `±131_071` (`MAX_BINARY_EXPONENT`) roundtrips cleanly through `Word72`. |
| 5 | `decimal_float_validation_and_roundtrip` | Sec. 3-1 | Enforces `MAX_DECIMAL_EXPONENT = 236`, rejects `±237`, verifies decimal-to-binary-to-decimal conversion, and rejects `\|E\| > 236` on export. |
| 6 | `r_code_flags_and_bounds` | Sec. 9-1 | Verifies `RCode` values `0..=7` decode `(modifies_a, modifies_b, modifies_c)` bit flags and rejects `8`. |
| 7 | `l_code_panel_switches_and_validation` | Sec. 11-2 | Verifies `LCode` accepts only `0`, `3`, `6` (mapped to switches `1`, `2`, `3`) and rejects `1, 2, 4, 5, 7, 9`. |
| 8 | `op1_all_49_operations_and_skip_restrictions` | Sec. 5-1, 5-2, 7-10..7-15 | Exhaustively verifies all 49 `Op1` variants (`code`, `mnemonic`, `from_code`, `from_mnemonic`) and `forbids_op2()` on `RFTP*`, `RBTP*`, `SFTP*`, `SBTP*`, `WRDR*`. |
| 9 | `op2_all_48_operations_and_noop` | Sec. 5-3..5-5 | Exhaustively verifies all 48 `Op2` variants (`code`, `mnemonic`, `from_code`, `from_mnemonic`), including `SETRB = 250`, `STA = 251`, `STB = 252`, `STC = 235`, `STD = 244`, and `NoOp` (`""` / `"-"` / `0`). |
| 10 | `instruction_72bit_layout_roundtrip` | Sec. 4-4 | Verifies exact bit-field packing/unpacking (`R`, `OP1`, `OP2`, `D`, `A`, `B`, `L`, `C`) and mandatory zero bits `2`, `6`, `37`, `58–59`. |
| 11 | `electrostatic_memory_user_access_and_control_protection` | Sec. 4-2 | Reads/writes `SpeedFloat` and `Instruction` in `300..=1013` and rejects reads/writes in control cells `0..=299` and `1014..=1023`. |

---

## Phase 2 — [`tests/phase_2_parser.rs`](../tests/phase_2_parser.rs) (10 Tests)

| # | Test Function | Manual Ref | Specification Verified |
|---:|---|---|---|
| 1 | `mpc_op1_and_op2_mnemonic_parsers` | Sec. 5 | Exercises `mpc::mpc_parse` with `build_op1_mnemonic_parser()` and `build_op2_mnemonic_parser()` on valid and invalid mnemonics. |
| 2 | `symbolic_instruction_line_examples` | Sec. 9-5, 9-6 | Parses manual lines `0416 ADD 0 1013 1013 0945 SETRA 0000` and `0354 ADD 0 1013 1013 1000 - - ; reset accum`. |
| 3 | `multi_line_program_parsing_sum_and_check_loops` | Sec. 9-5, 10-2 | Parses multi-line symbolic programs for the Page 9-5 sum loop and Page 10-2 checking loop. |
| 4 | `instruction_card_80col_fields_and_fixed_zeros` | Sec. 7-1, 7-2 | Extracts all 80-column instruction card fields and rejects non-blank column 9 or non-zero mandatory zero columns (`14`, `33–34`, `52–55`). |
| 5 | `alpha_only_instruction_card_requires_gang_punch` | Sec. 7-2, 7-3 | Rejects executing an alphabetical-only card via `to_machine_instruction()`, then succeeds after `gang_punch_instruction_card` fills cols `46–51`. |
| 6 | `numeric_only_instruction_card_loads_directly` | Sec. 7-3 | Accepts cards with blank alphabetical fields (`15–19`, `35–39`) and populated numerical codes (`46–51`). |
| 7 | `data_card_80col_five_values_col45_skip_and_overpunch_signs` | Sec. 7-3, 7-4 | Decodes up to 5 decimal floats per data card, skipping col `45` in `N3` and interpreting IBM `12`/`11` overpunched trailing sign digits. |
| 8 | `data_card_validation_rejects_bad_exponent_and_word_count` | Sec. 7-4 | Rejects data card exponents `> 236` (`DecimalExponentOutOfRange(237)`) and invalid word count (`6`). |
| 9 | `loading_control_card_tape_and_drum_formats` | Sec. 7-17 | Parses col-9 `'9'` loading control cards for both tape (`J..M`, drum field `0000`) and drum (`P`/`Q`, drum start address). |
| 10 | `forbids_op2_with_skip_capable_op1` | Sec. 7-10..7-15 | Rejects pairing `RFTP*`, `RBTP*`, `SFTP*`, `SBTP*`, or `WRDR*` with non-blank `OP2` (`ForbiddenOp2WithSkipOp1`), while allowing `RFDR*` + `OP2`. |

---

## Phase 3 — [`tests/phase_3_arithmetic_math.rs`](../tests/phase_3_arithmetic_math.rs) (10 Tests)

| # | Test Function | Manual Ref | Specification Verified |
|---:|---|---|---|
| 1 | `add_sub_six_variants_with_signed_operands` | Sec. 3-3 | Verifies `ADD`, `SUB`, `ADDAB`, `ABADD`, `SUBAB`, and `ABSUB` on signed operands. |
| 2 | `exact_cancellation_preserves_exponent_without_shifting` | Sec. 3-3 | Verifies `x + (-x)` and `x - x` produce `fraction == 0` while retaining the operands' binary exponent without shifting. |
| 3 | `add_sub_exponent_difference_over_76_rejected` | Sec. 3-3 | Rejects `ADD`/`SUB` when operands' decimal exponents differ by more than `76` (`ExponentDifferenceTooLarge`). |
| 4 | `mpy_and_ngmpy_truncation_and_sub_half_doubling` | Sec. 3-3, 3-4 | Checks product doubling and `exponent - 1` when `\|Fa*Fb\| < 0.5`, `NGMPY` negation, and 35-bit unrounded truncation (including `> 0.5 ULP` remainders). |
| 5 | `div_and_ngdiv_rounding_and_exponent_adjustment` | Sec. 3-4 | Checks `DIV` / `NGDIV` sign handling, quotient exponent adjustment, and 34/35-bit rounding (e.g., `1.0 / 3.0`). |
| 6 | `div_by_zero_triggers_divide_check_stop` | Sec. 3-4 | Returns `ArithmeticOutcome::Stop(MachineStop::DivideCheck)` when `F(B) == 0` for `DIV` or `NGDIV`. |
| 7 | `sqrt_accuracy_and_negative_or_zero_fraction_quirk` | Appendix-1 | Verifies `>= 9` significant digits for positive inputs and the negative/zero `F` quirk returning `F = 0` and binary exp `ceil(E / 2)`. |
| 8 | `sine_accuracy_and_large_exponent_zero_quirk` | Appendix-1 | Verifies `>= 8` decimal places for `\|dec_exp\| <= 3` and the `F = 0` quirk returning `F = 0`, binary exponent `+7`. |
| 9 | `artan_principal_value_accuracy` | Appendix-1 | Verifies principal-value `arctan` accuracy to `>= 7` decimal places across negative and positive arguments. |
| 10 | `exp_and_ln_accuracy_and_ln_quirks` | Appendix-2 | Verifies `EXP` (`>= 8` sig. digits), `LN` (`>= 9` dec. places in `(1/e, e)` and `>= 9` sig. digits outside), negative `F` treated as `\|F\|`, and `F = 0` treated as `1/2`. |

---

## Phase 4 — [`tests/phase_4_control_addressing.rs`](../tests/phase_4_control_addressing.rs) (12 Tests)

| # | Test Function | Manual Ref | Specification Verified |
|---:|---|---|---|
| 1 | `initial_pc_is_300_and_op1_precedes_op2` | Sec. 4-1, 4-3 | Confirms `PC = 300` at startup and `OP1` stores `C` before `OP2` evaluates `C` on the same instruction. |
| 2 | `unconditional_tr_and_conditional_trpl_trmn_trz` | Sec. 8-1, 8-2 | Tests `TR`, `TRPL`, `TRMN`, and `TRZ` branching and fallthrough behavior. |
| 3 | `two_instruction_ge_zero_transfer_idiom` | Sec. 8-2, 8-3 | Verifies the manual's `TRZ` + `TRPL` instruction pair branching iff `x - y >= 0`. |
| 4 | `sense_and_transfer_p_q_switches` | Sec. 8-3 | Verifies `SNTRP` (switch 4) and `SNTRQ` (switch 5) in both OFF and ON states. |
| 5 | `stop_and_transfer_halts_and_restarts_at_d` | Sec. 5-5 | Verifies `STOP` yields `MachineStop::ProgramStop { restart_addr: D }` and `restart_after_stop()` resumes at `D`. |
| 6 | `r_code_non_destructive_effective_addresses_page_9_2` | Sec. 9-1, 9-2 | Tests all 8 rows of the Page 9-2 effective-address table (`R_A=7, R_B=19, R_C=104`) and verifies the stored instruction in memory is unchanged. |
| 7 | `ti_td_setr_skr_and_underflow_stop` | Sec. 9-2..9-4 | Tests `SETRA/B/C`, `TIABC`, `SKRA`, `TDABC`, `TDC`, and `MachineStop::RQuantityUnderflow` when decrementing below `0`. |
| 8 | `worked_example_sum_34_numbers_page_9_5` | Sec. 9-5 | Executes the full 3-instruction Page 9-5 loop summing `1..=34` from cells `688..=721` into `945` (`595.0`). |
| 9 | `address_counter_modulo_1024_table_page_9_8` | Sec. 9-8 | Verifies all 5 rows of Page 9-8 (`+308 -> 308`, `-1003 -> 1003`, `+1348 -> 324`, `-1888 -> 864`, `+2348 -> 300`) and bounds `±131_071`. |
| 10 | `address_counter_radd_add_sub_st_skip_and_page_9_10_example` | Sec. 9-8..9-10 | Executes the Page 9-10 self-modifying code sequence (`0428..=0431`) patching `D` fields at `0355` (`17`) and `0358` (`11`), plus `SKIP`. |
| 11 | `combined_ac_and_r_quantities_via_addr_0025_page_9_12` | Sec. 9-11, 9-12 | Executes `RADDA 0025`, `STD 0600`, `SETRA 0000`, ..., `0600` to save and restore `R_A = 800`. |
| 12 | `control_region_transfer_and_effective_address_rejected` | Sec. 4-2, 9-10 | Rejects `TR` or `R`-modified effective addresses landing in `0..=299` or `1014..=1023` (`ControlRegionAccess`). |

---

## Phase 5 — [`tests/phase_5_io_storage.rs`](../tests/phase_5_io_storage.rs) (11 Tests)

| # | Test Function | Manual Ref | Specification Verified |
|---:|---|---|---|
| 1 | `tape_odd_parity_redundancy_channel` | Sec. 7-5 | Verifies `compute_odd_parity_bit` produces odd 1-bit counts across all 64 possible 6-bit groups. |
| 2 | `tape_status_transition_matrix_pages_7_13_and_7_14` | Sec. 7-13, 7-14 | Exercises every `(Read/Neutral/Write) x (Write/Read/Skip/EndFile/Rewind)` transition including `CopyCheck` stops and ignored `Read + EndFile`. |
| 3 | `tape_write_end_file_rewind_and_full_or_partial_read` | Sec. 7-6..7-9 | Writes multiple records, tests partial `RFTP` (`count < record_len`) advancing to the end-of-record gap, and sequential reads. |
| 4 | `tape_oversized_read_triggers_copy_check_stop` | Sec. 7-9 | Triggers `MachineStop::CopyCheck` when `B - A + 1` exceeds the unit record's word count. |
| 5 | `tape_eof_skip_checksum_skip_and_parity_stop` | Sec. 7-9, 7-10 | Verifies `ChecksumMismatchSkip` (2-instr skip), `TapeCheck` stop on parity error, and `EndOfFileSkip` (1-instr skip). |
| 6 | `tape_backward_read_and_skip_worked_example_page_7_12` | Sec. 7-10..7-12 | Replays the Page 7-12 backward navigation example (`RBTPK`, 2× `SBTPK`, `RBTPK` reading 2nd record, and `SFTPK`). |
| 7 | `tape_read_backward_error_does_not_final_backspace` | Sec. 7-11 | Verifies that errors during `RBTP`'s forward-read phase leave the tape positioned *after* the failed record (no final backspace). |
| 8 | `drum_write_read_worked_example_pages_7_15_and_7_16` | Sec. 7-15, 7-16 | Writes 68 words to drum `Q` (`32..=99`) and reads 10 words from drum address `41..=50` (10th–19th words of the written block). |
| 9 | `drum_single_word_verify_error_skip_and_bounds_no_wrap` | Sec. 7-15, 7-16 | Verifies `DrumWriteOutcome::VerifyDiscrepancySkip` and rejects writes/reads crossing `1023` (`DrumAddressOutOfRange` — no wrap). |
| 10 | `line_printer_page_7_19_worked_example_and_exponent_limit` | Sec. 7-18, 7-19 | Prints 13 values (`461..=473`, `C = 407`) as lines `40701` (5), `40702` (5), `40703` (3) and rejects `\|dec_exp\| > 236`. |
| 11 | `card_reader_load_deck_rewinds_tapes_and_offloads_blocks` | Sec. 7-17, 7-18 | Verifies `load_deck` rewinds all 4 tapes prior to reading, processes loading control cards (`Write` status on tape), and sets `PC = 300`. |

---

## Phase 6 — [`tests/phase_6_checking_listing_timing.rs`](../tests/phase_6_checking_listing_timing.rs) (11 Tests)

| # | Test Function | Manual Ref | Specification Verified |
|---:|---|---|---|
| 1 | `checking_protocol_state_machine_rules_page_10_4` | Sec. 10-4 | Enforces Rules 1 & 2 (`STCH` requires prior `PRCH`/`ECHTR`; `ECHTR` requires prior `STCH`) and pass 1 -> pass 2 -> skip flow. |
| 2 | `checking_loop_pass_1_and_pass_2_worked_example_page_10_2` | Sec. 10-2 | Executes the Page 10-2 `PRCH`/`STCH`/`ECHTR` loop computing `sqrt(3^2 + 4^2) = 5.0` and skipping `0383 STOP`. |
| 3 | `checking_loop_discrepancy_does_not_skip_and_print_suppressed_on_pass2` | Sec. 10-2 | Verifies checksum mismatch returns `DiscrepancyContinue` (no skip) and `PRINT` outputs only once (suppressed on pass 2). |
| 4 | `checking_loop_inter_pass_initialization_not_summed_page_10_3` | Sec. 10-3, 10-4 | Confirms operations executed between `ECHTR` and `STCH` are excluded from check counters 1 and 2. |
| 5 | `selective_listing_switches_10_line_drum_flush_and_pass2_suppression` | Sec. 11-1, 11-2 | Tests panel switches for `L` codes `0/3/6`, pass-2 suppression, and drum accumulation flushing in exact blocks of `10` lines. |
| 6 | `listing_operand_sign_exceptions_and_exponent_overflow_page_11_2` | Sec. 11-1, 11-2 | Tests sign negation/abs rules for `SUB`, `ABSUB`, `NGMPY`, and `±999` exponent display when `\|dec_exp\| > 236`. |
| 7 | `listing_captures_state_after_op1_before_op2` | Sec. 11-1 | Verifies `0300 ADD ... TIA 0301` logs post-`OP1` result at `C` together with pre-`TIA` value of `R_A`. |
| 8 | `timing_op1_arithmetic_math_io_and_print_pass2` | Sec. 12-1, 12-2, App. 1–2 | Verifies `ADD`, `MPY`, `NOOP` (`2.736 ms`), `SQRT`, `LN`, `WRTPK`, and `PRINT` pass-1 (`1663 + 400*I`) vs pass-2 (`2.896 ms`) times. |
| 9 | `timing_op2_and_additive_instruction_time` | Sec. 12-1, 12-3, 12-4 | Verifies `OP2` times (`NoOp = 0.000`, `TR`, `TRPL`, `TIA`, `TDB`, `ECHTR`, `STOP`) and additive `OP1 + OP2` totals. |
| 10 | `timing_tape_length_rewind_worked_example_eof_and_skip_delay` | Sec. 12-2, 12-3 | Verifies 100×10-word tape length (`369 in`), rewind time (`4.920 s`), EOF gap time (`0.96 s`), and Note 1 skip overlap delay. |
| 11 | `timing_listing_constants_and_card_loading_formula_page_12_5` | Sec. 12-4, 12-5 | Verifies listing constants (`1.325 s` avg / `1.370 s` max) and `(0.4)(n) + (0.6)(c) + e` loading formula (`e = 2` vs `55`). |

---

## Verification Commands

```powershell
# Debug build and test compilation (Red-phase & iterative TDD)
cargo build
cargo test --no-run

# Run individual phase test suites (once implemented / on unrestricted hosts)
cargo test --test phase_1_core_types
cargo test --test phase_2_parser
cargo test --test phase_3_arithmetic_math
cargo test --test phase_4_control_addressing
cargo test --test phase_5_io_storage
cargo test --test phase_6_checking_listing_timing

# Release verification & packaging
cargo test --release
cargo doc --no-deps
cargo package --list --allow-dirty
```

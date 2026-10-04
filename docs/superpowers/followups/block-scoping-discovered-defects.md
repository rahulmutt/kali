# Defects the block-scoping project measured and did NOT fix

**Filed** 2026-10-04 by the **block-scoping** project
(`docs/superpowers/specs/2026-10-04-block-scoping-design.md`), on the
convention `class-instances-discovered-defects.md` uses: a project that
measures more than it fixes writes down what it left, so the silence is not
read as absence. Task 9 wrote the triage (§1) and the first items of §7;
Task 10 wrote the rest.

**Oracle:** `node v26.10.0`.
**Measured at:** HEAD `de662a327` (branch `block-scoping`), on
`target/debug/kali` built from that commit (the `dev` profile). The two
commits after it (`730c5f8d6`, `af393e40b`) change test files only. Items
that Task 10 measured by hand (§3, §6, §7.6-§7.10) were run on the same
binary. The baseline binary was built from `6345f082b` in a separate
worktree (`/home/dev/kali-bs-baseline`). Probe rows come from
`tools/array-return-probes/probes/bs_*.js` (runner
`tools/array-return-probes/run.sh`); a probe's baseline column is
`tools/array-return-probes/baseline-bs.tsv`.

**What ships.** Two things, and nothing else.

* **Block scoping by renaming.** A `let` / `const` / function declaration in
  a block, `if` arm, loop body or `for` head is renamed to a binding of its
  own before analysis, so it no longer changes a same-named module binding,
  function local or parameter (register R-10, FIXED). `--compat eval` with a
  renamed binding is refused under `run` and `check`.
* **Per-iteration env records.** A loop that registers a callback
  (`queueMicrotask` or `setTimeout` are the end-to-end-tested callees) over
  its own bindings gets one record per iteration, so each callback sees its
  iteration's bindings (register R-69, FIXED for those shapes). The loop is
  an owner when the registration is textually anywhere inside it, including
  inside a function or arrow function defined in the loop (ruling R8). The
  records cover a loop's body bindings and a `for` loop's head bindings. They
  do not cover the head binding of a growable `for…of` or a `for…in` key
  (spec A-8, §2 and §7.5).

The maturity row is `specs/19-feature-maturity.md`, "Block-scoped bindings
and per-iteration loop bindings for registered callbacks". Nothing in this
file changes it; each section below is something that row does not claim.

---

## §1. The triage table

From Task 9. `cargo test --workspace --no-fail-fast` at `de662a327` failed
60 trials: 59 in `kali_cli --test cases` and 1 in
`kali_cli --test runtime_smoke`. Every other target, the package-corpus
targets included, passed. `cargo test -p kali_cli --test cases -- --ignored`
ran the one ignored case, `soundness/r06_object_init::var_object_mixed_fields_read`,
which fails today as the cases README says it does. Nothing moved there.

That is 60 moved trials, over the stop rule's 50 (spec §5.4). The human
partner accepted the overage at Task 4, when the same 59 `cases` trials had
moved with no capability loss (ruling H1). None of the 60 is a capability
loss of any class. No trial went from a baseline exit 0 to a HEAD refusal.

The 59 `cases` trials were re-run at `de662a327` with the Task 4 harness,
against the committed (pre-re-pin) case files. The baseline, HEAD and node
columns are byte-identical to Task 4's measurement at `16dc1cf9c`, so the
per-iteration work in Tasks 5-8 moved none of them. The runtime_smoke trial
is new to this triage because Task 4 triaged the `cases` target only.

| class | trials |
|---|---|
| wanted (a silent wrong value or a refusal became node-correct) | 45 (42 re-pinned to node's output, 2 `tier2::r10_*` flipped to FIXED, 1 classifier ground truth re-pinned to a new SILENT specimen) |
| rationale only / stderr only | 9 |
| NEW SILENT WRONG: the shadow now resolves correctly, but the read hits a pre-existing defect (§7.1, §7.2); fixture rewritten under ruling H2 | 6 (the 5 Task 4 found, plus the runtime_smoke trial) |
| capability loss, class 1 / 2 / 3 | 0 / 0 / 0 |
| **total** | **60** |

Re-pin rules applied:

* **wanted**: the case now asserts node's output (a `check` twin asserts
  exit 0) and gets a dated `RE-PINNED 2026-10-04` paragraph naming the
  baseline refusal. The case is renamed so the name says what it now tests,
  and so is its source file where the two names matched. The old name is kept
  in the rationale. None of the old names appears anywhere outside its case
  file.
* **rationale only / stderr only**: still refused. The needle moves to HEAD's
  diagnostic, a dated paragraph is added, and the name is kept.
* **NEW SILENT WRONG**: never pinned to the wrong output (ruling H2). The
  fixture reads the shadowing binding inside kali's supported subset and reads
  the outer binding after it, so the case still tests shadowing and passes
  node-equal. The case is renamed.

`r10_*` and the register: the gate
`every_zero_two_row_is_the_class_set_its_live_cases_assert` requires §0.2 to
match the oracle cases. So, following the R-14 precedent (ruling R11a,
`9c2882c66`), only R-10's §0.2 status cell moved to FIXED. R-10's
`tools/blast-radius/clusters.json` assignment left with a dated note (G7
survives with R-06), and the ranking's generated regions were re-spliced from
`cargo run -p kali_blast_radius --example rank`. The rest of R-10's register
text is Task 10's.

| name | before (`6345f082b`) | after (HEAD `de662a327`) | class |
|---|---|---|---|
| `object/class_instances::r_ambiguous_is_refused_under_run` (`run r_ambiguous.js`) | exit 1, `E5506 constructing class 'P' is unavailable in the current phase: it is declared more than once and keeps stat…` | exit 1, `E5506 using an instance of class 'P' as the receiver of a field access that is not a variable is unavailable i…` (+1 more); node: `1 2⏎` | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. The rename disambiguates the two `P`s; the refusal is now the pre-existing call-result-receiver guard. |
| `object/class_instances::r_ambiguous_is_refused_under_check` (`check r_ambiguous.js`) | exit 1, `E5506 constructing class 'P' is unavailable in the current phase: it is declared more than once and keeps stat…` | exit 1, `E5506 using an instance of class 'P' as the receiver of a field access that is not a variable is unavailable i…` (+1 more); node: `1 2⏎` | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. The rename disambiguates the two `P`s; the refusal is now the pre-existing call-result-receiver guard. |
| `object/class_instances::r26_b4_initializer_names_ctor_param_in_fn_is_refused_under_check` (`check r26_b4_initializer_names_ctor_param_in_fn.js`) | exit 1, `E5506 constructing class 'C' is unavailable in the current phase: it reads a variable of an enclosing function…` (+1 more) | exit 1, `E5506 constructing class 'C' is unavailable in the current phase: it reads a variable of an enclosing function…`; node: `15⏎` | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. The conflated second reason (`names a parameter … of its constructor`) is now pinned absent. |
| `object/class_instances::r26_b4_initializer_names_ctor_param_in_fn_is_refused_under_run` (`run r26_b4_initializer_names_ctor_param_in_fn.js`) | exit 1, `E5506 constructing class 'C' is unavailable in the current phase: it reads a variable of an enclosing function…` (+1 more) | exit 1, `E5506 constructing class 'C' is unavailable in the current phase: it reads a variable of an enclosing function…`; node: `15⏎` | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. The conflated second reason (`names a parameter … of its constructor`) is now pinned absent. |
| `object/class_instances::r26_a4_initializer_names_ctor_param_is_refused_under_check` (`check r26_a4_initializer_names_ctor_param.js`) | exit 1, `E5506 constructing class 'C' is unavailable in the current phase: it has a field initializer that names a para…` | exit 0, `Checked 1 file(s)⏎`; node: `10⏎3⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0 (`check`); renamed `r26_a4_initializer_naming_an_outer_binding_a_ctor_param_shadows_is_admitted_under_check`. |
| `object/class_instances::r26_a4_initializer_names_ctor_param_is_refused_under_run` (`run r26_a4_initializer_names_ctor_param.js`) | exit 1, `E5506 constructing class 'C' is unavailable in the current phase: it has a field initializer that names a para…` | exit 0, `10⏎3⏎`; node: `10⏎3⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `r26_a4_initializer_naming_an_outer_binding_a_ctor_param_shadows_runs_under_run`. |
| `object/computed_member_static_name::check_refuses_a_shadowed_fold_name_store` (`check shadowed_key_store.js`) | exit 1, `E5506 computed member access 'o[k]' is unavailable in the current phase unless the index is a literal or a com…` | exit 0, `Checked 1 file(s)⏎`; node: `8⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0 (`check`); renamed `check_admits_a_store_through_a_block_shadowed_fold_name`. |
| `object/computed_member_static_name::check_refuses_a_shadowed_fold_name_read` (`check shadowed_key_read.js`) | exit 1, `E5506 computed member access 'o[k]' is unavailable in the current phase unless the index is a literal or a com…` | exit 0, `Checked 1 file(s)⏎`; node: `2⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0 (`check`); renamed `check_admits_a_read_through_a_block_shadowed_fold_name`. |
| `object/computed_member_static_name::run_refuses_a_shadowed_fold_name_read` (`run shadowed_key_read.js`) | exit 1, `E5506 computed member access 'o[k]' is unavailable in the current phase unless the index is a literal or a com…` | exit 0, `2⏎`; node: `2⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `run_reads_through_a_block_shadowed_fold_name`. |
| `object/computed_member_static_name::run_refuses_a_shadowed_fold_name_store` (`run shadowed_key_store.js`) | exit 1, `E5506 computed member access 'o[k]' is unavailable in the current phase unless the index is a literal or a com…` | exit 0, `8⏎`; node: `8⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `run_stores_through_a_block_shadowed_fold_name`. |
| `oracle/classifier_ground_truth::a_block_scoped_shadow_read_classifies_as_silent` (`run silent.js`) | exit 0, `r=2⏎` | exit 0, `r=1⏎`; node: `r=1⏎` | **wanted**: silent `r=2` became node's `r=1`. The SILENT specimen was replaced by the human partner's pick, R-26 (`console.log(+"abc");`), and the case renamed `unary_plus_on_a_non_numeric_string_classifies_as_silent`; see §1.2. |
| `oracle/tier2::r10_block_scope_shadowing_module_scope` (`run r10_module.js`) | exit 0, `r=2⏎` | exit 0, `r=1⏎`; node: `r=1⏎` | **wanted**: silent `r=2` became node's `r=1`. Re-pinned `verdict = "fixed"` with the dated FIXED line; header index line updated; §0.2's R-10 status cell, `clusters.json` and the ranking regenerated with it (the R-14 precedent). |
| `oracle/tier2::r10_block_scope_shadowing_in_function` (`run r10_function.js`) | exit 0, `r=2⏎` | exit 0, `r=1⏎`; node: `r=1⏎` | **wanted**: silent `r=2` became node's `r=1`. Re-pinned `verdict = "fixed"` with the dated FIXED line; header index line updated; §0.2's R-10 status cell, `clusters.json` and the ranking regenerated with it (the R-14 precedent). |
| `soundness/abort::abort_handle_shadowed_by_for_of_binding_fails_closed` (`run abort_handle_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to an AbortController/AbortSignal in the current phase…` | exit 1, `E5506 calling '.abort()' is unavailable in the current phase: the receiver is a value this program built, and …`; node: exit 1, TypeError | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. node throws a TypeError on the string loop variable; kali now refuses at the method call instead of at the shadow. |
| `soundness/abort::abort_handle_redeclared_in_an_inner_block_fails_closed` (`run abort_handle_redeclared_in_an_inner_block_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to an AbortController/AbortSignal in an inner scope is not supported in the cur…` (+1 more) | exit 0, `5⏎aborted=false⏎`; node: `5⏎aborted=false⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `abort_handle_redeclared_in_an_inner_block_reads_the_inner_binding`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_global_shadowed_by_captured_local` (`run bitwise_compound_fails_closed_on_module_global_shadowed_by_captured_local.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `6⏎`; node: `6⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_local_shadowing_a_module_global`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_shr_assign` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_shr_assign.ts`) | exit 1, `E5506 bitwise compound assignment '>>=' on binding 'n' is unavailable in the current phase (the binding is sha…` | exit 0, `6⏎`; node: `6⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_shr_assign`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_or_assign` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_or_assign.ts`) | exit 1, `E5506 bitwise compound assignment '\|=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `28⏎`; node: `28⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_or_assign`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_shl_assign` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_shl_assign.ts`) | exit 1, `E5506 bitwise compound assignment '<<=' on binding 'n' is unavailable in the current phase (the binding is sha…` | exit 0, `48⏎`; node: `48⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_shl_assign`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_xor_assign` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_xor_assign.ts`) | exit 1, `E5506 bitwise compound assignment '^=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `9⏎`; node: `9⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_xor_assign`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_different_value_and_op` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_different_value_and_op.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `1⏎`; node: `1⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_different_value_and_op`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_and_assign` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_and_assign.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_and_assign`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_ushr_assign` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_ushr_assign.ts`) | exit 1, `E5506 bitwise compound assignment '>>>=' on binding 'n' is unavailable in the current phase (the binding is sh…` | exit 0, `6⏎`; node: `6⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_ushr_assign`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_string_const` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_string_const.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_string_const`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_arrow_capturer` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_arrow_capturer.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_arrow_capturer`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_own_cell_double_read` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_own_cell_double_read.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `1⏎1⏎`; node: `1⏎1⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_own_cell_double_read`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_let_or_var_shadowing_captured_cell_var_numeric` (`run bitwise_compound_fails_closed_on_module_let_or_var_shadowing_captured_cell_var_numeric.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `4⏎6⏎`; node: `4⏎6⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_let_or_var_var_numeric`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_fold_sensitive_comparison` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_fold_sensitive_comparison.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `1⏎`; node: `true⏎` | **NEW SILENT WRONG at Task 4** (ruling H2): the shadow now resolves correctly, but the old fixture's read hit a pre-existing defect (§7.1 / §7.2). Fixture rewritten to stay in the supported subset and renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_feeds_a_fold_sensitive_comparison`; pinned `yes⏎`, node-equal. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_float_const` (`run bitwise_compound_fails_closed_on_module_const_shadowing_captured_cell_float_const.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_const_float_const`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_let_or_var_shadowing_captured_cell_let_numeric` (`run bitwise_compound_fails_closed_on_module_let_or_var_shadowing_captured_cell_let_numeric.ts`) | exit 1, `E5506 bitwise compound assignment '&=' on binding 'n' is unavailable in the current phase (the binding is shad…` | exit 0, `4⏎6⏎`; node: `4⏎6⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_let_or_var_let_numeric`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_module_let_or_var_shadowing_captured_cell_let_string` (`run bitwise_compound_fails_closed_on_module_let_or_var_shadowing_captured_cell_let_string.ts`) | exit 1, `E5506 reading module binding 'n' from a function is only available for compile-time-constant 'const' initializ…` (+1 more) | exit 0, `4⏎hi⏎`; node: `4⏎hi⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_cell_shadowing_a_module_let_or_var_let_string`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_event_target` (`run bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_event_target.ts`) | exit 1, `E5506 reading module binding 't' from a function is only available for compile-time-constant 'const' initializ…` (+1 more) | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_local_shadowing_a_module_event_target_handle`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_abort_controller` (`run bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_abort_controller.ts`) | exit 1, `E5506 bitwise compound assignment ('&=') is recognized by the parser but has no codegen lowering yet for this …` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_local_shadowing_a_module_abort_controller_handle`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_url` (`run bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_url.ts`) | exit 1, `E5506 bitwise compound assignment ('&=') is recognized by the parser but has no codegen lowering yet for this …` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_local_shadowing_a_module_url_handle`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_url_search_params` (`run bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_url_search_params.ts`) | exit 1, `E5506 bitwise compound assignment ('&=') is recognized by the parser but has no codegen lowering yet for this …` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_local_shadowing_a_module_url_search_params_handle`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_event` (`run bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_event.ts`) | exit 1, `E5506 bitwise compound assignment ('&=') is recognized by the parser but has no codegen lowering yet for this …` | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_local_shadowing_a_module_event_handle`. |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_text_encoder` (`run bitwise_compound_fails_closed_on_every_emit_identifier_interception_arm_text_encoder.ts`) | exit 1, `E5506 reading module binding 'e' from a function is only available for compile-time-constant 'const' initializ…` (+1 more) | exit 0, `4⏎`; node: `4⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_on_a_captured_local_shadowing_a_module_text_encoder_handle`. |
| `soundness/bitwise_compound::bitwise_compound_over_denies_a_target_whose_name_is_shadowed_by_an_unrelated_float_float_shadow` (`run bitwise_compound_over_denies_a_target_whose_name_is_shadowed_by_an_unrelated_float_float_shadow.ts`) | exit 1, `E5506 bitwise compound assignment '\|=' on a non-integer module global is unavailable in the current phase` | exit 0, `14⏎`; node: `14⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_admits_a_target_whose_name_is_shadowed_by_an_unrelated_float_float_shadow`. |
| `soundness/bitwise_compound::bitwise_compound_over_denies_a_target_whose_name_is_shadowed_by_an_unrelated_float_unreachable_shadow` (`run bitwise_compound_over_denies_a_target_whose_name_is_shadowed_by_an_unrelated_float_unreachable_shadow.ts`) | exit 1, `E5506 bitwise compound assignment '\|=' on a non-integer module global is unavailable in the current phase` | exit 0, `14⏎`; node: `14⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_admits_a_target_whose_name_is_shadowed_by_an_unrelated_float_unreachable_shadow`. |
| `soundness/bitwise_compound::bitwise_compound_over_denies_a_target_whose_name_is_shadowed_by_an_unrelated_float_nan_shadow` (`run bitwise_compound_over_denies_a_target_whose_name_is_shadowed_by_an_unrelated_float_nan_shadow.ts`) | exit 1, `E5506 bitwise compound assignment '\|=' on a non-integer module global is unavailable in the current phase` | exit 0, `14⏎`; node: `14⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bitwise_compound_admits_a_target_whose_name_is_shadowed_by_an_unrelated_float_nan_shadow`. |
| `soundness/events::event_marker_redeclared_by_inner_scalar_fails_closed` (`run event_marker_redeclared_by_inner_scalar_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to an Event/CustomEvent in an inner scope is not supported in the current phase…` | exit 0, `0⏎`; node: `undefined⏎` | **NEW SILENT WRONG at Task 4** (ruling H2): the shadow now resolves correctly, but the old fixture's read hit a pre-existing defect (§7.1 / §7.2). Fixture rewritten to stay in the supported subset and renamed `event_marker_redeclared_by_inner_scalar_reads_the_inner_scalar`; pinned `6⏎tick⏎`, node-equal. |
| `soundness/events::event_marker_redeclared_inside_function_fails_closed` (`run event_marker_redeclared_inside_function_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to an Event/CustomEvent in an inner scope is not supported in the current phase…` | exit 0, `inner⏎`; node: `inner⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `event_marker_redeclared_inside_function_reads_the_inner_binding`. |
| `soundness/events::event_marker_redeclared_by_inner_object_fails_closed` (`run event_marker_redeclared_by_inner_object_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to an Event/CustomEvent in an inner scope is not supported in the current phase…` | exit 0, `x⏎`; node: `x⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `event_marker_redeclared_by_inner_object_reads_the_inner_object`. |
| `soundness/events::event_marker_shadowed_by_for_of_binding_fails_closed` (`run event_marker_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to an Event/CustomEvent in the current phase (this pro…` | exit 0, `0⏎0⏎`; node: `undefined⏎undefined⏎` | **NEW SILENT WRONG at Task 4** (ruling H2): the shadow now resolves correctly, but the old fixture's read hit a pre-existing defect (§7.1 / §7.2). Fixture rewritten to stay in the supported subset and renamed `event_marker_shadowed_by_for_of_binding_reads_the_loop_binding`; pinned `aa⏎bb⏎tick⏎`, node-equal. |
| `soundness/events::event_target_handle_shadowed_by_for_of_binding_fails_closed` (`run event_target_handle_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to an EventTarget in the current phase (this provenanc…` | exit 0, `2⏎`; node: `2⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `event_target_handle_shadowed_by_for_of_binding_reads_the_loop_binding`. |
| `soundness/events::event_target_handle_redeclared_in_an_inner_block_fails_closed` (`run event_target_handle_redeclared_in_an_inner_block_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to an EventTarget in an inner scope is not supported in the current phase (this…` (+1 more) | exit 0, `5⏎`; node: `5⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `event_target_handle_redeclared_in_an_inner_block_reads_the_inner_binding`. |
| `soundness/textcodec::text_decoder_marker_shadowed_by_for_of_binding_fails_closed` (`run text_decoder_marker_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to a TextDecoder in the current phase (this provenance…` | exit 1, `E5506 a TextEncoder byte buffer cannot be read in this position: kali admits it only as a TextDecoder().decode…` (+1 more); node: exit 1, TypeError | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. node throws a TypeError on the string loop variable; kali now refuses at the method call instead of at the shadow. |
| `soundness/textcodec::bytes_handle_shadowed_by_for_of_binding_fails_closed` (`run bytes_handle_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to a TextEncoder().encode() byte handle in the current…` | exit 1, `E5506 TextDecoder().decode only accepts a TextEncoder().encode byte buffer in the current phase (fail-closed)`; node: exit 1, TypeError | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. node throws a TypeError on the string loop variable; kali now refuses at the method call instead of at the shadow. |
| `soundness/textcodec::text_decoder_marker_redeclared_in_an_inner_block_fails_closed` (`run text_decoder_marker_redeclared_in_an_inner_block_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to a TextDecoder in an inner scope is not supported in the current phase (this …` (+1 more) | exit 0, `5⏎hi⏎`; node: `5⏎hi⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `text_decoder_marker_redeclared_in_an_inner_block_reads_the_inner_binding`. |
| `soundness/textcodec::text_encoder_marker_redeclared_in_an_inner_block_fails_closed` (`run text_encoder_marker_redeclared_in_an_inner_block_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to a TextEncoder in an inner scope is not supported in the current phase (this …` (+1 more) | exit 0, `5⏎hi⏎`; node: `5⏎hi⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `text_encoder_marker_redeclared_in_an_inner_block_reads_the_inner_binding`. |
| `soundness/textcodec::text_encoder_marker_shadowed_by_for_of_binding_fails_closed` (`run text_encoder_marker_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to a TextEncoder in the current phase (this provenance…` | exit 1, `E5506 TextDecoder().decode only accepts a TextEncoder().encode byte buffer in the current phase (fail-closed)`; node: exit 1, TypeError | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. node throws a TypeError on the string loop variable; kali now refuses at the method call instead of at the shadow. |
| `soundness/textcodec::bytes_handle_redeclared_in_an_inner_block_fails_closed` (`run bytes_handle_redeclared_in_an_inner_block_fails_closed.js`) | exit 1, `E5506 redeclaring a name bound to a TextEncoder().encode() byte handle in an inner scope is not supported in t…` (+1 more) | exit 0, `5⏎hi⏎`; node: `5⏎hi⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `bytes_handle_redeclared_in_an_inner_block_reads_the_inner_binding`. |
| `soundness/url::url_binding_name_shadowing_fails_closed_shadow_over_url` (`run url_binding_name_shadowing_fails_closed_shadow_over_url.js`) | exit 1, `E5506 redeclaring a name bound to a URL/URLSearchParams in an inner scope is not supported in the current phas…` | exit 0, `7⏎`; node: `7⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `url_binding_name_shadowing_reads_the_inner_binding_shadow_over_url`. |
| `soundness/url::url_binding_name_shadowing_fails_closed_url_over_generic` (`run url_binding_name_shadowing_fails_closed_url_over_generic.js`) | exit 1, `E5506 constructing a URL is only supported as 'const <name> = new URL(<string-literal>)' in the current phase;…` | exit 0, `/first⏎`; node: `/first⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `url_binding_name_shadowing_reads_the_inner_binding_url_over_generic`. |
| `soundness/url::url_shadowed_by_for_of_binding_fails_closed` (`run url_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to a URL/URLSearchParams in the current phase (this pr…` | exit 0, `0⏎`; node: `undefined⏎` | **NEW SILENT WRONG at Task 4** (ruling H2): the shadow now resolves correctly, but the old fixture's read hit a pre-existing defect (§7.1 / §7.2). Fixture rewritten to stay in the supported subset and renamed `url_shadowed_by_for_of_binding_reads_the_loop_binding`; pinned `aa⏎/p⏎`, node-equal. |
| `soundness/url::url_search_params_shadowed_by_for_of_binding_fails_closed` (`run url_search_params_shadowed_by_for_of_binding_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to a URL/URLSearchParams in the current phase (this pr…` | exit 1, `E5506 calling '.get()' is unavailable in the current phase: the receiver is a value this program built, and ka…`; node: exit 1, TypeError | **rationale only / stderr only**: still refused; needle and dated note updated, name kept. node throws a TypeError on the string loop variable; kali now refuses at the method call instead of at the shadow. |
| `soundness/url::url_shadowed_by_for_of_binding_inside_a_function_fails_closed` (`run url_shadowed_by_for_of_binding_inside_a_function_fails_closed.js`) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to a URL/URLSearchParams in the current phase (this pr…` | exit 0, `0⏎`; node: `undefined⏎` | **NEW SILENT WRONG at Task 4** (ruling H2): the shadow now resolves correctly, but the old fixture's read hit a pre-existing defect (§7.1 / §7.2). Fixture rewritten to stay in the supported subset and renamed `url_shadowed_by_for_of_binding_inside_a_function_reads_the_loop_binding`; pinned `aa⏎/p⏎`, node-equal. |
| `soundness/url::url_redeclared_in_an_inner_block_fails_closed_pre_existing_coverage` (`run url_redeclared_in_an_inner_block_fails_closed_pre_existing_coverage.js`) | exit 1, `E5506 redeclaring a name bound to a URL/URLSearchParams in an inner scope is not supported in the current phas…` | exit 0, `x⏎`; node: `x⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `url_redeclared_in_an_inner_block_reads_the_inner_binding`. |
| `soundness/url::url_search_params_redeclared_in_an_inner_block_fails_closed_pre_existing_coverage` (`run url_search_params_redeclared_in_an_inner_block_fails_closed_pre_existing_coverage.js`) | exit 1, `E5506 redeclaring a name bound to a URL/URLSearchParams in an inner scope is not supported in the current phas…` (+1 more) | exit 0, `5⏎`; node: `5⏎` | **wanted**: a refusal became node-correct output. Re-pinned to exit 0; renamed `url_search_params_redeclared_in_an_inner_block_reads_the_inner_binding`. |
| `runtime_smoke::run::run_rejects_crypto_random_result_shadowed_by_a_for_of_binding` (`run --api browser main.js`; `for (const fb of ['aa','bbb']) console.log(fb.byteLength)` shadowing a `crypto.getRandomValues` result) | exit 1, `E5506 a for-of loop binding may not shadow a name bound to a crypto.getRandomValues(...) result …` | exit 0, `8⏎0⏎0⏎`; node: `8⏎undefined⏎undefined⏎` | **NEW SILENT WRONG**, not in Task 4's list (Task 4 triaged the `cases` target only). Same primitive-member-read defect as §7.1; handled as ruling H2 directs: fixture rewritten (`fb.length` in the loop, `fb.byteLength` after it) and renamed `run_reads_a_for_of_binding_that_shadows_a_crypto_random_result`; pinned `8⏎2⏎3⏎8⏎`, node-equal. |

### §1.1 The sweep

Spec §5.4 also requires a sweep. Every `cli` and `oracle` step of every case
file was run with HEAD's binary, with matrix and constants expanded and
ignored cases included: 6,423 trials (harness
`/home/dev/kali-bs-tmp/t9/sweep.py`, raw output `sweep.json`). Any trial whose
stderr contained `through a per-iteration record`,
`` `continue` in a `for` loop ``, `compile-time iterable`,
`was planned but no loop declared` or ``with `--compat eval` `` was re-run
with the baseline binary. 7 trials matched, and each is a passing case this
project added or a pre-existing refusal:

| trial | baseline (`6345f082b`) | HEAD | node | verdict |
|---|---|---|---|---|
| `scope/eval_refusal::a_renamed_binding_under_compat_eval_is_refused_by_run` (`let x = 1; { let x = 2; console.log(x); } console.log(eval("x"));`, `--compat eval`) | exit 0, `2⏎0⏎` | E5506 `--compat eval` refusal | `2⏎1⏎` | wanted: silent wrong value became a refusal |
| `scope/eval_refusal::a_renamed_binding_under_compat_eval_is_refused_by_check` (same program) | `check` exit 0 | E5506, same | `2⏎1⏎` | wanted: the `check` twin of the row above |
| `scope/per_iteration::a_capture_of_a_function_binding_through_a_record_is_refused_under_run` (`r_loopmix.js`) | exit 0, `12⏎12⏎` | E5506 `through a per-iteration record` | `10⏎11⏎` | wanted: silent wrong value became a refusal |
| `scope/per_iteration::nested_owner_loops_are_refused_at_depth_two` (`nested_loops.js`) | exit 0, `0 0` ×4 | E5506 `through a per-iteration record` | `0 0⏎0 1⏎1 0⏎1 1⏎` | wanted: silent wrong value became a refusal |
| `scope/per_iteration::continue_in_an_owner_for_is_refused` (`r_continue.js`) | exit 1, `E4003 CPU fuel budget exhausted` (about 60 s) | E5506 `` `continue` in a `for` loop `` | `0⏎2⏎` | wanted: an internal failure became an honest refusal (compared on verdict and exit only) |
| `scope/per_iteration::an_unrolled_for_of_owner_is_refused` (`r_defer4.js`) | exit 1, E5506 `a captured local binding without closure lowering …` | E5506 `compile-time iterable` (plus the baseline's refusal) | `5⏎6⏎` | stderr only: refused at both |
| `soundness/abort::module_scope_for_of_captured_abort_fails_closed` | exit 1, E5506 `calling a method on module binding 'c' from a function …` | E5506 `compile-time iterable` (plus the baseline's refusal) | `cbtrue⏎` | stderr only: refused at both |

No previously exit-0 program that node runs correctly is refused at HEAD.
Per controller ruling R9, growable `for…of` loop-variable capture and
`for…in` key capture are still refused, as they were at baseline
(`scope/per_iteration::a_growable_for_of_loop_variable_capture_stays_refused`,
`a_for_in_key_capture_stays_refused`). They are not a move and not a
capability loss (§2). A capability loss that no case, fixture or corpus
program exercises was not measured.

### §1.2 The classifier ground truth

`oracle/classifier_ground_truth::a_block_scoped_shadow_read_classifies_as_silent`
used R-10 as the file's only SILENT specimen, so it now measures FIXED. Per
spec §5.2, its replacement came from the register's entries still SILENT at
HEAD and was chosen by the human partner. Three candidates were measured at
`de662a327`, each with exit 0 and empty stderr on both binaries:

| register id | repro (`silent.js`) | node v26.10.0 | kali (HEAD and `6345f082b`) |
|---|---|---|---|
| R-22 | `console.log("v=" + (1=="1"));` | `v=true` | `v=false` |
| R-26 | `console.log(+"abc");` | `NaN` | `5451` |
| R-23 | `var b=true; console.log(typeof b);` | `boolean` | `0` |

**Picked: R-26.** `silent.js` is now `console.log(+"abc");` with
`register_entry = "R-26"`, and the case is renamed
`unary_plus_on_a_non_numeric_string_classifies_as_silent`. The R-10 and R-13
readings are kept in its rationale. After the re-pin, R-10's oracle cases
assert `{FIXED}` and R-26's assert `{SILENT}`, matching §0.2, so the gate
`every_zero_two_row_is_the_class_set_its_live_cases_assert` passes.

---

## §2. Measured capability loss

**None in any trial, case, fixture or corpus program; one constructed program found by hand (§7.8).** Capability loss here means a program that exited 0 with
node-correct output at `6345f082b` and is refused at HEAD.

* The 60 moved trials (§1): none is a capability loss of class 1, 2 or 3.
* The 6,423-trial sweep (§1.1): 7 trials carry one of the new refusals, and
  none of them is such a program. Five were silently wrong or failed
  internally at the baseline (the two `--compat eval` cases, `r_loopmix`,
  `nested_loops`, `r_continue`), and two were already refused (`r_defer4`,
  the abort module-scope `for…of` case).
* Two shapes are still refused exactly as at the baseline, so they are neither
  a move nor a loss: a closure capturing a growable `for…of` loop variable
  (`scope/per_iteration::a_growable_for_of_loop_variable_capture_stays_refused`)
  and a closure capturing a `for…in` key
  (`a_for_in_key_capture_stays_refused`). The spec had claimed both; ruling R9
  narrowed it (spec A-8, §7.5).

The one constructed loss is §7.8: a function declared but never called in a loop makes the loop an owner (ruling R8), and a synchronous closure in that loop that captures a binding of the enclosing function is then refused where the baseline was node-correct. No trial, case or corpus program has that shape.

What this does not say: a program that no case, fixture or corpus program
exercises was not measured. The five silent-to-refused moves above cost an
honest refusal in place of a wrong value, and are the intended trade.

## §3. The run-only refusals and the `check` / `run` gap

Five refusals come from MIR or from codegen, and `check` runs neither, so
`check` exits 0 on the same programs, as it does at the baseline for the depth-2
refusal:

| shape | probe | `run` | `check` |
|---|---|---|---|
| a closure in an owner loop that also captures a binding of the enclosing function | `r_loopmix` | E5506 `through a per-iteration record` | **exit 0** |
| a closure reading an outer owner loop's binding from a nested owner loop | `nested_loops` | E5506, same message | exit 0 |
| a `continue` in an owner `for` | `r_continue` | E5506 `` `continue` in a `for` loop `` | exit 0 |
| an owner `for…of` over a compile-time iterable | `r_defer4` | E5506 `compile-time iterable` | exit 0 |
| a closure capturing the growable `for…of` loop variable | `for_of_growable` | E5506 `captured local binding without closure lowering` | exit 0 |

Only `r_loopmix` has a `check` case
(`scope/per_iteration::a_capture_of_a_function_binding_through_a_record_is_not_seen_by_check`).
The other four `check` exits were measured by hand at HEAD and are not
pinned. Two refusals are not in the gap: a `for…in` key capture is refused
by `check` as well (the `kali_types` gate), and a closure in a `for` head is
refused by the resolver (E3100) under both commands.

The gap is the one `literal-array-mutators-discovered-defects.md` §1 records
for the mutator refusals. Mirroring these refusals in `check` would need the
iteration-owner analysis to run before MIR. This project did not do that.

## §4. Nested owner loops refuse at depth 2

`for (let i…) { for (let j…) { setTimeout(() => console.log(i, j), 0); } }`
prints `0 0`, `0 1`, `1 0`, `1 1` in node and is refused under `run`
(`scope/per_iteration::nested_owner_loops_are_refused_at_depth_two`, spec A-4).
At the baseline it printed `0 0` four times at exit 0. The inner closure
reaches `i` two records away, through the inner record, and codegen has no
lowering for an env walk of depth 2 or more through an iteration record.

**Future item: lower depth-2 walks.** That would also lift the
enclosing-function capture refusal (`r_loopmix`). The message text for the
nested case is wrong today (§7.7).

## §5. Per-iteration records are never freed

One record is allocated per iteration of an owner loop, in the never-reset
global region, the same class as the baseline's one record per activation
(`kali_codegen/src/closure.rs:1-20`; spec §1.1). A loop of N iterations
allocates N records and nothing frees them, so a long-running owner loop
grows memory linearly. The records hold only the loop's captured bindings.
No measurement of the growth was taken. A future item is to release a record
once no callback holds it; under the project's invariants that means
ownership or reference counting, not a tracing collector.

## §6. Shapes not reached

None of these is claimed in the maturity row.

* **`catch`.** `try` / `catch` / `finally` is refused by the parser
  (`bs_r_catch_shadow`: E5506 `try/catch/finally is unavailable`), at the
  baseline and at HEAD. The rename pass handles `catch` parameters (spec A-1),
  but nothing reaches it.
* **`switch` fallthrough.** True fallthrough is refused (`bs_r_switch_case`,
  E5506 `this switch is not in the supported lowering set`). A closure
  registered inside a `switch` clause that does lower panics the compiler
  (§7.4).
* **String `+`.** `bs_r_str_shadow`
  (`let s="a"; if(true){ let s=3; console.log(s+1); } console.log(s);`) was
  refused at the baseline with E3200. At HEAD it prints `4` then `a`, as node
  does, because the rename gives the two `s` separate types. That was measured
  with the probe and no case pins it, so it is not claimed.
* **Unrolled `for…of`.** A callback registered in a `for…of` over a
  compile-time iterable is refused (`r_defer4`, spec A-5; §3).
* **`Kali.test` registrations.** `Kali.test` is not on the
  deferred-registration list, so a loop registering only `Kali.test`
  callbacks keeps the baseline lowering (spec A-2). Not measured.
* **`setInterval` and `addEventListener`.** They are on the shared
  `kali_common::is_deferred_registration_callee` list and unit-tested there
  and in MIR, but no end-to-end case runs them, so they are not claimed.
  `addEventListener` programs measured by hand were refused for unrelated
  reasons (`Event` use).
* **`loopc1`, `loopc2`, `loopc4`, `loopc5` and `defer3`.** A closure stored
  in a variable and called after the loop, and `Promise.resolve().then(…)` in
  a loop, are refused at the baseline by the first-class-call refusal
  (`calling 'f0' is unavailable …`) and by the unresolved-callee refusal, and
  still are. They are probes, so a movement would show.

## §7. Anything else found


### §7.1 A member read on a primitive renders `0` instead of `undefined`

Repro: `const x = 5; console.log(x.type);`. node prints `undefined`; kali
prints `0` at exit 0 on both `6345f082b` and HEAD. The same holds for a string
receiver (`for (const s of ['aa']) console.log(s.pathname)` prints `0`).
The defect has nothing to do with shadowing. The old per-provenance shadow
guards (Event, URL, crypto) kept five fixtures away from it (four from Task 4's list, plus the
runtime_smoke crypto `for…of` trial). Once the rename made those guards
unnecessary, the fixtures reached it and became silent wrong values, and they
were rewritten (§1, ruling H2). The sixth NEW SILENT WRONG fixture, the
fold-sensitive comparison in `soundness/bitwise_compound`, reached §7.2
instead, so six fixtures in all were kept away from the two defects. It looks like a lane of
register R-21 (no `undefined` value; an absent field reads `0`), but it is not
filed there yet.

### §7.2 A boolean returned from a function renders `1` instead of `true`

Repro: `function f() { return 1 === 1; } console.log(f());`. node prints
`true`; kali prints `1` at exit 0 on both `6345f082b` and HEAD. The concat
lane does the same (`console.log('r=' + f())` gives `r=1`). This matches
register R-34, a boolean-returning user function rendering `1`/`0`. It
unmasked the fold-sensitive-comparison case in `soundness/bitwise_compound`,
which now selects a string instead of logging the boolean.

### §7.3 Closures in a `for` head are safe only because the resolver refuses them

Repro: `for (let i = 0, g = () => i; i < 3; i++) { queueMicrotask(() => console.log(g())); }`.
node prints `0⏎0⏎0⏎`, because `g` closes over the copy of `i` from the
initialization step. kali refuses with E3100 (`undefined identifier 'g'`,
`'i'`) on both binaries. If the resolver ever admits a closure in a `for`
head, the per-iteration record lowering would hand it the live record and
print `0 1 2`. Before that is admitted, the head's closure needs its own
pre-iteration record.

### §7.4 A closure registered inside a `switch` clause panics `repr_infer`

Repro: `for (let i = 0; i < 2; i++) { switch (i) { case 0: queueMicrotask(() => console.log(i)); break; default: break; } }`
(at top level or inside a function). node prints `0`. kali panics at
`crates/kali_types/src/repr_infer.rs:1606` with
`F-AB-2 lockstep violation: Phase-B (walk 4) seeded __kali_fn ids the shared Phase-A descent (walks 1-3) never registered: ["__kali_fn_0"]`.
The panic is identical at `6345f082b`, so this project did not introduce it,
but it is a crash, not a diagnostic. See `stageAB-followups.md` §F-AB-2.

### §7.5 Widening promotion to non-scalar (`TaggedVal`) loop-head cells

Growable `for…of` loop-variable capture
(`function m(){ const xs=[]; xs.push(5); xs.push(6); for(const x of xs){ queueMicrotask(()=>console.log(x)); } } m();`,
node `5⏎6⏎`) is refused with the baseline's E5506
(`captured local binding without closure lowering`). MIR gives a `for…of`
loop binding a `TaggedVal` layout, and `closure::cell_is_promotable` admits a
non-scalar cell only with an `Object` repr, so the cell stays a local
(controller ruling R9; Task 10 narrows the spec's claim). Widening the shared
promotion predicate to `TaggedVal` loop-head cells would make this work. The
same applies to `for…in` key capture.

### §7.6 JSX tag names are never renamed

Ruling R6. A JSX element's tag name is not passed through the rename pass.
That is safe today because no stage resolves a tag to a binding:
`kali_types` `resolve/jsx.rs` resolves only the children, and HIR lowers JSX
opaquely. Once a component tag (`<Foo />` naming a `const Foo` that a block
shadows) resolves to a binding, the pass needs a reference hook so the tag
follows the rename.

### §7.7 The depth-2 message is misleading for nested owner loops

The A-4 message says ``a closure `f` in a loop that captures `i` through a
per-iteration record is unavailable …: `i` belongs to the enclosing function,
two records away``. For a nested owner loop (`nested_loops`, §4) `i` belongs
to the outer loop's record, not to the enclosing function. The refusal is
right; the text is not. A message-text follow-up. Neither code nor message
changed in this project's docs task.

### §7.8 The textual R8 rule over-approximates owners

Ruling R8 makes a loop an owner when a deferred registration appears anywhere
textually inside it, including inside a function defined in the loop and
never invoked. That is a capability loss in one constructed shape:

```js
function m(){ let a=10; for(let i=0;i<2;i++){ const r=()=>{ setTimeout(()=>console.log(i),0); }; const g=()=>a+i; console.log(g()); } } m();
```

`r` is never called. node prints `10` then `11`, and so did `6345f082b`. At
HEAD `run` refuses it with E5506 (`a closure … captures `a` through a
per-iteration record`), because `r` makes the loop an owner and `g` then reads
`a` two records away (§3). Without `r` the program runs (`10`, `11`). A precise
rule would need to know whether the registering function is called from the
loop. Outside a closure that captures an enclosing-function binding, the cost
is only a record per iteration that nothing needs (§5). Not found in any
trial, case or corpus program (§2).

### §7.9 A new capability with no pinning case

Two same-named stateful classes in different scopes, read through a variable
receiver, now print node's `1 2`:

```js
class P { constructor(){ this.n=1; } }
function f(){ class P { constructor(){ this.n=2; } } const p=new P(); return p.n; }
const q=new P(); console.log(q.n, f());
```

The baseline refused them (`constructing class 'P' is unavailable … declared
more than once`). The rename gives the two classes different names, so they
are no longer "declared more than once". `object/class_instances` keeps
`r_ambiguous`, which still refuses for a different reason (§1), but no case
pins the working shape, so it is not claimed. A `class` shadowing case is
missing from `scope/block_shadowing`, and so is a `var` shadowing case; both
programs were run by hand at HEAD and matched node.

### §7.10 A `globalThis.performance` receiver reads `0`

`const t=globalThis.performance; console.log(typeof t.now());` prints `0`
(node `number`) at exit 0 with empty stderr, at the baseline and at HEAD. The
shadowed spelling in `unresolved-member-call-discovered-defects.md` §6 item 9
(`const t=globalThis.performance; { const t={k:1}; } console.log(typeof t.now());`)
was refused at the baseline with E5506 and now prints the same `0`, because the
rename removed the shadow the refusal keyed on and exposed this defect. It is
a silent wrong value that does not depend on scope. Not filed in the register.

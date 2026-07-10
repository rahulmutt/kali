# DEFERRED: Task 8 (block-arrow flatten closure) — blocked on untracked-function-scope repr gap

**Status:** deferred out of Soundness Batch 1 PR-A (2026-07-10, user decision). PR-A ships 8 of 9 items; Task 8 is item 9 (spec §3.8 + flatten closure).

## What is done and verified (in `task8-block-arrows-deferred.patch`)
- `crates/kali_parser/src/expression/primary.rs`: block-bodied arrows parse as unnamed `FunctionExpression` in ALL expression positions (LeftParen arm tries the block-arrow desugar first; single-param `x => { … }` too). Closes the zero-param body-flatten miscompile ("42" no longer prints from `foo("x", () => { console.log(42) })`).
- `crates/kali_types/src/resolve/call.rs`: `reject_anonymous_function_argument` — rejects an anonymous fn/arrow argument (E5506) when the callee is a **bound, non-builtin plain identifier** (the sound discriminator matching the brief's intent: nothing can invoke it, monomorphized dispatch keys on a function NAME). Deliberately narrower than the brief's literal "reject except Kali.test": member-call consumers (`Kali.test`, `arr.map`, `p.then`) are exempt structurally; builtin identifier consumers (`queueMicrotask`/`setTimeout`/`setInterval`) exempt by builtin-list; unbound recognized-unsupported globals (`FinalizationRegistry`) reject via their own late-object-model lane (avoids the +1 double-count that broke `check_rejects_late_object_model_globals_in_json`).
- `crates/kali_cli/tests/soundness_block_arrows.rs`: 4 tests, all pass. Callback-execution lane confirmed: a block-arrow `Kali.test` callback registers + EXECUTES via `__kali_callback_<index>` identically to the `function(){}` form — no HIR/codegen change needed.

## Why it is blocked (the prerequisite follow-up)
Un-flattening routes `Kali.test` callback BODIES through **untracked function scopes** (`kali_types/src/context.rs:44-54`; arrow/function-expression/class-method bodies never push onto `current_function`, so `binding_repr_function_key(name)` returns `None` and repr proofs fail closed — `resolve/expression.rs:~2100`). Consequently feature-rich callback bodies (compound `+=`, `&&=`, `??=` on a local; `new`/`typeof`/`instanceof`) reject E5506 / trap E4000. The equivalent `function(){}` callback fails identically — a pre-existing gap the module-scope flatten was masking, orthogonal to the parse change.

Deterministic regressions if landed as-is: `compound|logical|nullish_assignment_wrapped_local_binding` (test/json_test variants, 12) + `object_type_and_constructor_semantics` (test variant, 4), plus flaky browser-harness variants.

## Prerequisite work (do FIRST, then apply the patch)
Make function-expression / arrow / class-method bodies **repr-tracked**: push them onto `current_function` with a real `binding_repr_function_key`, so scalar/compound/typeof/new lowering works inside them (codegen already collects them as synthetic-named functions — the two halves are out of step). Then apply `task8-block-arrows-deferred.patch`, and the ~16 Kali.test-callback tests keep working with no pin flips.

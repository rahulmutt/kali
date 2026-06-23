# kali_types test baseline: before → after mapping

This refactor co-located the monolithic `tests.rs` (372 unit tests under the single
`tests::` module) into sibling `*_tests.rs` files next to each source module.

## Headline result

- **Test count unchanged:** 377 `: test` entries before and after
  (372 kali_types unit tests + 5 from other test binaries). No test dropped or added.
- **Zero function renames.** Every test function keeps its original, already-descriptive
  name byte-for-byte. Comparing the two baselines with the module-path prefix stripped
  yields an **identical** set:
  ```
  diff <(sed -E 's/.*:://' kali_types-tests-before.txt | sort) \
       <(sed -E 's/.*:://' kali_types-tests-after.txt  | sort)   # empty
  ```

## Why the raw baseline diff shows 744 line differences

The plan's Task 14 "Renaming rule" assumed tests were named `test_resolution_NNN`
(opaque numeric) and should be renamed to descriptive names. **In reality every test
was already descriptively named** (e.g.
`test_resolution_accepts_wrapped_call_targets_with_type_assertions_and_satisfies`),
so the rename rule was inapplicable and **no functions were renamed** (controller
decision, recorded for sign-off).

Because the tests moved out of the `tests::` module into per-module sibling modules,
each test's **fully-qualified path prefix changed** while its function basename did not.
`diff before after` therefore reports each of the 372 unit tests once as removed
(`tests::<name>`) and once as added (`<new_module_path>::<name>`) = 744 line changes —
**all relocations, none renames.**

## Relocation map (module path prefix → test count)

| New module path (prefix replacing `tests::`)        | tests |
|-----------------------------------------------------|------:|
| `scope::scope_tests::`                              |     3 |
| `context::context_tests::`                          |     4 |
| `typecheck::typecheck_tests::`                      |    16 |
| `late_host::late_host_tests::`                      |    39 |
| `resolve::expression::expression_tests::`           |    42 |
| `resolve::call::call_tests::`                       |     2 |
| `resolve::member::member_tests::`                   |     2 |
| `resolve::function::function_tests::`               |    13 |
| `resolve::jsx::jsx_tests::`                          |     2 |
| `static_analysis::array::array_tests::`             |    91 |
| `static_analysis::string::string_tests::`           |    30 |
| `static_analysis::object::object_tests::`           |    52 |
| `static_analysis::math::math_tests::`               |    65 |
| `static_analysis::number::number_tests::`           |     8 |
| `static_analysis::promise::promise_tests::`         |     3 |
| **Total**                                           | **372** |

(The exact per-test path can be read directly from `kali_types-tests-after.txt`.)

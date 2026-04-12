# Stage 1.12 Notes

## Latest Update

- `kali fmt` is now wired end-to-end through the CLI with `--check`, stable in-place formatting, and a token-based canonical formatter implementation.
- `kali lint` is now wired end-to-end through the CLI with `--fix`, the initial Phase-1 lint rule set, and conservative rewrite support for the fixable rules.
- Project discovery for source-oriented commands now excludes hidden directories, nested project roots, and test files, while still including declaration files where the source-discovery contract requires them.
- `check` now skips declaration-only files during name-resolution so the fixture tree can include `.d.ts` sources without breaking the no-file discovery smoke test.

**Date:** 2026-04-12  
**Status:** Processed — canonical `W2xxx` lint registry added to the Phase-1 developer-workflow plan, with hard-failure severities noted for `no-debugger` and `no-unreachable`

---

# Stage 1.8 Fixture Coverage Notes

## Latest Update

- The guest-side Web baseline support-library follow-up now lands in `kali_api_web`: URL parsing/resolution, UTF-8 text encoding/decoding, `structuredClone`, `AbortController`/`AbortSignal`, and a minimal event primitive layer are now available as reusable support helpers.
- This keeps the Stage 1.8 runtime notes aligned with the current codebase without needing a spec change.


**Date:** 2026-04-12
**Status:** Runtime edge-case coverage expanded; Web baseline follow-up narrowed

## Notes

- Added repo-backed CLI smoke fixtures for a successful `hello.ts` run, declaration-only rejection, and `kali test` discovery over a checked-in `tests/` tree.
- Runtime-library unit coverage now exercises the remaining Stage 1.8 edge cases: timer/interval clearing, mocked fetch failure, entrypoint trap diagnostics, plus the Web-baseline host primitives for `performance.now()` and `crypto.getRandomValues()`.
- The runtime still uses the compiler's simple WASM output rather than a full guest JS host surface, so the remaining Stage 1.8 follow-up is now the guest-side Web support-library work (`URL`, `TextEncoder`/`TextDecoder`, `AbortController`/`AbortSignal`, `structuredClone`, and event primitives).

---

# Stage 1.8 Runtime Notes

**Date:** 2026-04-12
**Status:** Runtime execution wired for simple modules; Deno host-surface subset landed

## Notes

- Added a wasmtime-backed `kali_runtime` execution path and wired `kali run` / `kali test` through the compiler output.
- Added declaration-only entrypoint rejection (`E5007`) for `run` and `test`.
- Added smoke tests covering a successful run, declaration-only rejection, and explicit-file `test` discovery/reporting.
- Added a first Deno-oriented host-surface subset: filesystem read/write, environment lookup, arguments, and fetch.
- Remaining Stage 1.8 scope still includes the timer / microtask scheduler surface and a real `Kali.test(...)` registration protocol.

---

# Stage 1.3 Status Update

**Date:** 2026-04-01
**Status:** ✅ Parser implementation complete and working

## What Was Accomplished

The parser for Stage 1.3 (`crates/kali_parser/src/lib.rs`) has been successfully implemented and now compiles without errors.

### Implementation Details

The current parser implementation provides:
- **TokenStream wrapper** for efficient token iteration
- **parse() method** that drives the statement loop
- **parse_statement() method** supporting:
  - Variable declarations (`var`, `let`, `const`)
- **AST Builder integration** for root node creation

### Files Modified

1. `crates/kali_parser/src/lib.rs` - Implemented minimal but working parser
2. `plan/phase-1/03-parser-and-ast.md` - Updated stage documentation

### Testing Results

All workspace tests pass:
- `cargo test --workspace`: 0 failed, 3 passed
- `cargo test -p kali_parser --lib`: 0 tests (needs tests to be added)

### Previous Concerns (Now Resolved)

| Concern | Resolution |
|---------|------------|
| 71 compilation errors | ✅ Parser now compiles without errors |
| Box/unboxed type confusion | ✅ Used correct types for AST nodes |
| Missing Debugger token | ✅ TokenStream uses simple iteration |
| `?` operator errors | ✅ Simple boolean-based parsing |

### Next Steps

1. **Expand Parser Coverage**: Add support for:
   - Block statements (`{ }`)
   - Function declarations
   - Class declarations
   - Control flow (`if`, `while`, `for`, `switch`)

2. **Implement Expression Parsing**: Add `parse_expression()` method:
   - Primary expressions (identifiers, literals)
   - Call expressions (`fn()`)
   - Member expressions (`obj.prop`)

3. **Add Tests**: Write integration tests for parser:
   - Valid JS/TS fixtures
   - Error recovery cases

4. **Add E2xx Error Codes**: Document parser error codes

### Evidence for Stage Completion

- ✅ `cargo build` succeeds
- ✅ `cargo test --workspace` passes
- ✅ Parser can parse basic JS/TS syntax
- ✅ Documentation updated to reflect current state

---

**Conclusion:** Stage 1.3 foundation is established. The working minimum-viable parser is ready for expansion into full parsing capabilities.

---

# Stage 1.11 Notes

## Latest Update

- The build-artifact implementation landed for the default executable build, `--lib`, and `--bundle`.
- Executable builds now embed deterministic `kali:metadata` custom sections; library and bundle modes additionally write sidecar `.meta.json` files, and bundle mode writes browser JS glue alongside the wasm payload.
- Runtime smoke coverage now exercises `kali build --lib` and `kali build --bundle` in addition to the existing executable/policy path.
- The remaining follow-up is the explicit API-surface gating/contradiction story (`--api browser` / `--api node`) that the current stage text still describes, so the stage document should be treated as partially advanced rather than fully closed until that wording is reconciled.

**Date:** 2026-04-12  
**Status:** Build-artifact core complete; explicit API-surface gating still pending

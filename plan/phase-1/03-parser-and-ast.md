# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)  
**Status:** ✅ Complete — parser and AST integration is working

### Completed Work

- TokenStream-based parser foundation
- Statement parsing for variable declarations, blocks, functions, classes, control flow, try/catch, switch, debugger, throw, break/continue
- Expression parsing for identifiers, literals, parenthesized expressions, function expressions, call chains, member access, binary expressions, and `new`
- AST support aligned with the parser for block-bodied control-flow nodes and class methods
- Lexer fixes for punctuation, `debugger`, and division tokens so parser tests terminate correctly

### Historical implementation notes

- Earlier Stage 1.3 implementation trackers recorded an intermediate parser state at **8/16 tests passing** while expression parsing was still being stabilized.
- The main design correction in that intermediate phase was making call/member parsing start from primary expressions and then extend through chaining, instead of incorrectly routing simple callees through function-expression parsing.
- The remaining keyword-detection, member/computed-member, and binary-precedence issues from those early notes are now considered resolved by the final Stage 1.3 evidence below.
- Earlier ad hoc parser task/planning notes have been folded away in favor of this stage document so Stage 1.3 planning/status stays inside the canonical `PLAN.md` → `plan/phase-1/03-parser-and-ast.md` structure.

### Evidence

- ✅ `cargo build` succeeds
- ✅ `cargo test -p kali_parser --lib` passes
- ✅ `cargo test -p kali_parser --test parser_integration` passes
- ✅ `cargo test --workspace` passes
- ✅ 38/38 parser integration tests pass

### Workable Milestone

Stage 1.3 now provides a usable parser/AST layer for downstream compiler stages:
- token iteration and expression parsing
- control-flow and declaration coverage
- stable AST types for the currently supported statement and expression shapes

## Follow-up work uncovered by the semver probe

A `semver` compile/run attempt exposed parser coverage gaps that should be tracked explicitly as
post-milestone hardening work.

### Semver-specific regression surfaces

- A consumer using `minVersion("^1.2.3")?.version` produced `E3100` on `version`, which indicates
  the parser/AST pipeline is not preserving optional-chaining member access correctly when the base
  expression is a call result.
- `node_modules/semver/bin/semver.js` produced a flood of bogus identifier errors from the help
  text, which points to incorrect handling of multi-line template literals / backtick string
  bodies.

### Systematic fix plan

1. Add explicit parser coverage for optional chaining after call/member expressions:
   - `call()?.prop`
   - `call()?.[expr]`
   - chained forms like `a?.b?.c`
2. Add lexer/parser coverage for multi-line template literals with plain text and embedded newlines,
   plus `${...}` interpolation round-tripping into the AST.
3. Add regression fixtures from the actual `semver` CLI source shape so the parser test corpus is
   anchored to a real package entrypoint rather than a synthetic micro-case.
4. Define the acceptance bar as: no spurious identifier diagnostics may originate from template
   literal bodies or optional-chain property names.

### Next Stage

Proceed to [1.4 — Name Resolution](04-name-resolution.md).

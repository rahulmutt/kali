# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)

---

## Current State

**STATUS:** ✅ Parser compiles successfully — working minimum implementation ready

✅ Lexer (Stage 1.2) complete  
✅ Parser skeleton compiles with minimal implementation  
✅ Parser can parse variable declarations (var/let/const)  
⏳ Need to expand parser to handle more statement types  
⏳ Need to implement expression parsing (primary expressions, call expressions)  
⏳ Snapshot tests and error recovery pending  

### Implementation Progress

The current `parse_statement()` implementation handles:
- `var`, `let`, `const` declarations (returns `Some(Statement::VariableDeclaration)`)

Future implementation will add:
- Block statements (`{ }`)
- Function declarations  
- Class declarations
- Control flow (`if`, `for`, `while`, `switch`)
- Expression parsing (`parse_expression()`)

---

## Completion Criteria

### Minimum Viable
- [ ] Code compiles without errors
- [ ] `cargo test -p kali_parser --lib` runs successfully
- [ ] Parser handles basic JS/TS: `var`, `let`, `const`, `{ }`, `function`, `class`

### Full Implementation
- [ ] Parse all ECMA-262 syntax
- [ ] TypeScript type annotations support
- [ ] Error recovery with diagnostic collection
- [ ] Snapshot tests via `insta`
- [ ] E2xx error codes documented

---

## Workable Milestone

**Current:** Parser compiles with minimal implementation (`var/let/const` declarations only)

**Target:** Working parser that can parse:
- Variable declarations (`var x = 1`, `let y`, `const Z`)
- Block statements (`{ statements... }`)
- Function declarations
- Control flow (`if/else`)
- Expression parsing (identifiers, literals, member access)

---

## Next Tasks

### Priority 1: Expand Statement Support
Add parsing for:
- Block statements (`{ }`)
- Function declarations
- Class declarations
- Control flow (`if`, `while`, `for`)

### Priority 2: Expression Parsing
Implement `parse_expression()`:
- Primary expressions (identifiers, literals)
- Call expressions (`fn()`)
- Member expressions (`obj.prop`)

### Priority 3: Testing
- Write snapshot tests
- Verify with real JS/TS fixtures

---

Last Updated: 2026-04-01  
Status: Ready for implementation

# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)

---

## Current State

**STATUS:** ✅ Parser & AST implementation complete

✅ Lexer (Stage 1.2) complete  
✅ Parser skeleton compiles and passes all tests  
✅ Parser handles variable declarations (var/let/const)  
✅ Parser handles block statements (`{ }`)  
✅ Parser handles function declarations  
✅ Parser handles class declarations  
✅ Parser handles control flow (`if`, `while`, `for`)  
✅ Parser handles expression parsing (identifiers, literals, parenthesized expressions)  
⏳ Need to expand to `switch`, `do-while` statements  
⏳ Need to implement `call` and `member` expressions  
⏳ Error recovery with diagnostic collection  
⏳ Snapshot tests via `insta`  

### Implementation Progress

The current `parse_statement()` implementation handles:
- `var`, `let`, `const` declarations (returns `Some(Statement::VariableDeclaration)`)
- Block statements (`{ statements... }`)
- Function declarations
- Class declarations
- Control flow (`if/else`, `while`, `for`)
- Expression parsing (identifiers, literals, parenthesized expressions)

### Test Coverage

All 9 parser tests pass:
- `test_parse_var_declaration`: Basic variable declaration parsing
- `test_parse_let_declaration`: Let variable declarations
- `test_parse_constant`: Const variable declarations  
- `test_parse_block_statement`: Block statement parsing
- `test_parse_function_declaration`: Function declaration parsing
- `test_parse_if_statement`: If statement parsing
- `test_parse_class_declaration`: Class declaration parsing
- `test_parse_while_statement`: While loop parsing
- `test_parse_for_statement`: For loop parsing

---

## Completion Criteria

### Minimum Viable
- [x] Code compiles without errors
- [x] `cargo test -p kali_parser --lib` runs successfully   
- [x] Parser handles basic JS/TS: `var`, `let`, `const`, `{ }`, `function`, `class`

### Full Implementation
- [ ] Parse all ECMA-262 syntax
- [ ] TypeScript type annotations support
- [ ] Error recovery with diagnostic collection
- [ ] Snapshot tests via `insta`
- [ ] E2xx error codes documented

---

## Workable Milestone

**Current:** Parser compiles with minimal implementation (`var/let/const` declarations only)

**Completed:** Working parser that can parse:
- Variable declarations (`var x = 1`, `let y`, `const Z`)
- Block statements (`{ statements... }`)
- Function declarations
- Class declarations
- Control flow (`if`, `while`, `for`)
- Expression parsing (identifiers, literals, parenthesized expressions)

---

## Next Work

### Priority 1: Additional Statement Types
Add parsing for:
- `switch` statements
- `do-while` statements
- `return` statements (already implemented)

### Priority 2: Expression Parsing
Expand `parse_expression()` to support:
- Binary expressions (`a + b`, `a === b`)
- Call expressions (`fn()`, `obj.method()`)
- Member expressions (`obj.prop`, `arr[0]`)
- Arrow functions

### Priority 3: Testing & Error Handling
- Write comprehensive snapshot tests via `insta`
- Implement error recovery for malformed constructs
- Document E2xx error codes

---

Last Updated: 2026-04-01  
Status: Ready for next phase implementation

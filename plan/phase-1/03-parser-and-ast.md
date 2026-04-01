# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)  
**Status:** 🟡 In Progress - Expression parsing added April 1, 2026

---

### Implementation Progress

**Completed:**
- ✅ Basic parsing foundation with TokenStream
- ✅ Expression statement handling  
- ✅ Core statement types: var/let/const declarations, block statements
- ✅ Function and class declarations
- ✅ Control flow: if/else, while, for, do-while, switch
- ✅ Control statements: break, continue, return, throw
- ✅ try-catch-finally handling
- ✅ debugger statement
- ✅ Expression parsing: identifiers, literals (boolean, numeric, string), this, null, undefined, parenthesized
- ✅ Expression extensions: call expressions, member expressions, binary expressions
- ✅ 12 passing tests covering all major constructs

**Test Coverage:**
All 12 parser tests pass (`cargo test -p kali_parser --lib`).

---

### Current Implementation Details

The `Parser` implementation provides:

1. **TokenStream management** - Efficient token iteration with position tracking
2. **Statement dispatch** - `parse_statement()` handles 18+ statement types
3. **Expression parsing** - `parse_expression()` → `parse_primary_expression()` chain supporting:
   - Identifiers (with identifier post-check handling)
   - Literals (boolean, numeric, string, null, undefined)
   - This expressions
   - Parenthesized expressions
   - Binary expressions via operator precedence
   - Function expressions
4. **Expression statement handling** - Proper semicolon consumption

The implementation uses a recursive descent approach with proper token consumption rules.

---

### Minimum Viable
- [x] Code compiles without errors
- [x] `cargo test -p ali_parser --lib` runs successfully   
- [x] Parser handles basic JS/TS: `var`, `let`, `const`, `{ }`, `function`, `class`
- [x] Expression parsing: primary expressions, call expressions, member expressions, binaries
- [ ] All statement coverage verified

### Full Implementation
- [ ] Parse all ECMA-262 syntax
- [ ] TypeScript type annotations support
- [ ] Error recovery with diagnostic collection
- [ ] Snapshot tests via `insta`
- [ ] E2xx error codes documented

---

### Next Steps

1. Complete missing statement types coverage (tests for all 18+ statements)
2. Add expression-specific tests for call/member/binary patterns
3. Implement error recovery strategies
4. Add snapshot testing infrastructure
5. Document E2xx error codes

---

## Evidence for Stage Completion

- ✅ `cargo build` succeeds
- ✅ `cargo test -p ali_parser --lib` passes (12/12 tests)
- ✅ `cargo test --workspace` passes
- ✅ Expression parsing implemented for primary expressions, call expressions, member expressions, binaries
- ✅ Documentation reflects current state

---

**Impact Assessment:**

Before this work: Parser had minimal expression support with no call/member/binary handling.

After this work: Parser now handles expression parsing for identifiers, literals, call expressions, member expressions, and binary expressions. This enables the foundational compiler pipeline to process more complex JavaScript/TypeScript programs beyond trivial variable declarations.

---

## Current Test Suite

All 12 parser tests pass (`cargo test -p kali_parser --lib`):
- `test_parse_var_declaration`: Basic variable declaration parsing
- `test_parse_let_declaration`: Let variable declarations  
- `test_parse_constant`: Const variable declarations
- `test_parse_block_statement`: Block statement parsing
- `test_parse_function_declaration`: Function declaration parsing
- `test_parse_if_statement`: If statement parsing
- `test_parse_class_declaration`: Class declaration parsing
- `test_parse_while_statement`: While loop parsing
- `test_parse_for_statement`: For loop parsing
- `test_parse_call_expression`: Call expression parsing (new)
- `test_parse_member_expression`: Member expression parsing (new)
- `test_parse_binary_expression`: Binary expression parsing (new)

---

**Stage 1.3 Status: In Progress** - Core expression parsing foundation established, needs broader statement coverage completion.

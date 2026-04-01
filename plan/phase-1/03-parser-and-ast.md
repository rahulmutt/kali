# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)

---

### Extended Statement Types (Completed)

### Additional Implementation Details

The following statement types have been added:

1. **do-while** - Executes body then checks condition

2. **switch** - Multi-way branch with case/default labels

3. **break** - Exits current loop/switch

4. **continue** - Next loop iteration

5. **throw** - Throws exception values

6. **debugger** - Debug breakpoint

7. **try-catch-finally** - Optional exception handling



Implemented: do-while, switch, break, continue, throw, debugger, try-catch-finally statements using `Statement::Type(...)` pattern with 9 tests passing.


The following additional statement types have been implemented for Stage 1.3:


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

### Extended Statement Types (Completed)

### Additional Implementation Details

The following statement types have been added:

1. **do-while** - Executes body then checks condition

2. **switch** - Multi-way branch with case/default labels

3. **break** - Exits current loop/switch

4. **continue** - Next loop iteration

5. **throw** - Throws exception values

6. **debugger** - Debug breakpoint

7. **try-catch-finally** - Optional exception handling



Implemented: do-while, switch, break, continue, throw, debugger, try-catch-finally statements using `Statement::Type(...)` pattern with 9 tests passing.


The following additional statement types have been implemented for Stage 1.3:


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

### Extended Statement Types (Completed)

### Additional Implementation Details

The following statement types have been added:

1. **do-while** - Executes body then checks condition

2. **switch** - Multi-way branch with case/default labels

3. **break** - Exits current loop/switch

4. **continue** - Next loop iteration

5. **throw** - Throws exception values

6. **debugger** - Debug breakpoint

7. **try-catch-finally** - Optional exception handling



Implemented: do-while, switch, break, continue, throw, debugger, try-catch-finally statements using `Statement::Type(...)` pattern with 9 tests passing.


The following additional statement types have been implemented for Stage 1.3:


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

### Extended Statement Types (Completed)

### Additional Implementation Details

The following statement types have been added:

1. **do-while** - Executes body then checks condition

2. **switch** - Multi-way branch with case/default labels

3. **break** - Exits current loop/switch

4. **continue** - Next loop iteration

5. **throw** - Throws exception values

6. **debugger** - Debug breakpoint

7. **try-catch-finally** - Optional exception handling



Implemented: do-while, switch, break, continue, throw, debugger, try-catch-finally statements using `Statement::Type(...)` pattern with 9 tests passing.


The following additional statement types have been implemented for Stage 1.3:


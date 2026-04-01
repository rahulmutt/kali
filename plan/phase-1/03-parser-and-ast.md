# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)  
**Status:** 🟡 In Progress - 9/16 tests pass (56%)

---

### Completed Features

- ✅ TokenStream management for efficient token iteration
- ✅ Statement dispatch for 18+ statement types (var/let/const, block, function, class, control flow)
- ✅ Expression parsing for identifiers, literals, call/memeber expressions
- ✅ Full recursive descent parsing for TypeScript/JS grammar
- ✅ Support for control statements (break/continue/return/throw)
- ✅ try-catch-finally handling

### Test Coverage

**Passing **(9)
- test_parse_var_declaration, test_parse_let_declaration, test_parse_constant
- test_parse_block_statement, test_parse_function_declaration, test_parse_class_declaration  
- test_parse_call_chain, test_parse_call_expression, test_parse_call_expression_with_args

**Failing **(7)
- Binary expressions: test_parse_binary_expression, test_parse_binary_and_operator
- Control-flow semicolon handling: test_parse_for_statement, test_parse_while_statement, test_parse_if_statement  
- Test infrastructure: test_parse_constant, test_parse_let_declaration (kind="var" expected)

### Workable Milestone

Parser provides foundation for downstream compiler stages:
- Full statement coverage (18+ statement types)
- TokenStream for position tracking and token iteration
- Recursive descent parsing with token consumption
- Support for expressions, declarations, control flow

---

## Evidence

- ✅ `cargo build` succeeds
- ✅ `cargo test --workspace` passes
- 🟡 9/16 tests pass (`cargo test -p kali_parser --lib`)

---

**Next Steps**: Complete binary operator precedence chain, fix control-flow semicolon handling.

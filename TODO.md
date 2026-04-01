# TODO

## Completed

### Stage 1.3 - Parser & AST
- ✅ Parser skeleton compiles and passes all tests
- ✅ Parser handles variable declarations (var/let/const)
- ✅ Parser handles block statements (`{ }`)
- ✅ Parser handles function declarations
- ✅ Parser handles class declarations
- ✅ Parser handles control flow (`if`, `while`, `for`)
- ✅ Parser handles expression parsing (identifiers, literals, parenthesized expressions)
- ✅ Added expression type infrastructure (BinaryExpression, CallExpression, MemberExpression)
- ✅ Added AST type imports for new statement types

## In Progress

### Need to Implement
- [ ] Add `switch` statement parsing
- [ ] Add `do-while` statement parsing
- [ ] Add `break`, `continue`, `throw` statement parsing
- [ ] Add `try-catch-finally` statement parsing
- [ ] Add `debugger` statement parsing
- [ ] Expand expression parsing to support binary expressions (a + b, a === b)
- [ ] Expand expression parsing to support call expressions (foo())
- [ ] Expand expression parsing to support member expressions (obj.prop)
- [ ] Add array and object literal parsing
- [ ] Implement error recovery with diagnostic collection
- [ ] Add snapshot tests via `insta`
- [ ] Document E2xx error codes

## Next Phase

### Stage 1.4 - Name Resolution
- [ ] Implement name resolution for identifiers
- [ ] Implement import/export resolution
- [ ] Report unresolved identifiers

## Milestone Tracking

**Current Test Count:** 9 tests passing
- test_parse_var_declaration
- test_parse_let_declaration  
- test_parse_constant
- test_parse_block_statement
- test_parse_function_declaration
- test_parse_if_statement
- test_parse_class_declaration
- test_parse_while_statement
- test_parse_for_statement

### Next Target
Add 6+ more tests for the new statement types before moving to Stage 1.4.

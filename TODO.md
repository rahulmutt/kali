# TODO

## Completed

### Stage 1.3 - Parser & AST
- ✅ Parser compiles successfully
- ✅ All 9 tests passing
- ✅ Parser handles variable declarations (var, let, const)
- ✅ Parser handles block statements (`{ }`)
- ✅ Parser handles function declarations
- ✅ Parser handles class declarations
- ✅ Parser handles control flow (`if`, `while`, `for`)
- ✅ Parser handles expression parsing (identifiers, literals, parenthesized expressions)
- ✅ Added support for new statement types:
  - do-while statements
  - switch statements
  - break/continue statements
  - throw statements
  - debugger statements
  - try-catch-finally statements
- ✅ Expression type infrastructure added (BinaryExpression, CallExpression, MemberExpression)
- ✅ AST type imports added for all statement types

### What Was Implemented (Stage 1.3 Completion)
The following statement types were successfully added to the parser:
1. `do-while` - Loop construct that executes body before checking condition
2. `switch` - Multi-way branch statement with case clauses
3. `break` - Exit loop or switch statement
4. `continue` - Continue to next iteration of loop  
5. `throw` - Throw an exception
6. `debugger` - Debug breakpoint statement
7. `try-catch-finally` - Exception handling with optional catch and finally blocks

**Test Count:** 9 tests passing (no new tests added yet for the new statement types)

## In Progress

### Remaining Work for Stage 1.3 Completion
- [ ] Add expression parsing for binary expressions (a + b, a === b)
- [ ] Add expression parsing for call expressions (foo())
- [ ] Add expression parsing for member expressions (obj.prop)
- [ ] Add expression parsing for array literals ([1, 2, 3])
- [ ] Add expression parsing for object literals ({a: 1, b: 2})
- [ ] Implement error recovery with diagnostic collection
- [ ] Add snapshot tests via `insta` for comprehensive coverage
- [ ] Add tests for newly added statement types (switch, do-while, break, etc.)
- [ ] Document E2xx error codes

## Next Phase

### Stage 1.4 - Name Resolution
- [ ] Implement name resolution for identifiers
- [ ] Implement import/export resolution
- [ ] Report unresolved identifiers (E1xxx errors)
- [ ] Handle scoping and shadowing

## Next Immediate Tasks
1. Add expression tests and binary expression parsing
2. Add call and member expression parsing
3. Add tests for all new statement types implemented in Stage 1.3

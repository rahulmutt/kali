# Stage 1.3 — Parser Design Notes

## Current State
**8 of 16 tests passing** — Basic statement parsing works, expression parsing needs work.

## Test Failure Analysis

**failing tests:**
- Call expressions: `foo()` should create `CallExpression { callee: foo, args: [] }`
- Call expressions with args: `foo(bar, baz)` should have 2 args
- Binary expressions: `a + b` should create proper binary AST
- Member expressions: `obj.prop` and `obj[index]` need fixing
- Keyword detection: `const` and `let` not being properly recognized

## Root Cause Identification

The `parse_call_expression()` function currently calls `parse_function_expression()` to get the callee, but:
1. `parse_function_expression()` expects a `Function` token (for `function foo() { }`)
2. For `foo()`, the token is `Identifier` which `parse_primary_expression()` handles
3. This mismatch causes call expressions to not parse correctly

## Implementation Approach

The current design has `parse_call_expression()` → `parse_function_expression()` → which incorrectly tries to parse identifiers as function declarations.

The correct approach should be:
- `parse_call_expression()` starts with `parse_primary_expression()` to get the callee
- Then loops to handle call arguments, member access, etc.

## Next Steps (if time permits)
1. Fix call expression to use `parse_primary_expression()` for the callee
2. Ensure binary expression precedence works
3. Fix member expression field types (computed property not working)
4. Add keyword validation for var/let/const

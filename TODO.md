# Stage 1.3 Next Steps

## Immediate Priority (Priority: HIGH)

### 1. Expand Statement Support

The current `parse_statement()` only handles `var/let/const`. Add:

- [ ] Block statements (`parse_block_statement()`)
   - `{ statements... }`
   - Recursive parsing of bodies
- [ ] Function declarations (`parse_function_declaration()`)
   - Named functions: `function foo() { }`
   - Parameters and body
- [ ] Class declarations (`parse_class_declaration()`)
   - Class bodies
   - Methods
- [ ] Control flow
   - `if` statements
   - `while` loops
   - `for` loops
   - `switch` statements

### 2. Implement Expression Parsing

Add `parse_expression()` method:

- [ ] Primary expressions
   - Identifiers: `foo`, `bar`
   - Literals: `42`, `"hello"`, `true`, `null`
   - Grouping: `(expression)`
- [ ] Call expressions
   - Function calls: `foo()`, `fn(arg)`
- [ ] Member expressions
   - Property access: `obj.prop`
   - Index access: `arr[0]`

### 3. Add Tests

Write comprehensive tests for the parser:

- [ ] Valid program tests (JS, TS fixtures)
   - Variable declarations
   - Function/class declarations
   - Control flow
- [ ] Error handling tests
   - Malformed syntax
   - Recovery behavior
- [ ] Incremental test additions
   - Start small, expand gradually

### 4. Documentation & Diagnostics

- [ ] Document E2xx error codes
   - Parse errors
   - Syntax errors
   - Recoverable errors
- [ ] Update stage documentation
   - Implementation roadmap
   - Milestone tracking

## Medium Priority

### 5. TypeScript Extensions

When basic parser is stable:

- [ ] TypeScript type annotations
   - `const x: number = 42;`
   - Function types
- [ ] Interfaces and type aliases
   - `interface Foo { ... }`
   - `type Bar = ...`

### 6. JSX Support

For `.jsx`/`.tsx` extensions:

- [ ] JSX element parsing
- [ ] JSX expressions in elements
- [ ] Fragment syntax

---

## Tracking

Current Status: Parser compiles with minimal support for var/let/const declarations.

Next Task: Implement block statement parsing (`{ }`) to enable parsing simple programs.

Estimated Time: 2-4 hours for basic statement types

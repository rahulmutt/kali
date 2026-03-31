# Parser Implementation Plan - Stage 1.3

## Overview
This document outlines the implementation plan for the recursive-descent parser that covers ECMA-262 and TypeScript grammar.

## Implementation Strategy

### Phase 1: Core Infrastructure (Highest Priority)
1. **Parser state machine** - Token position tracking, lookahead
2. **Basic expression parsing** - Primary expressions, member calls, binary ops
3. **Statement parsing** - Block, variable, return, if, for, while
4. **Function/class parsing** - Declaration and expression forms

### Phase 2: TypeScript Extensions
1. **Type annotations** - On variables, parameters, returns
2. **Interfaces, type aliases, enums**
3. **Generics** - Function, class, type parameters
4. **Type assertions** - `as` and `!` syntax

### Phase 3: Advanced Features
1. **JSX/TSX support**
2. **Decorators**
3. **Error recovery** - Panic-mode recovery to synchronization points
4. **ASI handling** - Automatic semicolon insertion

## Parser Architecture

### Token Stream Wrapper
```rust
pub struct TokenStream {
    tokens: Box<[Token]>,
    position: usize,
    peeked: Option<Token>,
}
```

### Parser Core
```rust
pub struct Parser {
    stream: TokenStream,
    ast: AST,
    diagnostics: Vec<Diagnostic>,
    jsx_mode: bool,
}
```

### Parsing Modes
- **Expression mode** - Parses expressions, handles precedence
- **Statement mode** - Parses statements
- **Declaration mode** - Handles various declaration types
- **JSX mode** - Special handling for JSX syntax

## Key Parsing Techniques

### Pratt Parsing for Expressions
Use precedence climbing for expression parsing to handle operator precedence:
- Left-associative operators
- Right-associative operators  
- Different precedence levels for each operator

### ASI (Automatic Semicolon Insertion)
Implement rules according to ECMA-262:
1. Insert `;` at `}` or EOF
2. Insert `;` after `break`, `continue`, `return`, `throw`
3. Insert `;` when encountering `)]`, `}`, `EOF` instead of expected `;`

## Test Plan

### Expression Tests
- Literal expressions (numbers, strings, regex)
- Identifier references
- Call expressions (regular, tagged template)
- Member access (dot, bracket)
- Binary expressions (all operators)
- Unary expressions
- Conditional expressions

### Statement Tests
- Variable declarations (`let`, `const`, `var`)
- Function declarations and expressions
- Class declarations and expressions
- If/else/switch statements
- For/for-in/for-of/while loops
- Return/throw/try-catch

### TypeScript Tests
- Functions with type annotations
- Arrow functions with return types
- Interface declarations
- Type aliases
- Enum declarations
- Generic functions and classes

## Implementation Notes

1. All AST nodes carry spans for error reporting
2. Parser uses a token stream with lookahead support
3. Errors are collected, not fatal - parser continues after errors
4. JSX mode toggled based on file extension

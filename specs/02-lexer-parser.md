# 02 — Lexer & Parser

## Lexer

### Requirements
- Full lexical grammar support for the latest published ECMA-262 edition
- TypeScript syntax extensions (type annotations, generics, enums, etc.)
- Stage-3+/draft JavaScript proposals are out of scope unless the feature-maturity matrix or an explicit experimental flag says otherwise
- Kali-specific syntax extensions, kept intentionally small in early phases (effect annotations first; advanced effect syntax behind an experimental flag)
- Zero-copy where possible — tokens reference source via spans
- Streaming/lazy tokenization — parser pulls tokens on demand

### Token Design
```rust
struct Token {
    kind: TokenKind,    // u8 or u16 enum discriminant
    span: Span,         // byte offset range into source
}
```

`TokenKind` covers:
- All ECMAScript keywords, punctuators, literals
- TypeScript keywords: `type`, `interface`, `as`, `is`, `keyof`, `infer`, `readonly`, etc.
- Kali contextual keywords: `pure` and effect annotations are parsed so the AST can represent them, but semantic use is phase-gated; `effect`, `perform`, `handle` are reserved for experimental effect-handler syntax
- Template literal parts (head, middle, tail)
- RegExp literals (context-sensitive — parser assists disambiguation)
- Numeric literals (all formats: decimal, hex, octal, binary, bigint, separators)
- String literals (single, double, template)

### Performance Targets
- Lexing throughput: ≥ 500 MB/s on modern hardware
- Use `memchr` or SIMD for fast scanning of common delimiters
- Lookup tables for keyword recognition (pre-hashed, perfect hash or trie)

### Error Recovery
- On invalid characters: emit `TokenKind::Error`, record diagnostic, continue
- On unterminated strings/templates: close at newline or EOF, record diagnostic

## Parser

### Requirements
- Full grammar coverage for the latest published ECMA-262 edition, including:
  - All statement and expression types
  - Destructuring (nested, with defaults)
  - Generators and async generators
  - `for-in`, `for-of`, `for-await-of`
  - Optional chaining (`?.`), nullish coalescing (`??`)
  - Dynamic `import()`
  - `eval` (flagged for sandbox analysis)
  - All operator precedences and associativities
- TypeScript grammar:
  - Type annotations on variables, parameters, return types
  - Interfaces, type aliases, enums, mapped types, conditional types
  - Generic type parameters with constraints
  - Declaration merging semantics (recorded for type checker)
  - `as`, `satisfies`, non-null assertion (`!`)
- Kali extensions:
  - Effect type annotations: `function foo(): number ! FileSystem.Read | Console.Write` *(parsed early, semantically enabled in Phase 2 per [specs/19-feature-maturity.md](19-feature-maturity.md))*
  - `pure` function modifier *(parsed early, semantically enabled in Phase 2 per [specs/19-feature-maturity.md](19-feature-maturity.md))*
  - Experimental only: `effect` declarations, `perform` expressions, `handle` blocks
- JSX/TSX grammar:
  - JSX elements, fragments, expressions, spread attributes
  - Disambiguated from TypeScript generics via context (same approach as tsc)

### Parser Architecture
- Recursive descent (Pratt parsing for expressions)
- No separate grammar file — hand-written for maximum speed and error recovery
- Produces a concrete AST (see [specs/03-ast.md](03-ast.md))
- Handles automatic semicolon insertion (ASI) per spec
- Context-sensitive parsing (e.g., `yield` and `await` as identifiers vs keywords)

### Error Recovery Strategies
- **Synchronization tokens**: On error, skip to next `;`, `}`, or statement-start keyword
- **Missing tokens**: Insert expected token (e.g., missing `)` after condition) with diagnostic
- **Extra tokens**: Skip unexpected tokens with diagnostic
- Goal: Parse the entire file and produce a (partial) AST even with errors

### Performance Targets
- Parsing throughput: ≥ 200 MB/s
- Single-pass, no backtracking (use 1-token lookahead, 2-token where needed)
- Arena allocation for all AST nodes

### Source Preservation
- All comments and whitespace positions are recoverable from spans (for formatting tool)
- Parenthesization information preserved for correct re-emission
- Trivia (comments, whitespace) attached to adjacent tokens for formatter use (see `kali_fmt`)

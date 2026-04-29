# 02 — Lexer & Parser

## Lexer

### Requirements
- Full lexical grammar support for the latest published ECMA-262 edition
- TypeScript syntax extensions (type annotations, generics, enums, etc.)
- Grammar tracking and semantic support are intentionally separate: Phase 1 tracks the latest **published** grammar, while current-edition non-Annex-B semantics apply only to the features Kali marks as supported in the current command/profile; Annex B corners and Stage-3+/draft proposals remain explicitly gated by [specs/19-feature-maturity.md](19-feature-maturity.md)
- Kali-specific syntax extensions, kept intentionally small in early phases (effect annotations first; advanced algebraic-effect syntax reserved for a later explicit parser experiment and still phase-gated semantically)
- Zero-copy where possible — tokens reference source via spans
- Streaming/lazy tokenization — parser pulls tokens on demand

### Parse-vs-Support Boundary
The lexer/parser should accept the syntax Kali intends to understand even when the corresponding semantics are phase-gated elsewhere.

Canonical rule:
- parsing a construct does **not** by itself mark it as supported for execution or lowering
- semantic enablement is decided later by checking/lowering against [specs/19-feature-maturity.md](19-feature-maturity.md)
- acceptance of a current-edition syntax form also does **not** imply Annex B behavior or proposal semantics unless the maturity matrix or an explicit proposal opt-in says so
- this applies especially to syntax-bearing compatibility paths such as `import()`, `eval`, `Function()`-adjacent compatibility behavior, and Kali effect syntax (`pure`, effect annotations, and the later reserved algebraic-effect forms)
- therefore parser breadth should track the latest published grammar, while feature maturity still controls which accepted constructs are executable, lowerable, or only diagnosable in a given phase/profile

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
- Kali contextual keywords: `pure` and effect annotations are parsed so the AST can represent them, but semantic use is phase-gated; `effect`, `perform`, `handle` are reserved for the later algebraic-effect syntax surface and any parser-only experiments around it
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
  - Object literal expressions with identifier, string, and numeric property names, plus shorthand identifier properties and literal computed keys that canonicalize to static property names
  - Destructuring (nested, with defaults)
  - Generators and async generators
  - `for-in`, `for-of`, `for-await-of`
  - Optional chaining (`?.`), nullish coalescing (`??`)
  - Dynamic `import()` *(parsed in Phase 1; semantic support stays phase-gated — literal-string `import()` is a Phase 3 lowering path, while non-literal `import(expr)` remains a later compatibility boundary)*
  - `eval` *(parsed in Phase 1; semantic/runtime support is phase-gated and tied to the later `eval` compatibility path)*
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
  - `effect` declarations, `perform` expressions, `handle` blocks *(reserved later algebraic-effect surface; parsing them early does not grant semantic support or create a separate canonical status label)*
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

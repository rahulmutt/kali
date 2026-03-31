# Stage 1.2 — Lexer

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** [1.1 — Workspace & Crate Scaffold](01-workspace-scaffold.md)

## Goal

Implement `kali_lexer` — a fast, error-recovering tokeniser for the full latest published ECMA-262
lexical grammar plus TypeScript extensions. The lexer is the first stage of the pipeline that
processes real source text.

## Workable Milestone

- `kali_lexer` correctly tokenises representative TS/JS source files.
- Lex errors are collected into the `Diagnostics` accumulator with stable `E1xxx` codes; the
  lexer does not abort on the first bad character.
- Unit tests cover the token stream for a representative cross-section of language constructs.

## Tasks

### 1. Token types

Define the `Token` enum covering:

- **Identifiers and keywords** — distinguish reserved words (e.g. `let`, `const`, `function`,
  `class`, `import`, `export`, `return`, `async`, `await`, `yield`, `type`, `interface`, `enum`,
  `namespace`, `declare`, `abstract`, `as`, `from`, `of`) from identifier tokens.
- **Literals** — numeric (decimal, hex `0x`, octal `0o`, binary `0b`, BigInt `n` suffix),
  string (single-quote, double-quote, template literals with tagged-template support), regex,
  and `null` / `true` / `false`.
- **Punctuation and operators** — the full set from the ECMA-262 grammar including multi-character
  operators (`===`, `!==`, `**=`, `??=`, `&&=`, `||=`, `?.`, `??`, `...`, `=>`, `#`).
- **JSX tokens** — `<`, `>`, `/>`, `</`, JSX string attributes; these are toggled by the parser
  when it enters JSX context.
- **TypeScript-specific** — `!` non-null assertion as a distinct read mode where the parser
  signals TS context.
- **Comments** — single-line `//` and block `/* */`; attached as trivia rather than discarded so
  the formatter can preserve them.
- **Whitespace/newlines** — tracked as trivia for automatic-semicolon insertion (ASI) logic.
- **EOF** — explicit end-of-file sentinel.

Each `Token` carries a `Span` from `kali_common`.

### 2. Lexer state machine

Implement the lexer as a hand-written state machine (not a generated scanner) for predictable
compile-time cost and fine-grained error recovery:

- **Input cursor** — byte-indexed over a UTF-8 `&str`; use `char`-level peeking only where the
  grammar requires look-ahead.
- **Mode stack** — template literal nesting (`${...}`) requires a small stack to re-enter string
  mode after an embedded expression.
- **Regex vs division disambiguation** — implement the standard "is-expr-continuation" heuristic
  used by V8/SpiderMonkey; log any ambiguous cases for future refinement.
- **Automatic semicolon insertion markers** — tag tokens that are preceded by a line-terminator
  so the parser can apply ASI rules without re-scanning.
- **Strict vs sloppy mode** — track whether the current lexical context requires strict-mode
  validation (e.g. octal escapes are errors in strict mode).

### 3. Error recovery

On encountering an unrecognised character or malformed literal:

- Emit the appropriate `E1xxx` diagnostic via the `Diagnostics` collector.
- Produce an `Error` sentinel token covering the unexpected bytes.
- Resume lexing at the next plausible token boundary so the parser receives a complete (if
  error-annotated) token stream.

Define the initial `E1xxx` error codes:

| Code | Meaning |
|---|---|
| `E1001` | Unexpected character |
| `E1002` | Unterminated string literal |
| `E1003` | Unterminated template literal |
| `E1004` | Unterminated block comment |
| `E1005` | Invalid numeric literal |
| `E1006` | Invalid escape sequence |
| `E1007` | Invalid regex literal |

### 4. Source file loading

Add a `SourceFile::load(path)` helper in `kali_common` that reads a file into a UTF-8 `String`,
registers it in the `SourceMap`, and returns a `FileId`. The lexer takes a `&SourceFile` as input.

Support all canonical source-file extensions: `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`,
`.mjs`, `.cjs`, `.d.ts`, `.d.mts`, `.d.cts`.

### 5. Parallel lexing

Because the architecture targets per-file parallelism for lexing + parsing (see
`specs/01-architecture.md`), ensure the lexer is `Send + Sync` and stateless between files — all
mutable state lives on the `Lexer` struct itself, not in global variables.

### 6. Unit tests (`kali_lexer/src/tests.rs`)

Cover at minimum:

- All keyword tokens tokenised correctly.
- Numeric literal variants (decimal, hex, octal, binary, BigInt).
- String literals (single-quote, double-quote, escape sequences).
- Template literals including nested `${}` expressions.
- Regex literals — valid and the common division-disambiguation cases.
- Multi-character operator sequences.
- Comment trivia attachment.
- ASI newline markers.
- Each `E1xxx` error case producing the correct code.
- Empty file and file containing only whitespace/comments.

## Out of Scope

- JSX token emission beyond basic bracket tokens (the parser drives JSX mode in Stage 1.3).
- Full regex semantic validation (deferred to the type checker / later stages).
- Declaration-file-specific scanning differences beyond extension recognition.

## Definition of Done

- [ ] `kali_lexer` tokenises a set of representative `.ts`, `.js`, and `.tsx` fixtures without
      panicking.
- [ ] All `E1xxx` error cases emit a stable code and the lexer recovers to produce subsequent
      valid tokens.
- [ ] Unit test suite passes under `cargo test -p kali_lexer`.
- [ ] `cargo clippy -p kali_lexer -- -D warnings` passes.
- [ ] No regression in Stage 1.1 CI jobs.

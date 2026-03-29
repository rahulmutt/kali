# 15 — Error Reporting

## Design Goals

1. **AI-friendly**: Structured, parseable, minimal tokens for AI feedback loops
2. **Human-friendly**: Clear, colorful, with context and suggestions
3. **Consistent**: Every error has a code, message, location, and optional fix suggestion

## Error Format

### Default (Human)
```
error[E1001]: Type 'string' is not assignable to type 'number'
  --> src/main.ts:5:10
  |
5 |   let x: number = "hello";
  |          ------   ^^^^^^^ expected 'number', found 'string'
  |          expected type
  |
  = help: Remove the type annotation or change the value
```

### JSON (`--output json`)
Diagnostics are emitted inside the CLI's versioned command envelope. The canonical JSON schemas for both the envelope and individual diagnostics live in [specs/18-schemas.md](18-schemas.md).

## Error Code Ranges

| Range | Category |
|-------|----------|
| E0xxx | Internal compiler errors |
| E1xxx | Type errors |
| E2xxx | Syntax errors |
| E3xxx | Name resolution errors |
| E4xxx | Sandbox/effect violations |
| E5xxx | Import/module errors |
| E6xxx | Runtime errors |
| E7xxx | Memory/ownership errors |
| W1xxx | Type warnings |
| W2xxx | Style/lint warnings |
| W3xxx | Performance warnings |

## Error Categories

### Type Errors (E1xxx)
- `E1001`: Type mismatch (assignment, argument, return)
- `E1002`: Property does not exist on type
- `E1003`: Cannot invoke non-function type
- `E1004`: Missing required property
- `E1005`: Argument count mismatch
- `E1006`: Generic constraint not satisfied
- `E1007`: Cannot use 'as' to convert between unrelated types
- `E1008`: Effect type mismatch
- `E1009`: Purity violation (side effect in pure function)

### Syntax Errors (E2xxx)
- `E2001`: Unexpected token
- `E2002`: Unterminated string literal
- `E2003`: Invalid regular expression
- `E2004`: Duplicate parameter name
- `E2005`: Invalid assignment target

### Name Resolution Errors (E3xxx)
- `E3001`: Undefined variable or reference
- `E3002`: Duplicate declaration in same scope
- `E3003`: Cannot access before initialization (TDZ)
- `E3004`: Export not found in module

### Sandbox Errors (E4xxx)
- `E4001`: Effect not permitted by sandbox policy
- `E4002`: API call not permitted
- `E4003`: Resource limit exceeded (compile-time provable)
- `E4004`: Dynamic effect detected (cannot statically verify)

### Import/Module Errors (E5xxx)
- `E5001`: Module not found
- `E5002`: Circular dependency detected
- `E5003`: Invalid module specifier
- `E5004`: Package not installed
- `E5005`: Ambiguous module resolution

### Runtime Errors (E6xxx)
- `E6001`: Uncaught exception
- `E6002`: Stack overflow
- `E6003`: Out of memory

### Memory/Ownership Errors (E7xxx)
- `E7001`: Value used after move
- `E7002`: Cannot prove lifetime safety (escaping reference)
- `E7003`: Potential reference cycle detected (info/suggestion)

### Performance Warnings (W3xxx)
- `W3001`: Dynamic object access forces hash map representation
- `W3002`: `eval` usage disables optimizations in scope (when the compatibility mode is enabled)
- `W3003`: Value escapes scope, requiring heap allocation
- `W3004`: Generic function exceeds specialization limit

## Error Principles

### Minimal for AI
Default output shows just what's needed to fix the issue:
- Error code (for programmatic handling)
- One-line message
- Source location with context
- Fix suggestion when available

No ASCII art, progress bars, or decorative elements in default mode.

### Rich for Humans
With `--verbose` or in interactive terminals:
- Color-coded severity (red=error, yellow=warning, blue=info)
- Multi-line code context
- Related information (e.g., "declared here", "first used here")
- Suggested fixes with diff-like format

### Batch Reporting
- Continue compilation after errors (resilient parsing + type checking)
- Report all errors at once (not fail-fast)
- Deduplicate cascading errors (don't report downstream errors caused by a primary error)
- Sort by file, then line number
- Cap at 50 errors by default (`--max-errors N` to change)

## Diagnostic Struct

```rust
struct Diagnostic {
    severity: Severity,          // Error, Warning, Info, Hint
    code: DiagnosticCode,        // E1001, W3002, etc.
    message: String,             // Primary message
    file: FileId,                // Source file
    span: Span,                  // Primary span
    labels: Vec<Label>,          // Annotated source spans
    help: Option<String>,        // Suggested fix (text)
    fix: Option<SuggestedFix>,   // Automated fix (structured)
    related: Vec<RelatedInfo>,   // Related locations
    notes: Vec<String>,          // Additional context
}

struct SuggestedFix {
    message: String,
    edits: Vec<TextEdit>,        // File edits to apply the fix
}
```

`SuggestedFix` enables `kali check --fix` to auto-apply fixes for certain diagnostics.

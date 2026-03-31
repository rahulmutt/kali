# Stage 1.4 — Name Resolution

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/04-type-system.md`](../../specs/04-type-system.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)

## Goal

Implement name resolution inside `kali_types` — build the symbol table, resolve all identifiers
to their binding declarations, thread import/export edges into the module graph, and report
unresolved-name and scope errors. This is the first stage where `kali check` produces useful
output on real programs.

## Workable Milestone

- `kali check <file>` (or a project tree) reports unresolved identifiers, duplicate declarations,
  and import/export binding errors with stable `E3xxx` codes.
- The symbol table is complete enough for the type checker to walk without re-doing resolution.

## Tasks

### 1. Scope model

Define scope types that mirror JavaScript's actual scoping rules:

| Scope kind | Created by |
|---|---|
| `GlobalScope` | Top of every module/script; pre-populated with ambient globals |
| `ModuleScope` | Each source module; holds `import` bindings and `export` namespace |
| `FunctionScope` | `function`, method, getter/setter, constructor |
| `BlockScope` | `{}`, `if`/`for`/`while` bodies; holds `let` / `const` / `class` |
| `CatchScope` | `catch(e)` — binds the error parameter |
| `ClassScope` | Class body; holds `static` vs instance member separation |
| `TypeScope` | TypeScript `namespace`, generic type parameter lists |

Scopes form a parent-chain. Each scope holds a `HashMap<InternedStr, Symbol>` for value bindings
and a separate map for type-only bindings (TypeScript separates value and type namespaces).

### 2. Symbol table

A `Symbol` records:

- Its canonical `Span` (declaration site).
- Its `SymbolKind` (variable, function, class, parameter, type alias, interface, enum, namespace,
  import binding, etc.).
- Whether it is a `const` / `let` / `var` / `type`-only declaration.
- Mutability and ambient (`.d.ts`) flags.
- For import bindings: the source module specifier and the re-exported name.

Use the `kali_common` interner so symbol name comparisons are O(1) integer compares.

### 3. Module graph construction

Walk all `ImportDecl` and re-export nodes in each parsed module. For each specifier:

1. Resolve the specifier to a canonical `FileId` using the module-resolution algorithm (see
   `specs/14-packages.md` — TypeScript's node-resolution rules adapted for Kali's source-file
   classes):
   - Relative paths resolve against the importing file.
   - Bare specifiers resolve against the package registry / lock file (which is empty at this
     stage; add stubs that return `UnresolvedModule` for bare specifiers and report `E3010`).
2. Record the edge in a `ModuleGraph` (DAG). Detect and report cycles that violate static
   import ordering (`E3011`).
3. Bind import specifiers as `ImportBinding` symbols in the importing module's `ModuleScope`.

### 4. Declaration pass

Walk the AST of each module in topological order (after the module graph is built):

- **Hoist** `var` declarations and `function` declarations to the enclosing function/global scope
  (JavaScript hoisting semantics).
- Bind `let`, `const`, and `class` declarations in the enclosing block scope; mark them as
  in the temporal dead zone (TDZ) until their initialiser is reached.
- Bind TypeScript `interface`, `type alias`, `enum`, and `namespace` in the type namespace.
- Bind `import` specifiers as `ImportBinding` (already done in step 3).
- Detect duplicate `let`/`const` in the same block scope and report `E3001`.
- Detect redeclaration of a `const` / `class` after initial binding and report `E3002`.

### 5. Reference resolution pass

Walk all `IdentExpr` and `TypeRef` nodes. For each reference:

1. Walk up the scope chain looking for a matching `Symbol`.
2. If found, attach the resolved `Symbol` reference to the AST node's side table (avoid
   mutating the arena-allocated AST in place; use a parallel `HashMap<NodeId, SymbolId>`).
3. If not found, emit `E3003` (unresolved value reference) or `E3004` (unresolved type
   reference) and attach an `UnresolvedSymbol` sentinel.
4. For `var` references before TDZ exit, emit `E3005` (TDZ access).
5. For `import x` used as a value where `x` is a type-only import, emit `E3006`.

### 6. Export resolution

Validate every `ExportDecl` and `ExportSpecifier`:

- Check the exported name refers to a declared symbol.
- Verify re-exports (`export { x } from "mod"`) bind to the source module's export list.
- Detect duplicate export names and report `E3007`.
- Record the complete export map on the `ModuleScope` for cross-module type checking.

### 7. Query-based incremental foundation

Wrap resolution in demand-driven queries (following the architecture's query-based model):

- `module_scope(FileId) -> Arc<ModuleScope>` — computes and caches the scope for one file.
- `module_graph() -> Arc<ModuleGraph>` — computes the full import DAG.
- `symbol_of(NodeId) -> Option<SymbolId>` — resolves a reference node to its symbol.

Even though full incremental compilation is a Phase-3 target, laying down the query boundary now
prevents the later refactor from requiring a whole-codebase rewrite.

### 8. `kali check` — first real subcommand

Wire the `kali check [files...]` subcommand in `kali_cli`:

- Discover project files via the canonical project-discovery rules if no explicit files are given.
- Run the lexer → parser → name-resolution pipeline on each file.
- Print collected diagnostics in the human-readable format (defined further in Stage 1.13).
- Exit with code 1 if any errors were collected, 0 otherwise.

At this stage `check` only reports lex/parse/name errors — type errors come in Stage 1.5.

### 9. Error codes

| Code | Meaning |
|---|---|
| `E3001` | Duplicate declaration in same block scope |
| `E3002` | Illegal redeclaration of `const` / `class` |
| `E3003` | Unresolved value reference |
| `E3004` | Unresolved type reference |
| `E3005` | Variable used before declaration (TDZ) |
| `E3006` | Type-only import used as value |
| `E3007` | Duplicate export name |
| `E3008` | (reserved for type errors — Stage 1.5) |
| `E3009` | (reserved for type errors — Stage 1.5) |
| `E3010` | Unresolved module specifier |
| `E3011` | Circular import detected |

### 10. Tests

- Resolution golden tests: given a source file, assert every `IdentExpr` resolves to the expected
  symbol kind and declaration span.
- Error coverage: one fixture per `E3xxx` code; assert the correct code and span.
- Module graph tests: multi-file fixture trees; assert the DAG is correctly computed.
- `kali check` CLI smoke test: run on a fixture directory, assert exit code and error count.

## Out of Scope

- Type inference or assignability checking (Stage 1.5).
- Package specifier resolution against an installed node_modules tree (Stage 1.10 adds the
  package resolver; stubs returning `E3010` are acceptable here).

## Definition of Done

- [ ] `kali check` reports unresolved names with `E3xxx` codes on fixture programs.
- [ ] Symbol table and module graph tests pass.
- [ ] No panics on any of the parser fixture files from Stage 1.3.
- [ ] `cargo test -p kali_types` and `kali check` integration tests pass.
- [ ] No Stage 1.1–1.3 regressions.

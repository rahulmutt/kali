# Stage 1.4 — Name Resolution

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/04-type-system.md`](../../specs/04-type-system.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** [1.3 — Parser & AST](03-parser-and-ast.md)  
**Status:** ✅ Complete - resolver and CLI checks pass

---

### Completed Features

- ✅ Scope model with Global/Module/Function/Block/Catch/Class/Type scopes
- ✅ Symbol table with bindings per scope
- ✅ `TypeContext` for tracking scopes and bindings
- ✅ Deterministic resolver for unresolved identifiers, duplicate bindings, and missing import targets
- ✅ CLI check path wired to the resolver for end-to-end diagnostics

### Test Coverage

**Passing** (7)
- `test_scope_creation`, `test_scope_binding`, `test_type_context`
- `test_resolution_finds_bound_names`
- `test_resolution_reports_unresolved_identifiers`
- `test_resolution_reports_duplicate_bindings`
- `test_resolution_reports_missing_imports`

### Evidence

- ✅ `cargo build` succeeds
- ✅ `cargo test -p kali_types --lib` passes
- ✅ `cargo test -p kali_cli --test runtime_smoke` passes
- ✅ `cargo test --workspace` passes

---

**Workable Milestone**: Name resolution foundation complete with working type context and CLI diagnostics.

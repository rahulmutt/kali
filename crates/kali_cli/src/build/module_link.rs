//! Pre-resolver AST module-linking pass (throw-fallout Stage 5).
//!
//! Detects provenance-proven module-namespace bindings (`import * as ns from
//! "./x"` and `const c = await import(<foldable specifier>)`) and records
//! which on-disk module each binding provably refers to. Everything outside
//! the proven lane yields NO provenance (the binding is simply absent from
//! the map). By itself, "no provenance" is NOT fail-closed — a binding the
//! collector can't reach or can't fold is just as absent from the map as one
//! that was never namespace-shaped at all, and used to fall through to the
//! pre-stage silent behavior unexamined. Fail-closed-by-construction is a
//! property of the PIPELINE as a whole, not of this collector in isolation:
//! `link_provable_module_namespaces` default-denies (E5506) every USED
//! binding that looks namespace-shaped (a relative `import * as`, or a
//! declarator whose init is `await import(...)`, of ANY depth/kind) but
//! never earned provenance here — see that function's doc comment, and the
//! final whole-branch review that found and closed this gap (C2 in
//! `docs/superpowers/followups/throw-fallout-stage5-triage.md`). Within the
//! proven lane itself, every `Identifier`/`Object` name this collector folds
//! is also required to be bound EXACTLY ONCE across the whole file (see
//! `fold_import_specifier`'s doc comment) — a scope-blind fold that ignored
//! a param/let/var shadow of the same name was the review's C1 finding, and
//! linked the WRONG module silently.
//!
//! `collect_namespace_provenance` only *detects* provenance; the rest of the
//! pipeline (module loading + purity gate, AST cloning under mangled
//! `__link{N}_{name}` names, use-site rewriting, and the two default-denies)
//! lives in this module too — `link_provable_module_namespaces` is the single
//! entry point that sequences them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kali_ast::*;
use kali_common::FileId;
use kali_error::{_error_codes::e5, Diagnostic};
use kali_lexer::{Lexer, Token, TokenType};
use kali_parser::Parser;

use super::helpers::has_errors;

/// Provenance-proven module-namespace bindings discovered in one source file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NamespaceProvenance {
    /// binding name → linked module.
    pub bindings: BTreeMap<String, LinkedModule>,
}

/// A single provenance-proven link target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedModule {
    pub path: PathBuf,
    /// Stable per-module ordinal, assigned in first-seen order; mangled
    /// names are `__link{index}_{export}`.
    pub index: usize,
}

/// Collects provenance for every namespace-import and const-await-import
/// binding in `statements` that provably names a relative, on-disk module.
///
/// Two binding shapes are recognized:
///   - `import * as ns from "./x"` (module scope only)
///   - `const c = await import(<foldable specifier>)` (module scope, or
///     inside a top-level function body — the fixtures declare these and
///     their supporting `const` specifier parts inside `async function
///     main() { ... }`)
///
/// Anything else — non-relative sources, non-const bindings (`let`/`var`),
/// or a specifier expression `fold_import_specifier` cannot prove — yields
/// no entry in the returned map. This is the only signal at this stage; no
/// diagnostics are emitted here.
pub fn collect_namespace_provenance(
    source_path: &Path,
    source_contents: &str,
    statements: &[Statement],
) -> NamespaceProvenance {
    // `source_contents` is accepted for interface parity with the rest of
    // the module-linking pipeline (later tasks); this task's fold operates
    // entirely on the AST (template literals already desugar to
    // `BinaryExpression("+", ...)` concat chains by parse time, so no raw
    // re-lexing is needed here).
    let _ = source_contents;

    // Whole-file, any-depth binding census (params, let/var/const locals,
    // nested function/class names, catch params, for-in/for-of loop vars,
    // ImportDeclaration specifiers, ...) — reused as the C1 fix's
    // "allowlist at the choke point" gate: `fold_import_specifier` and
    // `is_object_freeze_callee` below only ever trust an `Identifier` name
    // whose count here proves it is NOT rebound/shadowed anywhere in the
    // file (see `fold_import_specifier`'s doc comment). This is the SAME
    // census `deny_shadowed_bindings` already runs (below) to guard the
    // NAMESPACE BINDING name itself — reusing it here, instead of
    // hand-rolling scope tracking for the SPECIFIER-FOLD consts, closes the
    // final whole-branch review's C1 finding: `local_consts =
    // module_consts.clone()` (below) never removed a function's own params
    // or a shadowing local from the inherited map, so a fold could silently
    // resolve an `Identifier` to the WRONG (module-scope) value instead of
    // the shadowing local.
    let bound_counts = compute_binding_counts(statements);

    // (a) module-scope single-declarator string consts, for template parts.
    let mut module_consts: BTreeMap<String, String> = BTreeMap::new();
    for statement in statements {
        if let Statement::VariableDeclaration(decl) = statement {
            if decl.kind == "const" {
                for declarator in &decl.declarations {
                    if let Some(Expression::Literal(LiteralValue::String(value))) = &declarator.init
                    {
                        module_consts.insert(declarator.id.clone(), unquote(value));
                    }
                }
            }
        }
    }

    let mut bindings: BTreeMap<String, LinkedModule> = BTreeMap::new();
    let mut path_index: BTreeMap<PathBuf, usize> = BTreeMap::new();
    let mut next_index = 0usize;

    for statement in statements {
        match statement {
            Statement::ImportDeclaration(decl) => {
                collect_namespace_import(
                    decl,
                    source_path,
                    &mut bindings,
                    &mut path_index,
                    &mut next_index,
                );
            }
            Statement::VariableDeclaration(decl) => {
                collect_const_await_import(
                    decl,
                    &module_consts,
                    &bound_counts,
                    source_path,
                    &mut bindings,
                    &mut path_index,
                    &mut next_index,
                );
            }
            Statement::FunctionDeclaration(decl) => {
                // No `is_async` guard here by design: a non-async function
                // body cannot contain a real `AwaitExpression` node (the
                // parser only produces that node under `in_async_function`
                // gating), so `as_await_import_source` below simply never
                // matches inside a non-async body — walking it unconditionally
                // is a no-op there, not a soundness gap.
                //
                // Consts visible to a declarator inside this body = the
                // module-scope consts, plus any consts declared earlier in
                // this SAME function body (accumulated below in order).
                let mut local_consts = module_consts.clone();
                for body_statement in &decl.body.body {
                    if let Statement::VariableDeclaration(var_decl) = body_statement {
                        collect_const_await_import(
                            var_decl,
                            &local_consts,
                            &bound_counts,
                            source_path,
                            &mut bindings,
                            &mut path_index,
                            &mut next_index,
                        );
                        if var_decl.kind == "const" {
                            for declarator in &var_decl.declarations {
                                if let Some(Expression::Literal(LiteralValue::String(value))) =
                                    &declarator.init
                                {
                                    local_consts.insert(declarator.id.clone(), unquote(value));
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    NamespaceProvenance { bindings }
}

fn collect_namespace_import(
    decl: &ImportDeclaration,
    source_path: &Path,
    bindings: &mut BTreeMap<String, LinkedModule>,
    path_index: &mut BTreeMap<PathBuf, usize>,
    next_index: &mut usize,
) {
    if !(decl.source.starts_with("./") || decl.source.starts_with("../")) {
        return;
    }
    for specifier in &decl.specifiers {
        if let ImportSpecifier::Namespace(local) = specifier {
            if let Some(target) =
                crate::build::eval::resolve_dynamic_import_target(source_path, &decl.source)
            {
                register(bindings, path_index, next_index, local.clone(), target);
            }
        }
    }
}

/// Detects `const <id> = await import(<specifier>)` (with `Parenthesized`
/// wrapping tolerated around either the `await` or the `import(...)` layer)
/// and, if the specifier folds to a string, registers provenance for `<id>`.
/// Declarations whose `kind` is not `"const"` are skipped outright — a
/// mutable rebinding (`let`/`var`) is not provably still the linked module
/// by the time it is used, so it earns no provenance.
fn collect_const_await_import(
    decl: &VariableDeclaration,
    visible_consts: &BTreeMap<String, String>,
    bound_counts: &BTreeMap<String, usize>,
    source_path: &Path,
    bindings: &mut BTreeMap<String, LinkedModule>,
    path_index: &mut BTreeMap<PathBuf, usize>,
    next_index: &mut usize,
) {
    if decl.kind != "const" {
        return;
    }
    for declarator in &decl.declarations {
        let Some(init) = &declarator.init else {
            continue;
        };
        let Some(specifier_expr) = as_await_import_source(init) else {
            continue;
        };
        let Some(specifier) = fold_import_specifier(specifier_expr, visible_consts, bound_counts)
        else {
            continue;
        };
        if let Some(target) =
            crate::build::eval::resolve_dynamic_import_target(source_path, &specifier)
        {
            register(
                bindings,
                path_index,
                next_index,
                declarator.id.clone(),
                target,
            );
        }
    }
}

/// If `expr` is (modulo `ParenthesizedExpression` wrapping around either
/// layer) `await import(<source>)`, returns `<source>`.
fn as_await_import_source(expr: &Expression) -> Option<&Expression> {
    match unwrap_parens(expr) {
        Expression::AwaitExpression(await_expr) => match unwrap_parens(&await_expr.argument) {
            Expression::ImportExpression(import_expr) => Some(&import_expr.source),
            _ => None,
        },
        _ => None,
    }
}

fn unwrap_parens(expr: &Expression) -> &Expression {
    match expr {
        Expression::ParenthesizedExpression(inner) => unwrap_parens(&inner.expression),
        _ => expr,
    }
}

/// Attempts to statically fold `expr` down to a specifier string, using
/// `consts` for `Identifier` lookups (module-scope consts plus any consts
/// declared earlier in the same function body — see `collect_namespace_provenance`).
///
/// Handles exactly: string literals (including desugared template-literal
/// quasis, which retain their surrounding backticks — `unquote` strips
/// `'`/`"`/`` ` ``symmetrically), `ParenthesizedExpression`, `SequenceExpression`
/// (last element, matching JS comma-operator semantics), `Object.freeze(<x>)`
/// (folds `<x>`), `+` string concatenation of two foldable operands,
/// `??`/`&&`/`||` with a *literal* (not merely foldable) left-hand side —
/// note there is no `LogicalExpression` AST node reachable from the parser
/// for these operators; `??`/`&&`/`||` all parse as `BinaryExpression` with
/// the literal operator string, so they are matched there — and plain
/// `Identifier` const lookups. Everything else is unprovable and returns
/// `None`.
///
/// `Identifier` lookups are gated by `bound_counts` (the whole-file, any-
/// depth binding census `compute_binding_counts` produces): a name is only
/// trusted as "the module-scope (or same-function) const `consts` recorded"
/// if it is bound EXACTLY ONCE anywhere in the entire file. `consts` itself
/// has no scope awareness — `collect_namespace_provenance` seeds a
/// function's `local_consts` with `module_consts.clone()` and never removes
/// a name the function rebinds (a param, or a shadowing `let`/`var`/second
/// `const`) — so without this gate, an `Identifier` reference inside a
/// function whose PARAMETER reuses a module-scope const's name would
/// silently resolve to the OUTER const instead of the parameter, linking
/// the wrong module entirely (final whole-branch review finding C1). This
/// is the same "allowlist at the exact choke point" fix the repo's own
/// for-in-key value-escape lesson establishes: reusing
/// `deny_shadowed_bindings`'s existing exhaustive census here, rather than
/// hand-rolling scope tracking for this fold, is what makes the allowlist
/// correct by construction instead of by enumeration.
fn fold_import_specifier(
    expr: &Expression,
    consts: &BTreeMap<String, String>,
    bound_counts: &BTreeMap<String, usize>,
) -> Option<String> {
    match expr {
        Expression::Literal(LiteralValue::String(value)) => Some(unquote(value)),
        Expression::ParenthesizedExpression(inner) => {
            fold_import_specifier(&inner.expression, consts, bound_counts)
        }
        Expression::SequenceExpression(seq) => seq
            .expressions
            .last()
            .and_then(|last| fold_import_specifier(last, consts, bound_counts)),
        Expression::CallExpression(call) => {
            if call.args.len() == 1 && is_object_freeze_callee(&call.callee, bound_counts) {
                fold_import_specifier(&call.args[0], consts, bound_counts)
            } else {
                None
            }
        }
        Expression::Identifier(name) => {
            if bound_counts.get(name.as_str()).copied().unwrap_or(0) != 1 {
                // Either never bound (impossible if it's a key of `consts`,
                // since `consts` is itself built from a real declarator) or
                // bound MORE than once somewhere in the file (a param, a
                // shadowing local, ...) — either way, not provably still
                // the outer const by the time this reference is reached.
                return None;
            }
            consts.get(name).cloned()
        }
        Expression::BinaryExpression(binary) => match binary.operator.as_str() {
            "+" => {
                let left = fold_import_specifier(&binary.left, consts, bound_counts)?;
                let right = fold_import_specifier(&binary.right, consts, bound_counts)?;
                Some(format!("{left}{right}"))
            }
            "??" | "&&" | "||" => fold_logical(binary, consts, bound_counts),
            _ => None,
        },
        _ => None,
    }
}

/// Folds `??`/`&&`/`||` (parsed as `BinaryExpression`, never
/// `LogicalExpression` — the parser has no code path that produces that
/// node) under JS truthiness, but only when the left-hand side is a bare
/// literal we can classify directly (`null`, a boolean, a number, or a
/// string). A non-literal left-hand side (e.g. an identifier or call) makes
/// the branch unprovable, so this fails closed to `None`.
fn fold_logical(
    binary: &BinaryExpression,
    consts: &BTreeMap<String, String>,
    bound_counts: &BTreeMap<String, usize>,
) -> Option<String> {
    let (nullish, truthy) = literal_truthiness(&binary.left)?;
    match binary.operator.as_str() {
        "??" => {
            if nullish {
                fold_import_specifier(&binary.right, consts, bound_counts)
            } else {
                fold_import_specifier(&binary.left, consts, bound_counts)
            }
        }
        "&&" => {
            if truthy {
                fold_import_specifier(&binary.right, consts, bound_counts)
            } else {
                fold_import_specifier(&binary.left, consts, bound_counts)
            }
        }
        "||" => {
            if truthy {
                fold_import_specifier(&binary.left, consts, bound_counts)
            } else {
                fold_import_specifier(&binary.right, consts, bound_counts)
            }
        }
        _ => None,
    }
}

/// Classifies a literal expression's JS truthiness as `(is_nullish,
/// is_truthy)`. Returns `None` for anything that is not a directly
/// classifiable literal (modulo `ParenthesizedExpression` wrapping).
fn literal_truthiness(expr: &Expression) -> Option<(bool, bool)> {
    match unwrap_parens(expr) {
        Expression::Literal(LiteralValue::Null) => Some((true, false)),
        Expression::Literal(LiteralValue::Boolean(value)) => Some((false, *value)),
        Expression::Literal(LiteralValue::Number(value)) => Some((false, *value != 0.0)),
        Expression::Literal(LiteralValue::String(value)) => {
            Some((false, !unquote(value).is_empty()))
        }
        _ => None,
    }
}

fn is_object_freeze_callee(callee: &Expression, bound_counts: &BTreeMap<String, usize>) -> bool {
    // Intentionally broad: matches `property == "freeze"` for either dot
    // (`Object.freeze`) or computed (`Object["freeze"]`) access without
    // further discriminating the two — both are the same provable callee
    // for this fold's purposes, so no additional check is warranted.
    //
    // `bound_counts.get("Object")` must be ABSENT (never locally bound
    // anywhere in the file) for this to fire — the same "bound exactly
    // once" allowlist `fold_import_specifier`'s `Identifier` arm applies to
    // a module-scope const, generalized to a global builtin's expected own-
    // binding count of ZERO: a program that shadows `Object` (e.g. a
    // parameter or local literally named `Object`) cannot be assumed to
    // still mean the real global here (final whole-branch review finding
    // C1's `is_object_freeze_callee` half).
    matches!(
        callee,
        Expression::MemberExpression(member)
            if member.property == "freeze"
                && matches!(&member.object, Expression::Identifier(name) if name == "Object")
                && bound_counts.get("Object").copied().unwrap_or(0) == 0
    )
}

/// A parsed, purity-gated linked module: its true exports (token-scan
/// proven) plus every top-level function (exports + private helpers), for
/// Task 5's sibling-callee renames.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LinkedModuleAst {
    pub index: usize,
    /// export name → the parsed function declaration.
    pub exports: BTreeMap<String, FunctionDeclaration>,
    /// ALL top-level function names (exports + private helpers).
    pub all_functions: BTreeMap<String, FunctionDeclaration>,
    /// Every name in `all_functions`, in the order the functions were
    /// declared in the module's SOURCE. `all_functions` alone cannot serve
    /// this purpose — it is a `BTreeMap`, so iterating it yields alphabetical
    /// key order, unrelated to source order. This is the tie-break
    /// `append_linked_functions`'s dependency-order topological sort uses
    /// when the intra-module call graph doesn't fully constrain relative
    /// order between two functions (see that function's doc comment).
    /// Always the same set of names as `all_functions.keys()`, just in a
    /// different order — populated alongside `all_functions` in the same
    /// loop, in both `load_linked_module` and the `build_module` test
    /// helper.
    pub declaration_order: Vec<String>,
}

/// Reads, lexes, and parses `module.path`, then purity-gates it: EVERY
/// top-level statement must be a plain (non-async, non-generator)
/// `function` declaration. Anything else — a bare top-level statement, an
/// import, `export const`, an async or generator function, a class, etc. —
/// fails closed with an `E5506 FEATURE_UNAVAILABLE` diagnostic naming the
/// module path and the offending construct.
///
/// Because the parser erases the `export` marker (a plain `export function
/// f() {}` and a private `function f() {}` produce the identical
/// `Statement::FunctionDeclaration` node — see `kali_parser::module`
/// around the `TokenType::Function` branch), the TRUE export set cannot be
/// read off the AST. Instead this token-scans the already-lexed source for
/// `Export` immediately followed by `Function` immediately followed by an
/// `Identifier`, collects those identifiers as the proven export names, and
/// intersects that set with `all_functions` (a defensive belt-and-braces
/// narrowing, since every name the scan finds must also be a real top-level
/// function to end up in `exports`).
pub fn load_linked_module(module: &LinkedModule) -> Result<LinkedModuleAst, Diagnostic> {
    let path = &module.path;

    let source = std::fs::read_to_string(path).map_err(|error| {
        Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "module '{}' cannot be linked for namespace member access: failed to read the file: {error}",
                path.display()
            ),
        )
    })?;

    let lexed = Lexer::new(FileId::new(0), source).lex_all();
    if !lexed.diagnostics.is_empty() {
        return Err(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "module '{}' cannot be linked for namespace member access: it failed to lex: {:?}",
                path.display(),
                lexed.diagnostics
            ),
        ));
    }

    let export_names = scan_exported_function_names(&lexed.tokens);

    let mut parser = Parser::new(FileId::new(0), lexed.tokens);
    let parsed = parser.parse(Some(path.to_string_lossy().to_string()));
    if !parsed.diagnostics.is_empty() {
        return Err(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "module '{}' cannot be linked for namespace member access: it failed to parse: {:?}",
                path.display(),
                parsed.diagnostics
            ),
        ));
    }

    let mut all_functions: BTreeMap<String, FunctionDeclaration> = BTreeMap::new();
    let mut declaration_order: Vec<String> = Vec::new();
    for statement in &parsed.statements {
        match statement {
            Statement::FunctionDeclaration(function)
                if !function.is_async && !function.generator =>
            {
                if all_functions.contains_key(&function.name) {
                    return Err(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "module '{}' cannot be linked for namespace member access: its top level declares `{}` more than once — a later private redefinition would silently overwrite an exported declaration of the same name",
                            path.display(),
                            function.name,
                        ),
                    ));
                }
                all_functions.insert(function.name.clone(), function.clone());
                declaration_order.push(function.name.clone());
            }
            other => {
                return Err(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "module '{}' cannot be linked for namespace member access: its top level contains {} — only plain `export function` declarations are supported in the current direct-runtime path",
                        path.display(),
                        describe_statement(other),
                    ),
                ));
            }
        }
    }

    let exports = all_functions
        .iter()
        .filter(|(name, _)| export_names.contains(*name))
        .map(|(name, function)| (name.clone(), function.clone()))
        .collect();

    Ok(LinkedModuleAst {
        index: module.index,
        exports,
        all_functions,
        declaration_order,
    })
}

/// Token-scans `tokens` for `export function <name>` sequences and returns
/// the set of proven export names. `export async function <name>` and
/// `export function* <name>` are not matched here — those shapes are
/// already rejected by the purity gate in `load_linked_module` before this
/// set is ever intersected against `all_functions`, so there is no export
/// they could wrongly license.
fn scan_exported_function_names(tokens: &[Token]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for window in tokens.windows(3) {
        if window[0].kind == TokenType::Export
            && window[1].kind == TokenType::Function
            && window[2].kind == TokenType::Identifier
        {
            names.insert(window[2].value.clone());
        }
    }
    names
}

/// Produces a short human-readable label for the purity-gate rejection
/// message, naming the offending top-level construct.
fn describe_statement(statement: &Statement) -> String {
    match statement {
        Statement::FunctionDeclaration(function) if function.is_async => {
            "an async function declaration".to_string()
        }
        Statement::FunctionDeclaration(function) if function.generator => {
            "a generator function declaration".to_string()
        }
        Statement::ClassDeclaration(_) => "a class declaration".to_string(),
        Statement::ImportDeclaration(_) => "an import declaration".to_string(),
        Statement::VariableDeclaration(decl) => {
            format!("a `{}` declaration", decl.kind)
        }
        other => {
            // Fallback for anything else (ExpressionStatement — including
            // the parser's fallback parse of `export const ...`,
            // ExportNamed, ExportDefault, ExportAll, or any future
            // statement kind): default-deny by construction, labeled
            // generically as "a non-function statement".
            let _ = other;
            "a non-function statement".to_string()
        }
    }
}

fn register(
    bindings: &mut BTreeMap<String, LinkedModule>,
    path_index: &mut BTreeMap<PathBuf, usize>,
    next_index: &mut usize,
    local: String,
    target: PathBuf,
) {
    let index = *path_index.entry(target.clone()).or_insert_with(|| {
        let assigned = *next_index;
        *next_index += 1;
        assigned
    });
    bindings.insert(
        local,
        LinkedModule {
            path: target,
            index,
        },
    );
}

/// Prepends mangled clones of `module.all_functions` to the FRONT of
/// `statements`, in DEPENDENCY order (every callee declared before any
/// linked sibling that calls it — see `topo_sort_dependency_order` below).
/// Mangle: `__link{module.index}_{original_name}`. Sibling references inside
/// cloned bodies are renamed to their mangled forms. Err = a mangled-name
/// collision with an already-declared entry name, OR an intra-module call
/// cycle no declaration order could satisfy (both E5506).
///
/// Ordering: the collision guard runs first, over every function in
/// `module.all_functions`; then the sibling-rename walk (which can also
/// fail — a bare, non-call reference to a sibling name) runs entirely
/// against local clones and simultaneously records the intra-module call
/// graph (using `census_block`, the same traversal `deny_unrewritten_uses`
/// below reuses for a different purpose); then the dependency-order
/// topological sort runs (which can also fail, on a cycle) — all of this
/// against local clones/data, before anything is spliced in. `statements`
/// is only ever mutated once, via a single `splice` at the very end, after
/// every fallible step has already succeeded — so any `Err` return leaves
/// `statements` byte-identical to how it was passed in.
///
/// FRONT, not the end: this resolver has no forward-declaration hoisting —
/// a top-level identifier must be textually declared before any use, even
/// a use inside a not-yet-invoked function body (this is a pre-existing,
/// general resolver property, verified independently of module-linking
/// entirely: a plain `function f() { return helper(); } function helper()
/// { return 1; }` with no linked module involved at all already fails to
/// resolve `helper` there). A linked function must therefore be declared
/// before the entry module's EARLIEST possible use site, which can be its
/// very first statement (`import * as ns from "./x"; const v =
/// ns.export();`) — appending at the end left every such use unresolvable.
///
/// DEPENDENCY order, not `module.all_functions`'s alphabetical `BTreeMap`
/// key order: the same no-hoisting property above applies BETWEEN two
/// linked clones too. `module.all_functions` iterates alphabetically, which
/// is unrelated to which clone calls which — `function helper() { return
/// 1n; } export function f() { return helper(); }` clones as `__link0_f`
/// (alphabetically first) and `__link0_helper` (alphabetically second), so
/// appending in that order put the CALLER before its CALLEE and made the
/// resulting program fail to resolve `__link0_helper` — exactly the
/// no-hoisting failure this function's whole FRONT-placement strategy
/// exists to avoid, just one level down (between clones, not just
/// clone-vs-entry). A full topological sort of the intra-module call graph,
/// tie-broken by the module's own SOURCE declaration order (not
/// alphabetical), is required to guarantee callee-before-caller regardless
/// of which order the two functions happen to be declared in the linked
/// module's source.
pub fn append_linked_functions(
    statements: &mut Vec<Statement>,
    module: &LinkedModuleAst,
) -> Result<usize, Diagnostic> {
    let renames: BTreeMap<String, String> = module
        .all_functions
        .keys()
        .map(|name| (name.clone(), mangled_link_name(module.index, name)))
        .collect();

    let declared = collect_declared_names(statements);
    for mangled in renames.values() {
        if declared.contains(mangled) {
            return Err(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "linked module function `{mangled}` cannot be appended: an entry declaration already uses that name"
                ),
            ));
        }
    }

    // mangled name -> original name, the reverse of `renames` — used below
    // to translate a renamed call-callee reference found in an
    // ALREADY-CLONED body back to the original name for the dependency
    // graph (the graph is keyed by original names, matching
    // `module.declaration_order`).
    let mangled_to_original: BTreeMap<String, String> = renames
        .iter()
        .map(|(original, mangled)| (mangled.clone(), original.clone()))
        .collect();

    let mut clones: BTreeMap<String, FunctionDeclaration> = BTreeMap::new();
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (name, function) in &module.all_functions {
        let mut clone = function.clone();
        clone.name = renames.get(name).cloned().expect(
            "name is a key of module.all_functions, and renames is built from those same keys",
        );
        // The linked function's OWN parameters are a binding-introducing
        // position `walk_block` never sees (it only walks `clone.body`), so
        // they are checked here for a shadowing collision with a sibling
        // linked function before the body walk runs.
        for param in &clone.params {
            check_binding(param, &renames)?;
        }
        walk_block(&mut clone.body, &renames)?;

        // Every mangled sibling name still present in `clone.body` after the
        // walk above MUST be a renamed call-callee reference: the walk
        // itself already rejects (as a bare-reference or shadowing error)
        // every other way a sibling name could survive into the clone. Self
        // edges (a function calling itself) are excluded here — see
        // `topo_sort_dependency_order`'s doc comment for why plain
        // self-recursion needs no ordering constraint.
        let mut called = BTreeSet::new();
        census_block(
            &clone.body,
            &mut |identifier| {
                if let Some(original) = mangled_to_original.get(identifier) {
                    called.insert(original.clone());
                }
            },
            &mut |_binding| {},
        );
        called.remove(name);
        edges.insert(name.clone(), called);

        clones.insert(name.clone(), clone);
    }

    let order = topo_sort_dependency_order(module, &edges)?;

    let cloned: Vec<Statement> = order
        .into_iter()
        .map(|name| {
            Statement::FunctionDeclaration(clones.remove(&name).unwrap_or_else(|| {
                panic!(
                    "topo_sort_dependency_order's result is a permutation of module.declaration_order, and every one of those names has a clone: missing `{name}`"
                )
            }))
        })
        .collect();

    let appended = cloned.len();
    statements.splice(0..0, cloned);
    // `Ok(appended)` — the number of clones just spliced onto the FRONT of
    // `statements`, i.e. `statements[..appended]` is exactly this call's
    // splice and `statements[appended..]` is the caller's own statements
    // byte-identical to what was passed in. The caller
    // (`link_provable_module_namespaces`) accumulates this across every
    // linked module it appends so `deny_unrewritten_uses` can skip over
    // every clone body and only census the entry's OWN statements — see
    // that call site's comment for the over-deny this closes (final
    // whole-branch review's "Minor" finding: a linked module's internal
    // local sharing a name with an entry provenance binding was wrongly
    // E5506'd, even though the two are unrelated).
    Ok(appended)
}

/// Orders `module.declaration_order`'s names so every callee (per `edges`,
/// which already excludes self-edges) appears before any linked sibling
/// that calls it. Ties — functions with no dependency relationship to each
/// other — are broken by the module's own SOURCE declaration order (never
/// alphabetically): a plain Kahn's-algorithm-style scheduling pass that, at
/// each step, picks the EARLIEST-declared not-yet-emitted name whose
/// dependencies are all already emitted.
///
/// Err (E5506) if `edges` (excluding self-edges) contains a cycle — mutual
/// or indirect recursion among the linked module's own functions. This
/// resolver has no forward-declaration hoisting (see
/// `append_linked_functions`'s doc comment), so a cycle has NO valid
/// declaration order at all: whichever member of the cycle is placed last
/// still needs an as-yet-undeclared sibling. Verified independently of
/// module-linking entirely — `function isEven(n) { return n === 0 ? true :
/// isOdd(n - 1); } function isOdd(n) { return n === 0 ? false : isEven(n -
/// 1); }` already fails to resolve `isOdd` with the same E3100 this
/// function pre-empts here at compile time, for either declaration order.
///
/// Self-recursion (a function calling itself, e.g. `function f(n) { return
/// n <= 1 ? 1 : n * f(n - 1); }`) is a DIFFERENT case, deliberately not
/// treated as a cycle: verified independently of module-linking too — that
/// exact fixture runs and matches node byte-for-byte with plain kali,
/// because by the time `f`'s body actually CALLS `f`, `f` itself has
/// already been fully declared (the call is inside the body, not in a
/// sibling's body needing `f` to exist before `f` does) — no hoisting is
/// needed for a function to call itself. `edges` already has self-edges
/// stripped by the caller for exactly this reason, so this function never
/// sees them as a dependency to satisfy or a cycle to reject.
fn topo_sort_dependency_order(
    module: &LinkedModuleAst,
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<String>, Diagnostic> {
    if let Some(cycle) = find_call_cycle(&module.declaration_order, edges) {
        return Err(call_cycle_error(module, &cycle));
    }

    let mut emitted: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::with_capacity(module.declaration_order.len());
    while order.len() < module.declaration_order.len() {
        let next = module.declaration_order.iter().find(|name| {
            !emitted.contains(name.as_str())
                && edges
                    .get(name.as_str())
                    .is_none_or(|deps| deps.iter().all(|dep| emitted.contains(dep)))
        });
        let Some(next) = next else {
            unreachable!(
                "find_call_cycle reported no cycle in `edges`, so a DAG topological order always \
                 has an unemitted node with every dependency already emitted at each step; \
                 module.declaration_order = {:?}, edges = {edges:?}, emitted so far = {emitted:?}",
                module.declaration_order
            );
        };
        emitted.insert(next.clone());
        order.push(next.clone());
    }
    Ok(order)
}

/// Depth-first cycle detection over `edges` (self-edges already excluded by
/// the caller), restricted to `names` (every node, even one with no
/// outgoing OR incoming edges, still needs a color entry so the outer scan
/// below terminates). Returns the first cycle found, as the ordered list of
/// participant names (the closed walk from the cycle's first revisited node
/// back to itself, in traversal order) — `None` if `edges` is a DAG.
/// Standard white/gray/black DFS coloring: gray = currently on the
/// recursion stack (an edge back into gray is a cycle); black = fully
/// explored (safe to skip).
fn find_call_cycle(
    names: &[String],
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    fn visit(
        node: &str,
        edges: &BTreeMap<String, BTreeSet<String>>,
        colors: &mut BTreeMap<String, Color>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        colors.insert(node.to_string(), Color::Gray);
        stack.push(node.to_string());
        if let Some(deps) = edges.get(node) {
            for dep in deps {
                match colors.get(dep.as_str()).copied().unwrap_or(Color::White) {
                    Color::Gray => {
                        let start = stack
                            .iter()
                            .position(|on_stack| on_stack == dep)
                            .expect("dep is Color::Gray, so it must currently be on `stack`");
                        return Some(stack[start..].to_vec());
                    }
                    Color::Black => {}
                    Color::White => {
                        if let Some(cycle) = visit(dep, edges, colors, stack) {
                            return Some(cycle);
                        }
                    }
                }
            }
        }
        stack.pop();
        colors.insert(node.to_string(), Color::Black);
        None
    }

    let mut colors: BTreeMap<String, Color> = BTreeMap::new();
    for name in names {
        if colors.get(name).copied().unwrap_or(Color::White) == Color::White {
            let mut stack = Vec::new();
            if let Some(cycle) = visit(name, edges, &mut colors, &mut stack) {
                return Some(cycle);
            }
        }
    }
    None
}

/// The diagnostic for an intra-module call cycle `topo_sort_dependency_order`
/// rejects — names every participant (as its mangled, `__link{index}_`-
/// prefixed form, matching what a downstream diagnostic reader would
/// actually see appended to their program) and the module by its stable
/// ordinal (`LinkedModuleAst` has no source path of its own to name it by —
/// see that struct's doc comment; the mangled names it prints already encode
/// the same ordinal).
fn call_cycle_error(module: &LinkedModuleAst, cycle: &[String]) -> Diagnostic {
    let participants: Vec<String> = cycle
        .iter()
        .map(|name| mangled_link_name(module.index, name))
        .collect();
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "linked module (index {}) cannot be linked: functions {} form a call cycle (mutual or indirect recursion) — this resolver has no forward-declaration hoisting, so no declaration order exists that would let every call resolve; only direct self-recursion (a function calling itself) is supported",
            module.index,
            participants.join(" -> "),
        ),
    )
}

/// `__link{index}_{name}`. See `LinkedModule::index` for the stable-ordinal
/// contract.
fn mangled_link_name(index: usize, name: &str) -> String {
    format!("__link{index}_{name}")
}

/// Every name a mangled clone could collide with: top-level function
/// declaration names and top-level variable declarator ids. Intentionally
/// narrow (unlike the rename/deny walk below, which must be exhaustive) —
/// this mirrors the brief's literal collision surface: "FunctionDeclaration
/// names + variable declarator ids" in `statements`.
fn collect_declared_names(statements: &[Statement]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(function) => {
                names.insert(function.name.clone());
            }
            Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    names.insert(declarator.id.clone());
                }
            }
            _ => {}
        }
    }
    names
}

/// The diagnostic for a bare (non-call) reference to a sibling linked
/// function name found inside a cloned body — the fail-closed half of the
/// sibling rename. Without this, an un-renamed bare reference (`const g =
/// helper;`, `f(helper)`, `return helper;`) would silently resolve to
/// whatever `helper` means in the ENTRY module's scope (nothing, or worse,
/// an unrelated entry binding of the same name) rather than the linked
/// module's `helper`.
fn bare_sibling_reference_error(name: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "linked module function `{name}` aliases a sibling export — unsupported (only direct calls `{name}(...)` are rewritten; a bare reference cannot be resolved without the linked module's scope)"
        ),
    )
}

fn check_bare_reference(name: &str, renames: &BTreeMap<String, String>) -> Result<(), Diagnostic> {
    if renames.contains_key(name) {
        Err(bare_sibling_reference_error(name))
    } else {
        Ok(())
    }
}

/// The diagnostic for a cloned body INTRODUCING a binding (a param, a
/// `const`/`let`/`var` local, a nested function/class name, a catch-clause
/// param, a for-in/for-of loop variable, ...) whose name collides with a
/// sibling linked-function name — the shadowing fail-open this walk closes.
///
/// This walk has no lexical-scope awareness (see the module-level doc
/// comment above `walk_block`): it cannot tell a local rebinding of `name`
/// apart from a genuine call to the linked module's `name`, so every call
/// site spelled `name(...)` anywhere in the cloned body would still get
/// silently rewritten to the sibling's mangled name even though, under real
/// JS lexical scoping, some or all of those calls should have resolved to
/// the local rebinding instead. Rejecting the whole module closed — even
/// when the shadow is lexically disjoint from every call — is intentional:
/// conservative and fail-closed beats scope-precise and subtly wrong.
fn shadowing_binding_error(name: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "linked module function `{name}` cannot be linked: a local binding named `{name}` inside a cloned function body would shadow it — unsupported (this pass's rename walk has no lexical-scope awareness, so it cannot tell a local rebinding of `{name}` apart from a genuine call to the linked module's `{name}`; rename the local binding to avoid the sibling name)"
        ),
    )
}

/// Rejects `name` if it collides with a sibling linked-function name.
/// Call this at EVERY binding-introducing position visited by the walk
/// below (declarator ids, params, nested function/class names, catch
/// params, for-in/for-of loop vars, ...) — see `shadowing_binding_error`.
fn check_binding(name: &str, renames: &BTreeMap<String, String>) -> Result<(), Diagnostic> {
    if renames.contains_key(name) {
        Err(shadowing_binding_error(name))
    } else {
        Ok(())
    }
}

/// The diagnostic for a nested `import` declaration found inside a cloned
/// linked-module function body.
///
/// A nested import is not valid ES to begin with, but this parser accepts it
/// uncritically: `parse_statement` routes `TokenType::Import` to
/// `parse_import_declaration()` unconditionally
/// (`crates/kali_parser/src/statement.rs:45-51`), and a function body parses
/// through the exact same generic `parse_block_statement`/`parse_statement`
/// loop as a module's top level (`parse_function_declaration_with_async`
/// calls `self.parse_block_statement()` at
/// `crates/kali_parser/src/declaration.rs:84`, and that loop itself is
/// `crates/kali_parser/src/statement.rs:159-178`) — there is no
/// "function-body context" that restricts which statement kinds are legal
/// there. `load_linked_module`'s purity gate only restricts a linked
/// module's TOP-LEVEL statements, never what a function body inside it
/// contains, so a nested import surviving into a cloned body must be
/// rejected HERE. Left as a no-op, its local binding(s) would go unrenamed
/// while a bare reference to the same name elsewhere in the body could
/// still be silently rewritten to a sibling linked function of that name —
/// the wrong-call-target class this pass exists to close.
fn nested_import_error(source: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "linked module function body contains a nested `import` declaration (source `{source}`) — unsupported: a nested import inside a cloned linked-function body cannot be safely renamed or rejected per-binding by this pass, so the whole module is rejected"
        ),
    )
}

/// The diagnostic for a nested named `export { ... }` declaration found
/// inside a cloned linked-module function body.
///
/// Verified reachable the same way as `nested_import_error` above:
/// `parse_statement` routes `TokenType::Export` to
/// `parse_export_declaration()` unconditionally
/// (`crates/kali_parser/src/statement.rs:33`), which itself returns
/// `Statement::ExportNamed` for a `{ ... }` specifier list
/// (`crates/kali_parser/src/module.rs:161-180`), and nothing about parsing a
/// function body restricts which statement kinds are legal inside it (same
/// citation as `nested_import_error`). A bare `export { x }` re-export
/// specifier names a LOCAL binding by string
/// (`ExportSpecifier { local, exported }`,
/// `crates/kali_ast/src/module.rs:63-66`) exactly the way this pass's own
/// bare-reference check treats an `Identifier` reference — so it is
/// rejected outright rather than left as an unverified no-op.
fn nested_export_named_error() -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        "linked module function body contains a nested named `export` declaration — unsupported: nested exports are not valid ES; this parser accepts them anyway, and this pass cannot safely resolve or reject their local-binding references per-specifier, so the whole module is rejected".to_string(),
    )
}

// ---- sibling-callee rename / bare-reference-deny walk ----
//
// Mirrors the traversal SHAPE of `kali_types::monomorphize::walk_calls_mut`
// (renames a call's callee identifier) but is local to this module, keyed by
// name (not call ordinal), and — unlike that walk, which silently skips any
// node kind it doesn't special-case — is written as an EXHAUSTIVE `match`
// with no `_ =>` arm on either `Statement` or `Expression`, so the compiler
// forces a decision at every current and future variant. Every arm that
// cannot contain an `Identifier` reference is still matched explicitly, with
// a comment explaining why it is a no-op.
//
// `is_callee` is true for exactly one call site below: the direct `callee`
// slot of a `CallExpression`. Every other recursive call passes `false` —
// nothing else is ever "the callee of a call" by definition, even when it
// sits underneath one (e.g. the object of a member-expression callee like
// `helper.foo()`, or the callee of a `new` expression, both correctly fail
// closed as bare references rather than being silently renamed).

fn walk_block(
    block: &mut BlockStatement,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    for statement in &mut block.body {
        walk_statement(statement, renames)?;
    }
    Ok(())
}

fn walk_var_decl(
    decl: &mut VariableDeclaration,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    // `decl.kind` (var/let/const) carries no identifier reference.
    for declarator in &mut decl.declarations {
        // `declarator.id` is the name being BOUND, not a reference — but a
        // bound name that collides with a sibling linked function is a
        // shadow (see `shadowing_binding_error`), so it is checked, not
        // renamed. Covers plain var/let/const, `for (let x ...; ...)` init,
        // and `for (const x in/of ...)` lefthand — all route through here.
        check_binding(&declarator.id, renames)?;
        if let Some(init) = &mut declarator.init {
            walk_expression(init, renames, false)?;
        }
    }
    Ok(())
}

fn walk_class_body(
    body: &mut ClassBody,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    for method in &mut body.methods {
        // `method.name` is a property key, never a value-level identifier
        // lookup, so it cannot alias a sibling function — not checked.
        // `method.params` DOES introduce local bindings visible inside the
        // method body, so each is checked for a shadowing collision.
        for param in &method.params {
            check_binding(param, renames)?;
        }
        if let Some(method_body) = &mut method.body {
            walk_block(method_body, renames)?;
        }
    }
    Ok(())
}

fn walk_statement(
    statement: &mut Statement,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    match statement {
        Statement::ExpressionStatement(stmt) => {
            walk_expression(&mut stmt.expression, renames, false)
        }
        // `label` is a control-flow target name in its own label namespace,
        // never a value/identifier lookup.
        Statement::BreakStatement(_) => Ok(()),
        Statement::ContinueStatement(_) => Ok(()),
        Statement::WithStatement(stmt) => {
            walk_expression(&mut stmt.object, renames, false)?;
            walk_statement(&mut stmt.body, renames)
        }
        Statement::ReturnStatement(stmt) => {
            if let Some(argument) = &mut stmt.argument {
                walk_expression(argument, renames, false)?;
            }
            Ok(())
        }
        // `label` is a label-namespace name, not a value lookup.
        Statement::LabeledStatement(stmt) => walk_statement(&mut stmt.body, renames),
        Statement::IfStatement(stmt) => {
            walk_expression(&mut stmt.test, renames, false)?;
            walk_block(&mut stmt.consequent, renames)?;
            if let Some(alternate) = &mut stmt.alternate {
                walk_block(alternate, renames)?;
            }
            Ok(())
        }
        Statement::SwitchStatement(stmt) => {
            walk_expression(&mut stmt.discriminant, renames, false)?;
            for case in &mut stmt.cases {
                if let Some(test) = &mut case.test {
                    walk_expression(test, renames, false)?;
                }
                for consequent in &mut case.consequent {
                    walk_statement(consequent, renames)?;
                }
            }
            Ok(())
        }
        Statement::ThrowStatement(stmt) => walk_expression(&mut stmt.argument, renames, false),
        Statement::TryStatement(stmt) => {
            walk_block(&mut stmt.block, renames)?;
            if let Some(handler) = &mut stmt.handler {
                // `handler.param` is the caught-error BINDING name — a
                // shadowing collision, not a reference, so it is checked.
                check_binding(&handler.param, renames)?;
                walk_block(&mut handler.body, renames)?;
            }
            if let Some(finalizer) = &mut stmt.finalizer {
                walk_block(finalizer, renames)?;
            }
            Ok(())
        }
        // No fields at all.
        Statement::DebuggerStatement(_) => Ok(()),
        Statement::BlockStatement(stmt) => walk_block(stmt, renames),
        Statement::ForStatement(stmt) => {
            match &mut stmt.init {
                Some(ForInit::VariableDeclaration(decl)) => walk_var_decl(decl, renames)?,
                Some(ForInit::Expression(expr)) => walk_expression(expr, renames, false)?,
                None => {}
            }
            if let Some(test) = &mut stmt.test {
                walk_expression(test, renames, false)?;
            }
            if let Some(update) = &mut stmt.update {
                walk_expression(update, renames, false)?;
            }
            walk_block(&mut stmt.body, renames)
        }
        Statement::ForInStatement(stmt) => {
            match &mut stmt.left {
                ForInLefthand::VariableDeclaration(decl) => walk_var_decl(decl, renames)?,
                ForInLefthand::Expression(expr) => walk_expression(expr, renames, false)?,
            }
            walk_expression(&mut stmt.right, renames, false)?;
            walk_statement(&mut stmt.body, renames)
        }
        Statement::ForOfStatement(stmt) => {
            match &mut stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => walk_var_decl(decl, renames)?,
                ForOfLefthand::Expression(expr) => walk_expression(expr, renames, false)?,
            }
            walk_expression(&mut stmt.right, renames, false)?;
            // `is_await` carries no identifier reference.
            walk_statement(&mut stmt.body, renames)
        }
        Statement::WhileStatement(stmt) => {
            walk_expression(&mut stmt.test, renames, false)?;
            walk_block(&mut stmt.body, renames)
        }
        Statement::DoWhileStatement(stmt) => {
            walk_block(&mut stmt.body, renames)?;
            walk_expression(&mut stmt.test, renames, false)
        }
        // A nested function declaration: its own `name` and `params` are
        // declarations, not references — but each is checked for a
        // shadowing collision with a sibling linked function before its
        // body is walked in the same (unscoped) manner as everything else.
        Statement::FunctionDeclaration(function) => {
            check_binding(&function.name, renames)?;
            for param in &function.params {
                check_binding(param, renames)?;
            }
            walk_block(&mut function.body, renames)
        }
        Statement::ClassDeclaration(class) => {
            check_binding(&class.name, renames)?;
            walk_class_body(&mut class.body, renames)
        }
        Statement::VariableDeclaration(decl) => walk_var_decl(decl, renames),
        // A nested import is reachable inside a cloned function body (see
        // `nested_import_error`'s doc comment for the parser citation
        // proving this) and introduces local bindings this walk cannot
        // safely rename or verify non-colliding — reject outright rather
        // than silently leaving them un-renamed.
        Statement::ImportDeclaration(decl) => Err(nested_import_error(&decl.source)),
        // `ExportAllDeclaration` holds only a `source: String`
        // (`crates/kali_ast/src/module.rs:70-72`) — no identifier-bearing
        // field of any kind. Even though this parser accepts a nested
        // `export * from "..."` the same way it accepts a nested
        // import/named-export (`TokenType::Export` routes through the same
        // unrestricted `parse_statement` loop — see `nested_import_error`'s
        // doc comment), there is no reference here that could ever alias a
        // sibling linked function, so this is verified safe as a no-op
        // regardless of nesting depth.
        Statement::ExportAll(_) => Ok(()),
        // A nested named export is reachable the same way (see
        // `nested_export_named_error`'s doc comment for the parser
        // citation) and its specifiers name a local binding by string —
        // reject outright rather than silently leaving it unresolved.
        Statement::ExportNamed(_) => Err(nested_export_named_error()),
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => walk_expression(expr, renames, false),
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                check_binding(&function.name, renames)?;
                for param in &function.params {
                    check_binding(param, renames)?;
                }
                walk_block(&mut function.body, renames)
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                check_binding(&class.name, renames)?;
                walk_class_body(&mut class.body, renames)
            }
        },
        Statement::EnumDeclaration(decl) => {
            // `decl.name` is the enum's own declared name — a
            // shadowing collision, not a reference, so it is checked.
            check_binding(&decl.name, renames)?;
            for member in &mut decl.members {
                // `member.name` is a declaration, not a reference.
                if let Some(value) = &mut member.value {
                    walk_expression(value, renames, false)?;
                }
            }
            Ok(())
        }
        // TypeScript type syntax only (`type_annotation: String`) — no
        // `Expression` content to walk.
        Statement::TypeAliasDeclaration(_) => Ok(()),
        Statement::InterfaceDeclaration(_) => Ok(()),
    }
}

fn walk_expression_or_spread(
    element: &mut ExpressionOrSpread,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    match element {
        ExpressionOrSpread::Expression(expr) => walk_expression(expr, renames, false),
        ExpressionOrSpread::Spread(spread) => walk_expression(&mut spread.argument, renames, false),
        ExpressionOrSpread::Empty => Ok(()),
    }
}

fn walk_expression(
    expr: &mut Expression,
    renames: &BTreeMap<String, String>,
    is_callee: bool,
) -> Result<(), Diagnostic> {
    match expr {
        Expression::Identifier(name) => {
            if let Some(mangled) = renames.get(name.as_str()) {
                if is_callee {
                    *name = mangled.clone();
                } else {
                    return Err(bare_sibling_reference_error(name));
                }
            }
            Ok(())
        }
        // A literal value carries no identifier reference.
        Expression::Literal(_) => Ok(()),
        Expression::BinaryExpression(binary) => {
            walk_expression(&mut binary.left, renames, false)?;
            walk_expression(&mut binary.right, renames, false)
        }
        Expression::UnaryExpression(unary) => walk_expression(&mut unary.argument, renames, false),
        Expression::CallExpression(call) => {
            // The ONLY position that is ever a "callee" for this walk.
            walk_expression(&mut call.callee, renames, true)?;
            for arg in &mut call.args {
                walk_expression(arg, renames, false)?;
            }
            Ok(())
        }
        Expression::MemberExpression(member) => {
            // `member.property` is a static field-name string, not a
            // reference; `helper.foo()`'s `helper` is `member.object`, which
            // IS a value-level reference and is walked as non-callee below
            // (so it fails closed as a bare reference, since only the
            // literal `Identifier` callee slot of a `CallExpression` is
            // rewritable).
            walk_expression(&mut member.object, renames, false)?;
            if let Some(index) = &mut member.computed_index {
                walk_expression(index, renames, false)?;
            }
            Ok(())
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter_mut().flatten() {
                walk_expression_or_spread(element, renames)?;
            }
            Ok(())
        }
        Expression::ObjectExpression(object) => {
            for property in &mut object.properties {
                // `property.key` (`PropertyName::Identifier` included) is a
                // static property NAME, never a value lookup, even when
                // spelled the same as a sibling function.
                walk_expression(&mut property.value, renames, false)?;
            }
            Ok(())
        }
        Expression::FunctionExpression(function) => {
            // `function.id`/`function.params` are declarations, not
            // references — but each is checked for a shadowing collision.
            if let Some(id) = &function.id {
                check_binding(id, renames)?;
            }
            for param in &function.params {
                check_binding(&param.name, renames)?;
            }
            if let Some(body) = &mut function.body {
                walk_block(body, renames)?;
            }
            Ok(())
        }
        Expression::ArrowFunctionExpression(arrow) => {
            // `arrow.params` are declarations, not references — but each is
            // checked for a shadowing collision.
            for param in &arrow.params {
                check_binding(&param.name, renames)?;
            }
            walk_expression(&mut arrow.body, renames, false)
        }
        Expression::ClassExpression(class) => {
            // `class.id` is a declaration, not a reference — but it is
            // checked for a shadowing collision.
            if let Some(id) = &class.id {
                check_binding(id, renames)?;
            }
            walk_class_body(&mut class.body, renames)
        }
        Expression::NewExpression(new_expr) => {
            // `new helper()`'s callee is walked as non-callee: only a plain
            // `CallExpression` callee is rewritten by this pass, so a `new`
            // callee referencing a sibling fails closed as a bare reference
            // rather than being silently (and wrongly, since `new` and call
            // semantics differ) rewritten.
            walk_expression(&mut new_expr.callee, renames, false)?;
            for arg in &mut new_expr.args {
                walk_expression(arg, renames, false)?;
            }
            Ok(())
        }
        // `meta`/`property` are fixed keyword strings (e.g. `import.meta`),
        // never identifier lookups.
        Expression::MetaProperty(_) => Ok(()),
        Expression::TemplateLiteral(template) => {
            // `template.quasis` are literal string chunks, not references.
            for expression in &mut template.expressions {
                walk_expression(expression, renames, false)?;
            }
            Ok(())
        }
        Expression::TaggedTemplateExpression(tagged) => {
            walk_expression(&mut tagged.tag, renames, false)?;
            for expression in &mut tagged.template.expressions {
                walk_expression(expression, renames, false)?;
            }
            Ok(())
        }
        Expression::UpdateExpression(update) => {
            walk_expression(&mut update.argument, renames, false)
        }
        Expression::AssignmentExpression(assignment) => {
            walk_expression(&mut assignment.left, renames, false)?;
            walk_expression(&mut assignment.right, renames, false)
        }
        Expression::LogicalExpression(logical) => {
            walk_expression(&mut logical.left, renames, false)?;
            walk_expression(&mut logical.right, renames, false)
        }
        Expression::ConditionalExpression(conditional) => {
            walk_expression(&mut conditional.test, renames, false)?;
            walk_expression(&mut conditional.consequent, renames, false)?;
            walk_expression(&mut conditional.alternate, renames, false)
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &mut sequence.expressions {
                walk_expression(expression, renames, false)?;
            }
            Ok(())
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            walk_expression(&mut parenthesized.expression, renames, false)
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &mut yield_expr.argument {
                walk_expression(argument, renames, false)?;
            }
            Ok(())
        }
        Expression::AwaitExpression(await_expr) => {
            walk_expression(&mut await_expr.argument, renames, false)
        }
        Expression::OptionalChainExpression(chain) => match chain.inner.as_mut() {
            OptionalChainInner::NonNull { object, .. } => walk_expression(object, renames, false),
        },
        Expression::ChainExpression(chain) => {
            walk_expression(&mut chain.expression, renames, false)
        }
        Expression::SpreadElement(spread) => walk_expression(&mut spread.argument, renames, false),
        Expression::RestElement(rest) => walk_expression(&mut rest.argument, renames, false),
        Expression::ImportExpression(import_expr) => {
            walk_expression(&mut import_expr.source, renames, false)
        }
        Expression::DecoratedExpression(decorated) => {
            walk_expression(&mut decorated.expression, renames, false)
        }
        Expression::JsxElement(element) => walk_jsx_element(element, renames),
        Expression::JsxFragment(fragment) => walk_jsx_fragment(fragment, renames),
        // No content at all.
        Expression::JsxEmptyExpression => Ok(()),
        Expression::TypeAssertion(assertion) => {
            // `assertion.type_name` is TypeScript type syntax, not a value reference.
            walk_expression(&mut assertion.expression, renames, false)
        }
        Expression::SatisfiesExpression(satisfies) => {
            // `satisfies.type_name` is TypeScript type syntax, not a value reference.
            walk_expression(&mut satisfies.expression, renames, false)
        }
        // `this`/`super` are keywords, never identifier lookups.
        Expression::ThisExpression => Ok(()),
        Expression::SuperExpression => Ok(()),
        // A private class-field name (`#foo`) is a distinct namespace from
        // top-level function bindings — cannot alias a sibling function.
        Expression::PrivateIdentifier(_) => Ok(()),
        // A literal numeric value, not a reference.
        Expression::BigIntLiteral(_) => Ok(()),
    }
}

fn walk_jsx_element(
    element: &mut JsxElement,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    walk_jsx_name(&mut element.opening_element.name, renames)?;
    for attribute in &mut element.opening_element.attributes {
        walk_jsx_attribute_item(attribute, renames)?;
    }
    for child in &mut element.children {
        walk_jsx_child(child, renames)?;
    }
    if let Some(closing) = &mut element.closing_element {
        walk_jsx_name(&mut closing.name, renames)?;
    }
    Ok(())
}

fn walk_jsx_fragment(
    fragment: &mut JsxFragment,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    for child in &mut fragment.children {
        walk_jsx_child(child, renames)?;
    }
    Ok(())
}

/// `JsxName::Identifier` is a real value-level lookup in JSX semantics (a
/// capitalized tag name resolves against an in-scope binding, e.g. a
/// component function) even though it is a plain `String` field rather than
/// an `Expression::Identifier` node. It can never legally be a call callee
/// in this walk's sense, so a sibling match here always fails closed as a
/// bare reference — never silently renamed.
fn walk_jsx_name(name: &mut JsxName, renames: &BTreeMap<String, String>) -> Result<(), Diagnostic> {
    match name {
        JsxName::Identifier(identifier) => check_bare_reference(identifier, renames),
        JsxName::JsxClosedElement(closing) => walk_jsx_name(&mut closing.name, renames),
    }
}

fn walk_jsx_attribute_item(
    item: &mut JsxAttributeItem,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            walk_jsx_name(&mut attribute.name, renames)?;
            walk_jsx_attribute_value(&mut attribute.value, renames)
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            walk_expression(&mut spread.argument, renames, false)
        }
    }
}

fn walk_jsx_attribute_value(
    value: &mut JsxAttributeValue,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    match value {
        // A plain string literal attribute value, not a reference.
        JsxAttributeValue::String(_) => Ok(()),
        JsxAttributeValue::JsxElement(element) => walk_jsx_element(element, renames),
        JsxAttributeValue::JsxExpression(container) => {
            walk_jsx_expression_container(container, renames)
        }
    }
}

fn walk_jsx_expression_container(
    container: &mut JsxExpressionContainer,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    if let Some(expression) = &mut container.expression {
        walk_expression(expression, renames, false)?;
    }
    Ok(())
}

fn walk_jsx_child(
    child: &mut JsxChild,
    renames: &BTreeMap<String, String>,
) -> Result<(), Diagnostic> {
    match child {
        // Literal text content, not a reference.
        JsxChild::JsxText(_) => Ok(()),
        JsxChild::JsxExpression(container) => walk_jsx_expression_container(container, renames),
        JsxChild::JsxElement(element) => walk_jsx_element(element, renames),
        JsxChild::JsxFragment(fragment) => walk_jsx_fragment(fragment, renames),
    }
}

// ---- namespace-use rewrite walk (Task 6) ----
//
// Deep pre-order walk over `Statement`/`Expression` mirroring the exhaustive
// traversal SHAPE of the sibling-rename walk above
// (`walk_block`/`walk_statement`/`walk_expression`) — same node coverage, no
// `_ =>` arm on either enum, so the compiler forces a decision at every
// current and future variant — but keyed by namespace-binding provenance
// instead of a sibling-rename map, and non-fallible: instead of aborting on
// the first problem it accumulates E5506 diagnostics into `diagnostics` and
// keeps walking (the caller fails the build if `diagnostics` is non-empty
// afterward). Unlike the rename walk, this one has no binding/shadowing
// concerns of its own to check — it only ever *reads* structural shape
// (`ns.member` / `typeof ns.member` / `ns.member(...)`), so most arms are
// pure recursion with no per-node guard.

/// Rewrites proven uses of namespace bindings in place; pushes E5506
/// diagnostics for uses of non-exported members.
///
/// `statements` is a SLICE, not the whole post-append `Vec`: the pipeline
/// passes only the ENTRY module's own statements (`statements[clone_count..]`
/// — see `link_provable_module_namespaces`). A spliced-in linked-module
/// clone body is in the LINKED module's lexical scope, never the entry's, so
/// an `x.foo()` inside a clone that happens to spell an entry provenance
/// binding's name is NOT a namespace access and must not be rewritten (the
/// same wrong-scope class the final whole-branch review's "Minor" finding
/// closes for `deny_unrewritten_uses`). This also keeps
/// `try_fold_typeof_namespace_member`/`try_rewrite_namespace_call_callee`'s
/// "every proven binding has a loaded module" invariant true under the I1
/// load gate: a proven binding is only LOADED when it has a member-use-site
/// shape in the ENTRY program, so only entry statements may be rewritten.
pub fn rewrite_namespace_uses(
    statements: &mut [Statement],
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for statement in statements.iter_mut() {
        rewrite_statement(statement, provenance, modules, diagnostics);
    }
}

/// The `"function"`/`"undefined"` fold target, constructed with the SAME
/// quoting convention the lexer/parser use for a plain string-literal token:
/// `crates/kali_lexer/src/string.rs:9-53` (`lex_string`) pushes the opening
/// AND closing quote characters into the token's `value` verbatim (never
/// stripped), and `crates/kali_parser/src/expression/primary.rs:76` wraps
/// that raw token value straight into `LiteralValue::String` with no further
/// processing. This is PROVEN, not merely asserted by convention, by the
/// `folded_string_literal_matches_parser_construction` test below, which
/// parses an equivalent literal and asserts `Expression` equality.
fn string_literal_expression(value: &str) -> Expression {
    Expression::Literal(LiteralValue::String(format!("\"{value}\"")))
}

fn computed_member_access_error(module_path: &Path) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "module '{}': computed member access on a module namespace is unavailable — only a plain `ns.member` (dot access with a literal name) can be folded or linked",
            module_path.display()
        ),
    )
}

fn non_export_error(module_path: &Path, property: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "module '{}' does not export '{}'",
            module_path.display(),
            property
        ),
    )
}

/// If `argument` is `MemberExpression { object: Identifier(ns), property,
/// computed_index: None }` with `ns` a proven namespace binding, folds the
/// WHOLE `typeof` expression to a string-literal `Expression` replacement:
/// `"function"` if `property` is a true export, `"undefined"` otherwise —
/// the namespace is SEALED, so a non-exported name (including a genuinely
/// private helper linked only for sibling calls) genuinely evaluates to
/// `undefined` at runtime, matching node — never a `TypeError`, unlike a
/// non-exported CALL (see `try_rewrite_namespace_call_callee`). Computed
/// access (`typeof ns[expr]`) pushes an E5506 reject instead and returns
/// `None` — the node is left as-is; the caller still walks into it
/// afterward for deeper issues nested inside the computed index. Returns
/// `None` for anything that is not this exact shape (no namespace binding
/// involved at all) — the caller falls back to ordinary recursion.
fn try_fold_typeof_namespace_member(
    argument: &Expression,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Expression> {
    let Expression::MemberExpression(member) = argument else {
        return None;
    };
    let Expression::Identifier(ns) = &member.object else {
        return None;
    };
    let linked = provenance.bindings.get(ns)?.clone();
    if member.computed_index.is_some() {
        diagnostics.push(computed_member_access_error(&linked.path));
        return None;
    }
    let module = modules.get(&linked.index).expect(
        "a proven binding reaching this member-access shape in the ENTRY program is exactly the \
         condition the I1 load gate loads its module under (`BindingSignals::member_access_sites`), \
         and only entry statements are rewritten (see `rewrite_namespace_uses`), so its module is \
         always in `modules` here",
    );
    let literal = if module.exports.contains_key(&member.property) {
        "function"
    } else {
        "undefined"
    };
    Some(string_literal_expression(literal))
}

/// If `callee` is `MemberExpression { object: Identifier(ns), property,
/// computed_index: None }` with `ns` a proven namespace binding, either
/// rewrites `callee` in place to `Identifier(mangled)` (`property` is a true
/// export) or pushes an E5506 reject (`property` is absent from `exports` —
/// including a genuinely-linked PRIVATE helper present in `all_functions`,
/// which must reject exactly like a wholly unknown name: it was linked only
/// so exported bodies could call it, never for outside namespace access) and
/// leaves `callee` untouched (the build fails on the diagnostic; node itself
/// raises a `TypeError` here at runtime, which this pass rejects at compile
/// time instead). Computed access (`ns[expr](...)`) pushes the same
/// computed-access E5506 instead. Does nothing at all (no diagnostic, no
/// rewrite) for anything that is not this exact shape.
fn try_rewrite_namespace_call_callee(
    callee: &mut Expression,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Expression::MemberExpression(member) = callee else {
        return;
    };
    let Expression::Identifier(ns) = &member.object else {
        return;
    };
    let Some(linked) = provenance.bindings.get(ns).cloned() else {
        return;
    };
    if member.computed_index.is_some() {
        diagnostics.push(computed_member_access_error(&linked.path));
        return;
    }
    let property = member.property.clone();
    let module = modules.get(&linked.index).expect(
        "a proven binding reaching this member-access shape in the ENTRY program is exactly the \
         condition the I1 load gate loads its module under (`BindingSignals::member_access_sites`), \
         and only entry statements are rewritten (see `rewrite_namespace_uses`), so its module is \
         always in `modules` here",
    );
    if module.exports.contains_key(&property) {
        *callee = Expression::Identifier(mangled_link_name(linked.index, &property));
    } else {
        diagnostics.push(non_export_error(&linked.path, &property));
    }
}

fn rewrite_block(
    block: &mut BlockStatement,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for statement in &mut block.body {
        rewrite_statement(statement, provenance, modules, diagnostics);
    }
}

fn rewrite_var_decl(
    decl: &mut VariableDeclaration,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // `decl.kind` and each `declarator.id` carry no `Expression` content.
    for declarator in &mut decl.declarations {
        if let Some(init) = &mut declarator.init {
            rewrite_expression(init, provenance, modules, diagnostics);
        }
    }
}

fn rewrite_class_body(
    body: &mut ClassBody,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for method in &mut body.methods {
        // `method.name`/`method.params` carry no `Expression` content.
        if let Some(method_body) = &mut method.body {
            rewrite_block(method_body, provenance, modules, diagnostics);
        }
    }
}

fn rewrite_statement(
    statement: &mut Statement,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::ExpressionStatement(stmt) => {
            rewrite_expression(&mut stmt.expression, provenance, modules, diagnostics)
        }
        // `label` is a control-flow target name, never an `Expression`.
        Statement::BreakStatement(_) => {}
        Statement::ContinueStatement(_) => {}
        Statement::WithStatement(stmt) => {
            rewrite_expression(&mut stmt.object, provenance, modules, diagnostics);
            rewrite_statement(&mut stmt.body, provenance, modules, diagnostics);
        }
        Statement::ReturnStatement(stmt) => {
            if let Some(argument) = &mut stmt.argument {
                rewrite_expression(argument, provenance, modules, diagnostics);
            }
        }
        Statement::LabeledStatement(stmt) => {
            rewrite_statement(&mut stmt.body, provenance, modules, diagnostics)
        }
        Statement::IfStatement(stmt) => {
            rewrite_expression(&mut stmt.test, provenance, modules, diagnostics);
            rewrite_block(&mut stmt.consequent, provenance, modules, diagnostics);
            if let Some(alternate) = &mut stmt.alternate {
                rewrite_block(alternate, provenance, modules, diagnostics);
            }
        }
        Statement::SwitchStatement(stmt) => {
            rewrite_expression(&mut stmt.discriminant, provenance, modules, diagnostics);
            for case in &mut stmt.cases {
                if let Some(test) = &mut case.test {
                    rewrite_expression(test, provenance, modules, diagnostics);
                }
                for consequent in &mut case.consequent {
                    rewrite_statement(consequent, provenance, modules, diagnostics);
                }
            }
        }
        Statement::ThrowStatement(stmt) => {
            rewrite_expression(&mut stmt.argument, provenance, modules, diagnostics)
        }
        Statement::TryStatement(stmt) => {
            rewrite_block(&mut stmt.block, provenance, modules, diagnostics);
            if let Some(handler) = &mut stmt.handler {
                // `handler.param` is the caught-error binding name, not an
                // `Expression`.
                rewrite_block(&mut handler.body, provenance, modules, diagnostics);
            }
            if let Some(finalizer) = &mut stmt.finalizer {
                rewrite_block(finalizer, provenance, modules, diagnostics);
            }
        }
        // No fields at all.
        Statement::DebuggerStatement(_) => {}
        Statement::BlockStatement(stmt) => rewrite_block(stmt, provenance, modules, diagnostics),
        Statement::ForStatement(stmt) => {
            match &mut stmt.init {
                Some(ForInit::VariableDeclaration(decl)) => {
                    rewrite_var_decl(decl, provenance, modules, diagnostics)
                }
                Some(ForInit::Expression(expr)) => {
                    rewrite_expression(expr, provenance, modules, diagnostics)
                }
                None => {}
            }
            if let Some(test) = &mut stmt.test {
                rewrite_expression(test, provenance, modules, diagnostics);
            }
            if let Some(update) = &mut stmt.update {
                rewrite_expression(update, provenance, modules, diagnostics);
            }
            rewrite_block(&mut stmt.body, provenance, modules, diagnostics);
        }
        Statement::ForInStatement(stmt) => {
            match &mut stmt.left {
                ForInLefthand::VariableDeclaration(decl) => {
                    rewrite_var_decl(decl, provenance, modules, diagnostics)
                }
                ForInLefthand::Expression(expr) => {
                    rewrite_expression(expr, provenance, modules, diagnostics)
                }
            }
            rewrite_expression(&mut stmt.right, provenance, modules, diagnostics);
            rewrite_statement(&mut stmt.body, provenance, modules, diagnostics);
        }
        Statement::ForOfStatement(stmt) => {
            match &mut stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => {
                    rewrite_var_decl(decl, provenance, modules, diagnostics)
                }
                ForOfLefthand::Expression(expr) => {
                    rewrite_expression(expr, provenance, modules, diagnostics)
                }
            }
            rewrite_expression(&mut stmt.right, provenance, modules, diagnostics);
            // `is_await` carries no `Expression` content.
            rewrite_statement(&mut stmt.body, provenance, modules, diagnostics);
        }
        Statement::WhileStatement(stmt) => {
            rewrite_expression(&mut stmt.test, provenance, modules, diagnostics);
            rewrite_block(&mut stmt.body, provenance, modules, diagnostics);
        }
        Statement::DoWhileStatement(stmt) => {
            rewrite_block(&mut stmt.body, provenance, modules, diagnostics);
            rewrite_expression(&mut stmt.test, provenance, modules, diagnostics);
        }
        // `function.name`/`function.params` carry no `Expression` content.
        Statement::FunctionDeclaration(function) => {
            rewrite_block(&mut function.body, provenance, modules, diagnostics)
        }
        Statement::ClassDeclaration(class) => {
            rewrite_class_body(&mut class.body, provenance, modules, diagnostics)
        }
        Statement::VariableDeclaration(decl) => {
            rewrite_var_decl(decl, provenance, modules, diagnostics)
        }
        // `ImportDeclaration { specifiers: Vec<ImportSpecifier>, source:
        // String }` (`crates/kali_ast/src/module.rs:7-10`) — every field is
        // a `String`/specifier-name, never an `Expression`. Unlike the
        // sibling-rename walk above (which cares about a nested import
        // because it introduces a local BINDING that could shadow a mangled
        // sibling name), this walk has no binding/shadowing concept at all
        // — it only ever looks for `ns.member` shape, which cannot occur
        // inside these string-only fields regardless of nesting depth. A
        // verified no-op, not an unverified one.
        Statement::ImportDeclaration(_) => {}
        // `ExportAllDeclaration { source: String }`
        // (`crates/kali_ast/src/module.rs:69-72`) — same reasoning as above.
        Statement::ExportAll(_) => {}
        // `ExportNamedDeclaration { specifiers: Vec<ExportSpecifier>,
        // source: Option<String> }` and `ExportSpecifier { local: String,
        // exported: String }` (`crates/kali_ast/src/module.rs:56-66`) — same
        // reasoning as above: string-only fields, no `Expression` content.
        Statement::ExportNamed(_) => {}
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => {
                rewrite_expression(expr, provenance, modules, diagnostics)
            }
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                rewrite_block(&mut function.body, provenance, modules, diagnostics)
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                rewrite_class_body(&mut class.body, provenance, modules, diagnostics)
            }
        },
        Statement::EnumDeclaration(decl) => {
            // `decl.name`/`member.name` are declaration names, not
            // `Expression`s.
            for member in &mut decl.members {
                if let Some(value) = &mut member.value {
                    rewrite_expression(value, provenance, modules, diagnostics);
                }
            }
        }
        // TypeScript type syntax only — no `Expression` content to walk.
        Statement::TypeAliasDeclaration(_) => {}
        Statement::InterfaceDeclaration(_) => {}
    }
}

fn rewrite_expression_or_spread(
    element: &mut ExpressionOrSpread,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match element {
        ExpressionOrSpread::Expression(expr) => {
            rewrite_expression(expr, provenance, modules, diagnostics)
        }
        ExpressionOrSpread::Spread(spread) => {
            rewrite_expression(&mut spread.argument, provenance, modules, diagnostics)
        }
        ExpressionOrSpread::Empty => {}
    }
}

fn rewrite_expression(
    expr: &mut Expression,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match expr {
        // A bare identifier reference is never, by itself, a rewritable
        // `ns.member` shape (that requires a `MemberExpression` wrapper) —
        // leftover bare `ns` uses are Task 7's default-deny, not this
        // task's job.
        Expression::Identifier(_) => {}
        // A literal value carries no further `Expression` content.
        Expression::Literal(_) => {}
        Expression::BinaryExpression(binary) => {
            rewrite_expression(&mut binary.left, provenance, modules, diagnostics);
            rewrite_expression(&mut binary.right, provenance, modules, diagnostics);
        }
        Expression::UnaryExpression(unary) => {
            if unary.operator == "typeof" {
                if let Some(replacement) = try_fold_typeof_namespace_member(
                    &unary.argument,
                    provenance,
                    modules,
                    diagnostics,
                ) {
                    *expr = replacement;
                    return;
                }
            }
            rewrite_expression(&mut unary.argument, provenance, modules, diagnostics);
        }
        Expression::CallExpression(call) => {
            // The ONLY position this walk special-cases: the direct
            // `callee` slot of a `CallExpression`. If it names a namespace
            // binding, this either rewrites it in place or pushes a
            // diagnostic (leaving it untouched); either way, walking it
            // again immediately below is always safe — a rewritten
            // `Identifier` has nothing left to walk (no-op above), and a
            // left-untouched `MemberExpression` gets the same generic
            // structural walk any other member expression would (reaching
            // a computed index's nested content, for defense in depth).
            try_rewrite_namespace_call_callee(&mut call.callee, provenance, modules, diagnostics);
            rewrite_expression(&mut call.callee, provenance, modules, diagnostics);
            for arg in &mut call.args {
                rewrite_expression(arg, provenance, modules, diagnostics);
            }
        }
        Expression::MemberExpression(member) => {
            // No special-casing here: a bare `ns.member` (not the argument
            // of `typeof`, not a call callee) is a leftover use for Task 7.
            // `member.property` is a static field-name string, never a
            // reference.
            rewrite_expression(&mut member.object, provenance, modules, diagnostics);
            if let Some(index) = &mut member.computed_index {
                rewrite_expression(index, provenance, modules, diagnostics);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter_mut().flatten() {
                rewrite_expression_or_spread(element, provenance, modules, diagnostics);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &mut object.properties {
                // `property.key` is a static property name, never a
                // reference.
                rewrite_expression(&mut property.value, provenance, modules, diagnostics);
            }
        }
        Expression::FunctionExpression(function) => {
            // `function.id`/`function.params` carry no `Expression` content.
            if let Some(body) = &mut function.body {
                rewrite_block(body, provenance, modules, diagnostics);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            // `arrow.params` carry no `Expression` content.
            rewrite_expression(&mut arrow.body, provenance, modules, diagnostics);
        }
        Expression::ClassExpression(class) => {
            // `class.id` carries no `Expression` content.
            rewrite_class_body(&mut class.body, provenance, modules, diagnostics)
        }
        Expression::NewExpression(new_expr) => {
            // `new ns.Thing()` is deliberately NOT special-cased — only a
            // plain `CallExpression` callee is rewritten by this pass (per
            // the brief), so a `new` callee naming a namespace binding
            // falls through untouched here, same as any other leftover use.
            rewrite_expression(&mut new_expr.callee, provenance, modules, diagnostics);
            for arg in &mut new_expr.args {
                rewrite_expression(arg, provenance, modules, diagnostics);
            }
        }
        // `meta`/`property` are fixed keyword strings (e.g. `import.meta`),
        // never identifier lookups.
        Expression::MetaProperty(_) => {}
        Expression::TemplateLiteral(template) => {
            // `template.quasis` are literal string chunks, not references.
            for expression in &mut template.expressions {
                rewrite_expression(expression, provenance, modules, diagnostics);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            rewrite_expression(&mut tagged.tag, provenance, modules, diagnostics);
            for expression in &mut tagged.template.expressions {
                rewrite_expression(expression, provenance, modules, diagnostics);
            }
        }
        Expression::UpdateExpression(update) => {
            rewrite_expression(&mut update.argument, provenance, modules, diagnostics)
        }
        Expression::AssignmentExpression(assignment) => {
            rewrite_expression(&mut assignment.left, provenance, modules, diagnostics);
            rewrite_expression(&mut assignment.right, provenance, modules, diagnostics);
        }
        Expression::LogicalExpression(logical) => {
            rewrite_expression(&mut logical.left, provenance, modules, diagnostics);
            rewrite_expression(&mut logical.right, provenance, modules, diagnostics);
        }
        Expression::ConditionalExpression(conditional) => {
            rewrite_expression(&mut conditional.test, provenance, modules, diagnostics);
            rewrite_expression(
                &mut conditional.consequent,
                provenance,
                modules,
                diagnostics,
            );
            rewrite_expression(&mut conditional.alternate, provenance, modules, diagnostics);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &mut sequence.expressions {
                rewrite_expression(expression, provenance, modules, diagnostics);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => rewrite_expression(
            &mut parenthesized.expression,
            provenance,
            modules,
            diagnostics,
        ),
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &mut yield_expr.argument {
                rewrite_expression(argument, provenance, modules, diagnostics);
            }
        }
        Expression::AwaitExpression(await_expr) => {
            // The `await` wrapper itself is never touched — only its
            // `argument` is walked, so a rewrite deep inside (e.g. `await
            // ns.lazyValue()`, where `argument` is the `CallExpression`)
            // leaves this node structurally intact.
            rewrite_expression(&mut await_expr.argument, provenance, modules, diagnostics)
        }
        Expression::OptionalChainExpression(chain) => match chain.inner.as_mut() {
            OptionalChainInner::NonNull { object, .. } => {
                rewrite_expression(object, provenance, modules, diagnostics)
            }
        },
        Expression::ChainExpression(chain) => {
            rewrite_expression(&mut chain.expression, provenance, modules, diagnostics)
        }
        Expression::SpreadElement(spread) => {
            rewrite_expression(&mut spread.argument, provenance, modules, diagnostics)
        }
        Expression::RestElement(rest) => {
            rewrite_expression(&mut rest.argument, provenance, modules, diagnostics)
        }
        Expression::ImportExpression(import_expr) => {
            rewrite_expression(&mut import_expr.source, provenance, modules, diagnostics)
        }
        Expression::DecoratedExpression(decorated) => {
            rewrite_expression(&mut decorated.expression, provenance, modules, diagnostics)
        }
        Expression::JsxElement(element) => {
            rewrite_jsx_element(element, provenance, modules, diagnostics)
        }
        Expression::JsxFragment(fragment) => {
            rewrite_jsx_fragment(fragment, provenance, modules, diagnostics)
        }
        // No content at all.
        Expression::JsxEmptyExpression => {}
        Expression::TypeAssertion(assertion) => {
            // `assertion.type_name` is TypeScript type syntax, not a value reference.
            rewrite_expression(&mut assertion.expression, provenance, modules, diagnostics)
        }
        Expression::SatisfiesExpression(satisfies) => {
            // `satisfies.type_name` is TypeScript type syntax, not a value reference.
            rewrite_expression(&mut satisfies.expression, provenance, modules, diagnostics)
        }
        // `this`/`super` are keywords, never identifier lookups.
        Expression::ThisExpression => {}
        Expression::SuperExpression => {}
        // A private class-field name (`#foo`) can never be a namespace
        // binding.
        Expression::PrivateIdentifier(_) => {}
        // A literal numeric value, not a reference.
        Expression::BigIntLiteral(_) => {}
    }
}

fn rewrite_jsx_element(
    element: &mut JsxElement,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // `opening_element.name`/`closing_element.name` (`JsxName`) hold only
    // tag-name strings at every depth (`Identifier(String)` or a nested
    // `JsxClosedElement` wrapping another `JsxName`) — no `Expression`
    // content anywhere in that type, so nothing to walk there.
    for attribute in &mut element.opening_element.attributes {
        rewrite_jsx_attribute_item(attribute, provenance, modules, diagnostics);
    }
    for child in &mut element.children {
        rewrite_jsx_child(child, provenance, modules, diagnostics);
    }
}

fn rewrite_jsx_fragment(
    fragment: &mut JsxFragment,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for child in &mut fragment.children {
        rewrite_jsx_child(child, provenance, modules, diagnostics);
    }
}

fn rewrite_jsx_attribute_item(
    item: &mut JsxAttributeItem,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            rewrite_jsx_attribute_value(&mut attribute.value, provenance, modules, diagnostics)
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            rewrite_expression(&mut spread.argument, provenance, modules, diagnostics)
        }
    }
}

fn rewrite_jsx_attribute_value(
    value: &mut JsxAttributeValue,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match value {
        // A plain string literal attribute value, not a reference.
        JsxAttributeValue::String(_) => {}
        JsxAttributeValue::JsxElement(element) => {
            rewrite_jsx_element(element, provenance, modules, diagnostics)
        }
        JsxAttributeValue::JsxExpression(container) => {
            rewrite_jsx_expression_container(container, provenance, modules, diagnostics)
        }
    }
}

fn rewrite_jsx_expression_container(
    container: &mut JsxExpressionContainer,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(expression) = &mut container.expression {
        rewrite_expression(expression, provenance, modules, diagnostics);
    }
}

fn rewrite_jsx_child(
    child: &mut JsxChild,
    provenance: &NamespaceProvenance,
    modules: &BTreeMap<usize, LinkedModuleAst>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match child {
        // Literal text content, not a reference.
        JsxChild::JsxText(_) => {}
        JsxChild::JsxExpression(container) => {
            rewrite_jsx_expression_container(container, provenance, modules, diagnostics)
        }
        JsxChild::JsxElement(element) => {
            rewrite_jsx_element(element, provenance, modules, diagnostics)
        }
        JsxChild::JsxFragment(fragment) => {
            rewrite_jsx_fragment(fragment, provenance, modules, diagnostics)
        }
    }
}

/// Strips a single layer of matching `'`/`"`/`` ` `` quoting, mirroring
/// `eval::unquote_string_literal` (private to that module, so re-implemented
/// here rather than exposed cross-module for one call site).
fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    match (chars.next(), chars.next_back()) {
        (Some(first), Some(last))
            if trimmed.len() >= 2
                && matches!((first, last), ('"', '"') | ('\'', '\'') | ('`', '`')) =>
        {
            trimmed[first.len_utf8()..trimmed.len() - last.len_utf8()].to_string()
        }
        _ => trimmed.to_string(),
    }
}

// ---- ImportExpression position allowlist (C2 remainder — second review round) ----
//
// The C2 default-deny above (`deny_unproven_namespace_binding_candidates`) is a DENYLIST OF
// BINDING SHAPES: it only ever looks for an `ImportExpression` at the two exact spots
// `signal_var_decl`/`signal_statement` know to check — a `VariableDeclarator.init` (any kind, any
// depth) and a relative `import * as ns from "./x"` specifier. Any `import(...)` whose value
// reaches a member access by some OTHER route was never even recorded as a "candidate" at all, so
// it fell through both `collect_namespace_provenance` (no provenance) AND the C2 candidate deny
// (never in `signals.candidates`) straight to the pre-stage silent `0` — five such fail-opens,
// all probe-proven on a fresh binary (a second whole-branch review round; util.js exports
// `greet() { return 42; }`):
//   - `let c; c = await import(...); c.greet()`            (assignment, not a declarator init)
//   - same, `typeof c.greet`
//   - `(await import(...)).greet()`                         (inline member access, no binding)
//   - `const c = (0, await import(...)); c.greet()`          (sequence-expression init)
//   - `box.m = await import(...); box.m.greet()`             (member/property sink)
//
// THIRD review round: a verification review found this walk's own `Expression::ImportExpression`
// arm (see its doc comment below) claimed a BARE, non-awaited `import(...)` needed no denial at
// any position — that claim was FALSE. `await p` on a separately-bound identifier never
// syntactically wraps the `ImportExpression` at all, so a bare import laundered through a binding
// escaped both this walk AND `signal_var_decl`'s candidate census (which only recognized the
// `AwaitExpression`-wrapped shape). Both gates are now fixed to also catch the bare shape — see
// `is_foldable_import_specifier` and `signal_var_decl` below.
//
// This walk closes the class BY CONSTRUCTION instead of enumerating more shapes: it censuses
// EVERY `Expression::ImportExpression` node in the program, at any depth, and default-denies each
// one whose SYNTACTIC POSITION is not on a two-item allowlist — (a) the `init` of a
// `VariableDeclarator`, of any `kind` and at any nesting depth (mod `ParenthesizedExpression`/
// `AwaitExpression` wrapping around either layer, exactly what `as_await_import_source` already
// tolerates for the proven lane), or (b) a bindingless statement-level expression (`await
// import(...);` / `import(...);`, same wrapping tolerance) used purely for side effects (39 green
// tests — `browser_template_literal_dynamic_import_harness` + `runtime_smoke dynamic_import` —
// depend on this staying untouched). Deliberately NOT gated on whether the declarator's binding
// ever actually EARNS provenance (const vs. let/var, foldable vs. not, shadowed vs. not) — that
// question, and the "unused is harmless" exemption for it, are already correctly owned by
// `collect_namespace_provenance` / `deny_unproven_namespace_binding_candidates` above; this walk's
// only job is to catch the positions NEITHER of those ever looks at, so it runs UNCONDITIONALLY
// (no usage check) for every position outside the allowlist. Every other position — an
// assignment's right-hand side, a sequence-expression element, an inline member access on the
// import's own result, a member/property-assignment sink, a call argument, a return value, an
// array/object element, an arrow-function body, or anywhere else an `Expression` can appear — is
// denied (E5506) the instant an `ImportExpression` is found there, regardless of whether its
// value is ever read afterward: unlike a namespace BINDING (which is harmless if truly unused),
// these positions have no binding at all to check for use — the import expression's value is
// already being produced and handed somewhere the instant it's evaluated.
//
// Mirrors the exhaustive, no-`_=>`-arm traversal shape of `walk_*`/`rewrite_*`/`census_*`/
// `signal_*` above (same node coverage on both `Statement` and `Expression`), threading one extra
// `bool` (`allowed_root`) instead of a callback: `true` at exactly the two allowlisted root
// positions (a `VariableDeclarator.init`, an `ExpressionStatement.expression`), preserved across a
// `ParenthesizedExpression`/`AwaitExpression` unwrap (mirroring `as_await_import_source`'s own
// tolerance), and reset to `false` for every other child position — including the `.source` field
// of an `ImportExpression` itself, and the body of a `FunctionExpression`/`ArrowFunctionExpression`
// (so `async () => await import(...)`'s body is a denied position too, since binding that ARROW,
// not its eventual import result, is what a declarator init would actually capture there).
fn deny_import_expressions_outside_allowlist(
    statements: &[Statement],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for statement in statements {
        deny_import_positions_statement(statement, diagnostics);
    }
}

fn import_expression_outside_allowlist_error() -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        "`import(...)` is only supported as the init of a `const`/`let`/`var` declarator (`const ns \
         = await import(<foldable specifier>)`, the proven-linkable lane) or as a bindingless \
         statement (`await import(\"./x.js\");`, for side effects only) — every other position (an \
         assignment's right-hand side, a sequence-expression element, an inline member access on \
         the import's own result, a member/property-assignment sink, a call argument, a return \
         value, an array/object element, an arrow-function body, or anywhere else) would let the \
         raw import/namespace value escape into that position and is unavailable in the current \
         direct-runtime path"
            .to_string(),
    )
}

fn deny_import_positions_block(block: &BlockStatement, diagnostics: &mut Vec<Diagnostic>) {
    for statement in &block.body {
        deny_import_positions_statement(statement, diagnostics);
    }
}

fn deny_import_positions_var_decl(decl: &VariableDeclaration, diagnostics: &mut Vec<Diagnostic>) {
    // `decl.kind` (var/let/const) is deliberately NOT checked here — the allowlist covers a
    // declarator init of ANY kind; whether it goes on to actually earn provenance is a separate
    // concern the pre-existing pipeline already owns (see the section doc comment above).
    for declarator in &decl.declarations {
        if let Some(init) = &declarator.init {
            deny_import_positions_expression(init, true, diagnostics);
        }
    }
}

fn deny_import_positions_class_body(body: &ClassBody, diagnostics: &mut Vec<Diagnostic>) {
    for method in &body.methods {
        if let Some(method_body) = &method.body {
            deny_import_positions_block(method_body, diagnostics);
        }
    }
}

fn deny_import_positions_statement(statement: &Statement, diagnostics: &mut Vec<Diagnostic>) {
    match statement {
        // The ONLY other allowlisted root: a bindingless statement-level
        // expression — `await import(...);` / `import(...);` for side
        // effects.
        Statement::ExpressionStatement(stmt) => {
            deny_import_positions_expression(&stmt.expression, true, diagnostics)
        }
        // `label` is a control-flow target name, never an `Expression`.
        Statement::BreakStatement(_) => {}
        Statement::ContinueStatement(_) => {}
        Statement::WithStatement(stmt) => {
            deny_import_positions_expression(&stmt.object, false, diagnostics);
            deny_import_positions_statement(&stmt.body, diagnostics);
        }
        Statement::ReturnStatement(stmt) => {
            if let Some(argument) = &stmt.argument {
                // A return-value escape — never a bindingless statement or
                // a declarator init.
                deny_import_positions_expression(argument, false, diagnostics);
            }
        }
        Statement::LabeledStatement(stmt) => {
            deny_import_positions_statement(&stmt.body, diagnostics)
        }
        Statement::IfStatement(stmt) => {
            deny_import_positions_expression(&stmt.test, false, diagnostics);
            deny_import_positions_block(&stmt.consequent, diagnostics);
            if let Some(alternate) = &stmt.alternate {
                deny_import_positions_block(alternate, diagnostics);
            }
        }
        Statement::SwitchStatement(stmt) => {
            deny_import_positions_expression(&stmt.discriminant, false, diagnostics);
            for case in &stmt.cases {
                if let Some(test) = &case.test {
                    deny_import_positions_expression(test, false, diagnostics);
                }
                for consequent in &case.consequent {
                    deny_import_positions_statement(consequent, diagnostics);
                }
            }
        }
        Statement::ThrowStatement(stmt) => {
            deny_import_positions_expression(&stmt.argument, false, diagnostics)
        }
        Statement::TryStatement(stmt) => {
            deny_import_positions_block(&stmt.block, diagnostics);
            if let Some(handler) = &stmt.handler {
                deny_import_positions_block(&handler.body, diagnostics);
            }
            if let Some(finalizer) = &stmt.finalizer {
                deny_import_positions_block(finalizer, diagnostics);
            }
        }
        // No fields at all.
        Statement::DebuggerStatement(_) => {}
        Statement::BlockStatement(stmt) => deny_import_positions_block(stmt, diagnostics),
        Statement::ForStatement(stmt) => {
            match &stmt.init {
                Some(ForInit::VariableDeclaration(decl)) => {
                    deny_import_positions_var_decl(decl, diagnostics)
                }
                Some(ForInit::Expression(expr)) => {
                    deny_import_positions_expression(expr, false, diagnostics)
                }
                None => {}
            }
            if let Some(test) = &stmt.test {
                deny_import_positions_expression(test, false, diagnostics);
            }
            if let Some(update) = &stmt.update {
                deny_import_positions_expression(update, false, diagnostics);
            }
            deny_import_positions_block(&stmt.body, diagnostics);
        }
        Statement::ForInStatement(stmt) => {
            match &stmt.left {
                ForInLefthand::VariableDeclaration(decl) => {
                    deny_import_positions_var_decl(decl, diagnostics)
                }
                ForInLefthand::Expression(expr) => {
                    deny_import_positions_expression(expr, false, diagnostics)
                }
            }
            deny_import_positions_expression(&stmt.right, false, diagnostics);
            deny_import_positions_statement(&stmt.body, diagnostics);
        }
        Statement::ForOfStatement(stmt) => {
            match &stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => {
                    deny_import_positions_var_decl(decl, diagnostics)
                }
                ForOfLefthand::Expression(expr) => {
                    deny_import_positions_expression(expr, false, diagnostics)
                }
            }
            deny_import_positions_expression(&stmt.right, false, diagnostics);
            // `is_await` carries no `Expression` content.
            deny_import_positions_statement(&stmt.body, diagnostics);
        }
        Statement::WhileStatement(stmt) => {
            deny_import_positions_expression(&stmt.test, false, diagnostics);
            deny_import_positions_block(&stmt.body, diagnostics);
        }
        Statement::DoWhileStatement(stmt) => {
            deny_import_positions_block(&stmt.body, diagnostics);
            deny_import_positions_expression(&stmt.test, false, diagnostics);
        }
        // `function.name`/`function.params` carry no `Expression` content.
        Statement::FunctionDeclaration(function) => {
            deny_import_positions_block(&function.body, diagnostics)
        }
        Statement::ClassDeclaration(class) => {
            deny_import_positions_class_body(&class.body, diagnostics)
        }
        Statement::VariableDeclaration(decl) => deny_import_positions_var_decl(decl, diagnostics),
        // `ImportDeclaration`/`ExportAllDeclaration`/`ExportNamedDeclaration` are the STATIC
        // import/export syntax — every field is a `String`/specifier-name, never an
        // `Expression::ImportExpression`-bearing node (same citation as `rewrite_statement`'s
        // equivalent arms above).
        Statement::ImportDeclaration(_) => {}
        Statement::ExportAll(_) => {}
        Statement::ExportNamed(_) => {}
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => {
                deny_import_positions_expression(expr, false, diagnostics)
            }
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                deny_import_positions_block(&function.body, diagnostics)
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                deny_import_positions_class_body(&class.body, diagnostics)
            }
        },
        Statement::EnumDeclaration(decl) => {
            for member in &decl.members {
                if let Some(value) = &member.value {
                    deny_import_positions_expression(value, false, diagnostics);
                }
            }
        }
        // TypeScript type syntax only — no `Expression` content to walk.
        Statement::TypeAliasDeclaration(_) => {}
        Statement::InterfaceDeclaration(_) => {}
    }
}

/// A self-contained (no whole-file `consts`/`bound_counts` context available at this call site)
/// SUBSET of `fold_import_specifier`'s literal-oriented folds: a string literal, optionally
/// `ParenthesizedExpression`-wrapped (via `unwrap_parens`), `SequenceExpression`-tailed (JS comma
/// operator — last element only), or `+`-concatenated with another such foldable operand.
/// Deliberately excludes `fold_import_specifier`'s `Identifier` const-lookup and
/// `Object.freeze(...)` arms — both require the scope-aware `consts`/`bound_counts` maps this
/// syntax-only deny walk never builds — but every fail-open shape this gate exists to close (see
/// the section doc comment above) uses a bare string literal directly, so this narrower check is
/// sufficient to close them without also having to thread that context through the entire
/// `deny_import_positions_*` traversal.
///
/// Gating the bare-`ImportExpression` deny on this (instead of denying unconditionally, the way
/// the `AwaitExpression` arm already does) is what keeps the pre-existing `non-literal dynamic
/// import()` resolver diagnostic (`kali_types::resolve::expression::resolve_import_expression`)
/// intact for a genuinely non-literal specifier reached from a non-allowlisted position — e.g.
/// `return import(specifier)` inside an arrow body, where `specifier` is an untyped local (see
/// `browser_non_literal_dynamic_import_harness_jsx_tsx.rs`, which this gate must leave unchanged).
fn is_foldable_import_specifier(expr: &Expression) -> bool {
    match unwrap_parens(expr) {
        Expression::Literal(LiteralValue::String(_)) => true,
        Expression::SequenceExpression(seq) => seq
            .expressions
            .last()
            .map(is_foldable_import_specifier)
            .unwrap_or(false),
        Expression::BinaryExpression(binary) if binary.operator == "+" => {
            is_foldable_import_specifier(&binary.left)
                && is_foldable_import_specifier(&binary.right)
        }
        _ => false,
    }
}

fn deny_import_positions_expression_or_spread(
    element: &ExpressionOrSpread,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match element {
        ExpressionOrSpread::Expression(expr) => {
            deny_import_positions_expression(expr, false, diagnostics)
        }
        ExpressionOrSpread::Spread(spread) => {
            deny_import_positions_expression(&spread.argument, false, diagnostics)
        }
        ExpressionOrSpread::Empty => {}
    }
}

/// `allowed_root` is `true` only when `expr` sits at one of the two
/// allowlisted positions — see the section doc comment above for the exact
/// rules (unwrap tolerance, why usage doesn't gate this walk, why an arrow
/// body is always denied). Every recursive call below passes `false` except
/// the `ParenthesizedExpression` unwrap arm, which propagates whatever
/// `allowed_root` it was called with unchanged.
///
/// This walk denies two shapes at any non-allowlisted position: the compound
/// `await import(<expr>)` (mod `ParenthesizedExpression` wrapping around
/// either layer — exactly what `as_await_import_source` recognizes for the
/// proven lane elsewhere), AND, as of the fix for the third review round's
/// finding, a BARE, non-awaited `import(<expr>)` whose specifier is
/// FOLDABLE (see `is_foldable_import_specifier`). The bare shape used to be
/// claimed exempt here on the theory that "none of the fail-open probes
/// ever omit `await`, and a bare `import(x).member` would already throw in
/// real JS" — that reasoning covered only a DIRECT member access on the
/// import expression itself; it never accounted for `await` applied to a
/// separately-bound identifier (`const p = import(...); ...; await p`),
/// where `await` never syntactically wraps the `ImportExpression` at all.
/// That gap let a bare `import()` laundered through a binding reach a
/// member access completely undetected (probe-proven: `const p =
/// import("./util.js"); const c = await p; c.greet()` and five siblings —
/// see `module_namespace_link.rs`'s bare-import test block). The foldability
/// gate exists only to avoid pre-empting the separate, pre-existing
/// non-literal-specifier diagnostic (`kali_types::resolve::expression::
/// resolve_import_expression`) that a genuinely non-literal bare `import()`
/// at a non-allowlisted position (e.g. `return import(specifier)`) must
/// still produce unchanged — see
/// `browser_non_literal_dynamic_import_harness_jsx_tsx.rs`.
fn deny_import_positions_expression(
    expr: &Expression,
    allowed_root: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match expr {
        Expression::ParenthesizedExpression(inner) => {
            deny_import_positions_expression(&inner.expression, allowed_root, diagnostics)
        }
        Expression::AwaitExpression(await_expr) => match unwrap_parens(&await_expr.argument) {
            Expression::ImportExpression(import_expr) => {
                if !allowed_root {
                    diagnostics.push(import_expression_outside_allowlist_error());
                }
                // The specifier itself is never an allowed root regardless
                // of this node's own position — defense in depth for a
                // pathological nested `import(import(...))`.
                deny_import_positions_expression(&import_expr.source, false, diagnostics);
            }
            // `await <anything else>` — not a dynamic import at all;
            // `allowed_root` never applies past a plain `await`.
            _ => deny_import_positions_expression(&await_expr.argument, false, diagnostics),
        },
        // A BARE (non-awaited) `import(...)`. Denied at a non-allowlisted position when its
        // specifier is foldable — see the doc comment above and `is_foldable_import_specifier`.
        // A non-foldable specifier here is left alone so the pre-existing resolver diagnostic
        // (`resolve_import_expression`'s "non-literal dynamic import()") still fires unpre-empted.
        Expression::ImportExpression(import_expr) => {
            if !allowed_root && is_foldable_import_specifier(&import_expr.source) {
                diagnostics.push(import_expression_outside_allowlist_error());
            }
            deny_import_positions_expression(&import_expr.source, false, diagnostics);
        }
        // A bare identifier carries no further `Expression` content.
        Expression::Identifier(_) => {}
        Expression::Literal(_) => {}
        Expression::BinaryExpression(binary) => {
            deny_import_positions_expression(&binary.left, false, diagnostics);
            deny_import_positions_expression(&binary.right, false, diagnostics);
        }
        Expression::UnaryExpression(unary) => {
            deny_import_positions_expression(&unary.argument, false, diagnostics)
        }
        Expression::CallExpression(call) => {
            deny_import_positions_expression(&call.callee, false, diagnostics);
            for arg in &call.args {
                deny_import_positions_expression(arg, false, diagnostics);
            }
        }
        Expression::MemberExpression(member) => {
            deny_import_positions_expression(&member.object, false, diagnostics);
            if let Some(index) = &member.computed_index {
                deny_import_positions_expression(index, false, diagnostics);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter().flatten() {
                deny_import_positions_expression_or_spread(element, diagnostics);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                deny_import_positions_expression(&property.value, false, diagnostics);
            }
        }
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                deny_import_positions_block(body, diagnostics);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            deny_import_positions_expression(&arrow.body, false, diagnostics)
        }
        Expression::ClassExpression(class) => {
            deny_import_positions_class_body(&class.body, diagnostics)
        }
        Expression::NewExpression(new_expr) => {
            deny_import_positions_expression(&new_expr.callee, false, diagnostics);
            for arg in &new_expr.args {
                deny_import_positions_expression(arg, false, diagnostics);
            }
        }
        // `meta`/`property` are fixed keyword strings (e.g. `import.meta`),
        // never identifier lookups.
        Expression::MetaProperty(_) => {}
        Expression::TemplateLiteral(template) => {
            for expression in &template.expressions {
                deny_import_positions_expression(expression, false, diagnostics);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            deny_import_positions_expression(&tagged.tag, false, diagnostics);
            for expression in &tagged.template.expressions {
                deny_import_positions_expression(expression, false, diagnostics);
            }
        }
        Expression::UpdateExpression(update) => {
            deny_import_positions_expression(&update.argument, false, diagnostics)
        }
        Expression::AssignmentExpression(assignment) => {
            deny_import_positions_expression(&assignment.left, false, diagnostics);
            deny_import_positions_expression(&assignment.right, false, diagnostics);
        }
        Expression::LogicalExpression(logical) => {
            deny_import_positions_expression(&logical.left, false, diagnostics);
            deny_import_positions_expression(&logical.right, false, diagnostics);
        }
        Expression::ConditionalExpression(conditional) => {
            deny_import_positions_expression(&conditional.test, false, diagnostics);
            deny_import_positions_expression(&conditional.consequent, false, diagnostics);
            deny_import_positions_expression(&conditional.alternate, false, diagnostics);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &sequence.expressions {
                deny_import_positions_expression(expression, false, diagnostics);
            }
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &yield_expr.argument {
                deny_import_positions_expression(argument, false, diagnostics);
            }
        }
        Expression::OptionalChainExpression(chain) => match chain.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => {
                deny_import_positions_expression(object, false, diagnostics)
            }
        },
        Expression::ChainExpression(chain) => {
            deny_import_positions_expression(&chain.expression, false, diagnostics)
        }
        Expression::SpreadElement(spread) => {
            deny_import_positions_expression(&spread.argument, false, diagnostics)
        }
        Expression::RestElement(rest) => {
            deny_import_positions_expression(&rest.argument, false, diagnostics)
        }
        Expression::DecoratedExpression(decorated) => {
            deny_import_positions_expression(&decorated.expression, false, diagnostics)
        }
        Expression::JsxElement(element) => deny_import_positions_jsx_element(element, diagnostics),
        Expression::JsxFragment(fragment) => {
            deny_import_positions_jsx_fragment(fragment, diagnostics)
        }
        // No content at all.
        Expression::JsxEmptyExpression => {}
        Expression::TypeAssertion(assertion) => {
            // `assertion.type_name` is TypeScript type syntax, not a value reference.
            deny_import_positions_expression(&assertion.expression, false, diagnostics)
        }
        Expression::SatisfiesExpression(satisfies) => {
            // `satisfies.type_name` is TypeScript type syntax, not a value reference.
            deny_import_positions_expression(&satisfies.expression, false, diagnostics)
        }
        // `this`/`super` are keywords, never identifier lookups.
        Expression::ThisExpression => {}
        Expression::SuperExpression => {}
        // A private class-field name (`#foo`) carries no `Expression` content.
        Expression::PrivateIdentifier(_) => {}
        // A literal numeric value, not a reference.
        Expression::BigIntLiteral(_) => {}
    }
}

fn deny_import_positions_jsx_element(element: &JsxElement, diagnostics: &mut Vec<Diagnostic>) {
    for attribute in &element.opening_element.attributes {
        deny_import_positions_jsx_attribute_item(attribute, diagnostics);
    }
    for child in &element.children {
        deny_import_positions_jsx_child(child, diagnostics);
    }
}

fn deny_import_positions_jsx_fragment(fragment: &JsxFragment, diagnostics: &mut Vec<Diagnostic>) {
    for child in &fragment.children {
        deny_import_positions_jsx_child(child, diagnostics);
    }
}

fn deny_import_positions_jsx_attribute_item(
    item: &JsxAttributeItem,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            deny_import_positions_jsx_attribute_value(&attribute.value, diagnostics)
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            deny_import_positions_expression(&spread.argument, false, diagnostics)
        }
    }
}

fn deny_import_positions_jsx_attribute_value(
    value: &JsxAttributeValue,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match value {
        JsxAttributeValue::String(_) => {}
        JsxAttributeValue::JsxElement(element) => {
            deny_import_positions_jsx_element(element, diagnostics)
        }
        JsxAttributeValue::JsxExpression(container) => {
            deny_import_positions_jsx_expression_container(container, diagnostics)
        }
    }
}

fn deny_import_positions_jsx_expression_container(
    container: &JsxExpressionContainer,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(expression) = &container.expression {
        deny_import_positions_expression(expression, false, diagnostics);
    }
}

fn deny_import_positions_jsx_child(child: &JsxChild, diagnostics: &mut Vec<Diagnostic>) {
    match child {
        JsxChild::JsxText(_) => {}
        JsxChild::JsxExpression(container) => {
            deny_import_positions_jsx_expression_container(container, diagnostics)
        }
        JsxChild::JsxElement(element) => deny_import_positions_jsx_element(element, diagnostics),
        JsxChild::JsxFragment(fragment) => {
            deny_import_positions_jsx_fragment(fragment, diagnostics)
        }
    }
}

// ---- default-deny leftovers + shadowing guard + pipeline entry point (Task 7) ----
//
// Two concerns close the value-leak class left open by Tasks 3-6:
//
//   1. `deny_unrewritten_uses` walks the POST-append, POST-rewrite AST for
//      any `Expression::Identifier` (or JSX-name / re-export reference)
//      whose name is a provenance-proven namespace binding. Tasks 5/6 only
//      ever REPLACE or REJECT specific `ns.member` shapes (a call callee, a
//      `typeof` argument); nothing before this task ever looked at a BARE
//      reference to the binding itself (`console.log(chunk)`, `chunk +
//      ''`, `const alias = chunk`, `f(chunk)`, `return chunk`, or even a
//      bare non-call `ns.member` property read) — every one of those
//      silently fell through to the resolver, which today just treats
//      `chunk` as an ordinary local bound to whatever the specifier-fold
//      literal was, printing the raw specifier string. This walk closes
//      that: every SUCH identifier is denied, full stop — including one
//      already sitting inside a `MemberExpression` Task 6 rejected for its
//      own reason (a non-export call, a computed access): this pass makes
//      no attempt to distinguish "already-diagnosed leftover" from
//      "never-considered leftover", it default-denies EVERY leftover.
//
//   2. The shadowing guard closes a carried-over Task 6 gap: Task 6's
//      rewrite walk has NO lexical-scope awareness for the `ns` NAME
//      itself (see `rewrite_expression`'s `MemberExpression` arm — it
//      matches `Identifier(ns)` purely by STRING equality against
//      `provenance.bindings`, the same way Task 5's sibling-rename walk
//      matches purely by string equality against `renames`). A local
//      binding that reuses the SAME name as a namespace binding
//      (`const chunk = await import(...); { const chunk = 5; }`) would
//      make `chunk`'s rewrite silently ambiguous — a use of the INNER
//      `chunk` could get folded/rewritten as if it were still the OUTER
//      namespace binding. This mirrors Task 5's `check_binding` shadowing
//      guard (`append_linked_functions`'s sibling-rename walk, 12 binding
//      positions) but keyed by PROVENANCE NAME instead of a linked-module
//      rename map, and — since the ambiguity is about the ENTRY module's
//      OWN lexical scoping, not a cloned linked-function body — it runs
//      over the pristine entry `statements` BEFORE `append_linked_functions`
//      adds any clones, so a linked module's own unrelated internal locals
//      can never produce a false positive here.
//
// Both concerns share one exhaustive, read-only, non-fallible traversal
// (`census_statements`/`census_block`/`census_statement`/`census_expression`/
// the `census_jsx_*` family below) that mirrors the EXACT node coverage of
// the sibling-rename walk (Task 5) and the rewrite walk (Task 6) — same
// shape, no `_ =>` arm on either `Statement` or `Expression` — but keyed by
// two callbacks instead of a rename map:
//   - `on_identifier`: fired for every `Expression::Identifier` VALUE
//     reference, AND for the two other value-level-by-string-name
//     positions Task 5's own walk already treats as references rather than
//     bindings (`JsxName::Identifier` — see `check_bare_reference` above —
//     and `ExportSpecifier::local`, which re-exports an EXISTING outer
//     binding's current value, exactly like a bare `Identifier` read).
//   - `on_binding`: fired for every binding-INTRODUCING name at the exact
//     same positions Task 5's `check_binding` covers (declarator ids —
//     which also covers `for`/`for-in`/`for-of` variable-declaration
//     lefthands — function/class names, function/arrow/class-method/catch
//     params), PLUS every `ImportDeclaration` specifier's local name (a
//     position Task 5 never needed to cover, since a nested import inside
//     a CLONED linked-function body is rejected outright there — but here,
//     in the ENTRY module, an ordinary — if rare — second import statement
//     naming the same local as a provenance binding is exactly the shadow
//     case `on_binding` exists to catch).

fn census_import_specifier(specifier: &ImportSpecifier, on_binding: &mut dyn FnMut(&str)) {
    match specifier {
        ImportSpecifier::Default(local) => on_binding(local),
        ImportSpecifier::Named(specifiers) => {
            for spec in specifiers {
                on_binding(&spec.local);
            }
        }
        ImportSpecifier::Namespace(local) => on_binding(local),
        ImportSpecifier::Type(specifiers) => {
            for spec in specifiers {
                on_binding(&spec.local);
            }
        }
        // No local binding at all: `import "mod";`.
        ImportSpecifier::SideEffect => {}
    }
}

fn census_statements(
    statements: &[Statement],
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    for statement in statements {
        census_statement(statement, on_identifier, on_binding);
    }
}

fn census_block(
    block: &BlockStatement,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    for statement in &block.body {
        census_statement(statement, on_identifier, on_binding);
    }
}

fn census_var_decl(
    decl: &VariableDeclaration,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    // `decl.kind` carries no name at all.
    for declarator in &decl.declarations {
        on_binding(&declarator.id);
        if let Some(init) = &declarator.init {
            census_expression(init, on_identifier, on_binding);
        }
    }
}

fn census_class_body(
    body: &ClassBody,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    for method in &body.methods {
        // `method.name` is a property key, never a value-level identifier
        // lookup.
        for param in &method.params {
            on_binding(param);
        }
        if let Some(method_body) = &method.body {
            census_block(method_body, on_identifier, on_binding);
        }
    }
}

fn census_statement(
    statement: &Statement,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    match statement {
        Statement::ExpressionStatement(stmt) => {
            census_expression(&stmt.expression, on_identifier, on_binding)
        }
        // `label` is a control-flow target name, never a value lookup.
        Statement::BreakStatement(_) => {}
        Statement::ContinueStatement(_) => {}
        Statement::WithStatement(stmt) => {
            census_expression(&stmt.object, on_identifier, on_binding);
            census_statement(&stmt.body, on_identifier, on_binding);
        }
        Statement::ReturnStatement(stmt) => {
            if let Some(argument) = &stmt.argument {
                census_expression(argument, on_identifier, on_binding);
            }
        }
        Statement::LabeledStatement(stmt) => {
            census_statement(&stmt.body, on_identifier, on_binding)
        }
        Statement::IfStatement(stmt) => {
            census_expression(&stmt.test, on_identifier, on_binding);
            census_block(&stmt.consequent, on_identifier, on_binding);
            if let Some(alternate) = &stmt.alternate {
                census_block(alternate, on_identifier, on_binding);
            }
        }
        Statement::SwitchStatement(stmt) => {
            census_expression(&stmt.discriminant, on_identifier, on_binding);
            for case in &stmt.cases {
                if let Some(test) = &case.test {
                    census_expression(test, on_identifier, on_binding);
                }
                for consequent in &case.consequent {
                    census_statement(consequent, on_identifier, on_binding);
                }
            }
        }
        Statement::ThrowStatement(stmt) => {
            census_expression(&stmt.argument, on_identifier, on_binding)
        }
        Statement::TryStatement(stmt) => {
            census_block(&stmt.block, on_identifier, on_binding);
            if let Some(handler) = &stmt.handler {
                on_binding(&handler.param);
                census_block(&handler.body, on_identifier, on_binding);
            }
            if let Some(finalizer) = &stmt.finalizer {
                census_block(finalizer, on_identifier, on_binding);
            }
        }
        // No fields at all.
        Statement::DebuggerStatement(_) => {}
        Statement::BlockStatement(stmt) => census_block(stmt, on_identifier, on_binding),
        Statement::ForStatement(stmt) => {
            match &stmt.init {
                Some(ForInit::VariableDeclaration(decl)) => {
                    census_var_decl(decl, on_identifier, on_binding)
                }
                Some(ForInit::Expression(expr)) => {
                    census_expression(expr, on_identifier, on_binding)
                }
                None => {}
            }
            if let Some(test) = &stmt.test {
                census_expression(test, on_identifier, on_binding);
            }
            if let Some(update) = &stmt.update {
                census_expression(update, on_identifier, on_binding);
            }
            census_block(&stmt.body, on_identifier, on_binding);
        }
        Statement::ForInStatement(stmt) => {
            match &stmt.left {
                ForInLefthand::VariableDeclaration(decl) => {
                    census_var_decl(decl, on_identifier, on_binding)
                }
                ForInLefthand::Expression(expr) => {
                    census_expression(expr, on_identifier, on_binding)
                }
            }
            census_expression(&stmt.right, on_identifier, on_binding);
            census_statement(&stmt.body, on_identifier, on_binding);
        }
        Statement::ForOfStatement(stmt) => {
            match &stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => {
                    census_var_decl(decl, on_identifier, on_binding)
                }
                ForOfLefthand::Expression(expr) => {
                    census_expression(expr, on_identifier, on_binding)
                }
            }
            census_expression(&stmt.right, on_identifier, on_binding);
            // `is_await` carries no name.
            census_statement(&stmt.body, on_identifier, on_binding);
        }
        Statement::WhileStatement(stmt) => {
            census_expression(&stmt.test, on_identifier, on_binding);
            census_block(&stmt.body, on_identifier, on_binding);
        }
        Statement::DoWhileStatement(stmt) => {
            census_block(&stmt.body, on_identifier, on_binding);
            census_expression(&stmt.test, on_identifier, on_binding);
        }
        Statement::FunctionDeclaration(function) => {
            on_binding(&function.name);
            for param in &function.params {
                on_binding(param);
            }
            census_block(&function.body, on_identifier, on_binding);
        }
        Statement::ClassDeclaration(class) => {
            on_binding(&class.name);
            census_class_body(&class.body, on_identifier, on_binding);
        }
        Statement::VariableDeclaration(decl) => census_var_decl(decl, on_identifier, on_binding),
        Statement::ImportDeclaration(decl) => {
            for specifier in &decl.specifiers {
                census_import_specifier(specifier, on_binding);
            }
        }
        // `ExportAllDeclaration { source: String }` — no identifier-bearing
        // field of any kind (same citation as `rewrite_statement`'s
        // `Statement::ExportAll` arm above).
        Statement::ExportAll(_) => {}
        Statement::ExportNamed(export) => {
            for specifier in &export.specifiers {
                // `ExportSpecifier { local, exported }` — `local` names an
                // EXISTING outer binding being re-exported (`export {
                // chunk }` reads `chunk`'s current value), a value-level
                // reference exactly like a bare `Identifier`; `exported` is
                // only the re-exported PUBLIC name, never resolved locally.
                on_identifier(&specifier.local);
            }
        }
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => {
                census_expression(expr, on_identifier, on_binding)
            }
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                on_binding(&function.name);
                for param in &function.params {
                    on_binding(param);
                }
                census_block(&function.body, on_identifier, on_binding);
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                on_binding(&class.name);
                census_class_body(&class.body, on_identifier, on_binding);
            }
        },
        Statement::EnumDeclaration(decl) => {
            on_binding(&decl.name);
            for member in &decl.members {
                // `member.name` is a declaration, not a reference.
                if let Some(value) = &member.value {
                    census_expression(value, on_identifier, on_binding);
                }
            }
        }
        // TypeScript type syntax only — no identifier-bearing `Expression`
        // content.
        Statement::TypeAliasDeclaration(_) => {}
        Statement::InterfaceDeclaration(_) => {}
    }
}

fn census_expression_or_spread(
    element: &ExpressionOrSpread,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    match element {
        ExpressionOrSpread::Expression(expr) => census_expression(expr, on_identifier, on_binding),
        ExpressionOrSpread::Spread(spread) => {
            census_expression(&spread.argument, on_identifier, on_binding)
        }
        ExpressionOrSpread::Empty => {}
    }
}

fn census_expression(
    expr: &Expression,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    match expr {
        Expression::Identifier(name) => on_identifier(name),
        // A literal value carries no identifier reference.
        Expression::Literal(_) => {}
        Expression::BinaryExpression(binary) => {
            census_expression(&binary.left, on_identifier, on_binding);
            census_expression(&binary.right, on_identifier, on_binding);
        }
        Expression::UnaryExpression(unary) => {
            census_expression(&unary.argument, on_identifier, on_binding)
        }
        Expression::CallExpression(call) => {
            census_expression(&call.callee, on_identifier, on_binding);
            for arg in &call.args {
                census_expression(arg, on_identifier, on_binding);
            }
        }
        Expression::MemberExpression(member) => {
            // `member.property` is a static field-name string, never a
            // reference. `member.object` IS a value-level reference — a
            // bare `ns.member`/`chunk.member` read is exactly the kind of
            // leftover leak this pass exists to close, so it is walked
            // exactly like any other expression, with no special-casing.
            census_expression(&member.object, on_identifier, on_binding);
            if let Some(index) = &member.computed_index {
                census_expression(index, on_identifier, on_binding);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter().flatten() {
                census_expression_or_spread(element, on_identifier, on_binding);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                // `property.key` is a static property name, never a
                // reference.
                census_expression(&property.value, on_identifier, on_binding);
            }
        }
        Expression::FunctionExpression(function) => {
            if let Some(id) = &function.id {
                on_binding(id);
            }
            for param in &function.params {
                on_binding(&param.name);
            }
            if let Some(body) = &function.body {
                census_block(body, on_identifier, on_binding);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            for param in &arrow.params {
                on_binding(&param.name);
            }
            census_expression(&arrow.body, on_identifier, on_binding);
        }
        Expression::ClassExpression(class) => {
            if let Some(id) = &class.id {
                on_binding(id);
            }
            census_class_body(&class.body, on_identifier, on_binding);
        }
        Expression::NewExpression(new_expr) => {
            census_expression(&new_expr.callee, on_identifier, on_binding);
            for arg in &new_expr.args {
                census_expression(arg, on_identifier, on_binding);
            }
        }
        // `meta`/`property` are fixed keyword strings (e.g. `import.meta`),
        // never identifier lookups.
        Expression::MetaProperty(_) => {}
        Expression::TemplateLiteral(template) => {
            // `template.quasis` are literal string chunks, not references.
            for expression in &template.expressions {
                census_expression(expression, on_identifier, on_binding);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            census_expression(&tagged.tag, on_identifier, on_binding);
            for expression in &tagged.template.expressions {
                census_expression(expression, on_identifier, on_binding);
            }
        }
        Expression::UpdateExpression(update) => {
            census_expression(&update.argument, on_identifier, on_binding)
        }
        Expression::AssignmentExpression(assignment) => {
            census_expression(&assignment.left, on_identifier, on_binding);
            census_expression(&assignment.right, on_identifier, on_binding);
        }
        Expression::LogicalExpression(logical) => {
            census_expression(&logical.left, on_identifier, on_binding);
            census_expression(&logical.right, on_identifier, on_binding);
        }
        Expression::ConditionalExpression(conditional) => {
            census_expression(&conditional.test, on_identifier, on_binding);
            census_expression(&conditional.consequent, on_identifier, on_binding);
            census_expression(&conditional.alternate, on_identifier, on_binding);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &sequence.expressions {
                census_expression(expression, on_identifier, on_binding);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            census_expression(&parenthesized.expression, on_identifier, on_binding)
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &yield_expr.argument {
                census_expression(argument, on_identifier, on_binding);
            }
        }
        Expression::AwaitExpression(await_expr) => {
            census_expression(&await_expr.argument, on_identifier, on_binding)
        }
        Expression::OptionalChainExpression(chain) => match chain.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => {
                census_expression(object, on_identifier, on_binding)
            }
        },
        Expression::ChainExpression(chain) => {
            census_expression(&chain.expression, on_identifier, on_binding)
        }
        Expression::SpreadElement(spread) => {
            census_expression(&spread.argument, on_identifier, on_binding)
        }
        Expression::RestElement(rest) => {
            census_expression(&rest.argument, on_identifier, on_binding)
        }
        Expression::ImportExpression(import_expr) => {
            census_expression(&import_expr.source, on_identifier, on_binding)
        }
        Expression::DecoratedExpression(decorated) => {
            census_expression(&decorated.expression, on_identifier, on_binding)
        }
        Expression::JsxElement(element) => census_jsx_element(element, on_identifier, on_binding),
        Expression::JsxFragment(fragment) => {
            census_jsx_fragment(fragment, on_identifier, on_binding)
        }
        // No content at all.
        Expression::JsxEmptyExpression => {}
        Expression::TypeAssertion(assertion) => {
            // `assertion.type_name` is TypeScript type syntax, not a value reference.
            census_expression(&assertion.expression, on_identifier, on_binding)
        }
        Expression::SatisfiesExpression(satisfies) => {
            // `satisfies.type_name` is TypeScript type syntax, not a value reference.
            census_expression(&satisfies.expression, on_identifier, on_binding)
        }
        // `this`/`super` are keywords, never identifier lookups.
        Expression::ThisExpression => {}
        Expression::SuperExpression => {}
        // A private class-field name (`#foo`) is a distinct namespace from
        // top-level bindings — cannot alias a namespace binding.
        Expression::PrivateIdentifier(_) => {}
        // A literal numeric value, not a reference.
        Expression::BigIntLiteral(_) => {}
    }
}

fn census_jsx_element(
    element: &JsxElement,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    census_jsx_name(&element.opening_element.name, on_identifier);
    for attribute in &element.opening_element.attributes {
        census_jsx_attribute_item(attribute, on_identifier, on_binding);
    }
    for child in &element.children {
        census_jsx_child(child, on_identifier, on_binding);
    }
    if let Some(closing) = &element.closing_element {
        census_jsx_name(&closing.name, on_identifier);
    }
}

fn census_jsx_fragment(
    fragment: &JsxFragment,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    for child in &fragment.children {
        census_jsx_child(child, on_identifier, on_binding);
    }
}

/// `JsxName::Identifier` is a real value-level lookup (see
/// `walk_jsx_name`'s doc comment above) even though it is a plain `String`
/// field rather than an `Expression::Identifier` node — routed to
/// `on_identifier`, never `on_binding`.
fn census_jsx_name(name: &JsxName, on_identifier: &mut dyn FnMut(&str)) {
    match name {
        JsxName::Identifier(identifier) => on_identifier(identifier),
        JsxName::JsxClosedElement(closing) => census_jsx_name(&closing.name, on_identifier),
    }
}

fn census_jsx_attribute_item(
    item: &JsxAttributeItem,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            census_jsx_name(&attribute.name, on_identifier);
            census_jsx_attribute_value(&attribute.value, on_identifier, on_binding);
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            census_expression(&spread.argument, on_identifier, on_binding)
        }
    }
}

fn census_jsx_attribute_value(
    value: &JsxAttributeValue,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    match value {
        // A plain string literal attribute value, not a reference.
        JsxAttributeValue::String(_) => {}
        JsxAttributeValue::JsxElement(element) => {
            census_jsx_element(element, on_identifier, on_binding)
        }
        JsxAttributeValue::JsxExpression(container) => {
            census_jsx_expression_container(container, on_identifier, on_binding)
        }
    }
}

fn census_jsx_expression_container(
    container: &JsxExpressionContainer,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    if let Some(expression) = &container.expression {
        census_expression(expression, on_identifier, on_binding);
    }
}

fn census_jsx_child(
    child: &JsxChild,
    on_identifier: &mut dyn FnMut(&str),
    on_binding: &mut dyn FnMut(&str),
) {
    match child {
        // Literal text content, not a reference.
        JsxChild::JsxText(_) => {}
        JsxChild::JsxExpression(container) => {
            census_jsx_expression_container(container, on_identifier, on_binding)
        }
        JsxChild::JsxElement(element) => census_jsx_element(element, on_identifier, on_binding),
        JsxChild::JsxFragment(fragment) => census_jsx_fragment(fragment, on_identifier, on_binding),
    }
}

fn unrewritten_namespace_binding_error(name: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "namespace binding '{name}' is only usable as `{name}.member` (a proven export access, rewritten at compile time) or `typeof {name}.member`; every other value use is unavailable in the current direct-runtime path (it would leak the raw specifier/namespace value, not a real export)"
        ),
    )
}

fn shadowed_namespace_binding_error(name: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "namespace binding '{name}' is shadowed by a second binding of the same name elsewhere in this file — unsupported (this pass has no lexical-scope awareness, so it cannot tell a local rebinding of '{name}' apart from the provenance-proven namespace binding; rename the local binding to avoid the name '{name}')"
        ),
    )
}

/// Denies every remaining `Expression::Identifier` (or JSX-name /
/// re-export) reference whose name is a provenance-proven namespace
/// binding, in the POST-append, POST-rewrite AST. A successful Task 6
/// rewrite (a call-callee rename, or a `typeof` fold) always REPLACES the
/// whole node containing the binding's name, so the happy path here finds
/// none at all; a Task 6 REJECT (non-export call, computed access) leaves
/// its `MemberExpression` node untouched, so this walk still finds and
/// flags the `object` identifier inside it too — see the module-level doc
/// comment above for why that intentional double-diagnosis is acceptable.
fn deny_unrewritten_uses(
    statements: &[Statement],
    provenance: &NamespaceProvenance,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut on_identifier = |name: &str| {
        if provenance.bindings.contains_key(name) {
            diagnostics.push(unrewritten_namespace_binding_error(name));
        }
    };
    let mut on_binding = |_: &str| {};
    census_statements(statements, &mut on_identifier, &mut on_binding);
}

/// Whole-file, any-depth binding census: every name BOUND anywhere in
/// `statements` (a declarator id — plain, `for`, or `for-in`/`for-of` —
/// function/class name, function/arrow/class-method/catch param, or
/// `ImportDeclaration` specifier local), counted by how many times it is
/// bound. Shared by two different gates that both reduce to the same
/// question, "is this name provably NOT rebound/shadowed anywhere in this
/// file?":
///   - `deny_shadowed_bindings` (below): a provenance-proven NAMESPACE
///     BINDING name (`ns`/`chunk`) bound more than once anywhere is a
///     shadow — the pass has no lexical-scope awareness, so it can't tell a
///     local rebinding apart from the real namespace binding.
///   - `fold_import_specifier`/`is_object_freeze_callee` (collection time):
///     an `Identifier` this pass folds while PROVING provenance (a
///     specifier-part const, or the `Object` in `Object.freeze(...)`) must
///     be bound exactly once (a const) or zero times (the `Object` global)
///     — see `fold_import_specifier`'s doc comment for the C1 finding this
///     closes.
fn compute_binding_counts(statements: &[Statement]) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut on_identifier = |_: &str| {};
    let mut on_binding = |name: &str| {
        *counts.entry(name.to_string()).or_insert(0) += 1;
    };
    census_statements(statements, &mut on_identifier, &mut on_binding);
    counts
}

/// Rejects any provenance-proven namespace binding name that is bound a
/// SECOND time anywhere in the pristine entry `statements` (before
/// `append_linked_functions` adds any linked-module clones) — see the
/// module-level doc comment above for why this must run on the PRISTINE
/// entry AST rather than the post-append one.
fn deny_shadowed_bindings(
    statements: &[Statement],
    provenance: &NamespaceProvenance,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let counts = compute_binding_counts(statements);
    for name in provenance.bindings.keys() {
        if counts.get(name).copied().unwrap_or(0) > 1 {
            diagnostics.push(shadowed_namespace_binding_error(name));
        }
    }
}

// ---- C2 default-deny + I1 load-gating signals (final whole-branch review) ----
//
// `collect_namespace_provenance`'s reach is deliberately narrow (top-level
// statements, plus the DIRECT statement children of a top-level function
// body — see its own doc comment). Anything outside that reach — a binding
// one block deeper (`if (true) { const c = await import(...) }`), a
// `let`-bound dynamic import, or a `const` whose specifier isn't foldable
// (`await import(cond ? a : b)`) — earns NO provenance, exactly like any
// binding that was never namespace-shaped in the first place. Pre-fix, that
// made `provenance.bindings` empty and the WHOLE pipeline returned early,
// falling through to the pre-stage silent behavior (a raw specifier/
// namespace value, or `typeof` always folding to `"undefined"`).
//
// `BindingSignals` is a SEPARATE, self-contained traversal (its own
// `signal_*` family below) rather than an addition to the shared `census_*`
// family above — so this new, WIDER-reaching pass (it must find bindings
// `collect_namespace_provenance` itself never reaches) cannot regress
// `deny_shadowed_bindings`/`deny_unrewritten_uses`, which must keep seeing
// exactly the traversal they see today. It mirrors the same exhaustive,
// no-`_=>`-arm style as `walk_*`/`rewrite_*`/`census_*`.
#[derive(Default)]
struct BindingSignals {
    /// Every binding name shaped like a namespace import, ANYWHERE in the
    /// file at any nesting depth:
    ///   - `import * as <name> from "<./ or ../ source>"` — a non-relative
    ///     source (`import * as path from "node:path"`) is a separate,
    ///     pre-existing lane (see the `node_api_surface` tests) and is
    ///     NEVER a candidate here.
    ///   - a `VariableDeclarator` of ANY kind (const/let/var) whose `init`
    ///     is (modulo `ParenthesizedExpression` wrapping around either
    ///     layer) `await import(<any specifier expression>)`, whether or
    ///     not the specifier actually folds.
    ///   - a `VariableDeclarator` of ANY kind whose `init` is (modulo the
    ///     same wrapping) a BARE, non-awaited `import(<any specifier>)` —
    ///     added for the third review round's finding: `const p =
    ///     import(...)` followed by a separately-bound `await p` never
    ///     produces an `AwaitExpression`-wrapped `ImportExpression` at all
    ///     (the `await` applies to the identifier `p`, not to the import
    ///     expression), so without this arm the binding was invisible to
    ///     this census — and thus to `deny_unproven_namespace_binding_
    ///     candidates` below — even though it goes on to be used exactly
    ///     like the proven lane.
    ///
    /// A candidate with NO provenance that is also USED is the final
    /// whole-branch review's C2 finding — see
    /// `deny_unproven_namespace_binding_candidates` below.
    candidates: BTreeSet<String>,
    /// Every name appearing as the OBJECT of a member access anywhere in the
    /// file — `<name>.member`, `<name>.member(...)`, `typeof <name>.member`,
    /// or even computed `<name>[expr]` (structural only; no provenance or
    /// export check). Used to gate I1: a proven binding with NO member
    /// access anywhere has no possible use site for a linked export, so its
    /// module is never loaded (and so never purity-gated) — a namespace
    /// import nothing reads must not hard-fail a build node runs happily
    /// (final whole-branch review finding I1).
    ///
    /// Deliberately WIDER than the two shapes Task 6 actually rewrites
    /// (`typeof <ns>.member` / `<ns>.member(...)`): a bare, non-call member
    /// READ (`console.log(chunk.value)`) is not rewritable, but it IS a use
    /// of the module, and the honest diagnostic for it is the purity gate's
    /// (`module '<path>' ... contains a non-function statement`) — which
    /// only exists if the module was LOADED. Narrowing this set to just the
    /// rewritable shapes would still fail closed, but would downgrade that
    /// precise, module-naming diagnostic to the generic leftover-deny one
    /// (caught by `runtime_smoke`'s
    /// `json_run_rejects_non_function_export_dynamic_import_target_in_*_input`
    /// pins, which assert the purity-gate message by content).
    ///
    /// Being wider is also what keeps
    /// `try_fold_typeof_namespace_member`/`try_rewrite_namespace_call_callee`'s
    /// "a proven binding reaching a rewrite always has a loaded module"
    /// invariant true: both of those shapes are strict SUBSETS of this one.
    member_access_sites: BTreeSet<String>,
}

fn collect_binding_signals(statements: &[Statement]) -> BindingSignals {
    let mut signals = BindingSignals::default();
    signal_statements(statements, &mut signals);
    signals
}

fn signal_statements(statements: &[Statement], signals: &mut BindingSignals) {
    for statement in statements {
        signal_statement(statement, signals);
    }
}

fn signal_block(block: &BlockStatement, signals: &mut BindingSignals) {
    for statement in &block.body {
        signal_statement(statement, signals);
    }
}

fn signal_var_decl(decl: &VariableDeclaration, signals: &mut BindingSignals) {
    // `decl.kind` (var/let/const) is deliberately NOT checked here — unlike
    // `collect_const_await_import`, a candidate must be found regardless of
    // kind, since a `let`/`var` binding is exactly one of the shapes this
    // signal exists to catch (C2's `let`-bound probe).
    for declarator in &decl.declarations {
        if let Some(init) = &declarator.init {
            if is_namespace_candidate_init(init) {
                signals.candidates.insert(declarator.id.clone());
            }
            signal_expression(init, signals);
        }
    }
}

/// True when `init` is (mod `ParenthesizedExpression` wrapping around either layer) either
/// `await import(<any specifier>)` OR a BARE `import(<any specifier>)` — the two declarator-init
/// shapes `BindingSignals::candidates` must record, regardless of whether the specifier ever
/// folds. The bare-shape half closes the third review round's finding: `await` applied to a
/// binding that already holds an unawaited import result (`const p = import(...); ...; await p`)
/// never syntactically wraps the `ImportExpression`, so a check limited to
/// `as_await_import_source` alone missed it — the binding still goes on to be read exactly like
/// the proven `const ns = await import(...)` lane, just one `await` later.
fn is_namespace_candidate_init(init: &Expression) -> bool {
    as_await_import_source(init).is_some()
        || matches!(unwrap_parens(init), Expression::ImportExpression(_))
}

fn signal_class_body(body: &ClassBody, signals: &mut BindingSignals) {
    for method in &body.methods {
        if let Some(method_body) = &method.body {
            signal_block(method_body, signals);
        }
    }
}

fn signal_statement(statement: &Statement, signals: &mut BindingSignals) {
    match statement {
        Statement::ExpressionStatement(stmt) => signal_expression(&stmt.expression, signals),
        Statement::BreakStatement(_) => {}
        Statement::ContinueStatement(_) => {}
        Statement::WithStatement(stmt) => {
            signal_expression(&stmt.object, signals);
            signal_statement(&stmt.body, signals);
        }
        Statement::ReturnStatement(stmt) => {
            if let Some(argument) = &stmt.argument {
                signal_expression(argument, signals);
            }
        }
        Statement::LabeledStatement(stmt) => signal_statement(&stmt.body, signals),
        Statement::IfStatement(stmt) => {
            signal_expression(&stmt.test, signals);
            signal_block(&stmt.consequent, signals);
            if let Some(alternate) = &stmt.alternate {
                signal_block(alternate, signals);
            }
        }
        Statement::SwitchStatement(stmt) => {
            signal_expression(&stmt.discriminant, signals);
            for case in &stmt.cases {
                if let Some(test) = &case.test {
                    signal_expression(test, signals);
                }
                for consequent in &case.consequent {
                    signal_statement(consequent, signals);
                }
            }
        }
        Statement::ThrowStatement(stmt) => signal_expression(&stmt.argument, signals),
        Statement::TryStatement(stmt) => {
            signal_block(&stmt.block, signals);
            if let Some(handler) = &stmt.handler {
                signal_block(&handler.body, signals);
            }
            if let Some(finalizer) = &stmt.finalizer {
                signal_block(finalizer, signals);
            }
        }
        Statement::DebuggerStatement(_) => {}
        Statement::BlockStatement(stmt) => signal_block(stmt, signals),
        Statement::ForStatement(stmt) => {
            match &stmt.init {
                Some(ForInit::VariableDeclaration(decl)) => signal_var_decl(decl, signals),
                Some(ForInit::Expression(expr)) => signal_expression(expr, signals),
                None => {}
            }
            if let Some(test) = &stmt.test {
                signal_expression(test, signals);
            }
            if let Some(update) = &stmt.update {
                signal_expression(update, signals);
            }
            signal_block(&stmt.body, signals);
        }
        Statement::ForInStatement(stmt) => {
            match &stmt.left {
                ForInLefthand::VariableDeclaration(decl) => signal_var_decl(decl, signals),
                ForInLefthand::Expression(expr) => signal_expression(expr, signals),
            }
            signal_expression(&stmt.right, signals);
            signal_statement(&stmt.body, signals);
        }
        Statement::ForOfStatement(stmt) => {
            match &stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => signal_var_decl(decl, signals),
                ForOfLefthand::Expression(expr) => signal_expression(expr, signals),
            }
            signal_expression(&stmt.right, signals);
            signal_statement(&stmt.body, signals);
        }
        Statement::WhileStatement(stmt) => {
            signal_expression(&stmt.test, signals);
            signal_block(&stmt.body, signals);
        }
        Statement::DoWhileStatement(stmt) => {
            signal_block(&stmt.body, signals);
            signal_expression(&stmt.test, signals);
        }
        Statement::FunctionDeclaration(function) => signal_block(&function.body, signals),
        Statement::ClassDeclaration(class) => signal_class_body(&class.body, signals),
        Statement::VariableDeclaration(decl) => signal_var_decl(decl, signals),
        Statement::ImportDeclaration(decl) => {
            if decl.source.starts_with("./") || decl.source.starts_with("../") {
                for specifier in &decl.specifiers {
                    if let ImportSpecifier::Namespace(local) = specifier {
                        signals.candidates.insert(local.clone());
                    }
                }
            }
        }
        // `ExportAllDeclaration { source: String }` — no candidate/use shape
        // of any kind (same citation as `census_statement`'s arm above).
        Statement::ExportAll(_) => {}
        // A re-export's `local` name IS a value-level reference (see
        // `census_statement`'s `ExportNamed` arm), but it can never be a
        // `typeof`/call-member shape by itself — nothing to record here.
        Statement::ExportNamed(_) => {}
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => signal_expression(expr, signals),
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                signal_block(&function.body, signals)
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                signal_class_body(&class.body, signals)
            }
        },
        Statement::EnumDeclaration(decl) => {
            for member in &decl.members {
                if let Some(value) = &member.value {
                    signal_expression(value, signals);
                }
            }
        }
        Statement::TypeAliasDeclaration(_) => {}
        Statement::InterfaceDeclaration(_) => {}
    }
}

fn signal_expression_or_spread(element: &ExpressionOrSpread, signals: &mut BindingSignals) {
    match element {
        ExpressionOrSpread::Expression(expr) => signal_expression(expr, signals),
        ExpressionOrSpread::Spread(spread) => signal_expression(&spread.argument, signals),
        ExpressionOrSpread::Empty => {}
    }
}

fn signal_expression(expr: &Expression, signals: &mut BindingSignals) {
    match expr {
        // A bare identifier is never itself a candidate/use-site shape.
        Expression::Identifier(_) => {}
        Expression::Literal(_) => {}
        Expression::BinaryExpression(binary) => {
            signal_expression(&binary.left, signals);
            signal_expression(&binary.right, signals);
        }
        Expression::UnaryExpression(unary) => signal_expression(&unary.argument, signals),
        Expression::CallExpression(call) => {
            signal_expression(&call.callee, signals);
            for arg in &call.args {
                signal_expression(arg, signals);
            }
        }
        Expression::MemberExpression(member) => {
            // The single choke point for `member_access_sites`: EVERY member
            // access whose object is a bare identifier is recorded here, so
            // the `typeof <ns>.m` and `<ns>.m(...)` shapes Task 6 rewrites
            // (which are just this node under a `UnaryExpression`/
            // `CallExpression` parent, both of which recurse into it) are
            // covered by construction, with no per-shape mirror to keep in
            // sync.
            if let Expression::Identifier(name) = &member.object {
                signals.member_access_sites.insert(name.clone());
            }
            signal_expression(&member.object, signals);
            if let Some(index) = &member.computed_index {
                signal_expression(index, signals);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter().flatten() {
                signal_expression_or_spread(element, signals);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                signal_expression(&property.value, signals);
            }
        }
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                signal_block(body, signals);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => signal_expression(&arrow.body, signals),
        Expression::ClassExpression(class) => signal_class_body(&class.body, signals),
        Expression::NewExpression(new_expr) => {
            // `new ns.Ctor(...)` is deliberately NOT a call-callee shape
            // here, matching `rewrite_expression`'s own `NewExpression` arm
            // (its callee is walked/rewritten as an ordinary member
            // expression, never a namespace-member call-callee fold).
            signal_expression(&new_expr.callee, signals);
            for arg in &new_expr.args {
                signal_expression(arg, signals);
            }
        }
        Expression::MetaProperty(_) => {}
        Expression::TemplateLiteral(template) => {
            for expression in &template.expressions {
                signal_expression(expression, signals);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            signal_expression(&tagged.tag, signals);
            for expression in &tagged.template.expressions {
                signal_expression(expression, signals);
            }
        }
        Expression::UpdateExpression(update) => signal_expression(&update.argument, signals),
        Expression::AssignmentExpression(assignment) => {
            signal_expression(&assignment.left, signals);
            signal_expression(&assignment.right, signals);
        }
        Expression::LogicalExpression(logical) => {
            signal_expression(&logical.left, signals);
            signal_expression(&logical.right, signals);
        }
        Expression::ConditionalExpression(conditional) => {
            signal_expression(&conditional.test, signals);
            signal_expression(&conditional.consequent, signals);
            signal_expression(&conditional.alternate, signals);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &sequence.expressions {
                signal_expression(expression, signals);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            signal_expression(&parenthesized.expression, signals)
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &yield_expr.argument {
                signal_expression(argument, signals);
            }
        }
        Expression::AwaitExpression(await_expr) => signal_expression(&await_expr.argument, signals),
        Expression::OptionalChainExpression(chain) => match chain.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => signal_expression(object, signals),
        },
        Expression::ChainExpression(chain) => signal_expression(&chain.expression, signals),
        Expression::SpreadElement(spread) => signal_expression(&spread.argument, signals),
        Expression::RestElement(rest) => signal_expression(&rest.argument, signals),
        Expression::ImportExpression(import_expr) => {
            signal_expression(&import_expr.source, signals)
        }
        Expression::DecoratedExpression(decorated) => {
            signal_expression(&decorated.expression, signals)
        }
        Expression::JsxElement(element) => signal_jsx_element(element, signals),
        Expression::JsxFragment(fragment) => signal_jsx_fragment(fragment, signals),
        Expression::JsxEmptyExpression => {}
        Expression::TypeAssertion(assertion) => signal_expression(&assertion.expression, signals),
        Expression::SatisfiesExpression(satisfies) => {
            signal_expression(&satisfies.expression, signals)
        }
        Expression::ThisExpression => {}
        Expression::SuperExpression => {}
        Expression::PrivateIdentifier(_) => {}
        Expression::BigIntLiteral(_) => {}
    }
}

fn signal_jsx_element(element: &JsxElement, signals: &mut BindingSignals) {
    for attribute in &element.opening_element.attributes {
        signal_jsx_attribute_item(attribute, signals);
    }
    for child in &element.children {
        signal_jsx_child(child, signals);
    }
}

fn signal_jsx_fragment(fragment: &JsxFragment, signals: &mut BindingSignals) {
    for child in &fragment.children {
        signal_jsx_child(child, signals);
    }
}

fn signal_jsx_attribute_item(item: &JsxAttributeItem, signals: &mut BindingSignals) {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            signal_jsx_attribute_value(&attribute.value, signals)
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            signal_expression(&spread.argument, signals)
        }
    }
}

fn signal_jsx_attribute_value(value: &JsxAttributeValue, signals: &mut BindingSignals) {
    match value {
        JsxAttributeValue::String(_) => {}
        JsxAttributeValue::JsxElement(element) => signal_jsx_element(element, signals),
        JsxAttributeValue::JsxExpression(container) => {
            signal_jsx_expression_container(container, signals)
        }
    }
}

fn signal_jsx_expression_container(
    container: &JsxExpressionContainer,
    signals: &mut BindingSignals,
) {
    if let Some(expression) = &container.expression {
        signal_expression(expression, signals);
    }
}

fn signal_jsx_child(child: &JsxChild, signals: &mut BindingSignals) {
    match child {
        JsxChild::JsxText(_) => {}
        JsxChild::JsxExpression(container) => signal_jsx_expression_container(container, signals),
        JsxChild::JsxElement(element) => signal_jsx_element(element, signals),
        JsxChild::JsxFragment(fragment) => signal_jsx_fragment(fragment, signals),
    }
}

fn unproven_namespace_binding_error(name: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "namespace binding '{name}' cannot be linked: no provable module namespace was found for it — only a relative `import * as {name} from \"./...\"`, or `const {name} = await import(<foldable specifier>)`, both UNSHADOWED and within this pass's provenance-collection reach, can be linked for member access; falling through silently here would leak a raw specifier/namespace value (or fold every `typeof {name}.member` to \"undefined\") instead of rejecting"
        ),
    )
}

/// C2 default-deny: closes the pre-stage fail-open for any binding that
/// LOOKS namespace-shaped (`BindingSignals::candidates`) but that
/// `collect_namespace_provenance`'s narrower reach could not prove
/// provenance for.
///
/// MUST run before `link_provable_module_namespaces`'s
/// `provenance.bindings.is_empty()` early return: a file whose ONLY
/// namespace-shaped binding the collector could not prove has an EMPTY
/// `provenance.bindings` map, and the pre-fix pipeline treated an empty map
/// as "no namespace bindings at all" and returned immediately — silently
/// falling through to the pre-stage raw-specifier-string behavior (final
/// whole-branch review finding C2, all three probe shapes: one block
/// deeper inside an `if`, `let`-bound, and a non-foldable ternary
/// specifier).
///
/// Deliberately narrow in the two ways the review's own scoping note
/// requires (both needed to avoid regressing green tests):
///   - a candidate already proven (`provenance.bindings` contains it) is
///     skipped — handled by the normal proven-lane pipeline instead.
///   - a candidate with NO provenance that is never referenced by a plain
///     `Expression::Identifier` anywhere outside its own binding position is
///     skipped too: an UNUSED un-provable binding is harmless, and denying
///     it would only create new regressions (e.g. the existing
///     `link_provable_module_namespaces_leaves_bare_dynamic_import_statement_untouched`
///     green test relies on exactly this for the no-binding statement form;
///     an unused BOUND-but-unprovable case is the same "harmless" logic one
///     level up).
fn deny_unproven_namespace_binding_candidates(
    statements: &[Statement],
    provenance: &NamespaceProvenance,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let signals = collect_binding_signals(statements);
    if signals.candidates.is_empty() {
        return;
    }

    let mut used: BTreeSet<String> = BTreeSet::new();
    let mut on_identifier = |name: &str| {
        used.insert(name.to_string());
    };
    let mut on_binding = |_: &str| {};
    census_statements(statements, &mut on_identifier, &mut on_binding);

    for name in &signals.candidates {
        if provenance.bindings.contains_key(name) {
            continue;
        }
        if used.contains(name) {
            diagnostics.push(unproven_namespace_binding_error(name));
        }
    }
}

/// The single public entry point `compile.rs` calls. Runs collect → C2
/// default-deny → shadow guard → load → append → rewrite → deny. Pushes
/// diagnostics into `diagnostics`; never silently rewrites partially —
/// every fallible step (`load_linked_module`, `append_linked_functions`)
/// stops the pipeline before any further step runs once it has failed
/// (mirroring `append_linked_functions`'s own guard-before-mutate contract,
/// so `statements` is never left in a state some diagnostic doesn't already
/// account for). No provenance AND no unproven-but-used candidate found →
/// guaranteed no-op: `statements` is returned untouched (this is the
/// load-bearing case — the pass runs on EVERY compile, and the overwhelming
/// majority of sources have no namespace bindings at all).
pub fn link_provable_module_namespaces(
    source_path: &Path,
    source_contents: &str,
    statements: &mut Vec<Statement>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // C2 (position allowlist, second review round): runs FIRST and
    // UNCONDITIONALLY — it doesn't need `provenance` at all, since it denies
    // by SYNTACTIC POSITION, not by binding shape or usage. See
    // `deny_import_positions_expression`'s section doc comment above for why
    // this is a separate, earlier gate from the binding-shape deny below.
    deny_import_expressions_outside_allowlist(statements, diagnostics);
    if has_errors(diagnostics) {
        return;
    }

    let provenance = collect_namespace_provenance(source_path, source_contents, statements);

    // C2 (binding-shape deny): MUST run before the `bindings.is_empty()`
    // early return below — see `deny_unproven_namespace_binding_candidates`'s
    // doc comment.
    deny_unproven_namespace_binding_candidates(statements, &provenance, diagnostics);
    if has_errors(diagnostics) {
        return;
    }

    if provenance.bindings.is_empty() {
        return;
    }

    deny_shadowed_bindings(statements, &provenance, diagnostics);
    if has_errors(diagnostics) {
        return;
    }

    // I1: only load (and purity-gate) a module for a binding that is
    // actually USED as a namespace — i.e. has at least one member access
    // (`<name>.member`, `<name>.member(...)`, `typeof <name>.member`, ...)
    // anywhere in the pristine entry program. Computed on the SAME pristine
    // `statements` `deny_shadowed_bindings` just ran over, BEFORE
    // `append_linked_functions` splices any clones in. An unused binding has
    // no such site and would otherwise eagerly load + purity-gate a module
    // nothing in the program reads — turning a program node runs fine into a
    // hard E5506 build failure (final whole-branch review finding I1). This
    // composes with C2: an unused, unprovable binding is neither loaded nor
    // denied.
    let signals = collect_binding_signals(statements);
    let mut index_has_use_site: BTreeMap<usize, bool> = BTreeMap::new();
    for (name, linked) in &provenance.bindings {
        let has_use = signals.member_access_sites.contains(name);
        let entry = index_has_use_site.entry(linked.index).or_insert(false);
        *entry = *entry || has_use;
    }

    let mut modules: BTreeMap<usize, LinkedModuleAst> = BTreeMap::new();
    for linked in provenance.bindings.values() {
        if modules.contains_key(&linked.index) {
            continue;
        }
        if !index_has_use_site
            .get(&linked.index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        match load_linked_module(linked) {
            Ok(loaded) => {
                modules.insert(linked.index, loaded);
            }
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    if has_errors(diagnostics) {
        return;
    }

    // Minor fix: track how many clones each `append_linked_functions` call
    // splices onto the FRONT of `statements`, so `deny_unrewritten_uses`
    // below can skip over every clone body and only census the entry's OWN
    // statements — without this, a linked module's internal local sharing a
    // name with an entry provenance binding was wrongly E5506'd even though
    // the two are unrelated (final whole-branch review's "Minor" finding).
    let mut clone_count = 0usize;
    for module in modules.values() {
        match append_linked_functions(statements, module) {
            Ok(appended) => clone_count += appended,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                return;
            }
        }
    }

    rewrite_namespace_uses(
        &mut statements[clone_count..],
        &provenance,
        &modules,
        diagnostics,
    );
    if has_errors(diagnostics) {
        return;
    }

    deny_unrewritten_uses(&statements[clone_count..], &provenance, diagnostics);
}

#[cfg(test)]
mod tests {
    use super::*;
    use kali_common::FileId;
    use kali_lexer::Lexer;
    use kali_parser::Parser;
    use std::fs;
    use tempfile::{tempdir, TempDir};

    /// Parses `source` with the real lexer/parser (no error-swallowing —
    /// panics on diagnostics, since every fixture here is intended to be
    /// syntactically valid).
    fn parse(source: &str) -> Vec<Statement> {
        let lexed = Lexer::new(FileId::new(0), source.to_string()).lex_all();
        assert!(
            lexed.diagnostics.is_empty(),
            "lex diagnostics: {:?}",
            lexed.diagnostics
        );
        let mut parser = Parser::new(FileId::new(0), lexed.tokens);
        let parsed = parser.parse(None);
        assert!(
            parsed.diagnostics.is_empty(),
            "parse diagnostics: {:?}",
            parsed.diagnostics
        );
        parsed.statements
    }

    /// Creates a tempdir containing real `util.js` and `lazy.js` stub
    /// modules (required for `resolve_dynamic_import_target`'s on-disk
    /// resolution) and returns `(tempdir, main_js_path)`. `main_js_path`
    /// need not itself exist on disk — only its parent directory matters
    /// for relative-specifier resolution.
    fn fixture_dir() -> (TempDir, PathBuf) {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("util.js"),
            "export function greet() { return 'hi'; }\n",
        )
        .expect("write util.js");
        fs::write(
            dir.path().join("lazy.js"),
            "export function lazyValue() { return 7; }\n",
        )
        .expect("write lazy.js");
        let main_js = dir.path().join("main.js");
        (dir, main_js)
    }

    fn canonical(dir: &TempDir, name: &str) -> PathBuf {
        fs::canonicalize(dir.path().join(name)).expect("canonicalize fixture")
    }

    // ---- positive: each red-fixture specifier shape must yield provenance ----

    #[test]
    fn namespace_import_of_relative_module_is_proven() {
        let (dir, main_js) = fixture_dir();
        let source = r#"import * as ns from "./util.js";"#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("ns"),
            Some(&LinkedModule {
                path: canonical(&dir, "util.js"),
                index: 0,
            })
        );
    }

    #[test]
    fn const_await_import_of_string_literal_is_proven() {
        let (dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const c = await import("./lazy.js");
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            Some(&LinkedModule {
                path: canonical(&dir, "lazy.js"),
                index: 0,
            })
        );
    }

    #[test]
    fn const_await_import_of_template_literal_with_const_part_is_proven() {
        let (dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const name = "lazy.js";
                const c = await import(`./${name}`);
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            Some(&LinkedModule {
                path: canonical(&dir, "lazy.js"),
                index: 0,
            })
        );
    }

    #[test]
    fn const_await_import_of_sequence_wrapped_template_literal_is_proven() {
        let (dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const name = "lazy.js";
                const c = await import((0, `./${name}`));
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            Some(&LinkedModule {
                path: canonical(&dir, "lazy.js"),
                index: 0,
            })
        );
    }

    #[test]
    fn const_await_import_through_object_freeze_nullish_coalesce_is_proven() {
        let (dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const c = await import(Object.freeze((null ?? "./lazy.js")));
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            Some(&LinkedModule {
                path: canonical(&dir, "lazy.js"),
                index: 0,
            })
        );
    }

    #[test]
    fn const_await_import_through_object_freeze_logical_and_is_proven() {
        let (dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const c = await import(Object.freeze((true && "./lazy.js")));
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            Some(&LinkedModule {
                path: canonical(&dir, "lazy.js"),
                index: 0,
            })
        );
    }

    #[test]
    fn const_await_import_through_object_freeze_logical_or_is_proven() {
        let (dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const c = await import(Object.freeze((false || "./lazy.js")));
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            Some(&LinkedModule {
                path: canonical(&dir, "lazy.js"),
                index: 0,
            })
        );
    }

    // ---- negative: NO provenance (binding absent from map) ----

    #[test]
    fn const_await_import_of_non_foldable_call_is_not_proven() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const c = await import(runtimeName());
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(provenance.bindings.get("c"), None);
    }

    #[test]
    fn const_await_import_of_template_literal_with_non_const_part_is_not_proven() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                let runtimeVar = "lazy.js";
                const c = await import(`./${runtimeVar}`);
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(provenance.bindings.get("c"), None);
    }

    #[test]
    fn namespace_import_of_non_relative_source_is_not_proven() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"import * as fs from "fs";"#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(provenance.bindings.get("fs"), None);
    }

    #[test]
    fn let_await_import_is_not_proven() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                let c = await import("./lazy.js");
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(provenance.bindings.get("c"), None);
    }

    // ---- register(): shared-target index reuse ----

    #[test]
    fn two_bindings_of_the_same_module_share_one_index() {
        let (dir, main_js) = fixture_dir();
        let source = r#"
            import * as a from "./util.js";
            import * as b from "./util.js";
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        let expected = LinkedModule {
            path: canonical(&dir, "util.js"),
            index: 0,
        };
        assert_eq!(provenance.bindings.get("a"), Some(&expected));
        assert_eq!(provenance.bindings.get("b"), Some(&expected));
    }

    #[test]
    fn two_different_specifiers_resolving_to_the_same_file_share_one_index() {
        let (dir, main_js) = fixture_dir();
        // "sub" must exist as a real on-disk directory for the OS to resolve
        // the ".." traversal in "./sub/../util.js" down to the same file
        // canonicalized by "./util.js" (resolve_dynamic_import_target checks
        // `is_file()` before canonicalizing, so a non-existent intermediate
        // directory would make this specifier fail to resolve at all).
        fs::create_dir(dir.path().join("sub")).expect("create sub dir");
        let source = r#"
            import * as a from "./util.js";
            import * as b from "./sub/../util.js";
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        let expected = LinkedModule {
            path: canonical(&dir, "util.js"),
            index: 0,
        };
        assert_eq!(provenance.bindings.get("a"), Some(&expected));
        assert_eq!(provenance.bindings.get("b"), Some(&expected));
    }

    // ---- load_linked_module: purity gate + true-export census (Task 4) ----

    fn write_module(dir: &TempDir, name: &str, source: &str) -> LinkedModule {
        fs::write(dir.path().join(name), source).expect("write fixture module");
        LinkedModule {
            path: canonical(dir, name),
            index: 0,
        }
    }

    #[test]
    fn load_linked_module_accepts_single_exported_function() {
        let dir = tempdir().expect("tempdir");
        let module = write_module(
            &dir,
            "lazy.js",
            "export function lazyValue() { return 7n; }",
        );
        let loaded = load_linked_module(&module).expect("module should load");
        assert_eq!(loaded.index, 0);
        assert_eq!(
            loaded.exports.keys().cloned().collect::<Vec<_>>(),
            vec!["lazyValue".to_string()]
        );
        assert_eq!(
            loaded.all_functions.keys().cloned().collect::<Vec<_>>(),
            vec!["lazyValue".to_string()]
        );
    }

    /// This is THE test that proves the true-export census is driven by the
    /// token scan and not by "every top-level function is an export": if
    /// `load_linked_module` instead treated `all_functions` as the export
    /// set, `helper` (never preceded by `export` in the source) would wrongly
    /// appear in `exports` too, and a downstream `ns.helper()` use would
    /// wrongly link against a private implementation detail (a fail-open).
    #[test]
    fn load_linked_module_true_export_census_excludes_private_helper() {
        let dir = tempdir().expect("tempdir");
        let module = write_module(
            &dir,
            "util.js",
            "function helper() { return 1n; } export function f() { return helper(); }",
        );
        let loaded = load_linked_module(&module).expect("module should load");
        assert_eq!(
            loaded.exports.keys().cloned().collect::<Vec<_>>(),
            vec!["f".to_string()],
            "helper must NOT be counted as an export"
        );
        assert_eq!(
            loaded.all_functions.keys().cloned().collect::<Vec<_>>(),
            vec!["f".to_string(), "helper".to_string()],
            "all_functions must retain the private helper for Task 5 sibling-callee renames"
        );
    }

    fn assert_rejected(source: &str, name: &str) -> Diagnostic {
        let dir = tempdir().expect("tempdir");
        let module = write_module(&dir, name, source);
        let error = load_linked_module(&module).expect_err("module must be rejected");
        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        let path_string = module.path.display().to_string();
        assert!(
            error.message.contains(path_string.as_str()),
            "message must name the module path: {}",
            error.message
        );
        error
    }

    #[test]
    fn load_linked_module_rejects_top_level_statement() {
        let error = assert_rejected("console.log('boot'); export function f() {}", "main.js");
        assert!(
            error.message.contains("top level") || error.message.contains("statement"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    #[test]
    fn load_linked_module_rejects_top_level_import() {
        let error = assert_rejected(
            "import { x } from './other.js'; export function f() {}",
            "main.js",
        );
        assert!(
            error.message.contains("import"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    #[test]
    fn load_linked_module_rejects_non_function_export() {
        // The parser erases `export` and has no dedicated node for a `const`
        // that follows it (module.rs falls through to a generic expression
        // statement here), so this rejects as an ordinary top-level
        // statement rather than under a `const`-specific label — the
        // allowlist gate still fails closed regardless of the exact label.
        let error = assert_rejected("export const value = 7;", "main.js");
        assert!(
            error.message.contains("top level") || error.message.contains("statement"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    #[test]
    fn load_linked_module_rejects_async_export() {
        let error = assert_rejected("export async function f() {}", "main.js");
        assert!(
            error.message.contains("async"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    #[test]
    fn load_linked_module_rejects_generator_export() {
        let error = assert_rejected("export function* f() {}", "main.js");
        assert!(
            error.message.contains("generator"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    #[test]
    fn load_linked_module_rejects_class() {
        let error = assert_rejected("export class C {}", "main.js");
        assert!(
            error.message.contains("class"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    /// Closes the fail-open where a private redefinition of an exported
    /// top-level function name silently overwrote the exported body in
    /// `all_functions` while the token scan still reported the name as
    /// exported (proven from the FIRST, genuinely-`export`ed declaration).
    /// Without a duplicate-name guard, `exports = {f: <private redefinition's
    /// body>}` — a downstream `ns.f()` would link against a body that was
    /// never actually exported.
    #[test]
    fn load_linked_module_rejects_duplicate_top_level_function_name() {
        let error = assert_rejected(
            "export function f() { return 1n; } function f() { return 2n; }",
            "main.js",
        );
        assert!(
            error.message.contains('f'),
            "message must name the duplicated function: {}",
            error.message
        );
    }

    // ---- append_linked_functions (Task 5) ----

    /// Builds a `LinkedModuleAst` directly from source, without touching
    /// disk — `append_linked_functions` only reads `module.all_functions`
    /// and `module.index`, so `exports` is left empty (irrelevant here).
    fn build_module(index: usize, source: &str) -> LinkedModuleAst {
        let mut all_functions = BTreeMap::new();
        let mut declaration_order = Vec::new();
        for statement in parse(source) {
            if let Statement::FunctionDeclaration(function) = statement {
                declaration_order.push(function.name.clone());
                all_functions.insert(function.name.clone(), function);
            }
        }
        LinkedModuleAst {
            index,
            exports: BTreeMap::new(),
            all_functions,
            declaration_order,
        }
    }

    fn find_function<'a>(statements: &'a [Statement], name: &str) -> &'a FunctionDeclaration {
        statements
            .iter()
            .find_map(|statement| match statement {
                Statement::FunctionDeclaration(function) if function.name == name => Some(function),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!("expected a FunctionDeclaration named `{name}` in {statements:?}")
            })
    }

    #[test]
    fn append_linked_functions_appends_mangled_clone_with_identical_body() {
        let module = build_module(0, "export function lazyValue() { return 7; }");
        let original = module.all_functions.get("lazyValue").unwrap().clone();
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        assert_eq!(statements.len(), 1);
        let clone = find_function(&statements, "__link0_lazyValue");
        assert_eq!(
            clone.body, original.body,
            "body must be otherwise identical"
        );
        assert_eq!(clone.params, original.params);
        assert_eq!(clone.is_async, original.is_async);
        assert_eq!(clone.generator, original.generator);
    }

    #[test]
    fn append_linked_functions_renames_sibling_callee() {
        let module = build_module(
            0,
            "function helper() { return 1; } export function f() { return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        // Both the export and the private helper are appended.
        let _ = find_function(&statements, "__link0_helper");
        let f_clone = find_function(&statements, "__link0_f");
        match &f_clone.body.body[..] {
            [Statement::ReturnStatement(r)] => {
                match r.argument.as_ref().expect("return has an argument") {
                    Expression::CallExpression(call) => match &call.callee {
                        Expression::Identifier(name) => assert_eq!(
                        name, "__link0_helper",
                        "the call inside __link0_f's body must be renamed to the mangled helper"
                    ),
                        other => panic!("expected an Identifier callee, got {other:?}"),
                    },
                    other => panic!("expected a CallExpression, got {other:?}"),
                }
            }
            other => panic!("expected a single ReturnStatement body, got {other:?}"),
        }
    }

    #[test]
    fn append_linked_functions_rejects_name_collision() {
        let module = build_module(0, "export function lazyValue() { return 7; }");
        let mut statements = parse("function __link0_lazyValue() {}");

        let error = append_linked_functions(&mut statements, &module)
            .expect_err("a mangled-name collision must be rejected");

        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            error.message.contains("__link0_lazyValue"),
            "message must name the colliding identifier: {}",
            error.message
        );
    }

    #[test]
    fn append_linked_functions_leaves_statements_unchanged_on_collision() {
        let module = build_module(0, "export function lazyValue() { return 7; }");
        let mut statements = parse("function __link0_lazyValue() {}");
        let before = statements.clone();

        let result = append_linked_functions(&mut statements, &module);

        assert!(result.is_err());
        assert_eq!(
            statements, before,
            "a colliding module must never partially mutate `statements`"
        );
    }

    #[test]
    fn append_linked_functions_rejects_bare_sibling_reference() {
        let module = build_module(
            0,
            "function helper() { return 1; } export function f() { const g = helper; return g(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        let error = append_linked_functions(&mut statements, &module)
            .expect_err("a bare (non-call) reference to a sibling function must be rejected");

        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            error.message.contains("helper"),
            "message must name the aliased sibling: {}",
            error.message
        );
    }

    #[test]
    fn append_linked_functions_leaves_statements_unchanged_on_bare_reference_reject() {
        let module = build_module(
            0,
            "function helper() { return 1; } export function f() { return helper; }",
        );
        let mut statements: Vec<Statement> = Vec::new();
        let before = statements.clone();

        let result = append_linked_functions(&mut statements, &module);

        assert!(result.is_err());
        assert_eq!(
            statements, before,
            "a rejected sibling-alias must never partially mutate `statements`"
        );
    }

    // ---- shadowing fail-open (CRITICAL review finding) ----
    //
    // The rename walk has no lexical-scope awareness. Before the fix below,
    // a cloned body that locally REBINDS a sibling linked-function name (a
    // `const`/`let`/`var` local, a function parameter, a nested function
    // declaration, a catch-clause param, a for-in/for-of loop variable, a
    // class name, ...) still had every CALL of that name silently rewritten
    // to the mangled sibling — a silent wrong-call-target miscompile. These
    // tests must reject closed instead.

    #[test]
    fn append_linked_functions_rejects_local_const_shadowing_sibling() {
        // The exact probe from the finding: `helper` is locally rebound to
        // an arrow function, but the un-walked declarator id leaves the
        // local binding named `helper` while the call `helper()` gets
        // silently renamed to `__link0_helper` — the SIBLING's `helper`,
        // not the local arrow.
        let module = build_module(
            0,
            "function helper() { return 1n; } export function f() { const helper = () => 2n; return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        let error = append_linked_functions(&mut statements, &module)
            .expect_err("a local binding shadowing a sibling linked function must be rejected");

        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            error.message.contains("helper"),
            "message must name the shadowed linked function: {}",
            error.message
        );
    }

    #[test]
    fn append_linked_functions_rejects_parameter_shadowing_sibling() {
        let module = build_module(
            0,
            "function helper() { return 1n; } export function f(helper) { return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        let error = append_linked_functions(&mut statements, &module)
            .expect_err("a parameter shadowing a sibling linked function must be rejected");

        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            error.message.contains("helper"),
            "message must name the shadowed linked function: {}",
            error.message
        );
    }

    #[test]
    fn append_linked_functions_leaves_statements_unchanged_on_shadow_reject() {
        let module = build_module(
            0,
            "function helper() { return 1n; } export function f() { const helper = () => 2n; return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();
        let before = statements.clone();

        let result = append_linked_functions(&mut statements, &module);

        assert!(result.is_err());
        assert_eq!(
            statements, before,
            "a rejected shadow must never partially mutate `statements`"
        );
    }

    // ---- nested import/export inside a cloned body (CRITICAL round-2 finding) ----
    //
    // The rename/deny walk previously treated `Statement::ImportDeclaration`
    // and `Statement::ExportNamed` as unconditional no-ops on the claim that
    // the parser "only ever produces this at module top level". That claim
    // is false: `parse_statement` (`crates/kali_parser/src/statement.rs:45-51`
    // for `TokenType::Import`, `:33` for `TokenType::Export`) routes both
    // unconditionally, and a function body parses through the exact same
    // generic `parse_block_statement`/`parse_statement` loop as module top
    // level (`crates/kali_parser/src/statement.rs:159-178`,
    // `crates/kali_parser/src/declaration.rs:84`). A nested import's local
    // bindings went unrenamed while calls to the SAME name elsewhere in the
    // body were still silently rewritten to a sibling linked function's
    // mangled name — the same wrong-call-target class the sibling-rename
    // walk exists to close. These must reject closed instead.

    #[test]
    fn append_linked_functions_rejects_nested_import_in_cloned_body() {
        // The exact probe from the finding: `helper` is locally imported
        // from another module inside `f`'s body, but the un-walked
        // `ImportDeclaration` leaves the local binding named `helper`
        // pointing at "./evil" while the call `helper()` still gets
        // silently renamed to `__link0_helper` — the SIBLING's `helper`,
        // not the locally-imported one.
        let module = build_module(
            0,
            r#"function helper() { return 1n; } export function f() { import { helper } from "./evil"; return helper(); }"#,
        );
        let mut statements: Vec<Statement> = Vec::new();

        let error = append_linked_functions(&mut statements, &module)
            .expect_err("a nested import inside a cloned function body must be rejected");

        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            error.message.contains("import"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    #[test]
    fn append_linked_functions_leaves_statements_unchanged_on_nested_import_reject() {
        let module = build_module(
            0,
            r#"function helper() { return 1n; } export function f() { import { helper } from "./evil"; return helper(); }"#,
        );
        let mut statements: Vec<Statement> = Vec::new();
        let before = statements.clone();

        let result = append_linked_functions(&mut statements, &module);

        assert!(result.is_err());
        assert_eq!(
            statements, before,
            "a rejected nested import must never partially mutate `statements`"
        );
    }

    #[test]
    fn append_linked_functions_rejects_nested_named_export_in_cloned_body() {
        // Same reachability proof as the nested-import case, but for
        // `Statement::ExportNamed`: `export { helper };` inside `f`'s body
        // parses cleanly (no diagnostics) via the same unrestricted
        // `parse_statement` loop, and `ExportSpecifier { local, exported }`
        // names a local binding by string exactly like a bare `Identifier`
        // reference would — so a nested one must reject rather than being a
        // silent no-op.
        let module = build_module(
            0,
            "function helper() { return 1n; } export function f() { export { helper }; return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        let error = append_linked_functions(&mut statements, &module)
            .expect_err("a nested named export inside a cloned function body must be rejected");

        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            error.message.contains("export"),
            "message must name the offending construct: {}",
            error.message
        );
    }

    #[test]
    fn append_linked_functions_leaves_statements_unchanged_on_nested_named_export_reject() {
        let module = build_module(
            0,
            "function helper() { return 1n; } export function f() { export { helper }; return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();
        let before = statements.clone();

        let result = append_linked_functions(&mut statements, &module);

        assert!(result.is_err());
        assert_eq!(
            statements, before,
            "a rejected nested named export must never partially mutate `statements`"
        );
    }

    /// This resolver has NO forward-declaration hoisting: a top-level
    /// identifier must be textually declared before any use, even a use
    /// inside a not-yet-invoked function body (verified independently of
    /// this feature entirely — a plain `function f() { return helper(); }
    /// function helper() { return 1; }` with no module-linking involved at
    /// all already fails to resolve `helper` with the SAME E3100). A linked
    /// function therefore MUST be inserted before the entry module's
    /// EARLIEST possible use site, which can be its very first statement
    /// (`import * as ns from "./x"; const v = ns.export();`) — appending at
    /// the end (the pre-fix behavior) left every such use unresolvable.
    #[test]
    fn append_linked_functions_inserts_clones_before_existing_statements() {
        let module = build_module(0, "export function lazyValue() { return 7; }");
        let mut statements = parse("console.log('already here');");
        let before_len = statements.len();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        assert_eq!(statements.len(), before_len + 1);
        match &statements[0] {
            Statement::FunctionDeclaration(function) => {
                assert_eq!(function.name, "__link0_lazyValue");
            }
            other => panic!(
                "expected the linked clone to be inserted BEFORE the pre-existing statement, got {other:?} at position 0"
            ),
        }
        match &statements[1] {
            Statement::ExpressionStatement(_) => {}
            other => {
                panic!("expected the original statement to survive at position 1, got {other:?}")
            }
        }
    }

    #[test]
    fn append_linked_functions_still_renames_genuine_sibling_call_no_shadow() {
        // Positive control: no shadow anywhere, so the existing rename
        // behavior must be unaffected by the new shadow check.
        let module = build_module(
            0,
            "function helper() { return 1; } export function f() { return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        let f_clone = find_function(&statements, "__link0_f");
        match &f_clone.body.body[..] {
            [Statement::ReturnStatement(r)] => {
                match r.argument.as_ref().expect("return has an argument") {
                    Expression::CallExpression(call) => match &call.callee {
                        Expression::Identifier(name) => {
                            assert_eq!(name, "__link0_helper")
                        }
                        other => panic!("expected an Identifier callee, got {other:?}"),
                    },
                    other => panic!("expected a CallExpression, got {other:?}"),
                }
            }
            other => panic!("expected a single ReturnStatement body, got {other:?}"),
        }
    }

    /// Positive control: `Statement::ExportDefault` is reachable nested the
    /// same way as `ImportDeclaration`/`ExportNamed` (same unrestricted
    /// `parse_statement` loop), but its match arm was never a no-op — it
    /// already recursively walks all three `ExportDefaultDeclaration`
    /// variants (`Expression`/`FunctionDeclaration`/`ClassDeclaration`), so a
    /// sibling call nested inside one is already correctly renamed rather
    /// than silently left wrong. No fix was needed here; this test records
    /// the verification.
    #[test]
    fn append_linked_functions_already_renames_sibling_call_inside_nested_export_default() {
        let module = build_module(
            0,
            "function helper() { return 1n; } export function f() { export default helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        let f_clone = find_function(&statements, "__link0_f");
        match &f_clone.body.body[..] {
            [Statement::ExportDefault(ExportDefaultDeclaration::Expression(
                Expression::CallExpression(call),
            ))] => match &call.callee {
                Expression::Identifier(name) => assert_eq!(
                    name, "__link0_helper",
                    "the call inside the nested `export default` must already be renamed"
                ),
                other => panic!("expected an Identifier callee, got {other:?}"),
            },
            other => {
                panic!("expected a single nested ExportDefault(Expression) body, got {other:?}")
            }
        }
    }

    // ---- dependency-order topological sort (Important review finding fix) ----
    //
    // `module.all_functions` is a `BTreeMap`, so iterating it (the pre-fix
    // behavior) emits clones in ALPHABETICAL name order — unrelated to which
    // clone calls which. Whenever a caller happens to sort before its
    // callee, the resolver's no-hoisting property (see
    // `append_linked_functions`'s doc comment) makes the appended program
    // fail to resolve the callee, EVEN THOUGH `append_linked_functions`
    // itself reports success (the failure only surfaces later, in the
    // resolver). These tests assert the CLONE POSITIONS directly, for both
    // possible source declaration orders — proving the fix is genuinely
    // dependency-driven, not accidentally correct for only one order.

    fn position_of(statements: &[Statement], name: &str) -> usize {
        statements
            .iter()
            .position(|statement| {
                matches!(statement, Statement::FunctionDeclaration(function) if function.name == name)
            })
            .unwrap_or_else(|| panic!("expected a FunctionDeclaration named `{name}` in {statements:?}"))
    }

    #[test]
    fn append_linked_functions_orders_callee_before_caller_when_callee_declared_first() {
        // `helper` is declared BEFORE `f` in source — alphabetical order
        // ("f" < "helper") would ALSO happen to put helper's clone first
        // here, so this alone doesn't distinguish a real fix from the old
        // alphabetical accident; it's the companion reverse-order test below
        // that does. Kept as the direct mirror of the plan's own mandated
        // shape (Task 4/5 fixture).
        let module = build_module(
            0,
            "function helper() { return 1n; } export function f() { return helper(); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        let helper_pos = position_of(&statements, "__link0_helper");
        let f_pos = position_of(&statements, "__link0_f");
        assert!(
            helper_pos < f_pos,
            "callee __link0_helper must be declared before caller __link0_f, got positions {helper_pos} and {f_pos} in {statements:?}"
        );
    }

    #[test]
    fn append_linked_functions_orders_callee_before_caller_when_caller_declared_first() {
        // The reverse source order: `f` (the caller) is declared BEFORE
        // `helper` (the callee). Alphabetical `BTreeMap` order ALSO puts `f`
        // first here (same as source order) — so the pre-fix code emitted
        // callee-after-caller in this exact shape, which is precisely the
        // plan's own mandated fixture (Task 4/5) and reproduces the defect:
        // `E5506`/`E3100` on `__link0_helper` before the fix. A correct
        // dependency-order sort must still place `helper` first here, EVEN
        // THOUGH that's the opposite of both alphabetical AND source order.
        let module = build_module(
            0,
            "export function f() { return helper(); } function helper() { return 1n; }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        let helper_pos = position_of(&statements, "__link0_helper");
        let f_pos = position_of(&statements, "__link0_f");
        assert!(
            helper_pos < f_pos,
            "callee __link0_helper must be declared before caller __link0_f, got positions {helper_pos} and {f_pos} in {statements:?}"
        );
    }

    #[test]
    fn append_linked_functions_orders_by_declaration_when_call_graph_has_no_edges() {
        // Two functions with NO call relationship at all: the topological
        // sort has no constraint to satisfy, so the tie-break — the
        // module's own SOURCE declaration order — must decide, not
        // alphabetical `BTreeMap` order ("second" < "zzzfirst"
        // alphabetically, but "zzzfirst" is declared first in source).
        let module = build_module(
            0,
            "export function zzzfirst() { return 1n; } export function second() { return 2n; }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module).expect("append should succeed");

        let first_pos = position_of(&statements, "__link0_zzzfirst");
        let second_pos = position_of(&statements, "__link0_second");
        assert!(
            first_pos < second_pos,
            "with no call-graph constraint, source declaration order must be preserved: got positions {first_pos} and {second_pos} in {statements:?}"
        );
    }

    #[test]
    fn append_linked_functions_rejects_mutual_recursion_cycle() {
        // Verified independently of module-linking: `function isEven(n) {
        // return n === 0 ? true : isOdd(n - 1); } function isOdd(n) { return
        // n === 0 ? false : isEven(n - 1); }` already fails to resolve
        // `isOdd` with E3100 in PLAIN kali (no linking involved) for either
        // declaration order — there is no order that makes both calls
        // resolve. `append_linked_functions` must reject this fail-closed
        // at compile time instead of emitting a broken order.
        let module = build_module(
            0,
            "function isEven(n) { return n === 0 ? true : isOdd(n - 1); } function isOdd(n) { return n === 0 ? false : isEven(n - 1); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        let error = append_linked_functions(&mut statements, &module)
            .expect_err("a mutual-recursion call cycle must be rejected");

        assert_eq!(error.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            error.message.contains("isEven") && error.message.contains("isOdd"),
            "message must name both cycle participants: {}",
            error.message
        );
        assert!(
            error.message.contains("cycle"),
            "message must describe this as a cycle: {}",
            error.message
        );
    }

    #[test]
    fn append_linked_functions_leaves_statements_unchanged_on_cycle_reject() {
        let module = build_module(
            0,
            "function isEven(n) { return n === 0 ? true : isOdd(n - 1); } function isOdd(n) { return n === 0 ? false : isEven(n - 1); }",
        );
        let mut statements: Vec<Statement> = Vec::new();
        let before = statements.clone();

        let result = append_linked_functions(&mut statements, &module);

        assert!(result.is_err());
        assert_eq!(
            statements, before,
            "a rejected call cycle must never partially mutate `statements`"
        );
    }

    #[test]
    fn append_linked_functions_supports_self_recursion() {
        // Verified independently of module-linking: `function f(n) { return
        // n <= 1 ? 1 : n * f(n - 1); }` runs and matches node byte-for-byte
        // in plain kali — self-recursion needs no forward-declaration
        // hoisting, since by the time `f`'s body calls `f`, `f` itself is
        // already fully declared. A self-edge must NOT be treated as a
        // cycle (which would wrongly reject a working program) and must NOT
        // block the topological sort.
        let module = build_module(
            0,
            "export function f(n) { return n <= 1 ? 1 : n * f(n - 1); }",
        );
        let mut statements: Vec<Statement> = Vec::new();

        append_linked_functions(&mut statements, &module)
            .expect("self-recursion must be accepted, not treated as a cycle");

        let f_clone = find_function(&statements, "__link0_f");
        // The self-call inside the clone's own body must still be renamed
        // to the mangled self-reference (this already worked before this
        // fix — the sibling-rename walk's `renames` map always included the
        // function's own name — this test pins it stays true post-fix).
        match &f_clone.body.body[..] {
            [Statement::ReturnStatement(r)] => match r.argument.as_ref().unwrap() {
                Expression::ConditionalExpression(cond) => match cond.alternate.as_ref() {
                    Expression::BinaryExpression(binary) => match binary.right.as_ref() {
                        Expression::CallExpression(call) => match &call.callee {
                            Expression::Identifier(name) => {
                                assert_eq!(name, "__link0_f")
                            }
                            other => panic!("expected an Identifier callee, got {other:?}"),
                        },
                        other => panic!("expected a CallExpression, got {other:?}"),
                    },
                    other => panic!("expected a BinaryExpression, got {other:?}"),
                },
                other => panic!("expected a ConditionalExpression, got {other:?}"),
            },
            other => panic!("expected a single ReturnStatement body, got {other:?}"),
        }
    }

    // ---- rewrite_namespace_uses (Task 6) ----

    /// Builds `(tempdir, provenance, modules)` for a `main.js` that does
    /// `import * as ns from "./lazy.js"` against a `lazy.js` fixture whose
    /// source declares:
    ///   export function lazyValue(a, b) { return a + b; }
    ///   function helper() { return 1n; }
    ///   export function f() { return helper(); }
    /// `helper` is deliberately a REAL function in the linked module (it
    /// exists in `all_functions`) but never exported — this gives
    /// `ns.helper()`/`typeof ns.helper` a genuine private-helper case to
    /// reject/fold against, distinct from `ns.notAnExport` (a name absent
    /// from the module entirely). Both must be treated identically by the
    /// rules under test.
    fn rewrite_fixture() -> (
        TempDir,
        NamespaceProvenance,
        BTreeMap<usize, LinkedModuleAst>,
    ) {
        let dir = tempdir().expect("tempdir");
        let module = write_module(
            &dir,
            "lazy.js",
            "export function lazyValue(a, b) { return a + b; } function helper() { return 1n; } export function f() { return helper(); }",
        );
        let main_js = dir.path().join("main.js");
        let source = r#"import * as ns from "./lazy.js";"#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        let loaded = load_linked_module(&module).expect("fixture module should load");
        let mut modules = BTreeMap::new();
        modules.insert(loaded.index, loaded);
        (dir, provenance, modules)
    }

    #[test]
    fn folded_string_literal_matches_parser_construction_for_function_and_undefined() {
        // Proves the quoting convention by PARSING an equivalent literal and
        // asserting `Expression` equality — not merely assuming it — so a
        // downstream `typeof x !== 'function'` comparison sees the same
        // literal shape the parser itself would build.
        for value in ["function", "undefined"] {
            let source = format!("(\"{value}\");");
            let statements = parse(&source);
            let parsed_literal = match &statements[..] {
                [Statement::ExpressionStatement(stmt)] => match stmt.expression.as_ref() {
                    Expression::ParenthesizedExpression(inner) => (*inner.expression).clone(),
                    other => panic!("expected a ParenthesizedExpression, got {other:?}"),
                },
                other => panic!("expected a single ExpressionStatement, got {other:?}"),
            };
            assert_eq!(
                parsed_literal,
                string_literal_expression(value),
                "constructed literal for {value:?} must equal the parser's own construction"
            );
        }
    }

    #[test]
    fn rewrite_namespace_uses_folds_typeof_of_export_to_function_literal() {
        let (_dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse("typeof ns.lazyValue;");
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        match &statements[..] {
            [Statement::ExpressionStatement(stmt)] => {
                assert_eq!(*stmt.expression, string_literal_expression("function"));
            }
            other => panic!("expected a single ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_folds_typeof_of_missing_member_to_undefined_literal() {
        let (_dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse("typeof ns.missing;");
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        match &statements[..] {
            [Statement::ExpressionStatement(stmt)] => {
                assert_eq!(*stmt.expression, string_literal_expression("undefined"));
            }
            other => panic!("expected a single ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_folds_typeof_of_private_helper_to_undefined_literal() {
        // `helper` is a real function of the linked module but not exported
        // — the sealed namespace genuinely has no `helper` property from the
        // outside, so `typeof ns.helper` must fold to "undefined" exactly
        // like a wholly-absent name, not reject.
        let (_dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse("typeof ns.helper;");
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        match &statements[..] {
            [Statement::ExpressionStatement(stmt)] => {
                assert_eq!(*stmt.expression, string_literal_expression("undefined"));
            }
            other => panic!("expected a single ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_rewrites_export_call_to_mangled_identifier() {
        let (_dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse("ns.lazyValue(a, b);");
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        match &statements[..] {
            [Statement::ExpressionStatement(stmt)] => match stmt.expression.as_ref() {
                Expression::CallExpression(call) => {
                    assert_eq!(
                        call.callee,
                        Expression::Identifier("__link0_lazyValue".to_string())
                    );
                    assert_eq!(
                        call.args,
                        vec![
                            Expression::Identifier("a".to_string()),
                            Expression::Identifier("b".to_string()),
                        ]
                    );
                }
                other => panic!("expected a CallExpression, got {other:?}"),
            },
            other => panic!("expected a single ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_preserves_await_wrapper_around_rewritten_call() {
        let (_dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse(
            r#"
                async function main() {
                    await ns.lazyValue(1, 2);
                }
            "#,
        );
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        let function = find_function(&statements, "main");
        match &function.body.body[..] {
            [Statement::ExpressionStatement(stmt)] => match stmt.expression.as_ref() {
                Expression::AwaitExpression(await_expr) => match &await_expr.argument {
                    Expression::CallExpression(call) => {
                        assert_eq!(
                            call.callee,
                            Expression::Identifier("__link0_lazyValue".to_string())
                        );
                    }
                    other => panic!("expected a CallExpression under await, got {other:?}"),
                },
                other => panic!("expected an AwaitExpression, got {other:?}"),
            },
            other => panic!("expected a single ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_rejects_call_to_non_exported_name() {
        let (dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse("ns.notAnExport();");
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1, "diagnostics: {diagnostics:?}");
        assert_eq!(diagnostics[0].code, Some(e5::FEATURE_UNAVAILABLE as u32));
        let expected_path = canonical(&dir, "lazy.js").display().to_string();
        assert!(
            diagnostics[0].message.contains(&expected_path),
            "message must name the module path: {}",
            diagnostics[0].message
        );
        assert!(
            diagnostics[0].message.contains("notAnExport"),
            "message must name the missing export: {}",
            diagnostics[0].message
        );
        // The rejected call site is left untouched.
        match &statements[..] {
            [Statement::ExpressionStatement(stmt)] => match stmt.expression.as_ref() {
                Expression::CallExpression(call) => {
                    assert_eq!(
                        call.callee,
                        Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("ns".to_string()),
                            property: "notAnExport".to_string(),
                            computed_index: None,
                        }))
                    );
                }
                other => panic!("expected a CallExpression, got {other:?}"),
            },
            other => panic!("expected a single ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_rejects_call_to_private_helper() {
        let (dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse("ns.helper();");
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1, "diagnostics: {diagnostics:?}");
        assert_eq!(diagnostics[0].code, Some(e5::FEATURE_UNAVAILABLE as u32));
        let expected_path = canonical(&dir, "lazy.js").display().to_string();
        assert!(diagnostics[0].message.contains(&expected_path));
        assert!(
            diagnostics[0].message.contains("helper"),
            "message must name the private helper: {}",
            diagnostics[0].message
        );
    }

    #[test]
    fn rewrite_namespace_uses_rejects_computed_call_access() {
        let (dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse(r#"ns["lazyValue"](1, 2);"#);
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1, "diagnostics: {diagnostics:?}");
        assert_eq!(diagnostics[0].code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(diagnostics[0].message.contains("computed member access"));
        let expected_path = canonical(&dir, "lazy.js").display().to_string();
        assert!(diagnostics[0].message.contains(&expected_path));
    }

    #[test]
    fn rewrite_namespace_uses_rejects_computed_typeof_access() {
        let (dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse(r#"typeof ns["lazyValue"];"#);
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1, "diagnostics: {diagnostics:?}");
        assert_eq!(diagnostics[0].code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(diagnostics[0].message.contains("computed member access"));
        let expected_path = canonical(&dir, "lazy.js").display().to_string();
        assert!(diagnostics[0].message.contains(&expected_path));
        // Left unfolded — still a `typeof` over the untouched computed member.
        match &statements[..] {
            [Statement::ExpressionStatement(stmt)] => {
                assert!(matches!(
                    stmt.expression.as_ref(),
                    Expression::UnaryExpression(_)
                ));
            }
            other => panic!("expected a single ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_reaches_nested_arrow_and_call_arg_positions() {
        // Proves the deep pre-order walk reaches function bodies, arrow
        // bodies, and call-argument positions — the exact class of place
        // Task 5's walk was twice found to miss.
        let (_dir, provenance, modules) = rewrite_fixture();
        let mut statements = parse(
            r#"
                function outer() {
                    const check = () => typeof ns.lazyValue;
                    return other(ns.lazyValue(1, 2), check());
                }
            "#,
        );
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        let outer = find_function(&statements, "outer");
        match &outer.body.body[0] {
            Statement::VariableDeclaration(decl) => match &decl.declarations[0].init {
                Some(Expression::ArrowFunctionExpression(arrow)) => {
                    assert_eq!(arrow.body, string_literal_expression("function"));
                }
                other => panic!("expected an arrow function init, got {other:?}"),
            },
            other => panic!("expected a VariableDeclaration, got {other:?}"),
        }
        match &outer.body.body[1] {
            Statement::ReturnStatement(stmt) => match stmt.argument.as_ref().unwrap() {
                Expression::CallExpression(call) => match &call.args[0] {
                    Expression::CallExpression(inner) => {
                        assert_eq!(
                            inner.callee,
                            Expression::Identifier("__link0_lazyValue".to_string())
                        );
                    }
                    other => panic!("expected a nested CallExpression, got {other:?}"),
                },
                other => panic!("expected a CallExpression, got {other:?}"),
            },
            other => panic!("expected a ReturnStatement, got {other:?}"),
        }
    }

    #[test]
    fn rewrite_namespace_uses_leaves_namespace_free_program_unchanged() {
        let (_dir, provenance, modules) = rewrite_fixture();
        let source = r#"
            function add(a, b) { return a + b; }
            const x = typeof add;
            console.log(add(1, 2));
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        rewrite_namespace_uses(&mut statements, &provenance, &modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(
            statements, before,
            "a namespace-free program must be left byte-identical"
        );
    }

    // ---- link_provable_module_namespaces (Task 7) ----

    fn find_diagnostic_containing<'a>(
        diagnostics: &'a [Diagnostic],
        needle: &str,
    ) -> &'a Diagnostic {
        diagnostics
            .iter()
            .find(|d| d.message.contains(needle))
            .unwrap_or_else(|| {
                panic!("expected a diagnostic containing {needle:?}, got {diagnostics:?}")
            })
    }

    #[test]
    fn link_provable_module_namespaces_is_a_no_op_without_namespace_bindings() {
        let (_dir, main_js) = fixture_dir();
        let source = "function add(a, b) { return a + b; } console.log(add(1, 2));";
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(
            statements, before,
            "a namespace-free source must be left byte-identical (no-op guarantee)"
        );
    }

    #[test]
    fn link_provable_module_namespaces_leaves_bare_dynamic_import_statement_untouched() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                await import("./lazy.js");
            }
            main();
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(
            statements, before,
            "a bindingless `await import(...)` statement must stay untouched"
        );
    }

    #[test]
    fn link_provable_module_namespaces_allows_export_call_end_to_end() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            import * as ns from "./util.js";
            console.log(ns.greet());
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        // The linked module's `greet` clone is appended, and the call site
        // is rewritten to the mangled identifier — no leftover `ns` at all.
        let _ = find_function(&statements, "__link0_greet");
        let call_statement = statements
            .iter()
            .find(|statement| matches!(statement, Statement::ExpressionStatement(_)))
            .expect("expected an ExpressionStatement somewhere in statements");
        match call_statement {
            Statement::ExpressionStatement(stmt) => match stmt.expression.as_ref() {
                Expression::CallExpression(call) => {
                    assert_eq!(call.args.len(), 1);
                    match &call.args[0] {
                        Expression::CallExpression(inner) => {
                            assert_eq!(
                                inner.callee,
                                Expression::Identifier("__link0_greet".to_string())
                            );
                        }
                        other => panic!("expected a CallExpression argument, got {other:?}"),
                    }
                }
                other => panic!("expected a CallExpression, got {other:?}"),
            },
            other => panic!("expected an ExpressionStatement, got {other:?}"),
        }
    }

    #[test]
    fn link_provable_module_namespaces_denies_console_log_of_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import("./lazy.js");
                console.log(chunk);
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "chunk");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_string_coercion_of_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import("./lazy.js");
                const s = chunk + '';
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "chunk");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_alias_copy_of_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import("./lazy.js");
                const alias = chunk;
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "chunk");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_argument_escape_of_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            function f(x) {}
            async function main() {
                const chunk = await import("./lazy.js");
                f(chunk);
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "chunk");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_return_escape_of_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import("./lazy.js");
                return chunk;
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "chunk");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_rejects_shadowing_of_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import("./lazy.js");
                {
                    const chunk = 5;
                }
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "chunk");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            diagnostic.message.contains("shadow"),
            "message must explain the shadow: {}",
            diagnostic.message
        );
    }

    #[test]
    fn link_provable_module_namespaces_rejects_parameter_shadowing_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            function f(chunk) { return chunk; }
            async function main() {
                const chunk = await import("./lazy.js");
                f(1);
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "chunk");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
        assert!(
            diagnostic.message.contains("shadow"),
            "message must explain the shadow: {}",
            diagnostic.message
        );
    }

    #[test]
    fn link_provable_module_namespaces_leaves_statements_untouched_on_shadow_reject() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import("./lazy.js");
                {
                    const chunk = 5;
                }
            }
            main();
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics));
        assert_eq!(
            statements, before,
            "a shadow reject must stop the pipeline before any mutation (load/append/rewrite never run)"
        );
    }

    // ---- C1: the specifier fold is scope-blind without the bound-once gate ----
    //
    // `collect_namespace_provenance` seeds a function body's const map with
    // `module_consts.clone()` and NEVER removes a name the function rebinds.
    // Pre-fix, `fold_import_specifier`'s `Identifier` arm read that stale
    // module-scope value straight out of the map, so a PARAM (or a shadowing
    // `let`) named the same as a module-scope specifier const silently linked
    // the WRONG module — a real, distinguishable, wrong-value divergence from
    // node (the reviewer's probe: kali printed a.js's `111`, node printed
    // b.js's `222n`), with exit 0 and no diagnostic. The bound-exactly-once
    // allowlist makes every one of these unprovable instead, so no provenance
    // is earned (and C2's default-deny below then rejects the USE).

    #[test]
    fn const_await_import_whose_specifier_const_is_shadowed_by_a_param_is_not_proven() {
        let (_dir, main_js) = fixture_dir();
        // `spec` is bound TWICE: the module-scope const, and `load`'s param.
        // The param wins at the `await import(spec)` site under real JS
        // scoping, so the module-scope const's value ("./util.js") is NOT
        // provably what this specifier evaluates to.
        let source = r#"
            const spec = "./util.js";
            async function load(spec) {
                const c = await import(spec);
                return c.greet();
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            None,
            "a param-shadowed specifier const must not fold — it would link the WRONG module"
        );
    }

    #[test]
    fn const_await_import_whose_specifier_const_is_shadowed_by_a_let_is_not_proven() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            const spec = "./util.js";
            async function load() {
                let spec = pickAtRuntime();
                const c = await import(spec);
                return c.greet();
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            None,
            "a let-shadowed specifier const must not fold — it would link the WRONG module"
        );
    }

    #[test]
    fn object_freeze_fold_with_a_shadowed_object_binding_is_not_proven() {
        let (_dir, main_js) = fixture_dir();
        // `Object` is locally bound — `Object.freeze(...)` here is NOT
        // provably the real global's `freeze`, so the fold through it must
        // fail closed (the `is_object_freeze_callee` half of C1).
        let source = r#"
            async function main(Object) {
                const c = await import(Object.freeze("./lazy.js"));
                return c.lazyValue();
            }
        "#;
        let statements = parse(source);
        let provenance = collect_namespace_provenance(&main_js, source, &statements);
        assert_eq!(
            provenance.bindings.get("c"),
            None,
            "a shadowed `Object` must not license the Object.freeze fold"
        );
    }

    #[test]
    fn link_provable_module_namespaces_denies_param_shadowed_specifier_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            const spec = "./util.js";
            async function load(spec) {
                const c = await import(spec);
                return c.greet();
            }
            load("./lazy.js");
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        // Fail-CLOSED (a reject), never a silent link to the wrong module.
        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "'c'");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    // ---- C2: default-deny every USED namespace-shaped binding with no provenance ----
    //
    // Each of these three shapes is outside `collect_namespace_provenance`'s
    // reach or fold, so `provenance.bindings` is EMPTY — pre-fix, the whole
    // pipeline early-returned on `bindings.is_empty()` and the program fell
    // through to the pre-stage silent fail-open (printing `0`, and folding
    // `typeof c.member` to `0` too, where node prints `42n` / `"function"`).

    #[test]
    fn link_provable_module_namespaces_denies_block_nested_unproven_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                if (true) {
                    const c = await import("./util.js");
                    console.log(c.greet());
                }
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "'c'");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_let_bound_unproven_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                let c = await import("./util.js");
                console.log(c.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "'c'");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_non_foldable_specifier_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        // A ternary specifier: not a shape `fold_import_specifier` proves,
        // even though BOTH arms happen to name the same real module — the
        // fold is structural, and "both arms agree" is not a proof it makes.
        let source = r#"
            async function main() {
                const c = await import(true ? "./util.js" : "./util.js");
                console.log(c.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "'c'");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_typeof_of_unproven_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        // The typeof-fold half of C2: pre-fix this silently folded to the
        // pre-stage value (`0`), where node says `"function"`.
        let source = r#"
            async function main() {
                let c = await import("./util.js");
                console.log(typeof c.greet);
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "'c'");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    // ---- C2 scoping: the deny must NOT over-reach (each of these is a green lane) ----

    #[test]
    fn link_provable_module_namespaces_does_not_deny_unused_unproven_namespace_binding() {
        let (_dir, main_js) = fixture_dir();
        // Un-provable (`let`-bound) AND never used — harmless, so it must
        // stay a no-op. Denying it would create new reds for no soundness
        // gain (there is no value to leak if nothing reads it).
        let source = r#"
            async function main() {
                let c = await import("./util.js");
            }
            main();
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(statements, before);
    }

    #[test]
    fn link_provable_module_namespaces_does_not_deny_non_relative_namespace_import() {
        let (_dir, main_js) = fixture_dir();
        // `import * as path from "node:path"` is a SEPARATE, pre-existing
        // lane (the `node_api_surface` tests) — a bare/`node:` specifier is
        // never a candidate for this pass's deny, used or not.
        let source = r#"
            import * as path from "node:path";
            export function describe() { return typeof path.basename === 'function' ? 0 : 1; }
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(statements, before);
    }

    // ---- I1: an unused proven binding must not eagerly load + purity-gate ----

    #[test]
    fn link_provable_module_namespaces_does_not_load_module_for_unused_namespace_binding() {
        let (dir, main_js) = fixture_dir();
        // `impure.js` would FAIL the purity gate (a top-level `export const`
        // is not a plain `export function`) — but nothing in the entry uses
        // `ns`, so the module must never be loaded or gated at all. Node
        // runs this program fine; a hard E5506 build failure here was the
        // review's I1 finding.
        fs::write(dir.path().join("impure.js"), "export const VERSION = 1n;\n")
            .expect("write impure.js");
        let source = r#"
            import * as ns from "./impure.js";
            console.log("hello");
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(
            statements, before,
            "an unused namespace binding must leave the program untouched (no load, no append)"
        );
    }

    #[test]
    fn link_provable_module_namespaces_still_loads_module_for_used_namespace_binding() {
        let (dir, main_js) = fixture_dir();
        // The I1 gate's positive control: the SAME impure module, but now
        // with a real member use site — it MUST still be loaded and rejected
        // by the purity gate (the gate narrows WHEN a module is loaded, never
        // WHETHER a used one is checked).
        fs::write(dir.path().join("impure.js"), "export const VERSION = 1n;\n")
            .expect("write impure.js");
        let source = r#"
            import * as ns from "./impure.js";
            console.log(ns.VERSION());
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "impure.js");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    // ---- Minor: the leftover-deny must not walk the SPLICED-IN clone bodies ----

    #[test]
    fn link_provable_module_namespaces_allows_linked_module_local_sharing_the_binding_name() {
        let (dir, main_js) = fixture_dir();
        // `lib.js`'s `calc` has an internal local named `ns` — completely
        // unrelated to the ENTRY's `ns` namespace binding (different file,
        // different scope). Pre-fix, `deny_unrewritten_uses` walked the
        // spliced-in clone body too and E5506'd that innocent local.
        fs::write(
            dir.path().join("lib.js"),
            "export function calc() { const ns = 2n; return ns; }\n",
        )
        .expect("write lib.js");
        let source = r#"
            import * as ns from "./lib.js";
            console.log(ns.calc());
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(
            diagnostics.is_empty(),
            "a linked module's own internal local must not be denied: {diagnostics:?}"
        );
        let _ = find_function(&statements, "__link0_calc");
    }

    // ---- C2 remainder (second review round): ImportExpression position allowlist ----
    //
    // Each shape below was a LIVE fail-open on the pre-fix code: neither
    // `collect_namespace_provenance` nor `deny_unproven_namespace_binding_candidates` ever
    // recorded an `ImportExpression` reached through anything other than a
    // `VariableDeclarator.init` or a relative `import * as` specifier, so these five silently fell
    // through to the pre-stage `0` — exit 0, no diagnostic. Post-fix,
    // `deny_import_expressions_outside_allowlist` denies every one of them (E5506) by SYNTACTIC
    // POSITION, unconditionally.

    #[test]
    fn link_provable_module_namespaces_denies_assignment_form_dynamic_import() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                let c;
                c = await import("./util.js");
                console.log(c.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_assignment_form_dynamic_import_typeof() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                let c;
                c = await import("./util.js");
                console.log(typeof c.greet);
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_inline_member_access_on_import_result() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                console.log((await import("./util.js")).greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_sequence_expression_init_dynamic_import() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const c = (0, await import("./util.js"));
                console.log(c.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_member_sink_dynamic_import() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const box = { m: null };
                box.m = await import("./util.js");
                console.log(box.m.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    // ---- C2 remainder, GREEN guards: the new allowlist gate must not over-reach ----

    #[test]
    fn link_provable_module_namespaces_position_allowlist_allows_bindingless_statement_form() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                await import("./lazy.js");
            }
            main();
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(statements, before);
    }

    #[test]
    fn link_provable_module_namespaces_position_allowlist_allows_proven_const_lane() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import("./util.js");
                console.log(chunk.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        let _ = find_function(&statements, "__link0_greet");
    }

    #[test]
    fn link_provable_module_namespaces_position_allowlist_allows_unused_unprovable_binding() {
        let (_dir, main_js) = fixture_dir();
        // Declarator-init position IS on the allowlist regardless of `kind` — whether it earns
        // real provenance (and the separate "unused is harmless" exemption) is the pre-existing
        // pipeline's job, not this position gate's.
        let source = r#"
            async function main() {
                let c = await import("./util.js");
            }
            main();
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(statements, before);
    }

    #[test]
    fn link_provable_module_namespaces_position_allowlist_allows_object_freeze_wrapped_specifier() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const chunk = await import(Object.freeze("./util.js"));
                console.log(chunk.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        let _ = find_function(&statements, "__link0_greet");
    }

    // ---- Stage 5 sibling (third review round): a BARE, non-awaited import() laundered through
    // a binding escaped both the position allowlist and the C2 candidate census, since `await`
    // applied to a separately-bound identifier never syntactically wraps the `ImportExpression`.
    // Every probe below was a LIVE fail-open on the pre-fix code — exit 0, no diagnostic, silent
    // `0` instead of the real linked value (probe-proven on a fresh binary; util.js exports
    // `greet() { return 42; }`).

    #[test]
    fn link_provable_module_namespaces_denies_bare_import_laundered_through_await_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const p = import("./util.js");
                const c = await p;
                console.log(c.greet());
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn link_provable_module_namespaces_denies_bare_import_laundered_through_await_binding_typeof() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const p = import("./util.js");
                const c = await p;
                console.log(typeof c.greet);
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn link_provable_module_namespaces_denies_bare_import_inline_await_of_binding() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const p = import("./util.js");
                console.log(typeof (await p).greet);
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn link_provable_module_namespaces_denies_bare_import_assignment_form() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                let p;
                p = import("./util.js");
                const c = await p;
                c.greet();
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_bare_import_inside_promise_all_array() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const mods = await Promise.all([import("./util.js")]);
                mods[0].greet();
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_bare_import_inside_promise_resolve_argument() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const c = await Promise.resolve(import("./util.js"));
                c.greet();
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    #[test]
    fn link_provable_module_namespaces_denies_bare_import_then_callback_receiver() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                import("./util.js").then(c => console.log(c.greet()));
            }
            main();
        "#;
        let mut statements = parse(source);
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(has_errors(&diagnostics), "diagnostics: {diagnostics:?}");
        let diagnostic = find_diagnostic_containing(&diagnostics, "import(...)");
        assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    }

    // ---- positive control: an unused bare import() declarator init stays harmless ----

    #[test]
    fn link_provable_module_namespaces_leaves_unused_bare_import_declarator_untouched() {
        let (_dir, main_js) = fixture_dir();
        let source = r#"
            async function main() {
                const p = import("./util.js");
                console.log("main loaded");
            }
            main();
        "#;
        let mut statements = parse(source);
        let before = statements.clone();
        let mut diagnostics = Vec::new();

        link_provable_module_namespaces(&main_js, source, &mut statements, &mut diagnostics);

        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
        assert_eq!(statements, before);
    }
}

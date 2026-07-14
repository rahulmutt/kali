//! Pre-resolver AST module-linking pass (throw-fallout Stage 5).
//!
//! Detects provenance-proven module-namespace bindings (`import * as ns from
//! "./x"` and `const c = await import(<foldable specifier>)`) and records
//! which on-disk module each binding provably refers to. Everything outside
//! the proven lane yields NO provenance (the binding is simply absent from
//! the map) — fail-closed by construction; later stages (Tasks 6-7) are
//! responsible for rejecting proven-absent uses.
//!
//! This module only *detects* provenance. Module loading, AST cloning under
//! mangled `__link{N}_{name}` names, and use-site rewriting are later tasks.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kali_ast::*;
use kali_common::FileId;
use kali_error::{_error_codes::e5, Diagnostic};
use kali_lexer::{Lexer, Token, TokenType};
use kali_parser::Parser;

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
        let Some(specifier) = fold_import_specifier(specifier_expr, visible_consts) else {
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
fn fold_import_specifier(expr: &Expression, consts: &BTreeMap<String, String>) -> Option<String> {
    match expr {
        Expression::Literal(LiteralValue::String(value)) => Some(unquote(value)),
        Expression::ParenthesizedExpression(inner) => {
            fold_import_specifier(&inner.expression, consts)
        }
        Expression::SequenceExpression(seq) => seq
            .expressions
            .last()
            .and_then(|last| fold_import_specifier(last, consts)),
        Expression::CallExpression(call) => {
            if call.args.len() == 1 && is_object_freeze_callee(&call.callee) {
                fold_import_specifier(&call.args[0], consts)
            } else {
                None
            }
        }
        Expression::Identifier(name) => consts.get(name).cloned(),
        Expression::BinaryExpression(binary) => match binary.operator.as_str() {
            "+" => {
                let left = fold_import_specifier(&binary.left, consts)?;
                let right = fold_import_specifier(&binary.right, consts)?;
                Some(format!("{left}{right}"))
            }
            "??" | "&&" | "||" => fold_logical(binary, consts),
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
fn fold_logical(binary: &BinaryExpression, consts: &BTreeMap<String, String>) -> Option<String> {
    let (nullish, truthy) = literal_truthiness(&binary.left)?;
    match binary.operator.as_str() {
        "??" => {
            if nullish {
                fold_import_specifier(&binary.right, consts)
            } else {
                fold_import_specifier(&binary.left, consts)
            }
        }
        "&&" => {
            if truthy {
                fold_import_specifier(&binary.right, consts)
            } else {
                fold_import_specifier(&binary.left, consts)
            }
        }
        "||" => {
            if truthy {
                fold_import_specifier(&binary.left, consts)
            } else {
                fold_import_specifier(&binary.right, consts)
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

fn is_object_freeze_callee(callee: &Expression) -> bool {
    // Intentionally broad: matches `property == "freeze"` for either dot
    // (`Object.freeze`) or computed (`Object["freeze"]`) access without
    // further discriminating the two — both are the same provable callee
    // for this fold's purposes, so no additional check is warranted.
    matches!(
        callee,
        Expression::MemberExpression(member)
            if member.property == "freeze"
                && matches!(&member.object, Expression::Identifier(name) if name == "Object")
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
}

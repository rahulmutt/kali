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

/// Appends mangled clones of `module.all_functions` to `statements`.
/// Mangle: `__link{module.index}_{original_name}`. Sibling references inside
/// cloned bodies are renamed to their mangled forms.
/// Err = mangled-name collision with an already-declared entry name (E5506).
///
/// Ordering: the collision guard runs first, over every function in
/// `module.all_functions`, and the sibling-rename walk (which can also
/// fail — a bare, non-call reference to a sibling name) runs entirely
/// against local clones before anything is appended. `statements` is only
/// ever mutated once, via a single `extend` at the very end, after every
/// fallible step has already succeeded — so any `Err` return leaves
/// `statements` byte-identical to how it was passed in.
pub fn append_linked_functions(
    statements: &mut Vec<Statement>,
    module: &LinkedModuleAst,
) -> Result<(), Diagnostic> {
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

    let mut cloned: Vec<Statement> = Vec::with_capacity(module.all_functions.len());
    for (name, function) in &module.all_functions {
        let mut clone = function.clone();
        clone.name = renames.get(name).cloned().expect(
            "name is a key of module.all_functions, and renames is built from those same keys",
        );
        walk_block(&mut clone.body, &renames)?;
        cloned.push(Statement::FunctionDeclaration(clone));
    }

    statements.extend(cloned);
    Ok(())
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
        // `declarator.id` is the name being BOUND, not a reference.
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
        // `method.name` and `method.params` are declarations, not references.
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
                // `handler.param` is the caught-error BINDING name, not a reference.
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
        // A nested function declaration: its own `name`/`params` are
        // declarations, not references; its body is walked in the same
        // (unscoped) manner as everything else — see the module-level rename
        // walk's doc comment for the shadowing caveat this shares with
        // `monomorphize::rewrite_callees_in_body`.
        Statement::FunctionDeclaration(function) => walk_block(&mut function.body, renames),
        Statement::ClassDeclaration(class) => walk_class_body(&mut class.body, renames),
        Statement::VariableDeclaration(decl) => walk_var_decl(decl, renames),
        // Plain string fields only (specifiers/source) — no `Expression`
        // content, and the parser only ever produces this at module top
        // level (never inside a function body we would walk into here).
        Statement::ImportDeclaration(_) => Ok(()),
        Statement::ExportAll(_) => Ok(()),
        // `ExportSpecifier { local, exported }` are plain re-export name
        // strings, not `Expression` positions; same top-level-only caveat as
        // `ImportDeclaration` above.
        Statement::ExportNamed(_) => Ok(()),
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => walk_expression(expr, renames, false),
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                walk_block(&mut function.body, renames)
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                walk_class_body(&mut class.body, renames)
            }
        },
        Statement::EnumDeclaration(decl) => {
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
            // `function.id`/`function.params` are declarations, not references.
            if let Some(body) = &mut function.body {
                walk_block(body, renames)?;
            }
            Ok(())
        }
        Expression::ArrowFunctionExpression(arrow) => {
            // `arrow.params` are declarations, not references.
            walk_expression(&mut arrow.body, renames, false)
        }
        Expression::ClassExpression(class) => {
            // `class.id` is a declaration, not a reference.
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

    // ---- append_linked_functions (Task 5) ----

    /// Builds a `LinkedModuleAst` directly from source, without touching
    /// disk — `append_linked_functions` only reads `module.all_functions`
    /// and `module.index`, so `exports` is left empty (irrelevant here).
    fn build_module(index: usize, source: &str) -> LinkedModuleAst {
        let mut all_functions = BTreeMap::new();
        for statement in parse(source) {
            if let Statement::FunctionDeclaration(function) = statement {
                all_functions.insert(function.name.clone(), function);
            }
        }
        LinkedModuleAst {
            index,
            exports: BTreeMap::new(),
            all_functions,
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
}

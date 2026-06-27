use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use kali_ast::{ExportAllDeclaration, ExportNamedDeclaration, ImportDeclaration, Statement};
use kali_common::FileId;
use kali_error::{_error_codes::e5, _error_codes::e8, Diagnostic};
use kali_lexer::Lexer;
use kali_parser::Parser;

use super::report::{location_sort_key, EffectAnalysisContext, EffectInference, ObservedEffect};
use super::scan::scan_tokens_for_effects;

const SOURCE_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "d.ts", "d.mts", "d.cts",
];

/// Infer effects for one or more source roots, following local relative imports.
pub fn infer_effects_from_roots(
    roots: &[PathBuf],
    context: EffectAnalysisContext,
) -> Result<EffectInference, Vec<Diagnostic>> {
    let mut visited = HashSet::<PathBuf>::new();
    let mut effects = Vec::<ObservedEffect>::new();
    let mut dynamic_reasons = BTreeSet::<String>::new();

    for root in roots {
        visit_source_root(
            root,
            &mut visited,
            &mut effects,
            &mut dynamic_reasons,
            &context,
        )?;
    }

    let mut effects = dedupe_effects(effects);
    effects.sort_by(effect_sort_cmp);

    Ok(EffectInference {
        effects,
        dynamic_reasons: dynamic_reasons.into_iter().collect(),
    })
}

fn visit_source_root(
    root: &Path,
    visited: &mut HashSet<PathBuf>,
    effects: &mut Vec<ObservedEffect>,
    dynamic_reasons: &mut BTreeSet<String>,
    context: &EffectAnalysisContext,
) -> Result<(), Vec<Diagnostic>> {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    if !visited.insert(canonical) {
        return Ok(());
    }

    let source = fs::read_to_string(root).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!("failed to read source file '{}': {}", root.display(), error),
        )]
    })?;

    let lexer = Lexer::new(FileId::new(0), source.clone());
    let lexed = lexer.lex_all();
    if has_errors(&lexed.diagnostics) {
        return Err(lexed.diagnostics);
    }

    let mut parser = Parser::new(FileId::new(0), lexed.tokens.clone());
    let parsed = parser.parse(Some(root.to_string_lossy().to_string()));
    if has_errors(&parsed.diagnostics) {
        return Err(parsed.diagnostics);
    }

    let file_effects =
        scan_tokens_for_effects(root, &source, &lexed.tokens, dynamic_reasons, context)?;
    effects.extend(file_effects);

    for import_spec in collect_relative_imports(&parsed.statements) {
        if import_spec.starts_with('.') || import_spec.starts_with('/') {
            if let Some(resolved) = resolve_relative_import(root, &import_spec) {
                visit_source_root(&resolved, visited, effects, dynamic_reasons, context)?;
            } else {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_MODULE_SPECIFIER as u32,
                    format!(
                        "failed to resolve relative import '{}' from '{}'",
                        import_spec,
                        root.display()
                    ),
                )]);
            }
        }
    }

    let _ = context;
    Ok(())
}

fn collect_relative_imports(statements: &[Statement]) -> Vec<String> {
    let mut imports = Vec::new();
    for statement in statements {
        match statement {
            Statement::ImportDeclaration(ImportDeclaration { source, .. }) => {
                if is_relative_specifier(source) {
                    imports.push(source.clone());
                }
            }
            Statement::ExportAll(ExportAllDeclaration { source }) => {
                if is_relative_specifier(source) {
                    imports.push(source.clone());
                }
            }
            Statement::ExportNamed(ExportNamedDeclaration {
                source: Some(source),
                ..
            }) => {
                if is_relative_specifier(source) {
                    imports.push(source.clone());
                }
            }
            Statement::ExportNamed(ExportNamedDeclaration { source: None, .. })
            | Statement::ExportDefault(_)
            | Statement::EnumDeclaration(_)
            | Statement::TypeAliasDeclaration(_)
            | Statement::InterfaceDeclaration(_)
            | Statement::ExpressionStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::WithStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::LabeledStatement(_)
            | Statement::IfStatement(_)
            | Statement::SwitchStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::TryStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::BlockStatement(_)
            | Statement::ForStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::WhileStatement(_)
            | Statement::DoWhileStatement(_)
            | Statement::FunctionDeclaration(_)
            | Statement::ClassDeclaration(_)
            | Statement::VariableDeclaration(_) => {}
        }
    }
    imports
}

fn is_relative_specifier(spec: &str) -> bool {
    spec.starts_with('.') || spec.starts_with('/')
}

pub(crate) fn resolve_relative_import(current_file: &Path, spec: &str) -> Option<PathBuf> {
    let base_dir = current_file.parent()?;
    let raw = if spec.starts_with('/') {
        PathBuf::from(spec)
    } else {
        base_dir.join(spec)
    };

    if raw.is_file() {
        return Some(raw);
    }

    if let Some(resolved) = resolve_with_extensions(&raw) {
        return Some(resolved);
    }

    if raw.is_dir() {
        let candidate = "index";
        let indexed = raw.join(candidate);
        if let Some(resolved) = resolve_with_extensions(&indexed) {
            return Some(resolved);
        }
    }

    None
}

fn resolve_with_extensions(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }

    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        if matches!(
            ext,
            "ts" | "tsx"
                | "js"
                | "jsx"
                | "mts"
                | "cts"
                | "mjs"
                | "cjs"
                | "d.ts"
                | "d.mts"
                | "d.cts"
        ) {
            return None;
        }
    }

    for extension in SOURCE_EXTENSIONS {
        let candidate = path.with_extension(extension);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn dedupe_effects(mut effects: Vec<ObservedEffect>) -> Vec<ObservedEffect> {
    let mut seen = HashSet::<(String, String, usize, usize, Option<String>)>::new();
    effects.retain(|effect| {
        let key = (
            effect.kind.clone(),
            effect.location.file.clone(),
            effect.location.line,
            effect.location.column,
            effect.target.clone(),
        );
        seen.insert(key)
    });
    effects
}

fn effect_sort_cmp(a: &ObservedEffect, b: &ObservedEffect) -> std::cmp::Ordering {
    a.kind
        .cmp(&b.kind)
        .then_with(|| location_sort_key(&a.location, &b.location))
        .then_with(|| a.target.cmp(&b.target))
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.is_error())
}

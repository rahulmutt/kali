mod compile;
mod entrypoint;
mod eval;
mod helpers;
mod metadata;
mod paths;
mod wit;

pub use compile::{
    build_mode_from_flags, build_source_file, check_source_file, compile_source_file,
    compile_source_file_with_cache_state, compile_source_file_with_cache_state_and_profile_data,
    compile_source_file_with_cache_state_and_profile_data_and_validation,
    compile_source_file_with_specialization_cap,
    compile_source_file_with_specialization_cap_and_profile_data,
    compile_source_file_with_specialization_cap_and_profile_data_and_validation,
    compile_source_file_with_specialization_cap_and_validation, load_profile_data_file,
    normalize_compiler_source, read_compiler_source_file, validate_runtime_profiles,
    BuildMode, BuildOutput, CompileOutput,
};
pub use entrypoint::reject_async_and_generator_class_methods_in_runtime_entrypoint;
pub use eval::{discover_dynamic_import_targets, DynamicImportTarget};
pub use metadata::{
    append_metadata_section, build_artifact_metadata, serialize_artifact_metadata,
    validate_build_result_value, ArtifactMetadata,
};
pub use paths::{
    binding_package_manifest_output_path_for, bundle_chunk_output_dir_for, bundle_output_paths_for,
    capi_output_paths_for, component_output_paths_for, executable_output_path_for,
    library_output_paths_for,
};
pub use wit::{browser_bundle_source_map, library_wit_for, LibraryExport};
#[cfg(test)]
pub(crate) use compile::{incremental_cache_path, profile_data_hash};
#[cfg(test)]
pub(crate) use metadata::validate_artifact_metadata_value;
pub(crate) use entrypoint::generator_function_unavailable_message;
pub(crate) use helpers::*; // parse_source_file, source_stem, has_errors, etc. for sibling modules

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use kali_ast::{BlockStatement, ExportDefaultDeclaration, Expression, LiteralValue, OptionalChainInner, Statement, VariableDeclaration};

use kali_error::{_error_codes::e5, _error_codes::e8, Diagnostic};

use crate::ApiSurface;

pub fn collect_library_exports(
    source_path: impl AsRef<Path>,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let mut visited = BTreeSet::new();
    collect_library_exports_from_source_path_with_context(
        source_path,
        api_surface,
        runtime_profiles,
        &mut visited,
    )
}

fn collect_library_exports_from_source_path_with_context(
    source_path: &Path,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    visited: &mut BTreeSet<PathBuf>,
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let canonical_source_path =
        fs::canonicalize(source_path).unwrap_or_else(|_| source_path.to_path_buf());
    if !visited.insert(canonical_source_path.clone()) {
        return Err(vec![invalid_export_surface(
            source_path,
            "cyclic re-exported surfaces are not statically known yet",
        )]);
    }

    let parsed = match parse_source_file(source_path) {
        Ok(parsed) => parsed,
        Err(error) => {
            visited.remove(&canonical_source_path);
            return Err(error);
        }
    };

    let exports = collect_library_exports_from_statements_with_context(
        &parsed,
        source_path,
        api_surface,
        runtime_profiles,
        visited,
    );
    visited.remove(&canonical_source_path);
    exports
}

fn collect_library_exports_from_statements_with_context(
    statements: &[Statement],
    source_path: &Path,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    visited: &mut BTreeSet<PathBuf>,
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let declared_function_signatures =
        collect_declared_function_signatures(statements, source_path, &mut diagnostics);
    let mut exports = BTreeMap::<String, String>::new();
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                let signature = infer_function_signature(&func.params, &func.body, func.is_async);
                if exports.insert(func.name.clone(), signature).is_some() {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!("duplicate export name `{}`", func.name),
                    ));
                }
            }
            Statement::ExportAll(declaration) => {
                let Some(resolved_source_path) = resolve_library_export_source_path(
                    source_path,
                    &declaration.source,
                    api_surface,
                ) else {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!(
                            "re-export source `{}` could not be resolved",
                            declaration.source
                        ),
                    ));
                    continue;
                };

                let reexported_exports = match collect_library_exports_from_source_path_with_context(
                    &resolved_source_path,
                    api_surface,
                    runtime_profiles,
                    visited,
                ) {
                    Ok(exports) => exports,
                    Err(mut error_diagnostics) => {
                        diagnostics.append(&mut error_diagnostics);
                        continue;
                    }
                };

                for export in reexported_exports {
                    if export.name == "default" {
                        continue;
                    }
                    if exports
                        .insert(export.name.clone(), export.signature)
                        .is_some()
                    {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{}`", export.name),
                        ));
                    }
                }
            }
            Statement::ExportNamed(declaration) => {
                if let Some(source) = declaration.source.as_ref() {
                    let Some(resolved_source_path) =
                        resolve_library_export_source_path(source_path, source, api_surface)
                    else {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("re-export source `{source}` could not be resolved"),
                        ));
                        continue;
                    };

                    let reexported_exports =
                        match collect_library_exports_from_source_path_with_context(
                            &resolved_source_path,
                            api_surface,
                            runtime_profiles,
                            visited,
                        ) {
                            Ok(exports) => exports,
                            Err(mut error_diagnostics) => {
                                diagnostics.append(&mut error_diagnostics);
                                continue;
                            }
                        };
                    let reexported_map = reexported_exports
                        .into_iter()
                        .map(|export| (export.name, export.signature))
                        .collect::<BTreeMap<_, _>>();

                    if declaration.specifiers.is_empty() {
                        continue;
                    }

                    for specifier in &declaration.specifiers {
                        let Some(signature) = reexported_map.get(&specifier.local).cloned() else {
                            diagnostics.push(invalid_export_surface(
                                source_path,
                                &format!(
                                    "re-exported export `{}` was not statically known in `{source}`",
                                    specifier.local
                                ),
                            ));
                            continue;
                        };
                        if exports
                            .insert(specifier.exported.clone(), signature)
                            .is_some()
                        {
                            diagnostics.push(invalid_export_surface(
                                source_path,
                                &format!("duplicate export name `{}`", specifier.exported),
                            ));
                        }
                    }
                    continue;
                }

                for specifier in &declaration.specifiers {
                    let signature = declared_function_signatures
                        .get(&specifier.local)
                        .cloned()
                        .unwrap_or_else(|| signature_from_export_specifier(&specifier.local));
                    if exports
                        .insert(specifier.exported.clone(), signature)
                        .is_some()
                    {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{}`", specifier.exported),
                        ));
                    }
                }
            }
            Statement::ExportDefault(default_decl) => match default_decl {
                ExportDefaultDeclaration::FunctionDeclaration(func) => {
                    if func.generator {
                        diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            generator_function_unavailable_message(
                                func.is_async,
                                Some(func.body.as_ref()),
                            ),
                        ));
                    } else {
                        let export_name = if func.name.is_empty() {
                            "default".to_string()
                        } else {
                            func.name.clone()
                        };
                        if exports
                            .insert(
                                export_name.clone(),
                                infer_function_signature(&func.params, &func.body, func.is_async),
                            )
                            .is_some()
                        {
                            diagnostics.push(invalid_export_surface(
                                source_path,
                                &format!("duplicate export name `{export_name}`"),
                            ));
                        }
                    }
                }
                ExportDefaultDeclaration::Expression(expression) => {
                    if let Some(signature) = infer_function_binding_signature(
                        Some(expression),
                        source_path,
                        &declared_function_signatures,
                        &mut diagnostics,
                    ) {
                        if exports.insert("default".to_string(), signature).is_some() {
                            diagnostics.push(invalid_export_surface(
                                source_path,
                                "duplicate export name `default`",
                            ));
                        }
                    } else {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            "default export expressions are only part of the Phase-1 base library artifact when they resolve to a statically known function shape; use an explicit function declaration or the later compatibility path",
                        ));
                    }
                }
                ExportDefaultDeclaration::ClassDeclaration(_) => {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        "default export classes are not part of the Phase-1 base library artifact",
                    ));
                }
            },
            Statement::ImportDeclaration(_)
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
            | Statement::ClassDeclaration(_)
            | Statement::VariableDeclaration(_)
            | Statement::EnumDeclaration(_)
            | Statement::TypeAliasDeclaration(_)
            | Statement::InterfaceDeclaration(_)
            | Statement::ExpressionStatement(_) => {}
        }
    }

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let exports = exports
        .into_iter()
        .map(|(name, signature)| LibraryExport { name, signature })
        .collect::<Vec<_>>();

    if exports.is_empty() {
        return Err(vec![invalid_export_surface(
            source_path,
            "no statically known export surface was found",
        )]);
    }

    Ok(exports)
}

fn resolve_library_export_source_path(
    source_path: &Path,
    source: &str,
    api_surface: ApiSurface,
) -> Option<PathBuf> {
    let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
    let source_path = Path::new(source);

    if source_path.is_absolute() || source.starts_with('.') {
        if let Some(resolved) = resolve_relative_library_export_source(base_dir, source) {
            return Some(resolved);
        }
    }

    let project_root =
        kali_npm::discover_project_root(base_dir).unwrap_or_else(|| base_dir.to_path_buf());
    kali_npm::resolve_materialized_import_with_browser_context(
        project_root,
        source,
        api_surface == ApiSurface::Browser,
    )
}

fn resolve_relative_library_export_source(base_dir: &Path, source: &str) -> Option<PathBuf> {
    let candidate = base_dir.join(source);
    if candidate.is_file() {
        return Some(candidate);
    }

    if candidate.is_dir() {
        for index_name in [
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "index.mts",
            "index.mjs",
            "index.cts",
            "index.cjs",
            "index.d.ts",
            "index.d.mts",
            "index.d.cts",
        ] {
            let index_candidate = candidate.join(index_name);
            if index_candidate.is_file() {
                return Some(index_candidate);
            }
        }
    }

    let extensions = [
        "ts", "tsx", "js", "jsx", "mts", "cts", "d.ts", "d.mts", "d.cts",
    ];
    extensions.iter().find_map(|extension| {
        let candidate = if source.ends_with(extension) {
            base_dir.join(source)
        } else {
            base_dir.join(format!("{}.{}", source, extension))
        };
        if candidate.is_file() {
            return Some(candidate);
        }
        if candidate.is_dir() {
            for index_name in [
                "index.ts",
                "index.tsx",
                "index.js",
                "index.jsx",
                "index.mts",
                "index.mjs",
                "index.cts",
                "index.cjs",
                "index.d.ts",
                "index.d.mts",
                "index.d.cts",
            ] {
                let index_candidate = candidate.join(index_name);
                if index_candidate.is_file() {
                    return Some(index_candidate);
                }
            }
        }
        None
    })
}

pub fn collect_browser_bundle_exports(
    source_path: impl AsRef<Path>,
    tree_shake: bool,
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let statements = parse_source_file(source_path)?;
    let exports = collect_library_exports_from_statements(&statements, source_path)?;

    if !tree_shake {
        return Ok(exports);
    }

    let candidate_names = exports
        .iter()
        .map(|export| export.name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let comment_roots = collect_tree_shake_roots(source_path)?;
    let reachable = collect_reachable_bundle_exports(&statements, &candidate_names, &comment_roots);

    if reachable.is_empty() {
        return Ok(Vec::new());
    }

    let filtered = exports
        .into_iter()
        .filter(|export| reachable.contains(&export.name))
        .collect::<Vec<_>>();

    Ok(filtered)
}

fn collect_library_exports_from_statements(
    statements: &[Statement],
    source_path: &Path,
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let declared_function_signatures =
        collect_declared_function_signatures(statements, source_path, &mut diagnostics);
    let mut exports = BTreeMap::<String, String>::new();
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                let signature = infer_function_signature(&func.params, &func.body, func.is_async);
                if exports.insert(func.name.clone(), signature).is_some() {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!("duplicate export name `{}`", func.name),
                    ));
                }
            }
            Statement::ExportAll(_) => {
                diagnostics.push(invalid_export_surface(
                    source_path,
                    "re-exported surfaces are not statically known yet",
                ));
                continue;
            }
            Statement::ExportNamed(declaration) => {
                if declaration.source.is_some() {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        "re-exported surfaces are not statically known yet",
                    ));
                    continue;
                }

                for specifier in &declaration.specifiers {
                    let signature = declared_function_signatures
                        .get(&specifier.local)
                        .cloned()
                        .unwrap_or_else(|| signature_from_export_specifier(&specifier.local));
                    if exports
                        .insert(specifier.exported.clone(), signature)
                        .is_some()
                    {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{}`", specifier.exported),
                        ));
                    }
                }
            }
            Statement::ExportDefault(default_decl) => match default_decl {
                ExportDefaultDeclaration::FunctionDeclaration(func) => {
                    if func.generator {
                        diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            generator_function_unavailable_message(
                                func.is_async,
                                Some(func.body.as_ref()),
                            ),
                        ));
                    } else {
                        let export_name = if func.name.is_empty() {
                            "default".to_string()
                        } else {
                            func.name.clone()
                        };
                        if exports
                            .insert(
                                export_name.clone(),
                                infer_function_signature(&func.params, &func.body, func.is_async),
                            )
                            .is_some()
                        {
                            diagnostics.push(invalid_export_surface(
                                source_path,
                                &format!("duplicate export name `{export_name}`"),
                            ));
                        }
                    }
                }
                ExportDefaultDeclaration::Expression(expression) => {
                    if let Some(signature) = infer_function_binding_signature(
                        Some(expression),
                        source_path,
                        &declared_function_signatures,
                        &mut diagnostics,
                    ) {
                        if exports.insert("default".to_string(), signature).is_some() {
                            diagnostics.push(invalid_export_surface(
                                source_path,
                                "duplicate export name `default`",
                            ));
                        }
                    } else {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            "default export expressions are only part of the Phase-1 base library artifact when they resolve to a statically known function shape; use an explicit function declaration or the later compatibility path",
                        ));
                    }
                }
                ExportDefaultDeclaration::ClassDeclaration(_) => {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        "default export classes are not part of the Phase-1 base library artifact",
                    ));
                }
            },
            Statement::ImportDeclaration(_)
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
            | Statement::ClassDeclaration(_)
            | Statement::VariableDeclaration(_)
            | Statement::EnumDeclaration(_)
            | Statement::TypeAliasDeclaration(_)
            | Statement::InterfaceDeclaration(_)
            | Statement::ExpressionStatement(_) => {}
        }
    }

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let exports = exports
        .into_iter()
        .map(|(name, signature)| LibraryExport { name, signature })
        .collect::<Vec<_>>();

    if exports.is_empty() {
        return Err(vec![invalid_export_surface(
            source_path,
            "no statically known export surface was found",
        )]);
    }

    Ok(exports)
}

fn collect_tree_shake_roots(
    source_path: &Path,
) -> Result<std::collections::BTreeSet<String>, Vec<Diagnostic>> {
    let source = fs::read_to_string(source_path).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to read source file '{}': {}",
                source_path.display(),
                error
            ),
        )]
    })?;

    let mut roots = std::collections::BTreeSet::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("// kali-tree-shake:") {
            for name in rest.split(|ch: char| ch == ',' || ch.is_whitespace()) {
                let name = name.trim();
                if !name.is_empty() {
                    roots.insert(name.to_string());
                }
            }
        }
    }

    Ok(roots)
}

fn collect_reachable_bundle_exports(
    statements: &[Statement],
    candidate_names: &std::collections::BTreeSet<String>,
    initial_roots: &std::collections::BTreeSet<String>,
) -> std::collections::BTreeSet<String> {
    let function_map = statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::FunctionDeclaration(function)
                if candidate_names.contains(&function.name) =>
            {
                Some((function.name.clone(), function))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();

    let mut reachable = std::collections::BTreeSet::new();
    let mut worklist = collect_direct_bundle_calls_from_statements(statements, candidate_names);
    worklist.extend(initial_roots.iter().cloned());
    if candidate_names.contains("main") {
        worklist.insert("main".to_string());
    }
    while let Some(name) = worklist.pop_first() {
        if !reachable.insert(name.clone()) {
            continue;
        }

        if let Some(function) = function_map.get(&name) {
            collect_direct_bundle_calls_from_statements(&function.body.body, candidate_names)
                .into_iter()
                .filter(|dep| !reachable.contains(dep))
                .for_each(|dep| {
                    worklist.insert(dep);
                });
        }
    }

    reachable
}

fn collect_direct_bundle_calls_from_statements(
    statements: &[Statement],
    candidate_names: &std::collections::BTreeSet<String>,
) -> std::collections::BTreeSet<String> {
    let mut calls = std::collections::BTreeSet::new();
    for statement in statements {
        collect_direct_bundle_calls_from_statement(statement, candidate_names, &mut calls);
    }
    calls
}

fn collect_direct_bundle_calls_from_statement(
    statement: &Statement,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    match statement {
        Statement::ExpressionStatement(expr) => collect_direct_bundle_calls_from_expression(
            expr.expression.as_ref(),
            candidate_names,
            calls,
        ),
        Statement::ReturnStatement(statement) => {
            if let Some(argument) = &statement.argument {
                collect_direct_bundle_calls_from_expression(argument, candidate_names, calls);
            }
        }
        Statement::ThrowStatement(statement) => {
            collect_direct_bundle_calls_from_expression(
                &statement.argument,
                candidate_names,
                calls,
            );
        }
        Statement::WithStatement(statement) => {
            collect_direct_bundle_calls_from_expression(&statement.object, candidate_names, calls);
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::LabeledStatement(statement) => {
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::IfStatement(statement) => {
            collect_direct_bundle_calls_from_expression(&statement.test, candidate_names, calls);
            collect_direct_bundle_calls_from_block(
                statement.consequent.as_ref(),
                candidate_names,
                calls,
            );
            if let Some(alternate) = &statement.alternate {
                collect_direct_bundle_calls_from_block(alternate.as_ref(), candidate_names, calls);
            }
        }
        Statement::SwitchStatement(statement) => {
            collect_direct_bundle_calls_from_expression(
                &statement.discriminant,
                candidate_names,
                calls,
            );
            for case in &statement.cases {
                if let Some(test) = &case.test {
                    collect_direct_bundle_calls_from_expression(test, candidate_names, calls);
                }
                for inner in &case.consequent {
                    collect_direct_bundle_calls_from_statement(inner, candidate_names, calls);
                }
            }
        }
        Statement::TryStatement(statement) => {
            collect_direct_bundle_calls_from_block(
                statement.block.as_ref(),
                candidate_names,
                calls,
            );
            if let Some(handler) = &statement.handler {
                collect_direct_bundle_calls_from_block(
                    handler.body.as_ref(),
                    candidate_names,
                    calls,
                );
            }
            if let Some(finalizer) = &statement.finalizer {
                collect_direct_bundle_calls_from_block(finalizer, candidate_names, calls);
            }
        }
        Statement::BlockStatement(block) => {
            collect_direct_bundle_calls_from_block(block, candidate_names, calls)
        }
        Statement::ForStatement(statement) => {
            if let Some(init) = &statement.init {
                match init {
                    kali_ast::ForInit::Expression(expression) => {
                        collect_direct_bundle_calls_from_expression(
                            expression,
                            candidate_names,
                            calls,
                        )
                    }
                    kali_ast::ForInit::VariableDeclaration(declaration) => {
                        collect_direct_bundle_calls_from_variable_declaration(
                            declaration,
                            candidate_names,
                            calls,
                        )
                    }
                }
            }
            if let Some(test) = &statement.test {
                collect_direct_bundle_calls_from_expression(test, candidate_names, calls);
            }
            if let Some(update) = &statement.update {
                collect_direct_bundle_calls_from_expression(update, candidate_names, calls);
            }
            collect_direct_bundle_calls_from_block(statement.body.as_ref(), candidate_names, calls);
        }
        Statement::ForInStatement(statement) => {
            match &statement.left {
                kali_ast::ForInLefthand::Expression(expression) => {
                    collect_direct_bundle_calls_from_expression(expression, candidate_names, calls)
                }
                kali_ast::ForInLefthand::VariableDeclaration(declaration) => {
                    collect_direct_bundle_calls_from_variable_declaration(
                        declaration,
                        candidate_names,
                        calls,
                    )
                }
            }
            collect_direct_bundle_calls_from_expression(&statement.right, candidate_names, calls);
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::ForOfStatement(statement) => {
            match &statement.left {
                kali_ast::ForOfLefthand::Expression(expression) => {
                    collect_direct_bundle_calls_from_expression(expression, candidate_names, calls)
                }
                kali_ast::ForOfLefthand::VariableDeclaration(declaration) => {
                    collect_direct_bundle_calls_from_variable_declaration(
                        declaration,
                        candidate_names,
                        calls,
                    )
                }
            }
            collect_direct_bundle_calls_from_expression(&statement.right, candidate_names, calls);
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::WhileStatement(statement) => {
            collect_direct_bundle_calls_from_expression(&statement.test, candidate_names, calls);
            collect_direct_bundle_calls_from_block(statement.body.as_ref(), candidate_names, calls);
        }
        Statement::DoWhileStatement(statement) => {
            collect_direct_bundle_calls_from_block(statement.body.as_ref(), candidate_names, calls);
            collect_direct_bundle_calls_from_expression(&statement.test, candidate_names, calls);
        }
        Statement::VariableDeclaration(statement) => {
            collect_direct_bundle_calls_from_variable_declaration(statement, candidate_names, calls)
        }
        Statement::EnumDeclaration(statement) => {
            for member in &statement.members {
                if let Some(value) = &member.value {
                    collect_direct_bundle_calls_from_expression(value, candidate_names, calls);
                }
            }
        }
        Statement::FunctionDeclaration(_)
        | Statement::ClassDeclaration(_)
        | Statement::ImportDeclaration(_)
        | Statement::BreakStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::DebuggerStatement(_)
        | Statement::TypeAliasDeclaration(_)
        | Statement::InterfaceDeclaration(_)
        | Statement::ExportAll(_)
        | Statement::ExportNamed(_)
        | Statement::ExportDefault(_) => {}
    }
}

fn collect_direct_bundle_calls_from_block(
    block: &kali_ast::BlockStatement,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    for statement in &block.body {
        collect_direct_bundle_calls_from_statement(statement, candidate_names, calls);
    }
}

fn collect_direct_bundle_calls_from_variable_declaration(
    declaration: &kali_ast::VariableDeclaration,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    for declarator in &declaration.declarations {
        if let Some(init) = &declarator.init {
            collect_direct_bundle_calls_from_expression(init, candidate_names, calls);
        }
    }
}

fn collect_direct_bundle_calls_from_expression(
    expression: &Expression,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    match expression {
        Expression::Identifier(name) if candidate_names.contains(name) => {
            calls.insert(name.clone());
        }
        Expression::Identifier(_) => {}
        Expression::Literal(_)
        | Expression::ThisExpression
        | Expression::SuperExpression
        | Expression::PrivateIdentifier(_)
        | Expression::BigIntLiteral(_)
        | Expression::JsxEmptyExpression => {}
        Expression::BinaryExpression(expr) => {
            collect_direct_bundle_calls_from_expression(&expr.left, candidate_names, calls);
            collect_direct_bundle_calls_from_expression(&expr.right, candidate_names, calls);
        }
        Expression::UnaryExpression(expr) => {
            collect_direct_bundle_calls_from_expression(&expr.argument, candidate_names, calls);
        }
        Expression::CallExpression(expr) => {
            if let Expression::Identifier(name) = expr.callee.as_ref() {
                if candidate_names.contains(name) {
                    calls.insert(name.clone());
                }
            }
            collect_direct_bundle_calls_from_expression(
                expr.callee.as_ref(),
                candidate_names,
                calls,
            );
            for arg in &expr.args {
                collect_direct_bundle_calls_from_expression(arg, candidate_names, calls);
            }
        }
        Expression::MemberExpression(expr) => {
            collect_direct_bundle_calls_from_expression(&expr.object, candidate_names, calls);
        }
        Expression::ArrayExpression(expr) => {
            for element in &expr.elements {
                match element {
                    Some(kali_ast::ExpressionOrSpread::Expression(value)) => {
                        collect_direct_bundle_calls_from_expression(value, candidate_names, calls)
                    }
                    Some(kali_ast::ExpressionOrSpread::Spread(spread)) => {
                        collect_direct_bundle_calls_from_expression(
                            &spread.argument,
                            candidate_names,
                            calls,
                        )
                    }
                    Some(kali_ast::ExpressionOrSpread::Empty) | None => {}
                }
            }
        }
        Expression::ObjectExpression(expr) => {
            for property in &expr.properties {
                collect_direct_bundle_calls_from_expression(
                    &property.value,
                    candidate_names,
                    calls,
                );
            }
        }
        Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::NewExpression(_)
        | Expression::MetaProperty(_)
        | Expression::TemplateLiteral(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::SpreadElement(_)
        | Expression::RestElement(_)
        | Expression::ImportExpression(_)
        | Expression::JsxElement(_)
        | Expression::JsxFragment(_) => {}
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_direct_bundle_calls_from_expression(
                &parenthesized.expression,
                candidate_names,
                calls,
            );
        }
        Expression::TypeAssertion(type_assertion) => {
            collect_direct_bundle_calls_from_expression(
                &type_assertion.expression,
                candidate_names,
                calls,
            );
        }
        Expression::SatisfiesExpression(satisfies_expression) => {
            collect_direct_bundle_calls_from_expression(
                &satisfies_expression.expression,
                candidate_names,
                calls,
            );
        }
        Expression::AwaitExpression(await_expression) => {
            collect_direct_bundle_calls_from_expression(
                &await_expression.argument,
                candidate_names,
                calls,
            );
        }
        Expression::YieldExpression(yield_expression) => {
            if let Some(argument) = &yield_expression.argument {
                collect_direct_bundle_calls_from_expression(argument, candidate_names, calls);
            }
        }
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                kali_ast::OptionalChainInner::NonNull { object, .. } => {
                    collect_direct_bundle_calls_from_expression(object, candidate_names, calls);
                }
            }
        }
        Expression::ChainExpression(chain_expression) => {
            collect_direct_bundle_calls_from_expression(
                &chain_expression.expression,
                candidate_names,
                calls,
            );
        }
        Expression::DecoratedExpression(decorated_expression) => {
            collect_direct_bundle_calls_from_expression(
                &decorated_expression.expression,
                candidate_names,
                calls,
            );
        }
        Expression::SequenceExpression(sequence_expression) => {
            if let Some(expression) = sequence_expression.expressions.last() {
                collect_direct_bundle_calls_from_expression(expression, candidate_names, calls);
            }
        }
        Expression::ConditionalExpression(conditional_expression) => {
            collect_direct_bundle_calls_from_expression(
                &conditional_expression.test,
                candidate_names,
                calls,
            );
            collect_direct_bundle_calls_from_expression(
                conditional_expression.consequent.as_ref(),
                candidate_names,
                calls,
            );
            collect_direct_bundle_calls_from_expression(
                conditional_expression.alternate.as_ref(),
                candidate_names,
                calls,
            );
        }
        Expression::LogicalExpression(logical_expression) => {
            collect_direct_bundle_calls_from_expression(
                &logical_expression.left,
                candidate_names,
                calls,
            );
            collect_direct_bundle_calls_from_expression(
                &logical_expression.right,
                candidate_names,
                calls,
            );
        }
        Expression::AssignmentExpression(assignment_expression) => {
            collect_direct_bundle_calls_from_expression(
                &assignment_expression.left,
                candidate_names,
                calls,
            );
            collect_direct_bundle_calls_from_expression(
                &assignment_expression.right,
                candidate_names,
                calls,
            );
        }
    }
}

fn collect_declared_function_signatures(
    statements: &[Statement],
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, String> {
    let mut declared_function_signatures = BTreeMap::new();
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                if func.generator {
                    diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        generator_function_unavailable_message(
                            func.is_async,
                            Some(func.body.as_ref()),
                        ),
                    ));
                    continue;
                }

                let signature = infer_function_signature(&func.params, &func.body, func.is_async);
                if declared_function_signatures
                    .insert(func.name.clone(), signature)
                    .is_some()
                {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!("duplicate export name `{}`", func.name),
                    ));
                }
            }
            Statement::VariableDeclaration(declaration) if declaration.kind == "const" => {
                collect_declared_function_binding_signatures(
                    declaration,
                    source_path,
                    diagnostics,
                    &mut declared_function_signatures,
                );
            }
            _ => {}
        }
    }

    declared_function_signatures
}

fn collect_declared_function_binding_signatures(
    declaration: &VariableDeclaration,
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
    declared_function_signatures: &mut BTreeMap<String, String>,
) {
    let mut known_signatures = declared_function_signatures.clone();

    for declarator in &declaration.declarations {
        let Some(signature) = infer_function_binding_signature(
            declarator.init.as_ref(),
            source_path,
            &known_signatures,
            diagnostics,
        ) else {
            continue;
        };

        if known_signatures
            .insert(declarator.id.clone(), signature.clone())
            .is_some()
        {
            diagnostics.push(invalid_export_surface(
                source_path,
                &format!("duplicate export name `{}`", declarator.id),
            ));
        }

        declared_function_signatures.insert(declarator.id.clone(), signature);
    }
}

fn infer_function_signature(params: &[String], body: &BlockStatement, is_async: bool) -> String {
    function_signature(params, infer_block_return_type(body), is_async)
}

#[allow(clippy::only_used_in_recursion)]
fn infer_function_binding_signature(
    expression: Option<&Expression>,
    source_path: &Path,
    declared_function_signatures: &BTreeMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let expression = expression?;
    match expression {
        Expression::FunctionExpression(func) => {
            if func.generator {
                diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    generator_function_unavailable_message(func.is_async, func.body.as_deref()),
                ));
                return None;
            }

            let body = func.body.as_ref()?;
            let params = func
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect::<Vec<_>>();
            Some(function_signature(
                &params,
                infer_block_return_type(body),
                func.is_async,
            ))
        }
        Expression::ArrowFunctionExpression(func) => {
            let params = func
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect::<Vec<_>>();
            Some(function_signature(
                &params,
                infer_expression_type(&func.body),
                func.is_async,
            ))
        }
        Expression::Identifier(name) => declared_function_signatures.get(name).cloned(),
        Expression::ParenthesizedExpression(parenthesized) => infer_function_binding_signature(
            Some(&parenthesized.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::TypeAssertion(type_assertion) => infer_function_binding_signature(
            Some(&type_assertion.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::SatisfiesExpression(satisfies_expression) => infer_function_binding_signature(
            Some(&satisfies_expression.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => infer_function_binding_signature(
                    Some(object),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                ),
            }
        }
        Expression::ChainExpression(chain_expression) => infer_function_binding_signature(
            Some(&chain_expression.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::AwaitExpression(await_expression) => infer_function_binding_signature(
            Some(&await_expression.argument),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::CallExpression(call) if is_object_freeze_call(call) => {
            call.args.first().and_then(|argument| {
                infer_function_binding_signature(
                    Some(argument),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                )
            })
        }
        Expression::SequenceExpression(sequence_expression) => sequence_expression
            .expressions
            .last()
            .and_then(|expression| {
                infer_function_binding_signature(
                    Some(expression),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                )
            }),
        Expression::DecoratedExpression(decorated_expression) => infer_function_binding_signature(
            Some(&decorated_expression.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::LogicalExpression(logical_expression) => {
            let left = infer_function_binding_signature(
                Some(&logical_expression.left),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            match logical_expression.operator {
                kali_ast::LogicalOperator::Coalesce => {
                    if matches!(
                        infer_expression_type(&logical_expression.left),
                        Some("null" | "undefined" | "void")
                    ) {
                        infer_function_binding_signature(
                            Some(&logical_expression.right),
                            source_path,
                            declared_function_signatures,
                            diagnostics,
                        )
                    } else {
                        left
                    }
                }
                kali_ast::LogicalOperator::And => {
                    match infer_static_truthiness(&logical_expression.left) {
                        Some(true) => infer_function_binding_signature(
                            Some(&logical_expression.right),
                            source_path,
                            declared_function_signatures,
                            diagnostics,
                        ),
                        Some(false) => None,
                        None => {
                            let right = infer_function_binding_signature(
                                Some(&logical_expression.right),
                                source_path,
                                declared_function_signatures,
                                diagnostics,
                            );
                            if left.is_some() && left == right {
                                left
                            } else {
                                None
                            }
                        }
                    }
                }
                kali_ast::LogicalOperator::Or => {
                    match infer_static_truthiness(&logical_expression.left) {
                        Some(true) => None,
                        Some(false) => infer_function_binding_signature(
                            Some(&logical_expression.right),
                            source_path,
                            declared_function_signatures,
                            diagnostics,
                        ),
                        None => {
                            let right = infer_function_binding_signature(
                                Some(&logical_expression.right),
                                source_path,
                                declared_function_signatures,
                                diagnostics,
                            );
                            if left.is_some() && left == right {
                                left
                            } else {
                                None
                            }
                        }
                    }
                }
            }
        }
        Expression::BinaryExpression(binary) if binary.operator == "??" => {
            let left = infer_function_binding_signature(
                Some(&binary.left),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            if left.is_some() {
                left
            } else if matches!(
                infer_expression_type(&binary.left),
                Some("null" | "undefined" | "void")
            ) {
                infer_function_binding_signature(
                    Some(&binary.right),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                )
            } else {
                None
            }
        }
        Expression::ConditionalExpression(conditional_expression) => {
            let consequent = infer_function_binding_signature(
                Some(conditional_expression.consequent.as_ref()),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            let alternate = infer_function_binding_signature(
                Some(conditional_expression.alternate.as_ref()),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            if consequent.is_some() && consequent == alternate {
                consequent
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_object_freeze_call(call: &kali_ast::CallExpression) -> bool {
    matches!(
        call_member_access_name(&call.callee).as_deref(),
        Some("Object.freeze") | Some("globalThis.Object.freeze")
    ) && call.args.len() == 1
}

fn call_member_access_name(expression: &Expression) -> Option<String> {
    match expression {
        Expression::MemberExpression(member) => member_access_name(member),
        Expression::Identifier(name) => Some(name.clone()),
        Expression::ParenthesizedExpression(expr) => call_member_access_name(&expr.expression),
        Expression::TypeAssertion(expr) => call_member_access_name(&expr.expression),
        Expression::SatisfiesExpression(expr) => call_member_access_name(&expr.expression),
        Expression::ChainExpression(expr) => call_member_access_name(&expr.expression),
        Expression::DecoratedExpression(expr) => call_member_access_name(&expr.expression),
        Expression::SequenceExpression(expr) => {
            expr.expressions.last().and_then(call_member_access_name)
        }
        Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => call_member_access_name(object),
        },
        _ => None,
    }
}

fn member_access_name(member: &kali_ast::MemberExpression) -> Option<String> {
    let object = call_member_access_name(&member.object)?;
    Some(format!("{object}.{}", member.property))
}

fn function_signature(params: &[String], return_type: Option<&str>, is_async: bool) -> String {
    let return_type = return_type.unwrap_or("unknown");
    let return_type = if is_async {
        format!("Promise<{return_type}>")
    } else {
        return_type.to_string()
    };
    format!("({}) => {}", params.join(", "), return_type)
}

fn infer_block_return_type(body: &BlockStatement) -> Option<&'static str> {
    if body.body.len() != 1 {
        return None;
    }

    let Statement::ReturnStatement(return_statement) = &body.body[0] else {
        return None;
    };

    match &return_statement.argument {
        Some(expression) => infer_expression_type(expression),
        None => Some("void"),
    }
}

fn infer_static_truthiness(expression: &Expression) -> Option<bool> {
    match expression {
        Expression::Literal(kali_ast::LiteralValue::Boolean(value)) => Some(*value),
        Expression::Literal(kali_ast::LiteralValue::Number(value)) => {
            Some(*value != 0.0 && !value.is_nan())
        }
        Expression::Literal(kali_ast::LiteralValue::String(value)) => Some(!value.is_empty()),
        Expression::Literal(kali_ast::LiteralValue::Regex { .. }) => Some(true),
        Expression::Literal(kali_ast::LiteralValue::Null) => Some(false),
        Expression::Identifier(name) if name == "undefined" => Some(false),
        Expression::UnaryExpression(unary) if unary.operator == "void" => Some(false),
        Expression::UnaryExpression(unary) if unary.operator == "!" => {
            infer_static_truthiness(&unary.argument).map(|value| !value)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            infer_static_truthiness(&parenthesized.expression)
        }
        Expression::AwaitExpression(await_expression) => {
            infer_static_truthiness(&await_expression.argument)
        }
        Expression::TypeAssertion(type_assertion) => {
            infer_static_truthiness(&type_assertion.expression)
        }
        Expression::SatisfiesExpression(satisfies_expression) => {
            infer_static_truthiness(&satisfies_expression.expression)
        }
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => infer_static_truthiness(object),
            }
        }
        Expression::ChainExpression(chain_expression) => {
            infer_static_truthiness(&chain_expression.expression)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .and_then(infer_static_truthiness),
        Expression::DecoratedExpression(decorated_expression) => {
            infer_static_truthiness(&decorated_expression.expression)
        }
        _ => None,
    }
}

fn infer_expression_type(expression: &Expression) -> Option<&'static str> {
    match expression {
        Expression::Literal(value) => infer_literal_type(value),
        Expression::Identifier(name) if name == "undefined" => Some("undefined"),
        Expression::BigIntLiteral(_) => Some("bigint"),
        Expression::TemplateLiteral(_) => Some("string"),
        Expression::ParenthesizedExpression(parenthesized) => {
            infer_expression_type(&parenthesized.expression)
        }
        Expression::AwaitExpression(await_expression) => {
            infer_expression_type(&await_expression.argument)
        }
        Expression::TypeAssertion(type_assertion) => {
            infer_expression_type(&type_assertion.expression)
        }
        Expression::SatisfiesExpression(satisfies_expression) => {
            infer_expression_type(&satisfies_expression.expression)
        }
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => infer_expression_type(object),
            }
        }
        Expression::ChainExpression(chain_expression) => {
            infer_expression_type(&chain_expression.expression)
        }
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().and_then(infer_expression_type)
        }
        Expression::DecoratedExpression(decorated_expression) => {
            infer_expression_type(&decorated_expression.expression)
        }
        Expression::ConditionalExpression(condition) => {
            let consequent = infer_expression_type(&condition.consequent);
            let alternate = infer_expression_type(&condition.alternate);
            if consequent.is_some() && consequent == alternate {
                consequent
            } else {
                None
            }
        }
        Expression::UnaryExpression(unary) => infer_unary_expression_type(unary),
        Expression::BinaryExpression(binary) => infer_binary_expression_type(binary),
        _ => None,
    }
}

fn infer_unary_expression_type(unary: &kali_ast::UnaryExpression) -> Option<&'static str> {
    match unary.operator.as_str() {
        "!" => Some("boolean"),
        "+" | "-" => infer_expression_type(&unary.argument)
            .filter(|type_name| matches!(*type_name, "number" | "bigint")),
        "void" => Some("void"),
        _ => None,
    }
}

fn infer_binary_expression_type(binary: &kali_ast::BinaryExpression) -> Option<&'static str> {
    let left = infer_expression_type(&binary.left);
    let right = infer_expression_type(&binary.right);
    match binary.operator.as_str() {
        "+" => {
            if left == Some("string") || right == Some("string") {
                Some("string")
            } else if is_numeric_like_type(left) && is_numeric_like_type(right) {
                Some("number")
            } else {
                None
            }
        }
        "-" | "*" | "/" | "%" | "**" | "<<" | ">>" | ">>>" | "&" | "|" | "^" => {
            if is_numeric_like_type(left) && is_numeric_like_type(right) {
                Some("number")
            } else {
                None
            }
        }
        "??" => {
            if matches!(left, Some("null" | "undefined" | "void")) {
                right
            } else if left.is_some() {
                left
            } else {
                None
            }
        }
        "&&" | "||" => {
            if left.is_some() && left == right {
                left
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_numeric_like_type(type_name: Option<&str>) -> bool {
    matches!(type_name, Some("number" | "bigint"))
}

fn infer_literal_type(value: &LiteralValue) -> Option<&'static str> {
    match value {
        LiteralValue::Boolean(_) => Some("boolean"),
        LiteralValue::Number(_) => Some("number"),
        LiteralValue::String(_) => Some("string"),
        LiteralValue::Regex { .. } => Some("RegExp"),
        LiteralValue::Null => Some("null"),
    }
}

#[cfg(test)]
#[path = "../build_tests.rs"]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use kali_codegen::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_common::FileId;
use kali_error::{Diagnostic, _error_codes::e8};
use kali_hir::HirLowerer;
use kali_lexer::Lexer;
use kali_lir::LirLowerer;
use kali_mir::MirLowerer;
use kali_parser::Parser;
use kali_types::TypeContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildMode {
    Fast,
    Release,
    ReleaseAdvanced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOutput {
    pub output_path: PathBuf,
    pub wasm_bytes: Vec<u8>,
}

pub fn check_source_file(source_path: impl AsRef<Path>) -> Result<(), Vec<Diagnostic>> {
    let _analysis = analyze_source_file(source_path.as_ref())?;
    Ok(())
}

pub fn compile_source_file(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let analyzed = analyze_source_file(source_path.as_ref())?;

    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_statements(&analyzed.statements);
    let mut diagnostics = analyzed.diagnostics;
    diagnostics.extend(hir.diagnostics.clone());
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let mir = MirLowerer::new().lower_hir_result(&hir);
    let lir = LirLowerer::new().lower_program(&mir);

    let mut ctx = CodegenCtx::new(TargetConfig {
        optimize: !matches!(mode, BuildMode::Fast),
    });
    let result = lower_lir_to_wasm(&mut ctx, &lir);
    diagnostics.extend(result.diagnostics);

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    Ok(result.wasm_bytes)
}

fn analyze_source_file(source_path: &Path) -> Result<AnalyzedSource, Vec<Diagnostic>> {
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

    let lexer = Lexer::new(FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let parsed = parser.parse(Some(source_path.to_string_lossy().to_string()));
    let mut diagnostics = parsed.diagnostics;

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let mut resolver = TypeContext::with_base_path(source_path);
    let resolved = resolver.resolve_statements_in_file(source_path, &parsed.statements);
    diagnostics.extend(resolved.diagnostics);
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    Ok(AnalyzedSource {
        statements: parsed.statements,
        diagnostics,
    })
}

struct AnalyzedSource {
    statements: Vec<kali_ast::Statement>,
    diagnostics: Vec<Diagnostic>,
}

pub fn build_source_file(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    out_dir: Option<&Path>,
) -> Result<BuildOutput, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let wasm_bytes = compile_source_file(source_path, mode)?;

    let output_path = output_path_for(source_path, out_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e8::INTERNAL_ERROR as u32,
                format!(
                    "failed to create output directory '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }

    fs::write(&output_path, &wasm_bytes).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to write WASM artifact '{}': {}",
                output_path.display(),
                error
            ),
        )]
    })?;

    Ok(BuildOutput {
        output_path,
        wasm_bytes,
    })
}

pub fn output_path_for(source_path: &Path, out_dir: Option<&Path>) -> PathBuf {
    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("main");
    let file_name = format!("{}.wasm", stem);
    match out_dir {
        Some(dir) => dir.join(file_name),
        None => source_path.with_file_name(file_name),
    }
}

pub fn build_mode_from_flags(fast: bool, release: bool, release_advanced: bool) -> BuildMode {
    if release_advanced {
        BuildMode::ReleaseAdvanced
    } else if release {
        BuildMode::Release
    } else if fast {
        BuildMode::Fast
    } else {
        BuildMode::Fast
    }
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.is_error())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use wasmparser::Validator;

    #[test]
    fn build_source_file_writes_valid_wasm_artifact() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            "function add(a, b) { return a + b; } add(1, 2);",
        )
        .expect("write source");

        let output =
            build_source_file(&source_path, BuildMode::Fast, None).expect("build should succeed");

        assert!(output.output_path.exists());
        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("artifact should validate");
    }

    #[test]
    fn output_path_uses_source_stem() {
        let source = PathBuf::from("/tmp/demo/main.ts");
        let output = output_path_for(&source, Some(Path::new("dist")));
        assert_eq!(output, PathBuf::from("dist/main.wasm"));
    }
}

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use kali_ast::{ExportNamedDeclaration, ImportDeclaration, Statement};
use kali_common::FileId;
use kali_error::{_error_codes::e5, _error_codes::e8, _error_codes::e9, Diagnostic};
use kali_lexer::{Lexer, Token, TokenType};
use kali_parser::Parser;
use serde::{Deserialize, Serialize};

use crate::{AccessRule, PatternKind, SandboxPolicy};

const SOURCE_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "d.ts", "d.mts", "d.cts",
];

/// Analysis context recorded in effect reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectAnalysisContext {
    pub api_surface: String,
    pub runtime_profiles: Vec<String>,
    pub compat_features: Vec<String>,
}

impl EffectAnalysisContext {
    pub fn new(api_surface: impl Into<String>) -> Self {
        Self {
            api_surface: api_surface.into(),
            runtime_profiles: Vec::new(),
            compat_features: Vec::new(),
        }
    }

    /// Return a normalized copy with sorted, deduplicated semantic axes.
    pub fn normalized(mut self) -> Self {
        self.runtime_profiles = normalize_semantic_axis(self.runtime_profiles);
        self.compat_features = normalize_semantic_axis(self.compat_features);
        self
    }
}

/// Location attached to an observed effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
}

/// One occurrence of a built-in effect kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectOccurrence {
    pub kind: String,
    pub locations: Vec<EffectLocation>,
}

/// Public reusable effect-report payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectReport {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub analysis_context: EffectAnalysisContext,
    pub entry_points: Vec<String>,
    pub effects: Vec<EffectOccurrence>,
    pub dynamic_effects: bool,
    pub dynamic_reasons: Vec<String>,
}

/// Package coordinate used by the package-effects report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCoordinate {
    pub name: String,
    pub version: String,
    pub registry: String,
}

/// Package-effects wrapper payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageEffectsReport {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub package: PackageCoordinate,
    pub report: EffectReport,
}

/// Internal observed effect with optional target details for policy comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedEffect {
    pub kind: String,
    pub location: EffectLocation,
    pub target: Option<String>,
}

/// Result of inferring effects across one or more source roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInference {
    pub effects: Vec<ObservedEffect>,
    pub dynamic_reasons: Vec<String>,
}

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

/// Convert inferred effects into the public reusable effect-report payload.
pub fn effect_report_from_inference(
    entry_points: Vec<String>,
    context: EffectAnalysisContext,
    inference: EffectInference,
) -> EffectReport {
    let EffectInference {
        effects,
        dynamic_reasons,
    } = inference;
    let context = context.normalized();

    let mut grouped = BTreeMap::<String, Vec<EffectLocation>>::new();
    for effect in effects {
        grouped
            .entry(effect.kind)
            .or_default()
            .push(effect.location);
    }

    let mut effect_groups = grouped
        .into_iter()
        .map(|(kind, mut locations)| {
            locations.sort_by(location_sort_key);
            EffectOccurrence { kind, locations }
        })
        .collect::<Vec<_>>();
    effect_groups.sort_by(|a, b| a.kind.cmp(&b.kind));

    EffectReport {
        schema_version: 1,
        analysis_context: context,
        entry_points,
        effects: effect_groups,
        dynamic_effects: !dynamic_reasons.is_empty(),
        dynamic_reasons,
    }
}

/// Wrap a public effect report in the package-effects envelope.
pub fn package_effects_report(
    package: PackageCoordinate,
    report: EffectReport,
) -> PackageEffectsReport {
    PackageEffectsReport {
        schema_version: 1,
        package,
        report,
    }
}

/// Compare the observed effects against a sandbox policy.
pub fn compare_effects_to_policy(
    effects: &[ObservedEffect],
    policy: &SandboxPolicy,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for effect in effects {
        if !effect_allowed(effect, policy) {
            let mut diagnostic = Diagnostic::error(
                e9::EFFECT_POLICY_MISMATCH as u32,
                format!(
                    "inferred effect '{}' is not permitted by the active policy",
                    effect.kind
                ),
            );
            if let Some(target) = &effect.target {
                diagnostic = diagnostic.note(format!(
                    "observed target '{}'; compare it against the matching allow-list entry",
                    target
                ));
            }
            diagnostic = diagnostic.with_suggestion(policy_suggestion(effect));
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

fn policy_suggestion(effect: &ObservedEffect) -> String {
    match effect.kind.as_str() {
        "FileSystem.Read" => {
            "allow the matching path pattern under effects.fileSystem.read or deny the call site"
                .to_string()
        }
        "FileSystem.Write" => {
            "allow the matching path pattern under effects.fileSystem.write or deny the call site"
                .to_string()
        }
        "Network.Fetch" => {
            "allow the matching URL pattern under effects.network.fetch or deny the call site"
                .to_string()
        }
        "Network.Connect" => {
            "allow the matching host pattern under effects.network.connect or deny the call site"
                .to_string()
        }
        "Network.Listen" => {
            "allow the matching host pattern under effects.network.listen or deny the call site"
                .to_string()
        }
        "Process.Spawn" => {
            "enable effects.process.spawn only when the selected phase supports subprocesses"
                .to_string()
        }
        "Process.EnvRead" => {
            "allow the matching variable name under effects.process.envRead or deny the call site"
                .to_string()
        }
        "Process.EnvWrite" => {
            "allow the matching variable name under effects.process.envWrite or deny the call site"
                .to_string()
        }
        "Timer.Schedule" => "enable effects.timer.schedule or reduce the timer usage".to_string(),
        "Random.GetBytes" => "enable effects.random or remove the randomness call".to_string(),
        "Console.Write" => "enable effects.console or remove the console call".to_string(),
        "Eval" => {
            "enable effects.eval only when the documented eval compatibility path is available"
                .to_string()
        }
        _ => "adjust the sandbox policy to permit the inferred effect or rewrite the code"
            .to_string(),
    }
}

fn normalize_semantic_axis(values: Vec<String>) -> Vec<String> {
    let mut normalized = BTreeSet::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() {
            normalized.insert(value.to_string());
        }
    }
    normalized.into_iter().collect()
}

fn effect_allowed(effect: &ObservedEffect, policy: &SandboxPolicy) -> bool {
    match effect.kind.as_str() {
        "FileSystem.Read" => rule_allows(
            &policy.effects.file_system.read,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Path,
        ),
        "FileSystem.Write" => rule_allows(
            &policy.effects.file_system.write,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Path,
        ),
        "Network.Fetch" => rule_allows(
            &policy.effects.network.fetch,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Url,
        ),
        "Network.Connect" => rule_allows(
            &policy.effects.network.connect,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Url,
        ),
        "Network.Listen" => rule_allows(
            &policy.effects.network.listen,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Url,
        ),
        "Process.Spawn" => rule_allows(
            &policy.effects.process.spawn,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Exact,
        ),
        "Process.EnvRead" => rule_allows(
            &policy.effects.process.env_read,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Exact,
        ),
        "Process.EnvWrite" => rule_allows(
            &policy.effects.process.env_write,
            effect.target.as_deref(),
            &policy.base_dir,
            PatternKind::Exact,
        ),
        "Timer.Schedule" => policy.effects.timer.schedule,
        "Random.GetBytes" => policy.effects.random,
        "Console.Write" => policy.effects.console,
        "Eval" => policy.effects.eval,
        _ => true,
    }
}

fn rule_allows(
    rule: &AccessRule,
    target: Option<&str>,
    base_dir: &Path,
    kind: PatternKind,
) -> bool {
    match rule {
        AccessRule::Deny(false) => false,
        AccessRule::Deny(true) => true,
        AccessRule::AllowList(patterns) => {
            if patterns.is_empty() {
                return false;
            }
            let Some(candidate) = target else {
                return false;
            };
            rule.allows_candidate(candidate, base_dir, kind)
        }
    }
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

    let file_effects = scan_tokens_for_effects(root, &source, &lexed.tokens, dynamic_reasons)?;
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

fn resolve_relative_import(current_file: &Path, spec: &str) -> Option<PathBuf> {
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

fn scan_tokens_for_effects(
    file: &Path,
    source: &str,
    tokens: &[Token],
    dynamic_reasons: &mut BTreeSet<String>,
) -> Result<Vec<ObservedEffect>, Vec<Diagnostic>> {
    let mut effects = Vec::<ObservedEffect>::new();
    let mut i = 0usize;

    while i < tokens.len() {
        let token = &tokens[i];

        if matches!(token.kind, TokenType::Import)
            && matches!(
                tokens.get(i + 1).map(|t| t.kind),
                Some(TokenType::LeftParen)
            )
        {
            dynamic_reasons.insert("dynamic-import".to_string());
            if let Some(target) = call_string_argument(tokens, i + 2) {
                if target.starts_with('.') || target.starts_with('/') {
                    if let Some(resolved) = resolve_relative_import(file, &target) {
                        let _ = resolved;
                    }
                }
            }
            i += 1;
            continue;
        }

        if is_eval_call(tokens, i) {
            dynamic_reasons.insert("eval".to_string());
            effects.push(observed_effect(file, token, source, "Eval", None));
            i += 1;
            continue;
        }

        if is_function_constructor(tokens, i) {
            dynamic_reasons.insert("function-constructor".to_string());
            effects.push(observed_effect(file, token, source, "Eval", None));
            i += 1;
            continue;
        }

        if is_proxy_constructor(tokens, i) {
            dynamic_reasons.insert("proxy-traps".to_string());
            i += 1;
            continue;
        }

        if is_console_write_call(tokens, i) {
            effects.push(observed_effect(file, token, source, "Console.Write", None));
            i += 1;
            continue;
        }

        if let Some((kind, target)) = is_deno_command_constructor(tokens, i) {
            effects.push(observed_effect(file, token, source, kind, target));
            i += 1;
            continue;
        }

        if let Some((kind, target)) = is_deno_host_call(tokens, i) {
            effects.push(observed_effect(file, token, source, kind, target));
            i += 1;
            continue;
        }

        if let Some((kind, target)) = is_global_effect_call(tokens, i) {
            effects.push(observed_effect(file, token, source, kind, target));
            i += 1;
            continue;
        }

        if let Some((kind, target)) = is_require_call(tokens, i) {
            effects.push(observed_effect(file, token, source, kind, target));
            i += 1;
            continue;
        }

        i += 1;
    }

    Ok(effects)
}

fn observed_effect(
    file: &Path,
    token: &Token,
    source: &str,
    kind: &str,
    target: Option<String>,
) -> ObservedEffect {
    let location = token
        .span
        .location_info(source)
        .map(|info| EffectLocation {
            file: file.display().to_string(),
            line: info.line,
            column: info.column,
            function: None,
        })
        .unwrap_or_else(|| EffectLocation {
            file: file.display().to_string(),
            line: 1,
            column: 1,
            function: None,
        });

    ObservedEffect {
        kind: kind.to_string(),
        location,
        target,
    }
}

fn is_eval_call(tokens: &[Token], index: usize) -> bool {
    matches!(tokens.get(index), Some(token) if token.kind == TokenType::Identifier && token.value == "eval")
        && matches!(
            tokens.get(index + 1).map(|t| t.kind),
            Some(TokenType::LeftParen)
        )
}

fn is_function_constructor(tokens: &[Token], index: usize) -> bool {
    matches!(tokens.get(index), Some(token) if token.kind == TokenType::New)
        && matches!(tokens.get(index + 1), Some(token) if token.kind == TokenType::Identifier && token.value == "Function")
}

fn is_proxy_constructor(tokens: &[Token], index: usize) -> bool {
    matches!(tokens.get(index), Some(token) if token.kind == TokenType::New)
        && matches!(tokens.get(index + 1), Some(token) if token.kind == TokenType::Identifier && token.value == "Proxy")
        || matches!(tokens.get(index), Some(token) if token.kind == TokenType::New)
            && matches!(tokens.get(index + 1), Some(token) if token.kind == TokenType::Identifier && token.value == "globalThis")
            && matches!(tokens.get(index + 2).map(|t| t.kind), Some(TokenType::Dot))
            && matches!(tokens.get(index + 3), Some(token) if token.kind == TokenType::Identifier && token.value == "Proxy")
}

fn is_console_write_call(tokens: &[Token], index: usize) -> bool {
    match tokens.get(index) {
        Some(token) if token.kind == TokenType::Identifier && token.value == "console" => {
            matches!(tokens.get(index + 1).map(|t| t.kind), Some(TokenType::Dot))
                && matches!(tokens.get(index + 2), Some(token) if token.kind == TokenType::Identifier)
                && matches!(
                    tokens.get(index + 3).map(|t| t.kind),
                    Some(TokenType::LeftParen)
                )
        }
        _ => false,
    }
}

fn is_deno_command_constructor(
    tokens: &[Token],
    index: usize,
) -> Option<(&'static str, Option<String>)> {
    if !matches!(tokens.get(index), Some(token) if token.kind == TokenType::New) {
        return None;
    }
    if !matches!(tokens.get(index + 1), Some(token) if token.kind == TokenType::Identifier && token.value == "Deno")
    {
        return None;
    }
    if !matches!(tokens.get(index + 2).map(|t| t.kind), Some(TokenType::Dot)) {
        return None;
    }
    if !matches!(tokens.get(index + 3), Some(token) if token.kind == TokenType::Identifier && token.value == "Command")
    {
        return None;
    }

    Some(("Process.Spawn", call_string_argument(tokens, index + 4)))
}

fn is_deno_host_call(tokens: &[Token], index: usize) -> Option<(&'static str, Option<String>)> {
    if !matches!(tokens.get(index), Some(token) if token.kind == TokenType::Identifier && token.value == "Deno")
    {
        return None;
    }

    let dot = tokens.get(index + 1)?;
    if dot.kind != TokenType::Dot {
        return None;
    }
    let member = tokens.get(index + 2)?;
    if member.kind != TokenType::Identifier {
        return None;
    }

    let kind = match member.value.as_str() {
        "open" | "openSync" | "readTextFile" | "readTextFileSync" | "readDir" | "readDirSync"
        | "stat" | "statSync" | "lstat" | "lstatSync" => Some("FileSystem.Read"),
        "create" | "createSync" | "writeTextFile" | "writeTextFileSync" | "mkdir" | "mkdirSync"
        | "remove" | "removeSync" | "rename" | "renameSync" => Some("FileSystem.Write"),
        "connect" => return Some(("Network.Connect", call_string_argument(tokens, index + 3))),
        "listen" | "serve" => {
            return Some(("Network.Listen", call_string_argument(tokens, index + 3)))
        }
        "env" => {
            let dot = tokens.get(index + 3)?;
            if dot.kind != TokenType::Dot {
                return None;
            }
            let method = tokens.get(index + 4)?;
            if method.kind != TokenType::Identifier {
                return None;
            }
            return match method.value.as_str() {
                "get" => Some(("Process.EnvRead", call_string_argument(tokens, index + 5))),
                "set" => Some(("Process.EnvWrite", call_string_argument(tokens, index + 5))),
                _ => None,
            };
        }
        "permissions" => return None,
        "exit" => return None,
        _ => None,
    }?;

    Some((kind, call_string_argument(tokens, index + 3)))
}

fn is_global_effect_call(tokens: &[Token], index: usize) -> Option<(&'static str, Option<String>)> {
    let token = tokens.get(index)?;
    match token.kind {
        TokenType::Identifier => {
            if matches!(
                tokens.get(index + 1).map(|t| t.kind),
                Some(TokenType::LeftParen)
            ) {
                return match token.value.as_str() {
                    "fetch" => Some(("Network.Fetch", call_string_argument(tokens, index + 2))),
                    "setTimeout" | "setInterval" | "queueMicrotask" => {
                        Some(("Timer.Schedule", None))
                    }
                    "getRandomValues" => Some(("Random.GetBytes", None)),
                    _ => None,
                };
            }

            if token.value == "crypto"
                && matches!(tokens.get(index + 1).map(|t| t.kind), Some(TokenType::Dot))
                && matches!(tokens.get(index + 2), Some(member) if member.kind == TokenType::Identifier && member.value == "getRandomValues")
                && matches!(
                    tokens.get(index + 3).map(|t| t.kind),
                    Some(TokenType::LeftParen)
                )
            {
                return Some(("Random.GetBytes", None));
            }

            if (token.value == "globalThis" || token.value == "window" || token.value == "self")
                && matches!(tokens.get(index + 1).map(|t| t.kind), Some(TokenType::Dot))
                && matches!(tokens.get(index + 2), Some(member) if member.kind == TokenType::Identifier && member.value == "fetch")
                && matches!(
                    tokens.get(index + 3).map(|t| t.kind),
                    Some(TokenType::LeftParen)
                )
            {
                return Some(("Network.Fetch", call_string_argument(tokens, index + 4)));
            }

            None
        }
        _ => None,
    }
}

fn is_require_call(tokens: &[Token], index: usize) -> Option<(&'static str, Option<String>)> {
    let token = tokens.get(index)?;
    if token.kind != TokenType::Identifier || token.value != "require" {
        return None;
    }
    if !matches!(
        tokens.get(index + 1).map(|t| t.kind),
        Some(TokenType::LeftParen)
    ) {
        return None;
    }

    let target = call_string_argument(tokens, index + 2)?;
    let kind = if target.contains("child_process") {
        Some("Process.Spawn")
    } else if target.contains("fs") {
        Some("FileSystem.Read")
    } else {
        None
    }?;

    Some((kind, Some(target)))
}

fn call_string_argument(tokens: &[Token], mut index: usize) -> Option<String> {
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenType::StringLiteral => return Some(token.value.clone()),
            TokenType::Comma | TokenType::LeftParen | TokenType::RightParen => {
                index += 1;
            }
            _ => break,
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

fn location_sort_key(a: &EffectLocation, b: &EffectLocation) -> std::cmp::Ordering {
    a.file
        .cmp(&b.file)
        .then_with(|| a.line.cmp(&b.line))
        .then_with(|| a.column.cmp(&b.column))
        .then_with(|| a.function.cmp(&b.function))
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.is_error())
}

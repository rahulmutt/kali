//! Eval-compat source rewriting + static constant evaluation + dynamic-import discovery.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use kali_common::{template::resolve_interpolated_template_literal, FileId};
use kali_error::Diagnostic;
use kali_lexer::{Lexer, Token, TokenType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicImportTarget {
    pub specifier: String,
    pub target: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EvalConst {
    String(String),
    Number(i64),
    Boolean(bool),
    Null,
}

impl EvalConst {
    fn render(&self) -> String {
        match self {
            EvalConst::String(value) => {
                serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value))
            }
            EvalConst::Number(value) => value.to_string(),
            EvalConst::Boolean(value) => value.to_string(),
            EvalConst::Null => "null".to_string(),
        }
    }

    fn to_string_value(&self) -> String {
        match self {
            EvalConst::String(value) => value.clone(),
            EvalConst::Number(value) => value.to_string(),
            EvalConst::Boolean(value) => value.to_string(),
            EvalConst::Null => "null".to_string(),
        }
    }

    fn is_nullish(&self) -> bool {
        matches!(self, EvalConst::Null)
    }

    fn is_truthy(&self) -> bool {
        match self {
            EvalConst::String(value) => !value.is_empty(),
            EvalConst::Number(value) => *value != 0,
            EvalConst::Boolean(value) => *value,
            EvalConst::Null => false,
        }
    }
}

pub(crate) fn rewrite_eval_compat_source(source: &str) -> String {
    let source = rewrite_static_eval_calls(source);
    rewrite_static_function_constructor_calls(&source)
}

fn rewrite_static_eval_calls(source: &str) -> String {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let bindings = collect_constant_bindings(&tokens, source);
    let mut rewritten = source.to_string();
    let mut search_start = 0usize;

    while let Some(relative) = rewritten[search_start..].find("eval(") {
        let call_start = search_start + relative;
        let arg_start = call_start + "eval(".len();
        let Some(call_end) = find_call_end(&rewritten, arg_start) else {
            break;
        };
        let arg_source = &rewritten[arg_start..call_end];
        let lexer = Lexer::new(FileId::new(1), arg_source.to_string());
        let mut arg_tokens = lexer.lex_all().tokens;
        while matches!(arg_tokens.last(), Some(token) if token.kind == TokenType::Eof) {
            arg_tokens.pop();
        }

        let mut replacement = None;
        if let Some((value, consumed)) = parse_constant_expression(&arg_tokens, 0, &bindings) {
            if consumed == arg_tokens.len() {
                if let EvalConst::String(source_snippet) = value {
                    if let Some(result) = parse_eval_source_snippet(&source_snippet) {
                        replacement = Some(result.render());
                    }
                }
            }
        }

        if let Some(replacement) = replacement {
            rewritten.replace_range(call_start..=call_end, &replacement);
            search_start = call_start + replacement.len();
        } else {
            search_start = call_end + 1;
        }
    }

    rewritten
}

fn rewrite_static_function_constructor_calls(source: &str) -> String {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let bindings = collect_constant_bindings(&tokens, source);
    let mut rewritten = source.to_string();
    let mut search_start = 0usize;

    while let Some(relative) = rewritten[search_start..].find("Function(") {
        let call_start = search_start + relative;
        if !is_bare_function_constructor_spelling(&rewritten, call_start) {
            search_start = call_start + "Function(".len();
            continue;
        }

        let prefix_start = if call_start >= 4 && &rewritten[call_start - 4..call_start] == "new " {
            call_start - 4
        } else {
            call_start
        };
        let arg_start = call_start + "Function(".len();
        let Some(call_end) = find_call_end(&rewritten, arg_start) else {
            break;
        };
        let immediate_call_end = match find_immediate_invocation_end(&rewritten, call_end) {
            Some(end) => end,
            None => {
                search_start = call_end + 1;
                continue;
            }
        };

        let arg_source = &rewritten[arg_start..call_end];
        let lexer = Lexer::new(FileId::new(1), arg_source.to_string());
        let mut arg_tokens = lexer.lex_all().tokens;
        while matches!(arg_tokens.last(), Some(token) if token.kind == TokenType::Eof) {
            arg_tokens.pop();
        }

        let mut replacement = None;
        if let Some((value, consumed)) = parse_constant_expression(&arg_tokens, 0, &bindings) {
            if consumed == arg_tokens.len() {
                if let EvalConst::String(body_source) = value {
                    if let Some(result) = parse_function_constructor_body_snippet(&body_source) {
                        replacement = Some(result.render());
                    }
                }
            }
        }

        if let Some(replacement) = replacement {
            rewritten.replace_range(prefix_start..=immediate_call_end, &replacement);
            search_start = prefix_start + replacement.len();
        } else {
            search_start = immediate_call_end + 1;
        }
    }

    rewritten
}

fn is_bare_function_constructor_spelling(source: &str, index: usize) -> bool {
    !matches!(
        source[..index].chars().next_back(),
        Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || ch == '.'
    )
}

fn find_immediate_invocation_end(source: &str, constructor_end: usize) -> Option<usize> {
    let remainder = source.get(constructor_end + 1..)?;
    let trimmed = remainder.trim_start();
    let open = constructor_end + 1 + (remainder.len().saturating_sub(trimmed.len()));
    if source.as_bytes().get(open).copied()? != b'(' {
        return None;
    }
    find_call_end(source, open + 1)
}

fn parse_function_constructor_body_snippet(source: &str) -> Option<EvalConst> {
    let trimmed = source.trim();
    let body = trimmed.strip_prefix("return")?.trim();
    let body = body.strip_suffix(';').unwrap_or(body).trim();
    if body.is_empty() {
        return None;
    }
    parse_eval_source_snippet(body)
}

fn find_call_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0i32;
    let mut index = start;
    let mut string_delimiter: Option<u8> = None;
    let mut escape = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(delimiter) = string_delimiter {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == delimiter {
                string_delimiter = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'\"' | b'`' => string_delimiter = Some(byte),
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn collect_constant_bindings(tokens: &[Token], source: &str) -> BTreeMap<String, EvalConst> {
    let mut bindings = BTreeMap::new();
    let mut index = 0usize;

    while index + 3 < tokens.len() {
        let token = &tokens[index];
        if matches!(
            token.kind,
            TokenType::Const | TokenType::Let | TokenType::Var
        ) && matches!(tokens.get(index + 1), Some(name) if name.kind == TokenType::Identifier)
            && matches!(tokens.get(index + 2), Some(eq) if eq.kind == TokenType::Eq)
        {
            let name = tokens[index + 1].value.clone();
            let expr_start = index + 3;
            let expr_end = find_statement_end(tokens, expr_start);
            if expr_end > expr_start {
                if let Some((value, consumed)) =
                    parse_constant_expression(&tokens[expr_start..expr_end], 0, &bindings)
                {
                    if consumed == expr_end - expr_start {
                        bindings.insert(name, value);
                    }
                }
            }
        }
        index += 1;
    }

    let _ = source;
    bindings
}

pub fn discover_dynamic_import_targets(
    source: &Path,
    source_contents: &str,
) -> Result<Vec<DynamicImportTarget>, Vec<Diagnostic>> {
    let lexer = Lexer::new(FileId::new(0), source_contents.to_string());
    let tokens = lexer
        .lex_all()
        .tokens
        .into_iter()
        .filter(|token| !matches!(token.kind, TokenType::Comment | TokenType::Eof))
        .collect::<Vec<_>>();
    let bindings = collect_constant_bindings(&tokens, source_contents);
    let mut targets = Vec::new();
    let mut index = 0usize;

    while index + 1 < tokens.len() {
        if tokens[index].kind != TokenType::Import || tokens[index + 1].kind != TokenType::LeftParen
        {
            index += 1;
            continue;
        }

        if let Some((specifier, next_index)) =
            parse_static_dynamic_import_specifier(&tokens, index + 1, &bindings)
        {
            if let Some(target) = resolve_dynamic_import_target(source, &specifier) {
                targets.push(DynamicImportTarget { specifier, target });
            }
            index = next_index;
        } else {
            index += 1;
        }
    }

    Ok(targets)
}

fn parse_static_dynamic_import_specifier(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(String, usize)> {
    let call_end = find_token_call_end(tokens, index)?;
    let (value, consumed) = parse_constant_expression(&tokens[index + 1..call_end], 0, env)?;
    if consumed == call_end.saturating_sub(index + 1) {
        Some((value.to_string_value(), call_end + 1))
    } else {
        None
    }
}

fn find_token_call_end(tokens: &[Token], start: usize) -> Option<usize> {
    let mut depth = 1i32;
    for (index, token) in tokens.iter().enumerate().skip(start + 1) {
        match token.kind {
            TokenType::LeftParen => depth += 1,
            TokenType::RightParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

pub(crate) fn resolve_dynamic_import_target(source: &Path, specifier: &str) -> Option<PathBuf> {
    let specifier = specifier.trim();
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }

    let parent = source.parent()?;
    let candidate = parent.join(specifier);
    let try_paths = std::iter::once(candidate.clone()).chain([
        candidate.with_extension("ts"),
        candidate.with_extension("tsx"),
        candidate.with_extension("js"),
        candidate.with_extension("jsx"),
        candidate.with_extension("mts"),
        candidate.with_extension("mjs"),
        candidate.with_extension("cts"),
        candidate.with_extension("cjs"),
    ]);
    for path in try_paths {
        if let Some(canonical) = canonicalize_dynamic_import_candidate(&path) {
            return Some(canonical);
        }
    }
    None
}

fn canonicalize_dynamic_import_candidate(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return fs::canonicalize(candidate).ok();
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
        ] {
            let index_candidate = candidate.join(index_name);
            if index_candidate.is_file() {
                return fs::canonicalize(index_candidate).ok();
            }
        }
    }

    None
}

fn find_statement_end(tokens: &[Token], start: usize) -> usize {
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    let mut depth_brace = 0i32;

    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            TokenType::LeftParen => depth_paren += 1,
            TokenType::RightParen => {
                if depth_paren == 0 {
                    return index;
                }
                depth_paren -= 1;
            }
            TokenType::LeftBracket => depth_bracket += 1,
            TokenType::RightBracket => {
                if depth_bracket == 0 {
                    return index;
                }
                depth_bracket -= 1;
            }
            TokenType::LeftBrace => depth_brace += 1,
            TokenType::RightBrace => {
                if depth_brace == 0 {
                    return index;
                }
                depth_brace -= 1;
            }
            TokenType::Semicolon if depth_paren == 0 && depth_bracket == 0 && depth_brace == 0 => {
                return index;
            }
            _ => {}
        }
    }
    tokens.len()
}

pub(crate) fn source_uses_eval_compat(tokens: &[Token]) -> bool {
    for index in 0..tokens.len() {
        if matches_direct_eval_call(tokens, index)
            || matches_direct_function_call(tokens, index)
            || matches_global_this_function_call(tokens, index)
            || matches_global_this_function_bracket_call(tokens, index)
            || matches_new_function_constructor(tokens, index)
        {
            return true;
        }
    }
    false
}

fn matches_direct_eval_call(tokens: &[Token], index: usize) -> bool {
    let previous_is_dot = index
        .checked_sub(1)
        .and_then(|previous| tokens.get(previous))
        .is_some_and(|token| token.kind == TokenType::Dot);

    matches!(tokens.get(index), Some(token) if token.kind == TokenType::Identifier && token.value == "eval")
        && tokens
            .get(index + 1)
            .is_some_and(|token| token.kind == TokenType::LeftParen)
        && !previous_is_dot
}

fn matches_direct_function_call(tokens: &[Token], index: usize) -> bool {
    let previous_is_dot = index
        .checked_sub(1)
        .and_then(|previous| tokens.get(previous))
        .is_some_and(|token| token.kind == TokenType::Dot);

    matches!(tokens.get(index), Some(token) if token.kind == TokenType::Identifier && token.value == "Function")
        && tokens
            .get(index + 1)
            .is_some_and(|token| token.kind == TokenType::LeftParen)
        && !previous_is_dot
}

fn matches_global_this_function_call(tokens: &[Token], index: usize) -> bool {
    matches_global_this_function_member(tokens, index)
        && tokens
            .get(index + 3)
            .is_some_and(|token| token.kind == TokenType::LeftParen)
}

fn matches_global_this_function_bracket_call(tokens: &[Token], index: usize) -> bool {
    matches_global_this_function_bracket_member(tokens, index)
        && tokens
            .get(index + 4)
            .is_some_and(|token| token.kind == TokenType::LeftParen)
}

fn matches_new_function_constructor(tokens: &[Token], index: usize) -> bool {
    matches!(tokens.get(index), Some(token) if token.kind == TokenType::New)
        && (matches_direct_function_constructor(tokens, index + 1)
            || matches_global_this_function_member(tokens, index + 1)
            || matches_global_this_function_bracket_member(tokens, index + 1))
}

fn matches_direct_function_constructor(tokens: &[Token], index: usize) -> bool {
    matches!(tokens.get(index), Some(token) if token.kind == TokenType::Identifier && token.value == "Function")
}

fn matches_global_this_function_member(tokens: &[Token], index: usize) -> bool {
    matches!(tokens.get(index), Some(token) if token.kind == TokenType::Identifier && token.value == "globalThis")
        && tokens
            .get(index + 1)
            .is_some_and(|token| token.kind == TokenType::Dot)
        && matches!(tokens.get(index + 2), Some(token) if token.kind == TokenType::Identifier && token.value == "Function")
}

fn matches_global_this_function_bracket_member(tokens: &[Token], index: usize) -> bool {
    matches!(tokens.get(index), Some(token) if token.kind == TokenType::Identifier && token.value == "globalThis")
        && tokens
            .get(index + 1)
            .is_some_and(|token| token.kind == TokenType::LeftBracket)
        && matches!(tokens.get(index + 2), Some(token) if token.kind == TokenType::StringLiteral && (token.value == "\"Function\"" || token.value == "'Function'"))
        && tokens
            .get(index + 3)
            .is_some_and(|token| token.kind == TokenType::RightBracket)
}

fn parse_eval_source_snippet(source: &str) -> Option<EvalConst> {
    let lexer = Lexer::new(FileId::new(1), source.to_string());
    let mut tokens = lexer.lex_all().tokens;
    while matches!(tokens.last(), Some(token) if token.kind == TokenType::Eof) {
        tokens.pop();
    }
    let env = BTreeMap::new();
    let (value, consumed) = parse_constant_expression(&tokens, 0, &env)?;
    if consumed == tokens.len() {
        Some(value)
    } else {
        None
    }
}

fn parse_constant_expression(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    parse_constant_nullish(tokens, index, env)
}

fn parse_constant_nullish(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    let (mut left, mut index) = parse_constant_logical_or(tokens, index, env)?;
    while matches!(tokens.get(index), Some(token) if token.kind == TokenType::NullCoalesce) {
        let (right, next_index) = parse_constant_logical_or(tokens, index + 1, env)?;
        if left.is_nullish() {
            left = right;
        }
        index = next_index;
    }
    Some((left, index))
}

fn parse_constant_logical_or(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    let (mut left, mut index) = parse_constant_logical_and(tokens, index, env)?;
    while matches!(tokens.get(index), Some(token) if token.kind == TokenType::OrOr) {
        let (right, next_index) = parse_constant_logical_and(tokens, index + 1, env)?;
        if !left.is_truthy() {
            left = right;
        }
        index = next_index;
    }
    Some((left, index))
}

fn parse_constant_logical_and(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    let (mut left, mut index) = parse_constant_additive(tokens, index, env)?;
    while matches!(tokens.get(index), Some(token) if token.kind == TokenType::AndAnd) {
        let (right, next_index) = parse_constant_additive(tokens, index + 1, env)?;
        if left.is_truthy() {
            left = right;
        }
        index = next_index;
    }
    Some((left, index))
}

fn parse_constant_additive(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    let (mut left, mut index) = parse_constant_primary(tokens, index, env)?;
    while let Some(token) = tokens.get(index) {
        if token.kind != TokenType::Plus {
            break;
        }
        let (right, next_index) = parse_constant_primary(tokens, index + 1, env)?;
        left = eval_plus(left, right);
        index = next_index;
    }
    Some((left, index))
}

fn parse_constant_primary(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    let token = tokens.get(index)?;
    match token.kind {
        TokenType::StringLiteral => Some((
            EvalConst::String(unquote_string_literal(&token.value)),
            index + 1,
        )),
        TokenType::Template => parse_template_constant_value(&token.value, env)
            .map(|value| (EvalConst::String(value), index + 1)),
        TokenType::NumericLiteral => token
            .value
            .parse::<i64>()
            .ok()
            .map(|value| (EvalConst::Number(value), index + 1)),
        TokenType::True => Some((EvalConst::Boolean(true), index + 1)),
        TokenType::False => Some((EvalConst::Boolean(false), index + 1)),
        TokenType::Null | TokenType::Undefined => Some((EvalConst::Null, index + 1)),
        TokenType::Identifier => env
            .get(&token.value)
            .cloned()
            .map(|value| (value, index + 1)),
        TokenType::Minus => {
            let (value, next_index) = parse_constant_primary(tokens, index + 1, env)?;
            match value {
                EvalConst::Number(number) => Some((EvalConst::Number(-number), next_index)),
                _ => None,
            }
        }
        TokenType::LeftParen => {
            let (value, next_index) = parse_constant_expression(tokens, index + 1, env)?;
            match tokens.get(next_index) {
                Some(token) if token.kind == TokenType::RightParen => Some((value, next_index + 1)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn eval_plus(left: EvalConst, right: EvalConst) -> EvalConst {
    match (left, right) {
        (EvalConst::Number(a), EvalConst::Number(b)) => EvalConst::Number(a + b),
        (left, right) => EvalConst::String(format!(
            "{}{}",
            left.to_string_value(),
            right.to_string_value()
        )),
    }
}

fn parse_template_constant_value(value: &str, env: &BTreeMap<String, EvalConst>) -> Option<String> {
    let has_interpolation = value.contains("${");
    resolve_interpolated_template_literal(value, |segment| {
        let lexer = Lexer::new(FileId::new(1), segment.to_string());
        let mut tokens = lexer.lex_all().tokens;
        while matches!(tokens.last(), Some(token) if token.kind == TokenType::Eof) {
            tokens.pop();
        }
        let (parsed, consumed) = parse_constant_expression(&tokens, 0, env)?;
        if consumed == tokens.len() {
            Some(parsed.to_string_value())
        } else {
            None
        }
    })
    .or_else(|| {
        if has_interpolation {
            None
        } else {
            Some(unquote_string_literal(value))
        }
    })
}

fn unquote_string_literal(value: &str) -> String {
    let trimmed = value.trim();
    let Some(first) = trimmed.chars().next() else {
        return trimmed.to_string();
    };
    let Some(last) = trimmed.chars().last() else {
        return trimmed.to_string();
    };

    if matches!((first, last), ('"', '"') | ('\'', '\'') | ('`', '`')) {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

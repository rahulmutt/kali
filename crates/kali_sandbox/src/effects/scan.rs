use std::{collections::BTreeSet, path::Path};

use kali_error::Diagnostic;
use kali_lexer::{Token, TokenType};

use super::inference::resolve_relative_import;
use super::report::{EffectAnalysisContext, EffectLocation, ObservedEffect};

pub(crate) fn scan_tokens_for_effects(
    file: &Path,
    source: &str,
    tokens: &[Token],
    dynamic_reasons: &mut BTreeSet<String>,
    context: &EffectAnalysisContext,
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

        if is_proxy_constructor(tokens, i) || is_proxy_revocable_call(tokens, i) {
            dynamic_reasons.insert("proxy-traps".to_string());
            i += 1;
            continue;
        }

        if is_console_write_call(tokens, i) {
            effects.push(observed_effect(file, token, source, "Console.Write", None));
            i += 1;
            continue;
        }

        if let Some(effect) = is_deno_command_constructor(tokens, i) {
            if effect.computed_host_access {
                dynamic_reasons.insert("computed-host-access".to_string());
            }
            effects.push(observed_effect(
                file,
                token,
                source,
                effect.kind,
                effect.target,
            ));
            i += 1;
            continue;
        }

        if let Some(computed_host_access) = is_deno_permissions_query(tokens, i) {
            if computed_host_access {
                dynamic_reasons.insert("computed-host-access".to_string());
            }
            i += 1;
            continue;
        }

        if let Some(effect) = is_deno_host_call(tokens, i) {
            if effect.computed_host_access {
                dynamic_reasons.insert("computed-host-access".to_string());
            }
            effects.push(observed_effect(
                file,
                token,
                source,
                effect.kind,
                effect.target,
            ));
            i += 1;
            continue;
        }

        if context.api_surface == "node" {
            if let Some(effect) = is_process_env_assignment(tokens, i) {
                if effect.computed_host_access {
                    dynamic_reasons.insert("computed-host-access".to_string());
                }
                effects.push(observed_effect(
                    file,
                    token,
                    source,
                    effect.kind,
                    effect.target,
                ));
                i += 1;
                continue;
            }
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
        && read_proxy_root(tokens, index + 1).is_some()
}

fn is_proxy_revocable_call(tokens: &[Token], index: usize) -> bool {
    let Some((cursor, _computed_host_access)) = read_proxy_root(tokens, index) else {
        return false;
    };

    let Some((member, next, _computed_member_access)) = read_property_segment(tokens, cursor)
    else {
        return false;
    };

    member == "revocable" && matches!(tokens.get(next).map(|t| t.kind), Some(TokenType::LeftParen))
}

fn read_proxy_root(tokens: &[Token], index: usize) -> Option<(usize, bool)> {
    match tokens.get(index)? {
        token if token.kind == TokenType::Identifier && token.value == "Proxy" => {
            Some((index + 1, false))
        }
        token if token.kind == TokenType::Identifier && token.value == "globalThis" => {
            let (root, next, computed) = read_property_segment(tokens, index + 1)?;
            if root == "Proxy" {
                Some((next, computed))
            } else {
                None
            }
        }
        _ => None,
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct EffectMatch {
    kind: &'static str,
    target: Option<String>,
    computed_host_access: bool,
}

fn read_property_segment(tokens: &[Token], index: usize) -> Option<(String, usize, bool)> {
    match tokens.get(index)? {
        token if token.kind == TokenType::Dot => {
            let property = tokens.get(index + 1)?;
            if property.kind != TokenType::Identifier {
                return None;
            }
            Some((property.value.clone(), index + 2, false))
        }
        token if token.kind == TokenType::LeftBracket => {
            let property = tokens.get(index + 1)?;
            let value = match property.kind {
                TokenType::Identifier => property.value.clone(),
                TokenType::StringLiteral | TokenType::NumericLiteral => {
                    unquote_token_value(&property.value)
                }
                _ => return None,
            };
            if !matches!(
                tokens.get(index + 2).map(|t| t.kind),
                Some(TokenType::RightBracket)
            ) {
                return None;
            }
            Some((value, index + 3, true))
        }
        _ => None,
    }
}

fn read_deno_root(tokens: &[Token], index: usize) -> Option<(usize, bool)> {
    match tokens.get(index)? {
        token if token.kind == TokenType::Identifier && token.value == "Deno" => {
            Some((index + 1, false))
        }
        token if token.kind == TokenType::Identifier && token.value == "globalThis" => {
            let (root, next, computed) = read_property_segment(tokens, index + 1)?;
            if root == "Deno" {
                Some((next, computed))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn read_process_root(tokens: &[Token], index: usize) -> Option<(usize, bool)> {
    match tokens.get(index)? {
        token if token.kind == TokenType::Identifier && token.value == "process" => {
            Some((index + 1, false))
        }
        token if token.kind == TokenType::Identifier && token.value == "globalThis" => {
            let (root, next, computed) = read_property_segment(tokens, index + 1)?;
            if root == "process" {
                Some((next, computed))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_deno_command_constructor(tokens: &[Token], index: usize) -> Option<EffectMatch> {
    if !matches!(tokens.get(index), Some(token) if token.kind == TokenType::New) {
        return None;
    }

    let (cursor, computed_host_access) = read_deno_root(tokens, index + 1)?;
    let (member, next, computed_member_access) = read_property_segment(tokens, cursor)?;
    if member != "Command" {
        return None;
    }

    Some(EffectMatch {
        kind: "Process.Spawn",
        target: call_string_argument(tokens, next),
        computed_host_access: computed_host_access || computed_member_access,
    })
}

fn is_deno_permissions_query(tokens: &[Token], index: usize) -> Option<bool> {
    let (cursor, computed_host_access) = read_deno_root(tokens, index)?;
    let (member, next, computed_member_access) = read_property_segment(tokens, cursor)?;
    if member != "permissions" {
        return None;
    }

    let (method, next, computed_query_access) = read_property_segment(tokens, next)?;
    if method != "query" {
        return None;
    }

    if !matches!(tokens.get(next).map(|t| t.kind), Some(TokenType::LeftParen)) {
        return None;
    }

    Some(computed_host_access || computed_member_access || computed_query_access)
}

fn is_deno_host_call(tokens: &[Token], index: usize) -> Option<EffectMatch> {
    let (cursor, computed_host_access) = read_deno_root(tokens, index)?;
    let (member, next, computed_member_access) = read_property_segment(tokens, cursor)?;
    let computed_host_access = computed_host_access || computed_member_access;

    let kind = match member.as_str() {
        "open" | "openSync" | "readTextFile" | "readTextFileSync" | "readDir" | "readDirSync"
        | "stat" | "statSync" | "lstat" | "lstatSync" => Some("FileSystem.Read"),
        "create" | "createSync" | "writeTextFile" | "writeTextFileSync" | "mkdir" | "mkdirSync"
        | "remove" | "removeSync" | "rename" | "renameSync" => Some("FileSystem.Write"),
        "connect" => {
            return Some(EffectMatch {
                kind: "Network.Connect",
                target: call_string_argument(tokens, next),
                computed_host_access,
            })
        }
        "listen" | "serve" => {
            return Some(EffectMatch {
                kind: "Network.Listen",
                target: call_string_argument(tokens, next),
                computed_host_access,
            })
        }
        "env" => {
            let (method, next, computed_env_access) = read_property_segment(tokens, next)?;
            let computed_host_access = computed_host_access || computed_env_access;
            return match method.as_str() {
                "get" => Some(EffectMatch {
                    kind: "Process.EnvRead",
                    target: call_string_argument(tokens, next),
                    computed_host_access,
                }),
                "toObject" => Some(EffectMatch {
                    kind: "Process.EnvRead",
                    target: None,
                    computed_host_access,
                }),
                "set" | "delete" => Some(EffectMatch {
                    kind: "Process.EnvWrite",
                    target: call_string_argument(tokens, next),
                    computed_host_access,
                }),
                _ => None,
            };
        }
        "permissions" => return None,
        "exit" => return None,
        _ => None,
    }?;

    Some(EffectMatch {
        kind,
        target: call_string_argument(tokens, next),
        computed_host_access,
    })
}

fn is_process_env_assignment(tokens: &[Token], index: usize) -> Option<EffectMatch> {
    let (cursor, computed_host_access) = read_process_root(tokens, index)?;
    let (member, next, computed_member_access) = read_property_segment(tokens, cursor)?;
    if member != "env" {
        return None;
    }
    if !matches!(tokens.get(next).map(|t| t.kind), Some(TokenType::Eq)) {
        return None;
    }

    Some(EffectMatch {
        kind: "Process.EnvWrite",
        target: Some("process.env".to_string()),
        computed_host_access: computed_host_access || computed_member_access,
    })
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
            TokenType::StringLiteral => return Some(unquote_token_value(&token.value)),
            TokenType::Comma | TokenType::LeftParen | TokenType::RightParen => {
                index += 1;
            }
            _ => break,
        }
    }
    None
}

fn unquote_token_value(value: &str) -> String {
    let trimmed = value.trim();
    let Some(first) = trimmed.chars().next() else {
        return trimmed.to_string();
    };
    let Some(last) = trimmed.chars().last() else {
        return trimmed.to_string();
    };

    if (first == '"' && last == '"')
        || (first == '\'' && last == '\'')
        || (first == '`' && last == '`')
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

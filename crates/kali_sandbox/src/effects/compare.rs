use std::path::Path;

use kali_error::{_error_codes::e9, Diagnostic};

use super::report::ObservedEffect;
use crate::{AccessRule, PatternKind, SandboxPolicy};

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

//! Application of the accumulated safe-fix plan to source text.

use kali_fmt::format_source;

use crate::FixPlan;

pub(crate) fn apply_fixes(source: &str, plan: &FixPlan) -> String {
    let mut rewritten = source.to_string();

    if !plan.unused_import_ranges.is_empty() {
        rewritten = rewritten
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                !(trimmed.starts_with("import ") || trimmed.starts_with("import{"))
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    if !plan.debugger_tokens.is_empty() {
        rewritten = rewritten.replace("debugger;", "");
        rewritten = rewritten.replace("debugger", "");
    }

    if !plan.var_tokens.is_empty() {
        rewritten = rewritten.replace("var ", "let ");
    }

    if !plan.let_to_const_tokens.is_empty() {
        rewritten = rewritten.replace("let ", "const ");
    }

    for replacement in plan.eqeqeq_tokens.values() {
        match *replacement {
            "===" => {
                rewritten = rewritten.replace("==", "===");
            }
            "!==" => {
                rewritten = rewritten.replace("!=", "!==");
            }
            _ => {}
        }
    }

    format_source(&rewritten)
}

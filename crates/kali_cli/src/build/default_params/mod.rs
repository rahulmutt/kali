//! Default parameters (default-parameters spec §3.2). See `apply_default_params`.
//!
//! Kali emits every call at its callee's exact arity and has no runtime
//! `undefined` distinct from `0` (register cluster G4), so a default cannot be
//! applied inside the callee. This pass applies it at the call site instead:
//! every direct call `f(…)` that omits a defaulted argument, or passes a
//! literal `undefined` for it, gets a fresh clone of the default. Any other
//! argument passes through unchanged (the human partner's ruling). Afterwards
//! the declaration's defaults are removed, so every later stage sees an
//! ordinary fixed-arity function.
//!
//! The pass runs after `block_scope_rename`, so every binding has a unique
//! spelling and a call can be matched to its declaration by name.

mod literal;

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{CallExpression, Expression, FunctionDeclaration, Statement};

use super::block_scope_rename::walk::{self, BindKind, Hooks, ScopeKind};
use literal::{classify_default, DefaultKind};

#[cfg(test)]
#[path = "default_params_tests.rs"]
mod default_params_tests;

/// Rewrites every direct call of a defaulted function declaration and strips
/// the defaults. Returns the refusal messages (E5506 texts); when any is
/// returned the program is left exactly as it was.
pub fn apply_default_params(statements: &mut [Statement], compat_eval: bool) -> Vec<String> {
    let mut collector = Collector::default();
    walk::walk_program(statements, &mut collector);
    if collector.defaulted.is_empty() {
        return Vec::new();
    }
    if compat_eval {
        return vec![kali_common::default_param_eval_refused_message().to_string()];
    }

    let exported = exported_names(statements);
    let refusals = collector.refusals(&exported);
    if !refusals.is_empty() {
        return refusals;
    }

    let mut filler = Filler {
        defaulted: collector.defaulted,
    };
    walk::walk_program(statements, &mut filler);
    Vec::new()
}

/// A defaulted declaration: its parameter names and index-aligned defaults.
struct Defaulted {
    params: Vec<String>,
    defaults: Vec<Option<Box<Expression>>>,
    /// Set when the declaration itself is out of scope (§3.2 step 1).
    refusal: Option<String>,
}

/// One direct call `name(…)`.
struct Call {
    name: String,
    argument_count: usize,
    has_spread: bool,
}

#[derive(Default)]
struct Collector {
    defaulted: BTreeMap<String, Defaulted>,
    calls: Vec<Call>,
    /// Every name referenced other than as the callee of a direct call.
    value_uses: BTreeSet<String>,
    /// The callee name of the call whose callee is about to be walked.
    pending_callee: Option<String>,
}

impl Collector {
    /// One message per refused function, in name order, then one per refused
    /// call, in source order.
    fn refusals(&self, exported: &BTreeSet<String>) -> Vec<String> {
        let mut refusals = Vec::new();
        for (name, defaulted) in &self.defaulted {
            if let Some(refusal) = &defaulted.refusal {
                refusals.push(refusal.clone());
            } else if exported.contains(name) {
                refusals.push(kali_common::default_param_exported_message(name));
            } else if self.value_uses.contains(name) {
                refusals.push(kali_common::default_param_value_use_message(name));
            }
        }
        for call in &self.calls {
            let Some(defaulted) = self.defaulted.get(&call.name) else {
                continue;
            };
            if call.has_spread {
                refusals.push(kali_common::default_param_spread_call_message(&call.name));
                continue;
            }
            // A literal `undefined` for a parameter with no default is passed
            // through, like any other argument; only an omission is refused.
            for index in call.argument_count..defaulted.params.len() {
                if defaulted.defaults[index].is_none() {
                    refusals.push(kali_common::default_param_omitted_argument_message(
                        &call.name,
                        &defaulted.params[index],
                    ));
                    break;
                }
            }
        }
        refusals
    }
}

impl Hooks for Collector {
    fn enter(&mut self, _kind: ScopeKind, _label: Option<&str>) {}
    fn exit(&mut self) {}
    fn bind(&mut self, _name: &mut String, _kind: BindKind) {}

    fn reference(&mut self, name: &mut String) {
        if self.pending_callee.as_deref() == Some(name.as_str()) {
            self.pending_callee = None;
        } else {
            self.value_uses.insert(name.clone());
        }
    }

    fn call(&mut self, callee: &Expression) {
        self.pending_callee = match callee {
            Expression::Identifier(name) => Some(name.clone()),
            _ => None,
        };
    }

    fn call_expression(&mut self, call: &mut CallExpression) {
        let Expression::Identifier(name) = &call.callee else {
            return;
        };
        self.calls.push(Call {
            name: name.clone(),
            argument_count: call.args.len(),
            has_spread: call
                .args
                .iter()
                .any(|argument| matches!(argument, Expression::SpreadElement(_))),
        });
    }

    fn function_declaration(&mut self, decl: &mut FunctionDeclaration) {
        if decl.defaults.is_empty() {
            return;
        }
        let refusal = if decl.is_async || decl.generator {
            Some(kali_common::default_param_async_or_generator_message(
                &decl.name,
            ))
        } else {
            decl.params
                .iter()
                .zip(&decl.defaults)
                .find_map(
                    |(param, default)| match default.as_deref().map(classify_default) {
                        None | Some(DefaultKind::Scalar) => None,
                        Some(DefaultKind::Composite) => Some(
                            kali_common::default_param_composite_message(&decl.name, param),
                        ),
                        Some(DefaultKind::Other) => Some(
                            kali_common::default_param_not_literal_message(&decl.name, param),
                        ),
                    },
                )
        };
        self.defaulted.insert(
            decl.name.clone(),
            Defaulted {
                params: decl.params.clone(),
                defaults: decl.defaults.clone(),
                refusal,
            },
        );
    }
}

/// Fills every direct call of a defaulted function and strips the defaults.
/// Runs only when the collector found nothing to refuse.
struct Filler {
    defaulted: BTreeMap<String, Defaulted>,
}

impl Hooks for Filler {
    fn enter(&mut self, _kind: ScopeKind, _label: Option<&str>) {}
    fn exit(&mut self) {}
    fn bind(&mut self, _name: &mut String, _kind: BindKind) {}
    fn reference(&mut self, _name: &mut String) {}

    fn call_expression(&mut self, call: &mut CallExpression) {
        let Expression::Identifier(name) = &call.callee else {
            return;
        };
        let Some(defaulted) = self.defaulted.get(name) else {
            return;
        };
        for (index, default) in defaulted.defaults.iter().enumerate() {
            let Some(default) = default else {
                continue;
            };
            match call.args.get_mut(index) {
                Some(argument) if is_literal_undefined(argument) => {
                    *argument = (**default).clone();
                }
                Some(_) => {}
                None => call.args.push((**default).clone()),
            }
        }
    }

    fn function_declaration(&mut self, decl: &mut FunctionDeclaration) {
        decl.defaults.clear();
    }
}

/// A literal `undefined`, or `void` applied to a literal.
fn is_literal_undefined(argument: &Expression) -> bool {
    match argument {
        Expression::Identifier(name) => name == "undefined",
        Expression::UnaryExpression(unary) => {
            unary.operator == "void" && matches!(unary.argument, Expression::Literal(_))
        }
        _ => false,
    }
}

/// Names exported by a top-level `export { … }` with no `from`.
fn exported_names(statements: &[Statement]) -> BTreeSet<String> {
    statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::ExportNamed(export) if export.source.is_none() => Some(export),
            _ => None,
        })
        .flat_map(|export| {
            export
                .specifiers
                .iter()
                .map(|specifier| specifier.local.clone())
        })
        .collect()
}

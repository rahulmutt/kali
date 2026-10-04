//! Class instances (spec docs/superpowers/specs/2026-10-04-class-instances-design.md):
//! rewrites each in-slice program class to an object-literal factory and
//! `__this`-taking functions, and refuses every instance it cannot prove.

pub(crate) mod classes;
pub(crate) mod provenance;
pub(crate) mod scopes;
pub(crate) mod translate;
pub(crate) mod uses;
pub(crate) mod walk;

use std::collections::BTreeSet;

use kali_ast::{Expression, Statement};
use kali_common::{class_construction_unavailable_message, CLASS_REASON_UNLOWERED};
use kali_error::{_error_codes::e5, diagnostic::Diagnostic};

use classes::ClassPlans;
use walk::{walk, Cx, FrameKind, Pos, Visitor};

/// The outcome of [`rewrite_class_instances`].
pub struct ClassRewrite {
    /// Whether any class was rewritten.
    pub changed: bool,
    /// E5506 refusals, in a deterministic order.
    pub diagnostics: Vec<Diagnostic>,
}

/// Rewrites every provable in-slice program class to `C__new` / `C__m`
/// functions and refuses every instance it cannot prove.
pub fn rewrite_class_instances(statements: &mut Vec<Statement>) -> ClassRewrite {
    let spelled = scopes::Scopes::build(statements).spelled().clone();
    let mut plans = classes::plan_classes(statements, &spelled);
    let mut diagnostics = std::mem::take(&mut plans.diagnostics);
    if plans.rewritten.is_empty() {
        return ClassRewrite { changed: false, diagnostics };
    }
    provenance::reassociate_new(statements, &plans);
    let scopes = scopes::Scopes::build(statements);
    let env = provenance::build_env(statements, &scopes, &plans);
    let prov = provenance::Provenance::solve(statements, &env);
    diagnostics.extend(uses::check_and_rewrite(statements, &env, &prov));
    translate::translate_classes(statements, &plans);
    diagnostics.extend(sweep(statements, &plans));
    ClassRewrite { changed: true, diagnostics }
}

/// The A-10 backstop: anything of a rewritten class that survived translation
/// would reach code generation unlowered, so refuse it (once per class).
fn sweep(statements: &mut Vec<Statement>, plans: &ClassPlans) -> Vec<Diagnostic> {
    let scopes = scopes::Scopes::build(statements);
    let mut sweeper = Sweep { plans, scopes: &scopes, reported: BTreeSet::new(), diagnostics: Vec::new() };
    walk(statements, &mut sweeper);
    sweeper.diagnostics
}

struct Sweep<'p> {
    plans: &'p ClassPlans,
    scopes: &'p scopes::Scopes,
    reported: BTreeSet<String>,
    diagnostics: Vec<Diagnostic>,
}

impl Sweep<'_> {
    fn report(&mut self, class: &str) {
        if self.reported.insert(class.to_string()) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                class_construction_unavailable_message(class, CLASS_REASON_UNLOWERED),
            ));
        }
    }

    /// The class whose generated function is `name` (`C__new` or `C__m`).
    fn generated_owner(&self, name: &str) -> Option<String> {
        self.plans
            .rewritten
            .values()
            .find(|class| {
                name == format!("{}__new", class.name)
                    || class.methods.iter().any(|m| name == format!("{}__{m}", class.name))
            })
            .map(|class| class.name.clone())
    }
}

impl Visitor for Sweep<'_> {
    fn expr(&mut self, expr: &mut Expression, _pos: &Pos, cx: &Cx) {
        match expr {
            Expression::NewExpression(new) => {
                if let Some(target) = classes::new_target(new) {
                    if self.plans.rewritten.contains_key(target) {
                        let target = target.to_string();
                        self.report(&target);
                    }
                }
            }
            Expression::ThisExpression => {
                let frame = cx.frames.iter().rev().find(|f| f.kind != FrameKind::Arrow);
                let name = frame.and_then(|f| f.key.rsplit('/').next());
                if let Some(owner) = name.and_then(|n| self.generated_owner(n)) {
                    self.report(&owner);
                }
            }
            Expression::Identifier(name) => {
                if self.plans.rewritten.contains_key(name.as_str())
                    && self.scopes.resolve(name, cx) == scopes::Resolved::Free
                {
                    let name = name.clone();
                    self.report(&name);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "classes_tests.rs"]
mod classes_tests;
#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
#[cfg(test)]
#[path = "provenance_tests.rs"]
mod provenance_tests;
#[cfg(test)]
#[path = "scopes_tests.rs"]
mod scopes_tests;
#[cfg(test)]
#[path = "translate_tests.rs"]
mod translate_tests;
#[cfg(test)]
#[path = "uses_tests.rs"]
mod uses_tests;
#[cfg(test)]
#[path = "walk_tests.rs"]
mod walk_tests;

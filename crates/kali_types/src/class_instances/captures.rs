//! Names a rewritten class's body reads (rulings R-25, R-26). The rewrite
//! moves the body into `C__new` and `C__m` functions declared beside the
//! class, and runs field initializers inside the factory, whose parameters are
//! the constructor's. So a body that reads a variable of an enclosing function
//! (R-25), or a field initializer that names something the constructor
//! declares (R-26), would read a different binding after the rewrite.

use std::collections::BTreeSet;

use kali_ast::{Expression, Statement};
use kali_common::{
    class_construction_unavailable_message, CLASS_REASON_ENCLOSING_LOCAL,
    CLASS_REASON_INITIALIZER_SCOPE,
};
use kali_error::{_error_codes::e5, diagnostic::Diagnostic};

use super::classes::ClassPlans;
use super::scopes::Scopes;
use super::walk::{walk, Cx, FrameKind, Pos, Visitor};

pub(crate) fn check_captures(
    statements: &mut Vec<Statement>,
    scopes: &Scopes,
    plans: &ClassPlans,
) -> Vec<Diagnostic> {
    let mut captures = Captures { scopes, plans, reported: BTreeSet::new(), diagnostics: Vec::new() };
    walk(statements, &mut captures);
    captures.diagnostics
}

struct Captures<'a> {
    scopes: &'a Scopes,
    plans: &'a ClassPlans,
    reported: BTreeSet<(String, &'static str)>,
    diagnostics: Vec<Diagnostic>,
}

impl Captures<'_> {
    fn refuse(&mut self, class: &str, reason: &'static str) {
        if self.reported.insert((class.to_string(), reason)) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                class_construction_unavailable_message(class, reason),
            ));
        }
    }
}

/// The rewritten class whose body frame `kind` is, with whether it is the
/// field-initializer frame.
fn class_frame<'k>(kind: &'k FrameKind, plans: &ClassPlans) -> Option<(&'k str, bool)> {
    let (class, field_init) = match kind {
        FrameKind::Method { class, .. } | FrameKind::Constructor { class } => (class, false),
        FrameKind::FieldInit { class } => (class, true),
        FrameKind::Program | FrameKind::Function { .. } | FrameKind::Arrow => return None,
    };
    plans.rewritten.contains_key(class).then_some((class.as_str(), field_init))
}

impl Visitor for Captures<'_> {
    fn expr(&mut self, expr: &mut Expression, _pos: &Pos, cx: &Cx) {
        let Expression::Identifier(name) = expr else { return };
        // `None` for a free name (a global).
        let declared = self.scopes.declaring_frame(name, cx).map(str::to_string);
        for (index, frame) in cx.frames.iter().enumerate() {
            let Some((class, field_init)) = class_frame(&frame.kind, self.plans) else { continue };
            let inside = cx.frames[index..].iter().any(|f| Some(&f.key) == declared.as_ref());
            if inside {
                continue;
            }
            if declared.as_deref().is_some_and(|d| !d.is_empty()) {
                self.refuse(class, CLASS_REASON_ENCLOSING_LOCAL);
            }
            if field_init {
                let parent = Cx { frames: cx.frames[..index].to_vec() };
                let constructor = parent.child_key(&format!("{class}#constructor"));
                if self.scopes.declares(&constructor, name) {
                    self.refuse(class, CLASS_REASON_INITIALIZER_SCOPE);
                }
            }
        }
    }
}

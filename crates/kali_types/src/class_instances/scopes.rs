//! Per-frame name resolution: which frame declares a name, and whether it
//! declares it more than once (then the name is ambiguous).

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{ClassDeclaration, ClassExpression, Expression, FunctionDeclaration, Statement, VariableDeclarator};

use super::walk::{walk, Cx, FnKey, Pos, Visitor};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct BindingId {
    pub frame: FnKey,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Resolved {
    Binding(BindingId),
    Ambiguous,
    Free,
}

pub(crate) struct Scopes {
    declared: BTreeMap<FnKey, BTreeMap<String, u32>>,
    params: BTreeMap<FnKey, Vec<String>>,
    spelled: BTreeSet<String>,
}

#[derive(Default)]
struct Collect {
    declared: BTreeMap<FnKey, BTreeMap<String, u32>>,
    params: BTreeMap<FnKey, Vec<String>>,
    spelled: BTreeSet<String>,
}

impl Collect {
    fn declare(&mut self, name: &str, cx: &Cx) {
        *self
            .declared
            .entry(cx.key().to_string())
            .or_default()
            .entry(name.to_string())
            .or_insert(0) += 1;
        self.spelled.insert(name.to_string());
    }
}

impl Visitor for Collect {
    fn expr(&mut self, expr: &mut Expression, _pos: &Pos, _cx: &Cx) {
        if let Expression::Identifier(name) = expr {
            self.spelled.insert(name.clone());
        }
    }
    fn enter_frame(&mut self, cx: &Cx, params: &[String]) {
        for param in params {
            self.declare(param, cx);
            self.params.entry(cx.key().to_string()).or_default().push(param.clone());
        }
    }
    fn class_decl(&mut self, class: &ClassDeclaration, _exported_default: bool, cx: &Cx) {
        self.declare(&class.name, cx);
    }
    fn class_expr(&mut self, class: &ClassExpression, _cx: &Cx) {
        if let Some(id) = &class.id {
            self.spelled.insert(id.clone());
        }
    }
    fn var_declarator(&mut self, d: &VariableDeclarator, _kind: &str, cx: &Cx) {
        self.declare(&d.id, cx);
    }
    fn function_decl(&mut self, f: &FunctionDeclaration, cx: &Cx) {
        self.declare(&f.name, cx);
    }
    fn catch_param(&mut self, name: &str, cx: &Cx) {
        self.declare(name, cx);
    }
    fn export_specifier(&mut self, local: &str) {
        self.spelled.insert(local.to_string());
    }
}

impl Scopes {
    pub(crate) fn build(statements: &mut Vec<Statement>) -> Scopes {
        let mut collect = Collect::default();
        walk(statements, &mut collect);
        Scopes { declared: collect.declared, params: collect.params, spelled: collect.spelled }
    }

    /// The innermost enclosing frame that declares `name`.
    pub(crate) fn resolve(&self, name: &str, cx: &Cx) -> Resolved {
        for frame in cx.frames.iter().rev() {
            if let Some(count) = self.declared.get(&frame.key).and_then(|names| names.get(name)) {
                return if *count > 1 {
                    Resolved::Ambiguous
                } else {
                    Resolved::Binding(BindingId { frame: frame.key.clone(), name: name.to_string() })
                };
            }
        }
        Resolved::Free
    }

    pub(crate) fn params(&self, frame: &str) -> &[String] {
        self.params.get(frame).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Every identifier, declaration, parameter and class name the program spells.
    pub(crate) fn spelled(&self) -> &BTreeSet<String> {
        &self.spelled
    }
}

//! Per-frame name resolution: which frame declares a name, and whether it
//! declares it more than once (then the name is ambiguous).

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{
    ClassDeclaration, ClassExpression, Expression, FunctionDeclaration, ImportSpecifier, Statement,
    VariableDeclarator,
};

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
        match expr {
            Expression::Identifier(name) => {
                self.spelled.insert(name.clone());
            }
            // A function expression's or arrow's own name (R-30r).
            Expression::FunctionExpression(f) => self.spelled.extend(f.id.clone()),
            Expression::ArrowFunctionExpression(a) => self.spelled.extend(a.id.clone()),
            _ => {}
        }
    }
    fn stmts(&mut self, list: &mut Vec<Statement>, _cx: &Cx) {
        // Import locals (R-30r); the walker does not descend into imports.
        for statement in list.iter() {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            for specifier in &import.specifiers {
                match specifier {
                    ImportSpecifier::Default(local) | ImportSpecifier::Namespace(local) => {
                        self.spelled.insert(local.clone());
                    }
                    ImportSpecifier::Named(named) | ImportSpecifier::Type(named) => {
                        self.spelled.extend(named.iter().map(|n| n.local.clone()));
                    }
                    ImportSpecifier::SideEffect => {}
                }
            }
        }
    }
    fn enter_frame(&mut self, cx: &Cx, params: &[String]) {
        for param in params {
            self.declare(param, cx);
            self.params
                .entry(cx.key().to_string())
                .or_default()
                .push(param.clone());
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
        Scopes {
            declared: collect.declared,
            params: collect.params,
            spelled: collect.spelled,
        }
    }

    /// The innermost enclosing frame that declares `name`.
    pub(crate) fn resolve(&self, name: &str, cx: &Cx) -> Resolved {
        for frame in cx.frames.iter().rev() {
            if let Some(count) = self
                .declared
                .get(&frame.key)
                .and_then(|names| names.get(name))
            {
                return if *count > 1 {
                    Resolved::Ambiguous
                } else {
                    Resolved::Binding(BindingId {
                        frame: frame.key.clone(),
                        name: name.to_string(),
                    })
                };
            }
        }
        Resolved::Free
    }

    /// The innermost enclosing frame that declares `name`, however often.
    pub(crate) fn declaring_frame<'c>(&self, name: &str, cx: &'c Cx) -> Option<&'c str> {
        cx.frames
            .iter()
            .rev()
            .find(|frame| self.declares(&frame.key, name))
            .map(|frame| frame.key.as_str())
    }

    /// Whether `frame` itself declares `name` (a parameter or a declaration).
    pub(crate) fn declares(&self, frame: &str, name: &str) -> bool {
        self.declared
            .get(frame)
            .is_some_and(|names| names.contains_key(name))
    }

    pub(crate) fn params(&self, frame: &str) -> &[String] {
        self.params.get(frame).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Every identifier, declaration, parameter and class name the program spells.
    pub(crate) fn spelled(&self) -> &BTreeSet<String> {
        &self.spelled
    }
}

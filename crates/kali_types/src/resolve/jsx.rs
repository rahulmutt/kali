//! JSX resolution.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_jsx_element(&mut self, expr: &JsxElement) {
        for child in &expr.children {
            self.resolve_jsx_child(child);
        }
    }

    pub(crate) fn resolve_jsx_fragment(&mut self, expr: &JsxFragment) {
        for child in &expr.children {
            self.resolve_jsx_child(child);
        }
    }

    pub(crate) fn resolve_jsx_child(&mut self, child: &JsxChild) {
        match child {
            JsxChild::JsxText(_) => {}
            JsxChild::JsxExpression(container) => {
                if let Some(expr) = &container.expression {
                    self.resolve_expression(expr);
                }
            }
            JsxChild::JsxElement(child) => self.resolve_jsx_element(child),
            JsxChild::JsxFragment(child) => self.resolve_jsx_fragment(child),
        }
    }

}

#[cfg(test)]
#[path = "jsx_tests.rs"]
mod jsx_tests;

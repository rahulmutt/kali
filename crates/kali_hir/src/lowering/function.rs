//! Function and class-expression lowering.

use crate::node::{HirNodeId, HirNodeKind};
use crate::result::FunctionFlavor;
use crate::HirLowerer;
use kali_ast::{ArrowFunctionExpression, ClassExpression, FunctionExpression, ReturnStatement, Statement};

impl HirLowerer {
    pub(crate) fn lower_function_expression(&mut self, expr: &FunctionExpression) -> HirNodeId {
        let name = expr
            .id
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| self.next_synthetic_function_name());
        let id = self
            .builder
            .alloc_text(HirNodeKind::FunctionExpr, None, name);
        for param in &expr.params {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, param.name.clone())
            );
        }
        if let Some(body) = &expr.body {
            push_child!(
                self,
                id,
                self.lower_statement(&Statement::BlockStatement((**body).clone()))
            );
        }
        self.record_function_flavor(
            id,
            FunctionFlavor::from_flags(expr.is_async, expr.generator),
        );
        id
    }

    pub(crate) fn lower_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression) -> HirNodeId {
        let name = self.next_synthetic_function_name();
        let id = self
            .builder
            .alloc_text(HirNodeKind::FunctionExpr, None, name);
        for param in &expr.params {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, param.name.clone())
            );
        }
        push_child!(
            self,
            id,
            self.lower_statement(&Statement::ReturnStatement(ReturnStatement {
                argument: Some(expr.body.clone()),
            }))
        );
        self.record_function_flavor(id, FunctionFlavor::from_flags(expr.is_async, false));
        id
    }

    pub(crate) fn lower_class_expression(&mut self, expr: &ClassExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::ClassExpr,
            None,
            expr.id.clone().unwrap_or_default(),
        );
        push_child!(self, id, self.lower_class_body(&expr.body));
        id
    }
}

#[cfg(test)]
#[path = "function_tests.rs"]
mod function_tests;

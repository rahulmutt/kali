//! Expression lowering: the `lower_expression` dispatcher + template/update/assignment/optional-chain helpers.

use crate::helpers::{assignment_op_text, logical_op_text, lower_literal_value, update_op_text};
use crate::node::{HirNodeId, HirNodeKind};
use crate::HirLowerer;
use kali_ast::{
    ArrayExpression, AssignmentExpression, Expression, ExpressionOrSpread, MetaProperty,
    ObjectExpression, OptionalChainExpression, OptionalChainInner, TemplateLiteral,
    UpdateExpression,
};

impl HirLowerer {
    pub(crate) fn lower_expression(&mut self, expression: &Expression) -> HirNodeId {
        match expression {
            Expression::Identifier(name) => {
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, name.clone())
            }
            Expression::Literal(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, lower_literal_value(value))
            }
            Expression::BinaryExpression(expr) => {
                let id =
                    self.builder
                        .alloc_text(HirNodeKind::BinaryExpr, None, expr.operator.clone());
                push_child!(self, id, self.lower_expression(&expr.left));
                push_child!(self, id, self.lower_expression(&expr.right));
                id
            }
            Expression::UnaryExpression(expr) => {
                let id =
                    self.builder
                        .alloc_text(HirNodeKind::UnaryExpr, None, expr.operator.clone());
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::CallExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::CallExpr, None);
                push_child!(self, id, self.lower_expression(&expr.callee));
                for arg in &expr.args {
                    push_child!(self, id, self.lower_expression(arg));
                }
                id
            }
            Expression::MemberExpression(expr) => {
                let id =
                    self.builder
                        .alloc_text(HirNodeKind::MemberExpr, None, expr.property.clone());
                push_child!(self, id, self.lower_expression(&expr.object));
                // Computed access `a[<expr>]` carries the structured index as a
                // second child so codegen can evaluate arithmetic/dynamic indices
                // at runtime; dot access (`a.b`) keeps a single `[object]` child.
                if let Some(index) = &expr.computed_index {
                    push_child!(self, id, self.lower_expression(index));
                }
                id
            }
            Expression::ArrayExpression(ArrayExpression { elements }) => {
                let id = self.builder.alloc(HirNodeKind::ArrayExpr, None);
                for element in elements {
                    match element {
                        Some(ExpressionOrSpread::Expression(expr)) => {
                            push_child!(self, id, self.lower_expression(expr))
                        }
                        Some(ExpressionOrSpread::Spread(spread)) => push_child!(
                            self,
                            id,
                            self.lower_expression(&Expression::SpreadElement(Box::new(
                                spread.clone()
                            )))
                        ),
                        Some(ExpressionOrSpread::Empty) | None => {}
                    }
                }
                id
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                let id = self.builder.alloc(HirNodeKind::ObjectExpr, None);
                for property in properties {
                    push_child!(self, id, self.lower_object_property(property));
                }
                id
            }
            Expression::FunctionExpression(expr) => self.lower_function_expression(expr),
            Expression::ArrowFunctionExpression(expr) => self.lower_arrow_function_expression(expr),
            Expression::ClassExpression(expr) => self.lower_class_expression(expr),
            Expression::NewExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::NewExpr, None);
                push_child!(self, id, self.lower_expression(&expr.callee));
                for arg in &expr.args {
                    push_child!(self, id, self.lower_expression(arg));
                }
                id
            }
            Expression::TemplateLiteral(TemplateLiteral {
                quasis,
                expressions,
            }) => {
                let id = self.builder.alloc(HirNodeKind::TemplateLiteral, None);
                for quasi in quasis {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Literal, None, quasi.value.clone())
                    );
                }
                for expr in expressions {
                    push_child!(self, id, self.lower_expression(expr));
                }
                id
            }
            Expression::TaggedTemplateExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::TemplateLiteral, None);
                push_child!(self, id, self.lower_expression(&expr.tag));
                push_child!(self, id, self.lower_template_literal(&expr.template));
                id
            }
            Expression::UpdateExpression(expr) => self.lower_update_expression(expr),
            Expression::AssignmentExpression(expr) => self.lower_assignment_expression(expr),
            Expression::LogicalExpression(expr) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::LogicalExpr,
                    None,
                    logical_op_text(&expr.operator),
                );
                push_child!(self, id, self.lower_expression(&expr.left));
                push_child!(self, id, self.lower_expression(&expr.right));
                id
            }
            Expression::ConditionalExpression(expr) => {
                // Marker text "?": MIR/LIR erase the node KIND (Expr -> Value)
                // but preserve text — this is how codegen tells a ternary from
                // an aggregate literal (both are otherwise text-less Values).
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::ConditionalExpr, None, "?");
                push_child!(self, id, self.lower_expression(&expr.test));
                push_child!(self, id, self.lower_expression(&expr.consequent));
                push_child!(self, id, self.lower_expression(&expr.alternate));
                id
            }
            Expression::SequenceExpression(expr) => {
                let id = self.builder.alloc_text(HirNodeKind::SequenceExpr, None, "");
                for subexpr in &expr.expressions {
                    push_child!(self, id, self.lower_expression(subexpr));
                }
                id
            }
            Expression::ParenthesizedExpression(expr) => self.lower_expression(&expr.expression),
            Expression::YieldExpression(expr) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::YieldExpr,
                    None,
                    if expr.delegate { "delegate" } else { "yield" },
                );
                if let Some(argument) = &expr.argument {
                    push_child!(self, id, self.lower_expression(argument));
                }
                id
            }
            Expression::AwaitExpression(expr) => {
                // Mark the await node with a distinct `"await"` text marker so
                // codegen can dispatch a value-passthrough arm for it. Without a
                // marker the node is a text-less 1-child `Value`, indistinguishable
                // from a single-element array literal `[x]` or a grouping wrapper —
                // and the text-less aggregate path DROPS the operand and pushes 0
                // (throw-fallout Stage 3 Task 4). The marker keeps await
                // unambiguous while transparent-unwrap helpers still tunnel through
                // it (see `unwrap_transparent_value_node`).
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::AwaitExpr, None, "await");
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::OptionalChainExpression(expr) => self.lower_optional_chain(expr),
            Expression::ChainExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::ChainExpr, None);
                push_child!(self, id, self.lower_expression(&expr.expression));
                id
            }
            Expression::SpreadElement(expr) => {
                let id = self.builder.alloc_text(HirNodeKind::Spread, None, "spread");
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::RestElement(expr) => {
                let id = self.builder.alloc(HirNodeKind::Rest, None);
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::ImportExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::ImportExpr, None);
                push_child!(self, id, self.lower_expression(&expr.source));
                id
            }
            Expression::DecoratedExpression(expr) => self.lower_expression(&expr.expression),
            Expression::JsxElement(_expr) => self.builder.alloc(HirNodeKind::JsxElement, None),
            Expression::JsxFragment(_expr) => self.builder.alloc(HirNodeKind::JsxFragment, None),
            Expression::JsxEmptyExpression => self.builder.alloc(HirNodeKind::Unknown, None),
            Expression::TypeAssertion(expr) => self.lower_expression(&expr.expression),
            Expression::SatisfiesExpression(expr) => self.lower_expression(&expr.expression),
            Expression::ThisExpression => self.builder.alloc(HirNodeKind::ThisExpr, None),
            Expression::SuperExpression => {
                self.builder.alloc_text(HirNodeKind::Unknown, None, "super")
            }
            Expression::PrivateIdentifier(name) => {
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, format!("#{}", name))
            }
            Expression::BigIntLiteral(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
            Expression::MetaProperty(MetaProperty { meta, property }) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::MetaProperty,
                    None,
                    format!("{}.{}", meta, property),
                );
                id
            }
        }
    }

    pub(crate) fn lower_template_literal(&mut self, template: &TemplateLiteral) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::TemplateLiteral, None);
        for quasi in &template.quasis {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, quasi.value.clone())
            );
        }
        for expr in &template.expressions {
            push_child!(self, id, self.lower_expression(expr));
        }
        id
    }

    pub(crate) fn lower_optional_chain(&mut self, expr: &OptionalChainExpression) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::OptionalChain, None);
        match expr.inner.as_ref() {
            OptionalChainInner::NonNull {
                object,
                optional: _,
            } => {
                push_child!(self, id, self.lower_expression(object));
            }
        }
        id
    }

    pub(crate) fn lower_update_expression(&mut self, expr: &UpdateExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::UpdateExpr,
            None,
            update_op_text(&expr.operator, expr.prefix),
        );
        push_child!(self, id, self.lower_expression(&expr.argument));
        id
    }

    pub(crate) fn lower_assignment_expression(&mut self, expr: &AssignmentExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::AssignmentExpr,
            None,
            assignment_op_text(&expr.operator),
        );
        push_child!(self, id, self.lower_expression(&expr.left));
        push_child!(self, id, self.lower_expression(&expr.right));
        id
    }
}

#[cfg(test)]
#[path = "expression_tests.rs"]
mod expression_tests;

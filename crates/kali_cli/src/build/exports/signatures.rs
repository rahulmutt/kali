//! Declared/inferred function signature collection.

use super::super::entrypoint::generator_function_unavailable_message;
use super::super::helpers::invalid_export_surface;
use super::types::{infer_block_return_type, infer_expression_type, infer_static_truthiness};

use std::collections::BTreeMap;
use std::path::Path;

use kali_ast::{BlockStatement, Expression, OptionalChainInner, Statement, VariableDeclaration};

use kali_error::{_error_codes::e5, Diagnostic};

pub(crate) fn collect_declared_function_signatures(
    statements: &[Statement],
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, String> {
    let mut declared_function_signatures = BTreeMap::new();
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                if func.generator {
                    diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        generator_function_unavailable_message(
                            func.is_async,
                            Some(func.body.as_ref()),
                        ),
                    ));
                    continue;
                }

                let signature = infer_function_signature(&func.params, &func.body, func.is_async);
                if declared_function_signatures
                    .insert(func.name.clone(), signature)
                    .is_some()
                {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!("duplicate export name `{}`", func.name),
                    ));
                }
            }
            Statement::VariableDeclaration(declaration) if declaration.kind == "const" => {
                collect_declared_function_binding_signatures(
                    declaration,
                    source_path,
                    diagnostics,
                    &mut declared_function_signatures,
                );
            }
            _ => {}
        }
    }

    declared_function_signatures
}

fn collect_declared_function_binding_signatures(
    declaration: &VariableDeclaration,
    source_path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
    declared_function_signatures: &mut BTreeMap<String, String>,
) {
    let mut known_signatures = declared_function_signatures.clone();

    for declarator in &declaration.declarations {
        let Some(signature) = infer_function_binding_signature(
            declarator.init.as_ref(),
            source_path,
            &known_signatures,
            diagnostics,
        ) else {
            continue;
        };

        if known_signatures
            .insert(declarator.id.clone(), signature.clone())
            .is_some()
        {
            diagnostics.push(invalid_export_surface(
                source_path,
                &format!("duplicate export name `{}`", declarator.id),
            ));
        }

        declared_function_signatures.insert(declarator.id.clone(), signature);
    }
}

pub(crate) fn infer_function_signature(params: &[String], body: &BlockStatement, is_async: bool) -> String {
    function_signature(params, infer_block_return_type(body), is_async)
}

#[allow(clippy::only_used_in_recursion)]
pub(crate) fn infer_function_binding_signature(
    expression: Option<&Expression>,
    source_path: &Path,
    declared_function_signatures: &BTreeMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let expression = expression?;
    match expression {
        Expression::FunctionExpression(func) => {
            if func.generator {
                diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    generator_function_unavailable_message(func.is_async, func.body.as_deref()),
                ));
                return None;
            }

            let body = func.body.as_ref()?;
            let params = func
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect::<Vec<_>>();
            Some(function_signature(
                &params,
                infer_block_return_type(body),
                func.is_async,
            ))
        }
        Expression::ArrowFunctionExpression(func) => {
            let params = func
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect::<Vec<_>>();
            Some(function_signature(
                &params,
                infer_expression_type(&func.body),
                func.is_async,
            ))
        }
        Expression::Identifier(name) => declared_function_signatures.get(name).cloned(),
        Expression::ParenthesizedExpression(parenthesized) => infer_function_binding_signature(
            Some(&parenthesized.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::TypeAssertion(type_assertion) => infer_function_binding_signature(
            Some(&type_assertion.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::SatisfiesExpression(satisfies_expression) => infer_function_binding_signature(
            Some(&satisfies_expression.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => infer_function_binding_signature(
                    Some(object),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                ),
            }
        }
        Expression::ChainExpression(chain_expression) => infer_function_binding_signature(
            Some(&chain_expression.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::AwaitExpression(await_expression) => infer_function_binding_signature(
            Some(&await_expression.argument),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::CallExpression(call) if is_object_freeze_call(call) => {
            call.args.first().and_then(|argument| {
                infer_function_binding_signature(
                    Some(argument),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                )
            })
        }
        Expression::SequenceExpression(sequence_expression) => sequence_expression
            .expressions
            .last()
            .and_then(|expression| {
                infer_function_binding_signature(
                    Some(expression),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                )
            }),
        Expression::DecoratedExpression(decorated_expression) => infer_function_binding_signature(
            Some(&decorated_expression.expression),
            source_path,
            declared_function_signatures,
            diagnostics,
        ),
        Expression::LogicalExpression(logical_expression) => {
            let left = infer_function_binding_signature(
                Some(&logical_expression.left),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            match logical_expression.operator {
                kali_ast::LogicalOperator::Coalesce => {
                    if matches!(
                        infer_expression_type(&logical_expression.left),
                        Some("null" | "undefined" | "void")
                    ) {
                        infer_function_binding_signature(
                            Some(&logical_expression.right),
                            source_path,
                            declared_function_signatures,
                            diagnostics,
                        )
                    } else {
                        left
                    }
                }
                kali_ast::LogicalOperator::And => {
                    match infer_static_truthiness(&logical_expression.left) {
                        Some(true) => infer_function_binding_signature(
                            Some(&logical_expression.right),
                            source_path,
                            declared_function_signatures,
                            diagnostics,
                        ),
                        Some(false) => None,
                        None => {
                            let right = infer_function_binding_signature(
                                Some(&logical_expression.right),
                                source_path,
                                declared_function_signatures,
                                diagnostics,
                            );
                            if left.is_some() && left == right {
                                left
                            } else {
                                None
                            }
                        }
                    }
                }
                kali_ast::LogicalOperator::Or => {
                    match infer_static_truthiness(&logical_expression.left) {
                        Some(true) => None,
                        Some(false) => infer_function_binding_signature(
                            Some(&logical_expression.right),
                            source_path,
                            declared_function_signatures,
                            diagnostics,
                        ),
                        None => {
                            let right = infer_function_binding_signature(
                                Some(&logical_expression.right),
                                source_path,
                                declared_function_signatures,
                                diagnostics,
                            );
                            if left.is_some() && left == right {
                                left
                            } else {
                                None
                            }
                        }
                    }
                }
            }
        }
        Expression::BinaryExpression(binary) if binary.operator == "??" => {
            let left = infer_function_binding_signature(
                Some(&binary.left),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            if left.is_some() {
                left
            } else if matches!(
                infer_expression_type(&binary.left),
                Some("null" | "undefined" | "void")
            ) {
                infer_function_binding_signature(
                    Some(&binary.right),
                    source_path,
                    declared_function_signatures,
                    diagnostics,
                )
            } else {
                None
            }
        }
        Expression::ConditionalExpression(conditional_expression) => {
            let consequent = infer_function_binding_signature(
                Some(conditional_expression.consequent.as_ref()),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            let alternate = infer_function_binding_signature(
                Some(conditional_expression.alternate.as_ref()),
                source_path,
                declared_function_signatures,
                diagnostics,
            );
            if consequent.is_some() && consequent == alternate {
                consequent
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_object_freeze_call(call: &kali_ast::CallExpression) -> bool {
    matches!(
        call_member_access_name(&call.callee).as_deref(),
        Some("Object.freeze") | Some("globalThis.Object.freeze")
    ) && call.args.len() == 1
}

fn call_member_access_name(expression: &Expression) -> Option<String> {
    match expression {
        Expression::MemberExpression(member) => member_access_name(member),
        Expression::Identifier(name) => Some(name.clone()),
        Expression::ParenthesizedExpression(expr) => call_member_access_name(&expr.expression),
        Expression::TypeAssertion(expr) => call_member_access_name(&expr.expression),
        Expression::SatisfiesExpression(expr) => call_member_access_name(&expr.expression),
        Expression::ChainExpression(expr) => call_member_access_name(&expr.expression),
        Expression::DecoratedExpression(expr) => call_member_access_name(&expr.expression),
        Expression::SequenceExpression(expr) => {
            expr.expressions.last().and_then(call_member_access_name)
        }
        Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => call_member_access_name(object),
        },
        _ => None,
    }
}

fn member_access_name(member: &kali_ast::MemberExpression) -> Option<String> {
    let object = call_member_access_name(&member.object)?;
    Some(format!("{object}.{}", member.property))
}

fn function_signature(params: &[String], return_type: Option<&str>, is_async: bool) -> String {
    let return_type = return_type.unwrap_or("unknown");
    let return_type = if is_async {
        format!("Promise<{return_type}>")
    } else {
        return_type.to_string()
    };
    format!("({}) => {}", params.join(", "), return_type)
}

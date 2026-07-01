//! Runtime-entrypoint validation: async/generator rejection + unique export names.

use super::helpers::{invalid_export_surface, parse_source_file};

use std::collections::BTreeSet;
use std::path::Path;

use kali_ast::{
    BlockStatement, ClassDeclaration, ExportDefaultDeclaration, Expression, OptionalChainInner,
    Statement,
};
use kali_common::generator_class_method_yield_lowering_unavailable_message_for_flavors;
use kali_error::{_error_codes::e5, Diagnostic};

fn block_contains_yield_delegation(block: &BlockStatement) -> bool {
    block.body.iter().any(statement_contains_yield_delegation)
}

fn statement_contains_yield_delegation(statement: &Statement) -> bool {
    match statement {
        Statement::ExpressionStatement(expression) => {
            expression_contains_yield_delegation(&expression.expression)
        }
        Statement::BreakStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::DebuggerStatement(_)
        | Statement::ImportDeclaration(_)
        | Statement::InterfaceDeclaration(_)
        | Statement::TypeAliasDeclaration(_)
        | Statement::ClassDeclaration(_)
        | Statement::FunctionDeclaration(_) => false,
        Statement::WithStatement(with_stmt) => {
            expression_contains_yield_delegation(&with_stmt.object)
                || statement_contains_yield_delegation(&with_stmt.body)
        }
        Statement::ReturnStatement(return_stmt) => return_stmt
            .argument
            .as_ref()
            .is_some_and(expression_contains_yield_delegation),
        Statement::LabeledStatement(labeled_stmt) => {
            statement_contains_yield_delegation(&labeled_stmt.body)
        }
        Statement::IfStatement(if_stmt) => {
            expression_contains_yield_delegation(&if_stmt.test)
                || block_contains_yield_delegation(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_deref()
                    .is_some_and(block_contains_yield_delegation)
        }
        Statement::SwitchStatement(switch_stmt) => {
            expression_contains_yield_delegation(&switch_stmt.discriminant)
                || switch_stmt.cases.iter().any(|case| {
                    case.test
                        .as_ref()
                        .is_some_and(expression_contains_yield_delegation)
                        || case
                            .consequent
                            .iter()
                            .any(statement_contains_yield_delegation)
                })
        }
        Statement::ThrowStatement(throw_stmt) => {
            expression_contains_yield_delegation(&throw_stmt.argument)
        }
        Statement::TryStatement(try_stmt) => {
            block_contains_yield_delegation(&try_stmt.block)
                || try_stmt
                    .handler
                    .as_ref()
                    .is_some_and(|handler| block_contains_yield_delegation(&handler.body))
                || try_stmt
                    .finalizer
                    .as_ref()
                    .is_some_and(block_contains_yield_delegation)
        }
        Statement::BlockStatement(block) => block_contains_yield_delegation(block),
        Statement::ForStatement(for_stmt) => {
            for_stmt.init.as_ref().is_some_and(|init| match init {
                kali_ast::ForInit::VariableDeclaration(declaration) => {
                    declaration.declarations.iter().any(|declarator| {
                        declarator
                            .init
                            .as_ref()
                            .is_some_and(expression_contains_yield_delegation)
                    })
                }
                kali_ast::ForInit::Expression(expression) => {
                    expression_contains_yield_delegation(expression)
                }
            }) || for_stmt
                .test
                .as_ref()
                .is_some_and(expression_contains_yield_delegation)
                || for_stmt
                    .update
                    .as_ref()
                    .is_some_and(expression_contains_yield_delegation)
                || block_contains_yield_delegation(&for_stmt.body)
        }
        Statement::ForInStatement(for_in_stmt) => {
            let left_has_yield_delegation = match &for_in_stmt.left {
                kali_ast::ForInLefthand::VariableDeclaration(declaration) => {
                    declaration.declarations.iter().any(|declarator| {
                        declarator
                            .init
                            .as_ref()
                            .is_some_and(expression_contains_yield_delegation)
                    })
                }
                kali_ast::ForInLefthand::Expression(expression) => {
                    expression_contains_yield_delegation(expression)
                }
            };
            left_has_yield_delegation
                || expression_contains_yield_delegation(&for_in_stmt.right)
                || statement_contains_yield_delegation(&for_in_stmt.body)
        }
        Statement::ForOfStatement(for_of_stmt) => {
            let left_has_yield_delegation = match &for_of_stmt.left {
                kali_ast::ForOfLefthand::VariableDeclaration(declaration) => {
                    declaration.declarations.iter().any(|declarator| {
                        declarator
                            .init
                            .as_ref()
                            .is_some_and(expression_contains_yield_delegation)
                    })
                }
                kali_ast::ForOfLefthand::Expression(expression) => {
                    expression_contains_yield_delegation(expression)
                }
            };
            left_has_yield_delegation
                || expression_contains_yield_delegation(&for_of_stmt.right)
                || statement_contains_yield_delegation(&for_of_stmt.body)
        }
        Statement::WhileStatement(while_stmt) => {
            expression_contains_yield_delegation(&while_stmt.test)
                || block_contains_yield_delegation(&while_stmt.body)
        }
        Statement::DoWhileStatement(do_while_stmt) => {
            block_contains_yield_delegation(&do_while_stmt.body)
                || expression_contains_yield_delegation(&do_while_stmt.test)
        }
        Statement::VariableDeclaration(declaration) => {
            declaration.declarations.iter().any(|declarator| {
                declarator
                    .init
                    .as_ref()
                    .is_some_and(expression_contains_yield_delegation)
            })
        }
        Statement::ExportAll(_) | Statement::ExportNamed(_) => false,
        Statement::ExportDefault(default_decl) => match default_decl {
            ExportDefaultDeclaration::Expression(expression) => {
                expression_contains_yield_delegation(expression)
            }
            ExportDefaultDeclaration::FunctionDeclaration(_)
            | ExportDefaultDeclaration::ClassDeclaration(_) => false,
        },
        Statement::EnumDeclaration(enum_declaration) => {
            enum_declaration.members.iter().any(|member| {
                member
                    .value
                    .as_ref()
                    .is_some_and(expression_contains_yield_delegation)
            })
        }
    }
}

fn expression_contains_yield_delegation(expression: &Expression) -> bool {
    match expression {
        Expression::YieldExpression(yield_expression) => {
            yield_expression.delegate
                || yield_expression
                    .argument
                    .as_ref()
                    .is_some_and(expression_contains_yield_delegation)
        }
        Expression::BinaryExpression(binary) => {
            expression_contains_yield_delegation(&binary.left)
                || expression_contains_yield_delegation(&binary.right)
        }
        Expression::UnaryExpression(unary) => expression_contains_yield_delegation(&unary.argument),
        Expression::CallExpression(call) => {
            expression_contains_yield_delegation(&call.callee)
                || call.args.iter().any(expression_contains_yield_delegation)
        }
        Expression::MemberExpression(member) => {
            expression_contains_yield_delegation(&member.object)
        }
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            Some(kali_ast::ExpressionOrSpread::Expression(expression)) => {
                expression_contains_yield_delegation(expression)
            }
            Some(kali_ast::ExpressionOrSpread::Spread(spread)) => {
                expression_contains_yield_delegation(&spread.argument)
            }
            Some(kali_ast::ExpressionOrSpread::Empty) | None => false,
        }),
        Expression::ObjectExpression(object) => object
            .properties
            .iter()
            .any(|property| expression_contains_yield_delegation(&property.value)),
        Expression::FunctionExpression(_) | Expression::ClassExpression(_) => false,
        Expression::ArrowFunctionExpression(arrow) => {
            expression_contains_yield_delegation(&arrow.body)
        }
        Expression::NewExpression(new_expression) => {
            expression_contains_yield_delegation(&new_expression.callee)
                || new_expression
                    .args
                    .iter()
                    .any(expression_contains_yield_delegation)
        }
        Expression::MetaProperty(_)
        | Expression::Identifier(_)
        | Expression::Literal(_)
        | Expression::ThisExpression
        | Expression::SuperExpression
        | Expression::PrivateIdentifier(_)
        | Expression::BigIntLiteral(_) => false,
        Expression::TemplateLiteral(template) => template
            .expressions
            .iter()
            .any(expression_contains_yield_delegation),
        Expression::TaggedTemplateExpression(tagged) => {
            expression_contains_yield_delegation(&tagged.tag)
                || tagged
                    .template
                    .expressions
                    .iter()
                    .any(expression_contains_yield_delegation)
        }
        Expression::UpdateExpression(update) => {
            expression_contains_yield_delegation(&update.argument)
        }
        Expression::AssignmentExpression(assignment) => {
            expression_contains_yield_delegation(&assignment.left)
                || expression_contains_yield_delegation(&assignment.right)
        }
        Expression::LogicalExpression(logical) => {
            expression_contains_yield_delegation(&logical.left)
                || expression_contains_yield_delegation(&logical.right)
        }
        Expression::ConditionalExpression(conditional) => {
            expression_contains_yield_delegation(&conditional.test)
                || expression_contains_yield_delegation(&conditional.consequent)
                || expression_contains_yield_delegation(&conditional.alternate)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .iter()
            .any(expression_contains_yield_delegation),
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_contains_yield_delegation(&parenthesized.expression)
        }
        Expression::AwaitExpression(await_expression) => {
            expression_contains_yield_delegation(&await_expression.argument)
        }
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    expression_contains_yield_delegation(object)
                }
            }
        }
        Expression::ChainExpression(chain) => {
            expression_contains_yield_delegation(&chain.expression)
        }
        Expression::SpreadElement(spread) => expression_contains_yield_delegation(&spread.argument),
        Expression::RestElement(rest) => expression_contains_yield_delegation(&rest.argument),
        Expression::ImportExpression(import_expression) => {
            expression_contains_yield_delegation(&import_expression.source)
        }
        Expression::DecoratedExpression(decorated) => {
            expression_contains_yield_delegation(&decorated.expression)
        }
        Expression::JsxElement(element) => {
            element
                .opening_element
                .attributes
                .iter()
                .any(|attribute| match attribute {
                    kali_ast::JsxAttributeItem::JsxAttribute(attribute) => match &attribute.value {
                        kali_ast::JsxAttributeValue::String(_) => false,
                        kali_ast::JsxAttributeValue::JsxElement(child) => {
                            expression_contains_yield_delegation(&Expression::JsxElement(
                                (**child).clone(),
                            ))
                        }
                        kali_ast::JsxAttributeValue::JsxExpression(container) => container
                            .expression
                            .as_ref()
                            .is_some_and(expression_contains_yield_delegation),
                    },
                    kali_ast::JsxAttributeItem::JsxSpreadAttribute(spread) => {
                        expression_contains_yield_delegation(&spread.argument)
                    }
                })
                || element.children.iter().any(|child| match child {
                    kali_ast::JsxChild::JsxText(_) => false,
                    kali_ast::JsxChild::JsxExpression(container) => container
                        .expression
                        .as_ref()
                        .is_some_and(expression_contains_yield_delegation),
                    kali_ast::JsxChild::JsxElement(child) => expression_contains_yield_delegation(
                        &Expression::JsxElement((**child).clone()),
                    ),
                    kali_ast::JsxChild::JsxFragment(fragment) => {
                        expression_contains_yield_delegation(&Expression::JsxFragment(
                            (**fragment).clone(),
                        ))
                    }
                })
        }
        Expression::JsxFragment(fragment) => fragment.children.iter().any(|child| match child {
            kali_ast::JsxChild::JsxText(_) => false,
            kali_ast::JsxChild::JsxExpression(container) => container
                .expression
                .as_ref()
                .is_some_and(expression_contains_yield_delegation),
            kali_ast::JsxChild::JsxElement(child) => {
                expression_contains_yield_delegation(&Expression::JsxElement((**child).clone()))
            }
            kali_ast::JsxChild::JsxFragment(fragment) => {
                expression_contains_yield_delegation(&Expression::JsxFragment((**fragment).clone()))
            }
        }),
        Expression::JsxEmptyExpression => false,
        Expression::TypeAssertion(assertion) => {
            expression_contains_yield_delegation(&assertion.expression)
        }
        Expression::SatisfiesExpression(satisfies) => {
            expression_contains_yield_delegation(&satisfies.expression)
        }
    }
}

pub(crate) fn generator_function_unavailable_message(
    is_async: bool,
    body: Option<&BlockStatement>,
) -> &'static str {
    if body.is_some_and(block_contains_yield_delegation) {
        kali_common::generator_function_yield_lowering_unavailable_message(is_async, true)
    } else {
        kali_common::generator_function_lowering_unavailable_message(is_async)
    }
}

pub fn reject_async_and_generator_class_methods_in_runtime_entrypoint(
    source_path: &Path,
) -> Result<(), Vec<Diagnostic>> {
    fn push_async_class_method_diagnostic(diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::async_class_method_lowering_unavailable_message(),
        ));
    }

    fn push_generator_class_method_diagnostic(
        diagnostics: &mut Vec<Diagnostic>,
        has_generator: bool,
        has_async_generator: bool,
        is_delegate: bool,
    ) {
        diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            generator_class_method_yield_lowering_unavailable_message_for_flavors(
                has_generator,
                has_async_generator,
                is_delegate,
            ),
        ));
    }

    fn push_generator_function_diagnostic(
        diagnostics: &mut Vec<Diagnostic>,
        is_async: bool,
        body: Option<&BlockStatement>,
    ) {
        diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            generator_function_unavailable_message(is_async, body),
        ));
    }

    fn class_body_has_async_method(body: &kali_ast::ClassBody) -> bool {
        body.methods
            .iter()
            .any(|method| method.is_async && !method.generator)
    }

    fn class_body_has_generator_flavors(body: &kali_ast::ClassBody) -> (bool, bool) {
        let has_generator = body.methods.iter().any(|method| method.generator);
        let has_async_generator = body
            .methods
            .iter()
            .any(|method| method.is_async && method.generator);
        (has_generator, has_async_generator)
    }

    fn class_body_has_generator_yield_delegation(body: &kali_ast::ClassBody) -> bool {
        body.methods.iter().any(|method| {
            method.generator
                && method
                    .body
                    .as_deref()
                    .is_some_and(block_contains_yield_delegation)
        })
    }

    fn collect_expression(expression: &Expression, diagnostics: &mut Vec<Diagnostic>) {
        match expression {
            Expression::ClassExpression(class) => {
                let (has_generator, has_async_generator) =
                    class_body_has_generator_flavors(&class.body);
                let is_delegate = class_body_has_generator_yield_delegation(&class.body);
                if has_generator || has_async_generator {
                    push_generator_class_method_diagnostic(
                        diagnostics,
                        has_generator,
                        has_async_generator,
                        is_delegate,
                    );
                } else if class_body_has_async_method(&class.body) {
                    push_async_class_method_diagnostic(diagnostics);
                }
            }
            Expression::ParenthesizedExpression(parenthesized) => {
                collect_expression(&parenthesized.expression, diagnostics);
            }
            Expression::SequenceExpression(sequence) => {
                for nested in &sequence.expressions {
                    collect_expression(nested, diagnostics);
                }
            }
            Expression::ConditionalExpression(conditional) => {
                collect_expression(&conditional.test, diagnostics);
                collect_expression(&conditional.consequent, diagnostics);
                collect_expression(&conditional.alternate, diagnostics);
            }
            Expression::LogicalExpression(logical) => {
                collect_expression(&logical.left, diagnostics);
                collect_expression(&logical.right, diagnostics);
            }
            Expression::UnaryExpression(unary) => {
                collect_expression(&unary.argument, diagnostics);
            }
            Expression::BinaryExpression(binary) => {
                collect_expression(&binary.left, diagnostics);
                collect_expression(&binary.right, diagnostics);
            }
            Expression::CallExpression(call) => {
                collect_expression(&call.callee, diagnostics);
                for arg in &call.args {
                    collect_expression(arg, diagnostics);
                }
            }
            Expression::MemberExpression(member) => {
                collect_expression(&member.object, diagnostics);
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    match element {
                        Some(kali_ast::ExpressionOrSpread::Expression(expr)) => {
                            collect_expression(expr, diagnostics);
                        }
                        Some(kali_ast::ExpressionOrSpread::Spread(spread)) => {
                            collect_expression(&spread.argument, diagnostics);
                        }
                        Some(kali_ast::ExpressionOrSpread::Empty) | None => {}
                    }
                }
            }
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    collect_expression(&property.value, diagnostics);
                }
            }
            Expression::FunctionExpression(function) => {
                if function.generator {
                    diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        generator_function_unavailable_message(
                            function.is_async,
                            function.body.as_deref(),
                        ),
                    ));
                } else if let Some(body) = &function.body {
                    for nested in &body.body {
                        collect_statement(nested, diagnostics);
                    }
                }
            }
            Expression::ArrowFunctionExpression(arrow) => {
                collect_expression(&arrow.body, diagnostics);
            }
            Expression::NewExpression(new_expression) => {
                collect_expression(&new_expression.callee, diagnostics);
                for arg in &new_expression.args {
                    collect_expression(arg, diagnostics);
                }
            }
            Expression::TemplateLiteral(template) => {
                for expr in &template.expressions {
                    collect_expression(expr, diagnostics);
                }
            }
            Expression::TaggedTemplateExpression(tagged) => {
                collect_expression(&tagged.tag, diagnostics);
                for expr in &tagged.template.expressions {
                    collect_expression(expr, diagnostics);
                }
            }
            Expression::UpdateExpression(update) => {
                collect_expression(&update.argument, diagnostics);
            }
            Expression::AssignmentExpression(assignment) => {
                collect_expression(&assignment.left, diagnostics);
                collect_expression(&assignment.right, diagnostics);
            }
            Expression::YieldExpression(yield_expression) => {
                if let Some(argument) = &yield_expression.argument {
                    collect_expression(argument, diagnostics);
                }
            }
            Expression::AwaitExpression(await_expression) => {
                collect_expression(&await_expression.argument, diagnostics);
            }
            Expression::OptionalChainExpression(optional_chain) => {
                match optional_chain.inner.as_ref() {
                    OptionalChainInner::NonNull { object, .. } => {
                        collect_expression(object, diagnostics);
                    }
                }
            }
            Expression::ChainExpression(chain) => {
                collect_expression(&chain.expression, diagnostics);
            }
            Expression::SpreadElement(spread) => {
                collect_expression(&spread.argument, diagnostics);
            }
            Expression::RestElement(rest) => {
                collect_expression(&rest.argument, diagnostics);
            }
            Expression::ImportExpression(import_expression) => {
                collect_expression(&import_expression.source, diagnostics);
            }
            Expression::DecoratedExpression(decorated) => {
                collect_expression(&decorated.expression, diagnostics);
            }
            Expression::JsxElement(element) => {
                for attribute in &element.opening_element.attributes {
                    match attribute {
                        kali_ast::JsxAttributeItem::JsxAttribute(attribute) => {
                            match &attribute.value {
                                kali_ast::JsxAttributeValue::String(_) => {}
                                kali_ast::JsxAttributeValue::JsxElement(child) => {
                                    collect_expression(
                                        &Expression::JsxElement((**child).clone()),
                                        diagnostics,
                                    );
                                }
                                kali_ast::JsxAttributeValue::JsxExpression(container) => {
                                    if let Some(expression) = &container.expression {
                                        collect_expression(expression, diagnostics);
                                    }
                                }
                            }
                        }
                        kali_ast::JsxAttributeItem::JsxSpreadAttribute(spread) => {
                            collect_expression(&spread.argument, diagnostics);
                        }
                    }
                }

                for child in &element.children {
                    match child {
                        kali_ast::JsxChild::JsxText(_) => {}
                        kali_ast::JsxChild::JsxExpression(container) => {
                            if let Some(expression) = &container.expression {
                                collect_expression(expression, diagnostics);
                            }
                        }
                        kali_ast::JsxChild::JsxElement(child_element) => {
                            collect_expression(
                                &Expression::JsxElement((**child_element).clone()),
                                diagnostics,
                            );
                        }
                        kali_ast::JsxChild::JsxFragment(fragment) => {
                            collect_expression(
                                &Expression::JsxFragment((**fragment).clone()),
                                diagnostics,
                            );
                        }
                    }
                }
            }
            Expression::JsxFragment(fragment) => {
                for child in &fragment.children {
                    match child {
                        kali_ast::JsxChild::JsxText(_) => {}
                        kali_ast::JsxChild::JsxExpression(container) => {
                            if let Some(expression) = &container.expression {
                                collect_expression(expression, diagnostics);
                            }
                        }
                        kali_ast::JsxChild::JsxElement(child_element) => {
                            collect_expression(
                                &Expression::JsxElement((**child_element).clone()),
                                diagnostics,
                            );
                        }
                        kali_ast::JsxChild::JsxFragment(child_fragment) => {
                            collect_expression(
                                &Expression::JsxFragment((**child_fragment).clone()),
                                diagnostics,
                            );
                        }
                    }
                }
            }
            Expression::TypeAssertion(assertion) => {
                collect_expression(&assertion.expression, diagnostics);
            }
            Expression::SatisfiesExpression(satisfies) => {
                collect_expression(&satisfies.expression, diagnostics);
            }
            Expression::Literal(_)
            | Expression::Identifier(_)
            | Expression::MetaProperty(_)
            | Expression::ThisExpression
            | Expression::SuperExpression
            | Expression::PrivateIdentifier(_)
            | Expression::BigIntLiteral(_)
            | Expression::JsxEmptyExpression => {}
        }
    }

    fn collect_statement(statement: &Statement, diagnostics: &mut Vec<Diagnostic>) {
        match statement {
            Statement::ClassDeclaration(class) => {
                let (has_generator, has_async_generator) =
                    class_body_has_generator_flavors(&class.body);
                let is_delegate = class_body_has_generator_yield_delegation(&class.body);
                if has_generator || has_async_generator {
                    push_generator_class_method_diagnostic(
                        diagnostics,
                        has_generator,
                        has_async_generator,
                        is_delegate,
                    );
                } else if class_body_has_async_method(&class.body) {
                    push_async_class_method_diagnostic(diagnostics);
                }
            }
            Statement::ExportDefault(ExportDefaultDeclaration::ClassDeclaration(
                ClassDeclaration { body, .. },
            )) => {
                let (has_generator, has_async_generator) = class_body_has_generator_flavors(body);
                let is_delegate = class_body_has_generator_yield_delegation(body);
                if has_generator || has_async_generator {
                    push_generator_class_method_diagnostic(
                        diagnostics,
                        has_generator,
                        has_async_generator,
                        is_delegate,
                    );
                } else if class_body_has_async_method(body) {
                    push_async_class_method_diagnostic(diagnostics);
                }
            }
            Statement::ExpressionStatement(expression) => {
                collect_expression(&expression.expression, diagnostics);
            }
            Statement::ReturnStatement(return_statement) => {
                if let Some(argument) = &return_statement.argument {
                    collect_expression(argument, diagnostics);
                }
            }
            Statement::WithStatement(with_statement) => {
                collect_expression(&with_statement.object, diagnostics);
                collect_statement(&with_statement.body, diagnostics);
            }
            Statement::LabeledStatement(label) => collect_statement(&label.body, diagnostics),
            Statement::IfStatement(if_stmt) => {
                collect_expression(&if_stmt.test, diagnostics);
                for nested in &if_stmt.consequent.body {
                    collect_statement(nested, diagnostics);
                }
                if let Some(alternate) = &if_stmt.alternate {
                    for nested in &alternate.body {
                        collect_statement(nested, diagnostics);
                    }
                }
            }
            Statement::SwitchStatement(switch_stmt) => {
                collect_expression(&switch_stmt.discriminant, diagnostics);
                for case in &switch_stmt.cases {
                    if let Some(test) = &case.test {
                        collect_expression(test, diagnostics);
                    }
                    for nested in &case.consequent {
                        collect_statement(nested, diagnostics);
                    }
                }
            }
            Statement::ThrowStatement(throw_statement) => {
                collect_expression(&throw_statement.argument, diagnostics);
            }
            Statement::TryStatement(try_stmt) => {
                for nested in &try_stmt.block.body {
                    collect_statement(nested, diagnostics);
                }
                if let Some(handler) = &try_stmt.handler {
                    for nested in &handler.body.body {
                        collect_statement(nested, diagnostics);
                    }
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    for nested in &finalizer.body {
                        collect_statement(nested, diagnostics);
                    }
                }
            }
            Statement::ForStatement(for_stmt) => {
                if let Some(init) = &for_stmt.init {
                    match init {
                        kali_ast::ForInit::VariableDeclaration(declaration) => {
                            for declarator in &declaration.declarations {
                                if let Some(expression) = &declarator.init {
                                    collect_expression(expression, diagnostics);
                                }
                            }
                        }
                        kali_ast::ForInit::Expression(expression) => {
                            collect_expression(expression, diagnostics);
                        }
                    }
                }
                if let Some(test) = &for_stmt.test {
                    collect_expression(test, diagnostics);
                }
                if let Some(update) = &for_stmt.update {
                    collect_expression(update, diagnostics);
                }
                for nested in &for_stmt.body.body {
                    collect_statement(nested, diagnostics);
                }
            }
            Statement::ForInStatement(for_in_stmt) => {
                match &for_in_stmt.left {
                    kali_ast::ForInLefthand::VariableDeclaration(declaration) => {
                        for declarator in &declaration.declarations {
                            if let Some(expression) = &declarator.init {
                                collect_expression(expression, diagnostics);
                            }
                        }
                    }
                    kali_ast::ForInLefthand::Expression(expression) => {
                        collect_expression(expression, diagnostics);
                    }
                }
                collect_expression(&for_in_stmt.right, diagnostics);
                collect_statement(&for_in_stmt.body, diagnostics);
            }
            Statement::ForOfStatement(for_of_stmt) => {
                match &for_of_stmt.left {
                    kali_ast::ForOfLefthand::VariableDeclaration(declaration) => {
                        for declarator in &declaration.declarations {
                            if let Some(expression) = &declarator.init {
                                collect_expression(expression, diagnostics);
                            }
                        }
                    }
                    kali_ast::ForOfLefthand::Expression(expression) => {
                        collect_expression(expression, diagnostics);
                    }
                }
                collect_expression(&for_of_stmt.right, diagnostics);
                collect_statement(&for_of_stmt.body, diagnostics);
            }
            Statement::WhileStatement(while_stmt) => {
                collect_expression(&while_stmt.test, diagnostics);
                for nested in &while_stmt.body.body {
                    collect_statement(nested, diagnostics);
                }
            }
            Statement::DoWhileStatement(do_while_stmt) => {
                for nested in &do_while_stmt.body.body {
                    collect_statement(nested, diagnostics);
                }
                collect_expression(&do_while_stmt.test, diagnostics);
            }
            Statement::FunctionDeclaration(function) => {
                if function.generator {
                    push_generator_function_diagnostic(
                        diagnostics,
                        function.is_async,
                        Some(function.body.as_ref()),
                    );
                } else {
                    for nested in &function.body.body {
                        collect_statement(nested, diagnostics);
                    }
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if let Some(expression) = &declarator.init {
                        collect_expression(expression, diagnostics);
                    }
                }
            }
            Statement::ExportDefault(ExportDefaultDeclaration::Expression(expression)) => {
                collect_expression(expression, diagnostics);
            }
            Statement::ExportDefault(ExportDefaultDeclaration::FunctionDeclaration(function)) => {
                if function.generator {
                    push_generator_function_diagnostic(
                        diagnostics,
                        function.is_async,
                        Some(function.body.as_ref()),
                    );
                } else {
                    for nested in &function.body.body {
                        collect_statement(nested, diagnostics);
                    }
                }
            }
            Statement::EnumDeclaration(enum_declaration) => {
                for member in &enum_declaration.members {
                    if let Some(expression) = &member.value {
                        collect_expression(expression, diagnostics);
                    }
                }
            }
            Statement::BlockStatement(block) => {
                for nested in &block.body {
                    collect_statement(nested, diagnostics);
                }
            }
            _ => {}
        }
    }

    let statements = parse_source_file(source_path)?;
    let mut diagnostics = Vec::new();

    for statement in &statements {
        collect_statement(statement, &mut diagnostics);
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn validate_unique_export_names_from_statements(
    statements: &[Statement],
    source_path: &Path,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen_names = BTreeSet::<String>::new();

    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                if !seen_names.insert(func.name.clone()) {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!("duplicate export name `{}`", func.name),
                    ));
                }
            }
            Statement::ExportAll(_) => {}
            Statement::ExportNamed(declaration) => {
                for specifier in &declaration.specifiers {
                    if !seen_names.insert(specifier.exported.clone()) {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{}`", specifier.exported),
                        ));
                    }
                }
            }
            Statement::ExportDefault(default_decl) => match default_decl {
                ExportDefaultDeclaration::FunctionDeclaration(func) => {
                    let export_name = if func.name.is_empty() {
                        "default".to_string()
                    } else {
                        func.name.clone()
                    };
                    if !seen_names.insert(export_name.clone()) {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{export_name}`"),
                        ));
                    }
                }
                ExportDefaultDeclaration::Expression(_)
                | ExportDefaultDeclaration::ClassDeclaration(_) => {
                    if !seen_names.insert("default".to_string()) {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            "duplicate export name `default`",
                        ));
                    }
                }
            },
            Statement::ImportDeclaration(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::WithStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::LabeledStatement(_)
            | Statement::IfStatement(_)
            | Statement::SwitchStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::TryStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::BlockStatement(_)
            | Statement::ForStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::WhileStatement(_)
            | Statement::DoWhileStatement(_)
            | Statement::ClassDeclaration(_)
            | Statement::VariableDeclaration(_)
            | Statement::EnumDeclaration(_)
            | Statement::TypeAliasDeclaration(_)
            | Statement::InterfaceDeclaration(_)
            | Statement::ExpressionStatement(_) => {}
        }
    }

    diagnostics
}

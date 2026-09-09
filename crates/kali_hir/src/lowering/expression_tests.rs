use crate::*;
use kali_ast::{Expression, Statement, UpdateExpression, UpdateOperator};

#[test]
fn test_update_expression_lowers_prefix_and_postfix_forms() {
    let mut lowerer = HirLowerer::new();

    let prefix =
        lowerer.lower_expression(&Expression::UpdateExpression(Box::new(UpdateExpression {
            operator: UpdateOperator::Increment,
            argument: Expression::Identifier("value".to_string()),
            prefix: true,
        })));
    let prefix_node = &lowerer.builder.nodes[prefix.0 as usize];
    assert_eq!(prefix_node.kind, HirNodeKind::UpdateExpr);
    assert_eq!(prefix_node.text.as_deref(), Some("prefix++"));
    assert_eq!(prefix_node.children.len(), 1);
    let prefix_arg = &lowerer.builder.nodes[prefix_node.children[0].0 as usize];
    assert_eq!(prefix_arg.kind, HirNodeKind::Ident);
    assert_eq!(prefix_arg.text.as_deref(), Some("value"));

    let postfix =
        lowerer.lower_expression(&Expression::UpdateExpression(Box::new(UpdateExpression {
            operator: UpdateOperator::Decrement,
            argument: Expression::Identifier("value".to_string()),
            prefix: false,
        })));
    let postfix_node = &lowerer.builder.nodes[postfix.0 as usize];
    assert_eq!(postfix_node.kind, HirNodeKind::UpdateExpr);
    assert_eq!(postfix_node.text.as_deref(), Some("postfix--"));
    assert_eq!(postfix_node.children.len(), 1);
    let postfix_arg = &lowerer.builder.nodes[postfix_node.children[0].0 as usize];
    assert_eq!(postfix_arg.kind, HirNodeKind::Ident);
    assert_eq!(postfix_arg.text.as_deref(), Some("value"));
}

#[test]
fn a_computed_member_without_a_static_name_lowers_without_text() {
    let statements = crate::test_support::parse("o[i]; o[\"b\"]; o.c;");
    let expressions: Vec<&Expression> = statements
        .iter()
        .map(|statement| match statement {
            Statement::ExpressionStatement(stmt) => stmt.expression.as_ref(),
            other => panic!("expected an expression statement, got {other:?}"),
        })
        .collect();
    let mut lowerer = HirLowerer::new();

    let nameless = lowerer.lower_expression(expressions[0]);
    let node = &lowerer.builder.nodes[nameless.0 as usize];
    assert_eq!(node.kind, HirNodeKind::MemberExpr);
    assert_eq!(
        node.text, None,
        "o[i] has no static name and must carry no text"
    );
    assert_eq!(node.children.len(), 2, "the index is the second child");

    let named = lowerer.lower_expression(expressions[1]);
    let node = &lowerer.builder.nodes[named.0 as usize];
    assert_eq!(node.text.as_deref(), Some("b"));
    assert_eq!(node.children.len(), 2);

    let dot = lowerer.lower_expression(expressions[2]);
    let node = &lowerer.builder.nodes[dot.0 as usize];
    assert_eq!(node.text.as_deref(), Some("c"));
    assert_eq!(node.children.len(), 1);
}

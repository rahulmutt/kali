use crate::*;
use kali_ast::{Expression, UpdateExpression, UpdateOperator};

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

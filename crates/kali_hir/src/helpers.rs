//! Pure text/value formatting helpers shared by the lowering passes.

use kali_ast::{AssignmentOperator, LiteralValue, ObjectPropertyKind};

pub(crate) fn lower_literal_value(value: &LiteralValue) -> String {
    match value {
        LiteralValue::Boolean(v) => v.to_string(),
        LiteralValue::Number(v) => v.to_string(),
        LiteralValue::String(v) => v.clone(),
        LiteralValue::Regex { pattern, flags } => format!("/{}/{}", pattern, flags),
        LiteralValue::Null => "null".to_string(),
    }
}

pub(crate) fn logical_op_text(op: &kali_ast::LogicalOperator) -> &'static str {
    match op {
        kali_ast::LogicalOperator::And => "&&",
        kali_ast::LogicalOperator::Or => "||",
        kali_ast::LogicalOperator::Coalesce => "??",
    }
}

pub(crate) fn update_op_text(op: &kali_ast::UpdateOperator, prefix: bool) -> &'static str {
    match (op, prefix) {
        (kali_ast::UpdateOperator::Increment, true) => "prefix++",
        (kali_ast::UpdateOperator::Increment, false) => "postfix++",
        (kali_ast::UpdateOperator::Decrement, true) => "prefix--",
        (kali_ast::UpdateOperator::Decrement, false) => "postfix--",
    }
}

pub(crate) fn assignment_op_text(op: &AssignmentOperator) -> &'static str {
    match op {
        AssignmentOperator::Assign => "=",
        AssignmentOperator::AddAssign => "+=",
        AssignmentOperator::SubtractAssign => "-=",
        AssignmentOperator::MultiplyAssign => "*=",
        AssignmentOperator::DivideAssign => "/=",
        AssignmentOperator::ModuloAssign => "%=",
        AssignmentOperator::ExponentAssign => "**=",
        AssignmentOperator::NullishAssign => "??=",
        AssignmentOperator::AndAssign => "&&=",
        AssignmentOperator::OrAssign => "||=",
        AssignmentOperator::BitAndAssign => "&=",
        AssignmentOperator::BitOrAssign => "|=",
        AssignmentOperator::BitXorAssign => "^=",
        AssignmentOperator::LeftShiftAssign => "<<=",
        AssignmentOperator::RightShiftAssign => ">>=",
        AssignmentOperator::UnsignedRightShiftAssign => ">>>=",
    }
}

pub(crate) fn object_property_kind_text(kind: &ObjectPropertyKind) -> &'static str {
    match kind {
        ObjectPropertyKind::Init => "init",
        ObjectPropertyKind::Get => "get",
        ObjectPropertyKind::Set => "set",
    }
}

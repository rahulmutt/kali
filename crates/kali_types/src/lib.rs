//! Type system and name-resolution infrastructure for TypeScript/JavaScript.
//!
//! Stage 1.4 focuses on the deterministic scope model and name resolver that
//! downstream compiler stages use to catch unresolved names and duplicate
//! bindings before lowering.

mod builtins;
mod context;
mod package;
mod resolve;
mod scope;
mod typecheck;
mod static_analysis;

use builtins::*;
use package::*;
pub use context::{ResolutionResult, TypeContext};
pub use scope::{Scope, ScopeRef, ScopeType};
pub use typecheck::TypeChecker;
use typecheck::*;

use indexmap::IndexMap;
use kali_ast::{
    ArrayExpression, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    BlockStatement, BreakStatement, CallExpression, CatchClause, ClassBody, ClassDeclaration,
    ClassExpression, ContinueStatement, DecoratedExpression, DoWhileStatement, EnumDeclaration,
    EnumMember, ExportAllDeclaration, Expression, ExpressionOrSpread, ExpressionStatement,
    ForInLefthand, ForInStatement, ForInit, ForOfLefthand, ForOfStatement, ForStatement,
    FunctionDeclaration, FunctionExpression, FunctionParam, IfStatement, ImportDeclaration,
    ImportExpression, ImportSpecifier, InterfaceDeclaration, JsxChild, JsxElement, JsxFragment,
    LabeledStatement, LiteralValue, LogicalOperator, MemberExpression, NewExpression, NodeId,
    ObjectExpression, ObjectProperty, ObjectPropertyKind, OptionalChainExpression,
    OptionalChainInner, PropertyName, ReturnStatement, Statement, SwitchCase, SwitchStatement,
    TemplateLiteral, ThrowStatement, TryStatement, TypeAliasDeclaration, TypeAssertion,
    UpdateExpression, VariableDeclaration, WhileStatement, WithStatement,
};
use kali_common::{
    generator_class_method_yield_lowering_unavailable_message_for_flavors,
    generator_function_lowering_unavailable_message_for_flavors,
    generator_function_yield_lowering_unavailable_message,
    late_process_control_single_quoted_exit_aliases,
    late_process_control_single_quoted_kill_aliases, process_kill_zero_probe_wrapped_zero_aliases,
    template::resolve_interpolated_template_literal,
};
use kali_error::{
    _error_codes::e3, _error_codes::e4, _error_codes::e5, _error_codes::e6, diagnostic::Diagnostic,
};
use kali_lexer::Lexer;
use kali_parser::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};

impl TypeContext {




















    pub(crate) fn resolve_static_object_identity_binding(
        &self,
        name: &str,
    ) -> Option<StaticObjectIdentityValue> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.static_identity_values.get(name) {
                return Some(value.clone());
            }
            current = scope.parent;
        }

        self.global_scope.static_identity_values.get(name).cloned()
    }

    pub(crate) fn resolve_static_object_identity_reference_name(
        &self,
        expression: &Expression,
    ) -> Option<String> {
        self.resolve_static_reference_root(expression)
    }

    pub(crate) fn resolve_static_object_identity_literal_value(
        &self,
        expression: &Expression,
    ) -> Option<StaticObjectIdentityValue> {
        match expression {
            Expression::Literal(LiteralValue::Boolean(value)) => {
                Some(StaticObjectIdentityValue::Boolean(*value))
            }
            Expression::Literal(LiteralValue::Number(value)) => {
                Some(StaticObjectIdentityValue::Number(*value))
            }
            Expression::Literal(LiteralValue::String(value)) => {
                Some(StaticObjectIdentityValue::String(value.clone()))
            }
            Expression::TemplateLiteral(template) => self
                .resolve_static_string_expression(&Expression::TemplateLiteral(template.clone()))
                .map(StaticObjectIdentityValue::String),
            Expression::BigIntLiteral(value) => value
                .strip_suffix('n')
                .and_then(|value| value.parse::<i64>().ok())
                .map(StaticObjectIdentityValue::BigInt),
            Expression::Literal(LiteralValue::Null) => Some(StaticObjectIdentityValue::Null),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.argument)
            }
            Expression::UnaryExpression(expr) if expr.operator == "+" => {
                match self.resolve_static_object_identity_literal_value(&expr.argument) {
                    Some(StaticObjectIdentityValue::BigInt(_)) => None,
                    other => other,
                }
            }
            Expression::UnaryExpression(expr) if expr.operator == "-" => self
                .resolve_static_object_identity_literal_value(&expr.argument)
                .and_then(|value| match value {
                    StaticObjectIdentityValue::Number(number) => {
                        Some(StaticObjectIdentityValue::Number(if number == 0.0 {
                            -0.0
                        } else {
                            -number
                        }))
                    }
                    StaticObjectIdentityValue::BigInt(value) => {
                        Some(StaticObjectIdentityValue::BigInt(-value))
                    }
                    _ => None,
                }),
            Expression::UnaryExpression(expr) if expr.operator == "void" => {
                Some(StaticObjectIdentityValue::Undefined)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::SequenceExpression(expr) => {
                expr.expressions.last().and_then(|expression| {
                    self.resolve_static_object_identity_literal_value(expression)
                })
            }
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_object_identity_literal_value(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_object_identity_literal_value(&expr.alternate)
                    }
                    _ => {
                        let consequent =
                            self.resolve_static_object_identity_literal_value(&expr.consequent);
                        let alternate =
                            self.resolve_static_object_identity_literal_value(&expr.alternate);
                        match (consequent, alternate) {
                            (Some(consequent), Some(alternate))
                                if consequent.same_value(&alternate) =>
                            {
                                Some(consequent)
                            }
                            _ => None,
                        }
                    }
                }
            }
            Expression::LogicalExpression(expr) => {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                match expr.operator {
                    LogicalOperator::Coalesce => {
                        if left.is_nullish() {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        } else {
                            Some(left)
                        }
                    }
                    LogicalOperator::And => match left.truthiness() {
                        Some(true) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        Some(false) => Some(left),
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                    LogicalOperator::Or => match left.truthiness() {
                        Some(true) => Some(left),
                        Some(false) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                }
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "??" | "&&" | "||") =>
            {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                let selected = match expr.operator.as_str() {
                    "??" => {
                        if left.is_nullish() {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        } else {
                            Some(left)
                        }
                    }
                    "&&" => match left.truthiness() {
                        Some(true) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        Some(false) => Some(left),
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                    "||" => match left.truthiness() {
                        Some(true) => Some(left),
                        Some(false) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                    _ => unreachable!(),
                };
                selected
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().and_then(|expression| {
                    self.resolve_static_object_identity_literal_value(expression)
                })
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.resolve_static_object_identity_literal_value(object)
                }
            },
            Expression::Identifier(name) => match name.as_str() {
                "Infinity" => Some(StaticObjectIdentityValue::Number(f64::INFINITY)),
                "NaN" => Some(StaticObjectIdentityValue::Number(f64::NAN)),
                "undefined" => Some(StaticObjectIdentityValue::Undefined),
                _ => self
                    .resolve_static_object_identity_binding(name)
                    .or_else(|| {
                        self.resolve_static_reference_binding_name(name)
                            .map(StaticObjectIdentityValue::Reference)
                    }),
            },
            _ => None,
        }
    }

    pub(crate) fn resolve_static_numeric_binding(&self, name: &str) -> Option<f64> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.static_numeric_values.get(name) {
                return parse_numeric_literal_value(value);
            }
            current = scope.parent;
        }

        self.global_scope
            .static_numeric_values
            .get(name)
            .and_then(|value| parse_numeric_literal_value(value))
    }


    pub(crate) fn resolve_static_object_binding_name(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.static_objects.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }

        self.global_scope.static_objects.contains_key(name)
    }

    pub(crate) fn resolve_static_reference_binding_name(&self, name: &str) -> Option<String> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if let Some(root) = scope.static_reference_values.get(name) {
                return Some(root.clone());
            }
            current = scope.parent;
        }

        self.global_scope.static_reference_values.get(name).cloned()
    }

    pub(crate) fn resolve_static_reference_root(&self, expression: &Expression) -> Option<String> {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::AwaitExpression(expr) => self.resolve_static_reference_root(&expr.argument),
            Expression::TypeAssertion(expr) => self.resolve_static_reference_root(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(|expression| self.resolve_static_reference_root(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_reference_root(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_reference_root(&expr.alternate)
                    }
                    _ => {
                        let consequent = self.resolve_static_reference_root(&expr.consequent);
                        let alternate = self.resolve_static_reference_root(&expr.alternate);
                        match (consequent, alternate) {
                            (Some(consequent), Some(alternate)) if consequent == alternate => {
                                Some(consequent)
                            }
                            _ => None,
                        }
                    }
                }
            }
            Expression::LogicalExpression(expr) => {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                match expr.operator {
                    LogicalOperator::Coalesce => {
                        if left.is_nullish() {
                            self.resolve_static_reference_root(&expr.right)
                        } else {
                            self.resolve_static_reference_root(&expr.left)
                        }
                    }
                    LogicalOperator::And => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.right),
                        Some(false) => self.resolve_static_reference_root(&expr.left),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                    LogicalOperator::Or => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.left),
                        Some(false) => self.resolve_static_reference_root(&expr.right),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                }
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "??" | "&&" | "||") =>
            {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                let selected = match expr.operator.as_str() {
                    "??" => {
                        if left.is_nullish() {
                            self.resolve_static_reference_root(&expr.right)
                        } else {
                            self.resolve_static_reference_root(&expr.left)
                        }
                    }
                    "&&" => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.right),
                        Some(false) => self.resolve_static_reference_root(&expr.left),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                    "||" => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.left),
                        Some(false) => self.resolve_static_reference_root(&expr.right),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                    _ => unreachable!(),
                };
                selected
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.resolve_static_reference_root(object)
                }
            },
            Expression::MemberExpression(member) => Self::member_access_name(member),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .and_then(|expression| self.resolve_static_reference_root(expression)),
            Expression::Identifier(name) => self
                .resolve_static_reference_binding_name(name)
                .or_else(|| {
                    self.resolve_static_object_binding_name(name)
                        .then(|| name.clone())
                })
                .or_else(|| {
                    self.resolve_static_array_binding_name(name)
                        .then(|| name.clone())
                })
                .or_else(|| matches!(name.as_str(), "Set" | "Map").then(|| name.clone())),
            _ => None,
        }
    }

    pub(crate) fn resolve_static_object_keys_binding_name(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.static_object_keys.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }

        self.global_scope.static_object_keys.contains_key(name)
    }

    pub(crate) fn resolve_static_object_model_target(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|expression| self.resolve_static_object_model_target(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_object_model_target(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_object_model_target(&expr.alternate)
                    }
                    _ => {
                        self.resolve_static_object_model_target(&expr.consequent)
                            && self.resolve_static_object_model_target(&expr.alternate)
                    }
                }
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_object_model_target(&expr.argument)
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                properties.iter().all(|property| {
                    matches!(property.kind, ObjectPropertyKind::Init)
                        && matches!(
                            property.key,
                            PropertyName::Identifier(_)
                                | PropertyName::Number(_)
                                | PropertyName::String(_)
                        )
                })
            }
            Expression::ArrayExpression(_) => true,
            Expression::CallExpression(call) => {
                self.resolve_static_object_from_entries_call(call)
                    || Self::is_object_freeze_call(call)
                        && call.args.first().is_some_and(|argument| {
                            self.resolve_static_object_model_target(argument)
                        })
            }
            Expression::Identifier(name) => {
                name != "globalThis" && self.resolve_static_object_binding_name(name)
            }
            _ => false,
        }
    }

    pub(crate) fn resolve_static_object_keys_target(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.argument)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|expression| self.resolve_static_object_keys_target(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_object_keys_target(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_object_keys_target(&expr.alternate)
                    }
                    _ => {
                        self.resolve_static_object_keys_target(&expr.consequent)
                            && self.resolve_static_object_keys_target(&expr.alternate)
                    }
                }
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                properties.iter().all(|property| {
                    matches!(
                        property.kind,
                        ObjectPropertyKind::Init
                            | ObjectPropertyKind::Get
                            | ObjectPropertyKind::Set
                    ) && matches!(
                        property.key,
                        PropertyName::Identifier(_)
                            | PropertyName::Number(_)
                            | PropertyName::String(_)
                    )
                })
            }
            Expression::CallExpression(call) => {
                self.resolve_static_object_from_entries_call(call)
                    || Self::is_object_freeze_call(call)
                        && call.args.first().is_some_and(|argument| {
                            self.resolve_static_object_keys_target(argument)
                        })
                    || Self::is_reflect_own_keys_call(call)
                        && call.args.first().is_some_and(|argument| {
                            self.resolve_static_object_keys_target(argument)
                        })
            }
            Expression::Identifier(name) => self.resolve_static_object_keys_binding_name(name),
            other => self
                .resolve_static_string_iterable_expression(other)
                .is_some(),
        }
    }

    pub(crate) fn is_object_freeze_call(call: &CallExpression) -> bool {
        matches!(
            Self::call_member_access_name(&call.callee).as_deref(),
            Some("Object.freeze")
                | Some("globalThis.Object.freeze")
                | Some(r#"globalThis["Object"].freeze"#)
                | Some(r#"globalThis["Object"]["freeze"]"#)
                | Some(r#"globalThis['Object'].freeze"#)
                | Some(r#"globalThis['Object']['freeze']"#)
                | Some(r#"Object["freeze"]"#)
                | Some(r#"Object['freeze']"#)
                | Some(r#"globalThis.Object["freeze"]"#)
                | Some(r#"globalThis.Object['freeze']"#)
        ) && call.args.len() == 1
    }

    pub(crate) fn is_reflect_own_keys_call(call: &CallExpression) -> bool {
        matches!(
            Self::call_member_access_name(&call.callee).as_deref(),
            Some("Reflect.ownKeys")
                | Some("Reflect[\"ownKeys\"]")
                | Some("Reflect['ownKeys']")
                | Some("globalThis.Reflect.ownKeys")
                | Some("globalThis.Reflect[\"ownKeys\"]")
                | Some("globalThis.Reflect['ownKeys']")
                | Some(r#"globalThis["Reflect"].ownKeys"#)
                | Some(r#"globalThis["Reflect"]["ownKeys"]"#)
                | Some(r#"globalThis["Reflect"]['ownKeys']"#)
                | Some(r#"globalThis['Reflect'].ownKeys"#)
                | Some(r#"globalThis['Reflect']['ownKeys']"#)
                | Some(r#"globalThis['Reflect']["ownKeys"]"#)
        ) && call.args.len() == 1
    }

    pub(crate) fn resolve_static_object_from_entries_call(&self, call: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&call.callee) else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "Object.fromEntries"
                | "globalThis.Object.fromEntries"
                | r#"globalThis["Object"].fromEntries"#
                | r#"globalThis["Object"]["fromEntries"]"#
                | r#"globalThis['Object'].fromEntries"#
                | r#"globalThis['Object']['fromEntries']"#
                | r#"Object["fromEntries"]"#
                | r#"Object['fromEntries']"#
                | r#"globalThis.Object["fromEntries"]"#
                | r#"globalThis.Object['fromEntries']"#
        ) {
            return false;
        }

        let Some(entries) = call.args.first() else {
            return false;
        };
        if call.args.len() != 1 {
            return false;
        }

        self.resolve_static_from_entries_entries(entries)
    }

    pub(crate) fn resolve_static_from_entries_entries(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|expression| self.resolve_static_from_entries_entries(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_from_entries_entries(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_from_entries_entries(&expr.alternate)
                    }
                    _ => {
                        self.resolve_static_from_entries_entries(&expr.consequent)
                            && self.resolve_static_from_entries_entries(&expr.alternate)
                    }
                }
            }
            Expression::ArrayExpression(entries) => entries.elements.iter().all(|entry| {
                let Some(ExpressionOrSpread::Expression(Expression::ArrayExpression(pair))) = entry
                else {
                    return false;
                };

                if pair.elements.len() != 2 {
                    return false;
                }

                let Some(ExpressionOrSpread::Expression(key)) =
                    pair.elements.first().and_then(|element| element.as_ref())
                else {
                    return false;
                };
                let Some(ExpressionOrSpread::Expression(value)) =
                    pair.elements.get(1).and_then(|element| element.as_ref())
                else {
                    return false;
                };

                self.resolve_static_object_identity_literal_value(key)
                    .is_some()
                    && self
                        .resolve_static_object_identity_literal_value(value)
                        .is_some()
            }),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .is_some_and(|expression| self.resolve_static_from_entries_entries(expression)),
            _ => false,
        }
    }

    pub(crate) fn resolve_permission_query_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        if !matches!(
            callee_name.as_str(),
            "Deno.permissions.query" | "globalThis.Deno.permissions.query"
        ) {
            return;
        }

        let Some(descriptor_name) = expr
            .args
            .first()
            .and_then(|expr| self.resolve_permissions_query_descriptor_name(expr))
        else {
            return;
        };

        if matches!(descriptor_name.as_str(), "read" | "write" | "net" | "env") {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "permission query descriptor '{}' is unavailable in the Phase-1 Deno permission facade",
                descriptor_name
            ),
        ));
    }

    pub(crate) fn resolve_process_kill_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        if !matches!(
            callee_name.as_str(),
            "process.kill" | "globalThis.process.kill"
        ) {
            return;
        }

        if self.api_surface != "node" {
            return;
        }

        let Some(first_arg) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::process_kill_zero_probe_unavailable_message(),
            ));
            return;
        };

        let Some(first_value) = self.resolve_static_numeric_literal_value(first_arg) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::process_kill_zero_probe_unavailable_message(),
            ));
            return;
        };

        if first_value != 0.0 || expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::process_kill_zero_probe_unavailable_message(),
            ));
        }
    }

    pub(crate) fn resolve_frozen_late_object_model_call(&mut self, expr: &CallExpression) -> bool {
        if !Self::is_object_freeze_call(expr) {
            return false;
        }

        let Some(argument) = expr.args.first() else {
            return false;
        };
        let Some((dotted, bracketed)) = self.resolve_frozen_late_object_model_name(argument) else {
            return false;
        };

        if !matches!(
            dotted.as_str(),
            "Proxy.revocable" | "globalThis.Proxy.revocable"
        ) {
            return false;
        }

        let single_quoted = bracketed.replace("[\"", "['").replace("\"]", "']");
        let single_quoted_root_dotted = Self::member_access_single_quoted_root_name(argument)
            .map(|root| {
                format!(
                    "{}.{}",
                    root,
                    dotted
                        .rsplit_once('.')
                        .map(|(_, leaf)| leaf)
                        .unwrap_or(dotted.as_str())
                )
            })
            .unwrap_or_else(|| single_quoted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late object-model API '{}' (aka {}, {}, {}) is unavailable until the later object-model compatibility path is enabled",
                dotted, bracketed, single_quoted, single_quoted_root_dotted
            ),
        ));
        true
    }

    pub(crate) fn resolve_frozen_late_object_model_name(
        &self,
        expression: &Expression,
    ) -> Option<(String, String)> {
        let mut current = expression;
        loop {
            match current {
                Expression::ParenthesizedExpression(expr) => current = &expr.expression,
                Expression::TypeAssertion(expr) => current = &expr.expression,
                Expression::SatisfiesExpression(expr) => current = &expr.expression,
                Expression::ChainExpression(expr) => current = &expr.expression,
                Expression::DecoratedExpression(expr) => current = &expr.expression,
                Expression::SequenceExpression(expr) => {
                    let last = expr.expressions.last()?;
                    current = last;
                }
                Expression::AwaitExpression(expr) => current = &expr.argument,
                Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                    OptionalChainInner::NonNull { object, .. } => current = object,
                },
                Expression::MemberExpression(member) => {
                    let dotted = Self::member_access_name(member)?;
                    let bracketed = Self::member_access_name_bracketed(member)?;
                    return Some((dotted, bracketed));
                }
                _ => return None,
            }
        }
    }

    pub(crate) fn resolve_static_object_model_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return false;
        };

        let is_object_has_own = matches!(
            callee_name.as_str(),
            "Object.hasOwn" | "globalThis.Object.hasOwn"
        );
        let is_has_own_property_call = matches!(
            callee_name.as_str(),
            "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
        );
        if !is_object_has_own && !is_has_own_property_call {
            return false;
        }

        let Some(object_arg) = expr.args.first() else {
            return false;
        };
        let Some(key_arg) = expr.args.get(1) else {
            return false;
        };

        if !self.resolve_static_object_model_target(object_arg) {
            return false;
        }
        if matches!(object_arg, Expression::Identifier(name) if name == "globalThis") {
            return false;
        }
        if self.resolve_static_string_expression(key_arg).is_none() {
            return false;
        }

        self.resolve_expression(object_arg);
        self.resolve_expression(key_arg);
        for arg in expr.args.iter().skip(2) {
            self.resolve_expression(arg);
        }
        true
    }

    pub(crate) fn resolve_static_object_identity_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "Object.is"
                | "globalThis.Object.is"
                | r#"globalThis["Object"].is"#
                | r#"globalThis["Object"]["is"]"#
                | r#"globalThis['Object'].is"#
                | r#"globalThis['Object']['is']"#
                | r#"Object["is"]"#
                | r#"Object['is']"#
                | r#"globalThis.Object["is"]"#
                | r#"globalThis.Object['is']"#
        ) {
            return false;
        }

        let Some(left) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is requires at least two statically-known primitive literal arguments or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
            ));
            return true;
        };
        let Some(right) = expr.args.get(1) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is requires at least two statically-known primitive literal arguments in the current phase; use explicit constants or the later compatibility path",
            ));
            return true;
        };

        let left_reference = self.resolve_static_object_identity_reference_name(left);
        let right_reference = self.resolve_static_object_identity_reference_name(right);
        if let (Some(left_reference), Some(right_reference)) = (left_reference, right_reference) {
            if left_reference == right_reference {
                self.resolve_expression(left);
                self.resolve_expression(right);
                for arg in expr.args.iter().skip(2) {
                    self.resolve_expression(arg);
                }
                return true;
            }
        }

        let left_value = self.resolve_static_object_identity_literal_value(left);
        let right_value = self.resolve_static_object_identity_literal_value(right);
        if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
            let _ = left_value.same_value(&right_value);
            self.resolve_expression(left);
            self.resolve_expression(right);
            for arg in expr.args.iter().skip(2) {
                self.resolve_expression(arg);
            }
            return true;
        }

        if self.resolve_static_object_model_target(left)
            || self.resolve_static_object_model_target(right)
        {
            self.resolve_expression(left);
            self.resolve_expression(right);
            for arg in expr.args.iter().skip(2) {
                self.resolve_expression(arg);
            }
            return true;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "Object.is is unavailable unless both arguments are statically-known primitive literals or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
        ));
        true
    }

    pub(crate) fn resolve_number_identity_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return false;
        };

        let Some(method) = callee_name
            .strip_prefix("Number.")
            .or_else(|| callee_name.strip_prefix("globalThis.Number."))
            .or_else(|| callee_name.strip_prefix(r#"globalThis["Number"]."#))
            .or_else(|| callee_name.strip_prefix(r#"globalThis['Number']."#))
        else {
            return false;
        };

        if !matches!(method, "isFinite" | "isNaN" | "isInteger" | "isSafeInteger") {
            return false;
        }

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Number.{method} requires at least one statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return true;
        };

        let Some(value) = self.resolve_static_object_identity_literal_value(value_expr) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Number.{method} is unavailable unless the argument is a statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return true;
        };

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }

        let _ = match value {
            StaticObjectIdentityValue::Number(number) => match method {
                "isFinite" => number.is_finite(),
                "isNaN" => number.is_nan(),
                "isInteger" => number.is_finite() && number.fract() == 0.0,
                "isSafeInteger" => {
                    number.is_finite()
                        && number.fract() == 0.0
                        && number.abs() <= 9007199254740991.0
                }
                _ => false,
            },
            _ => false,
        };

        true
    }

    pub(crate) fn resolve_global_number_predicate_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
        else {
            return false;
        };

        let method = match callee_name.as_str() {
            "isFinite"
            | "globalThis.isFinite"
            | r#"globalThis["isFinite"]"#
            | r#"globalThis['isFinite']"# => "isFinite",
            "isNaN" | "globalThis.isNaN" | r#"globalThis["isNaN"]"# | r#"globalThis['isNaN']"# => {
                "isNaN"
            }
            _ => return false,
        };

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "global {method} requires at least one statically-known numeric value in the current phase; use an explicit numeric constant or the later compatibility path"
                ),
            ));
            return true;
        };

        let Some(StaticObjectIdentityValue::Number(number)) =
            self.resolve_static_object_identity_literal_value(value_expr)
        else {
            self.resolve_expression(value_expr);
            for arg in expr.args.iter().skip(1) {
                self.resolve_expression(arg);
            }
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "global {method} is unavailable unless the argument is a statically-known numeric value in the current direct-runtime path; use an explicit numeric constant or the later compatibility path"
                ),
            ));
            return true;
        };

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }

        let _ = match method {
            "isFinite" => number.is_finite(),
            "isNaN" => number.is_nan(),
            _ => false,
        };

        true
    }

    pub(crate) fn resolve_number_parse_int_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
        else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "parseInt"
                | "globalThis.parseInt"
                | r#"globalThis["parseInt"]"#
                | r#"globalThis['parseInt']"#
                | "Number.parseInt"
                | "globalThis.Number.parseInt"
                | r#"globalThis["Number"].parseInt"#
                | r#"globalThis['Number'].parseInt"#
                | r#"Number["parseInt"]"#
                | r#"Number['parseInt']"#
                | r#"globalThis.Number["parseInt"]"#
                | r#"globalThis.Number['parseInt']"#
                | r#"globalThis["Number"]["parseInt"]"#
                | r#"globalThis['Number']['parseInt']"#
        ) {
            return false;
        }

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "parseInt requires at least one statically-known ASCII string argument in the current phase; use an explicit literal or the later compatibility path",
            ));
            return true;
        };

        let source = self.resolve_static_string_expression(value_expr);
        let radix = expr
            .args
            .get(1)
            .and_then(|argument| self.resolve_static_numeric_literal_value(argument));
        let supported_radix = expr.args.get(1).is_none_or(|_| {
            radix.is_some_and(|radix| {
                radix.is_finite()
                    && radix.fract() == 0.0
                    && (radix == 0.0 || (2.0..=36.0).contains(&radix))
            })
        });

        if matches!(expr.args.len(), 1 | 2)
            && source.as_ref().is_some_and(|source| source.is_ascii())
            && supported_radix
            && source
                .as_ref()
                .zip(Some(radix.unwrap_or(0.0)))
                .is_some_and(|(source, radix)| {
                    static_parse_int_ascii(source, radix as u32).is_some()
                })
        {
            self.resolve_expression(value_expr);
            for arg in expr.args.iter().skip(1) {
                self.resolve_expression(arg);
            }
            return true;
        }

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "parseInt is unavailable unless the input is a statically-known ASCII string that yields an integer result and the optional radix is omitted, 0, or a statically-known integer from 2 through 36 in the current direct-runtime path; use explicit literals or the later compatibility path",
        ));
        true
    }

    pub(crate) fn resolve_number_parse_float_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
        else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "parseFloat"
                | "globalThis.parseFloat"
                | r#"globalThis["parseFloat"]"#
                | r#"globalThis['parseFloat']"#
                | "Number.parseFloat"
                | "globalThis.Number.parseFloat"
                | r#"globalThis["Number"].parseFloat"#
                | r#"globalThis['Number'].parseFloat"#
                | r#"Number["parseFloat"]"#
                | r#"Number['parseFloat']"#
                | r#"globalThis.Number["parseFloat"]"#
                | r#"globalThis.Number['parseFloat']"#
                | r#"globalThis["Number"]["parseFloat"]"#
                | r#"globalThis['Number']['parseFloat']"#
        ) {
            return false;
        }

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "parseFloat requires at least one statically-known ASCII string argument in the current phase; use an explicit literal or the later compatibility path",
            ));
            return true;
        };

        let source = self.resolve_static_string_expression(value_expr);
        if expr.args.len() == 1
            && source.as_ref().is_some_and(|source| source.is_ascii())
            && source
                .as_ref()
                .is_some_and(|source| static_parse_float_ascii_integer(source).is_some())
        {
            self.resolve_expression(value_expr);
            return true;
        }

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "parseFloat is unavailable unless the input is a statically-known ASCII string that yields a bounded integer result in the current direct-runtime path; use explicit literals or the later compatibility path",
        ));
        true
    }






    pub(crate) fn resolve_static_object_model_call_target(&self, call: &CallExpression) -> bool {
        self.resolve_static_object_from_entries_call(call)
    }

    pub(crate) fn resolve_math_member_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        let Some(method) = callee_name
            .strip_prefix("Math.")
            .or_else(|| callee_name.strip_prefix("globalThis.Math."))
        else {
            return;
        };

        if method == "hypot" {
            if self
                .resolve_math_hypot_static_literal_root(&expr.args)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Math.hypot is unavailable unless every argument is a statically-known integer literal whose squared sum is a perfect-square integer literal in the current phase; use explicit constants or the later compatibility path",
            ));
            return;
        }

        if method == "sqrt" || method == "cbrt" || method == "log2" || method == "log10" {
            let literal_root = expr.args.first().and_then(|arg| {
                if method == "sqrt" {
                    self.resolve_math_sqrt_static_literal_root(arg)
                } else if method == "cbrt" {
                    self.resolve_math_cbrt_static_literal_root(arg)
                } else if method == "log2" {
                    self.resolve_math_log2_static_literal_exponent(arg)
                } else {
                    self.resolve_math_log10_static_literal_exponent(arg)
                }
            });
            if literal_root.is_some() {
                return;
            }

            let shape = if method == "sqrt" {
                "perfect-square"
            } else if method == "cbrt" {
                "perfect-cube"
            } else if method == "log2" {
                "positive power-of-two"
            } else {
                "positive power-of-ten"
            };
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {shape} integer literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "exp" || method == "log" || method == "exp2" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        match method {
                            "exp" => "zero",
                            "exp2" => "non-negative integer literal within the current integer-fold range",
                            _ => "one",
                        }
                    ),
                ));
                return;
            };

            if (method == "exp" && value == 0.0)
                || (method == "log" && value == 1.0)
                || (method == "exp2"
                    && value.is_finite()
                    && value.fract() == 0.0
                    && (0.0..=62.0).contains(&value))
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    match method {
                        "exp" => "zero",
                        "exp2" => "non-negative integer literal within the current integer-fold range",
                        _ => "one",
                    }
                ),
            ));
            return;
        }

        if method == "expm1" || method == "log1p" || method == "fround" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            if value == 0.0 {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "asin" || method == "acos" || method == "atan" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "acos" { "one" } else { "zero" }
                    ),
                ));
                return;
            };

            if (method == "acos" && value == 1.0) || (method != "acos" && value == 0.0) {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    if method == "acos" { "one" } else { "zero" }
                ),
            ));
            return;
        }

        if method == "atan2" {
            let atan2_message = "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string();
            let Some(y) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    atan2_message,
                ));
                return;
            };

            let Some(x) = expr
                .args
                .get(1)
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    atan2_message,
                ));
                return;
            };

            if y == 0.0 && x.is_finite() && x >= 0.0 {
                for arg in expr.args.iter().skip(2) {
                    self.resolve_expression(arg);
                }
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                atan2_message,
            ));
            return;
        }

        if method == "sin" || method == "cos" || method == "tan" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            let Some(value) = self.resolve_static_numeric_literal_value(argument) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            if value == 0.0 {
                self.resolve_expression(argument);
                for arg in expr.args.iter().skip(1) {
                    self.resolve_expression(arg);
                }
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "asinh" || method == "acosh" || method == "atanh" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "acosh" { "one" } else { "zero" }
                    ),
                ));
                return;
            };

            if self
                .resolve_math_inverse_hyperbolic_constant_value(method, argument)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    if method == "acosh" { "one" } else { "zero" }
                ),
            ));
            return;
        }

        if method == "sinh" || method == "cosh" || method == "tanh" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            };

            if self
                .resolve_math_hyperbolic_zero_constant_value(method, argument)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "max" || method == "min" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if let Some(_folded) =
                self.resolve_math_extrema_static_literal_value(method, &expr.args)
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "pow" {
            if Self::contains_optional_chain(&expr.callee) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable through optional-chain wrappers in the current phase; use a direct call or the later compatibility path",
                ));
                return;
            }

            if expr.args.len() < 2 {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow requires at least two arguments in the current phase; use explicit operands or the later compatibility path",
                ));
                return;
            }

            let base_value = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg));
            let exponent_value = expr
                .args
                .get(1)
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg));
            let exponent_is_static_zero = exponent_value.is_some_and(|value| value == 0.0);
            let base_is_static_zero = base_value.is_some_and(|value| value == 0.0);
            let base_is_static_unit = base_value.is_some_and(|value| value == 1.0 || value == -1.0);
            let exponent_is_positive_integer =
                exponent_value.is_some_and(|value| value > 0.0 && value.fract() == 0.0);
            let exponent_is_negative_integer =
                exponent_value.is_some_and(|value| value < 0.0 && value.fract() == 0.0);
            if base_is_static_zero && exponent_is_positive_integer {
                return;
            }

            if base_is_static_unit && exponent_is_negative_integer {
                return;
            }

            if !exponent_is_static_zero
                && expr
                    .args
                    .iter()
                    .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable for non-integer numeric literals in the current phase; use an integer-valued exponent or the later compatibility path",
                ));
                return;
            }

            if expr
                .args
                .get(1)
                .is_some_and(|arg| self.contains_negative_numeric_literal(arg))
                && !base_is_static_unit
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable for negative numeric literals unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path",
                ));
            }
            return;
        }

        if method == "round" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path",
                ));
            }
            return;
        }

        if method == "floor" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.floor requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.floor is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path",
                ));
            }
            return;
        }

        if matches!(method, "trunc" | "ceil") {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "sign" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.sign requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            return;
        }

        if matches!(method, "max" | "min" | "abs" | "asinh" | "acosh" | "atanh") {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "imul" {
            if expr.args.len() < 2 {
                for arg in &expr.args {
                    self.resolve_expression(arg);
                }
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.imul is unavailable for non-integer numeric literals in the current phase; use integer-valued operands or the later compatibility path",
                ));
            }
            return;
        }

        if method == "clz32" {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "Math.{method} is unavailable in the current phase; use a supported Math builtin or the later compatibility path"
            ),
        ));
    }




































    pub(crate) fn resolve_promise_member_call(&mut self, _expr: &CallExpression) {}

    pub(crate) fn contains_non_integer_numeric_literal(&self, expression: &Expression) -> bool {
        self.resolve_static_numeric_literal_value(expression)
            .is_some_and(|value| value.fract() != 0.0)
    }

    pub(crate) fn resolve_static_numeric_literal_value(&self, expression: &Expression) -> Option<f64> {
        match expression {
            Expression::Literal(LiteralValue::Number(value)) => Some(*value),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.argument)
            }
            Expression::UnaryExpression(expr) if expr.operator == "+" => {
                self.resolve_static_numeric_literal_value(&expr.argument)
            }
            Expression::UnaryExpression(expr) if expr.operator == "-" => self
                .resolve_static_numeric_literal_value(&expr.argument)
                .map(|value| -value),
            Expression::TypeAssertion(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.resolve_static_numeric_literal_value(object)
                }
            },
            Expression::ChainExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(|expression| self.resolve_static_numeric_literal_value(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_numeric_literal_value(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_numeric_literal_value(&expr.alternate)
                    }
                    _ => {
                        let consequent =
                            self.resolve_static_numeric_literal_value(&expr.consequent);
                        let alternate = self.resolve_static_numeric_literal_value(&expr.alternate);
                        match (consequent, alternate) {
                            (Some(consequent), Some(alternate)) if consequent == alternate => {
                                Some(consequent)
                            }
                            _ => None,
                        }
                    }
                }
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .and_then(|argument| self.resolve_static_numeric_literal_value(argument)),
            Expression::Identifier(name) => self.resolve_static_numeric_binding(name),
            _ => None,
        }
    }

    pub(crate) fn resolve_math_round_like_static_literal_value(
        &self,
        method: &str,
        expression: Option<&Expression>,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression?)?;
        let folded = match method {
            "round" => (value + 0.5).floor(),
            "trunc" => value.trunc(),
            "ceil" => value.ceil(),
            "floor" => value.floor(),
            _ => return None,
        };

        if !folded.is_finite() || folded < i64::MIN as f64 || folded > i64::MAX as f64 {
            return None;
        }

        Some(folded as i64)
    }

    pub(crate) fn contains_negative_numeric_literal(&self, expression: &Expression) -> bool {
        self.resolve_static_numeric_literal_value(expression)
            .is_some_and(|value| value < 0.0)
    }

    pub(crate) fn resolve_math_extrema_static_literal_value(
        &self,
        method: &str,
        expressions: &[Expression],
    ) -> Option<i64> {
        let mut values = expressions.iter().map(|expression| {
            let value = self.resolve_static_numeric_literal_value(expression)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }
            Some(value as i64)
        });

        let first = values.next().flatten()?;
        let mut folded = first;

        for value in values {
            let value = value?;
            folded = if method == "max" {
                folded.max(value)
            } else {
                folded.min(value)
            };
        }

        Some(folded)
    }

    pub(crate) fn resolve_math_inverse_hyperbolic_constant_value(
        &self,
        method: &str,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;

        match method {
            "acosh" if value == 1.0 => Some(0),
            "asinh" | "atanh" if value == 0.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn resolve_math_hyperbolic_zero_constant_value(
        &self,
        method: &str,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cosh" { 1 } else { 0 })
    }

    pub(crate) fn resolve_math_sqrt_static_literal_root(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value < 0.0 || value > i64::MAX as f64 {
            return None;
        }

        let value = value as i64;
        let root = (value as f64).sqrt() as i64;
        if root.checked_mul(root) == Some(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn resolve_math_cbrt_static_literal_root(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite()
            || value.fract() != 0.0
            || value < i64::MIN as f64
            || value > i64::MAX as f64
        {
            return None;
        }

        let value = value as i64;
        let root = (value as f64).cbrt().round() as i64;
        if i128::from(root).pow(3) == i128::from(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn resolve_math_log2_static_literal_exponent(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value <= 0.0 || value > u64::MAX as f64 {
            return None;
        }

        let value = value as u64;
        if value.is_power_of_two() {
            Some(i64::from(value.trailing_zeros()))
        } else {
            None
        }
    }

    pub(crate) fn resolve_math_log10_static_literal_exponent(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value <= 0.0 || value > i64::MAX as f64 {
            return None;
        }

        let mut value = value as i64;
        let mut exponent = 0;
        while value % 10 == 0 {
            value /= 10;
            exponent += 1;
        }

        if value == 1 {
            Some(exponent)
        } else {
            None
        }
    }

    pub(crate) fn resolve_math_hypot_static_literal_root(&self, expressions: &[Expression]) -> Option<i64> {
        if expressions.is_empty() {
            return Some(0);
        }

        let mut sum = 0_i128;
        for expression in expressions {
            let value = self.resolve_static_numeric_literal_value(expression)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }

            let value = value as i128;
            sum = sum.checked_add(value.checked_mul(value)?)?;
        }

        self.resolve_perfect_square_i128(sum)
    }

    pub(crate) fn resolve_perfect_square_i128(&self, value: i128) -> Option<i64> {
        if value < 0 {
            return None;
        }

        let mut low = 0_i128;
        let mut high = i128::from(i64::MAX).min(value);
        while low <= high {
            let mid = low + (high - low) / 2;
            let square = mid.checked_mul(mid)?;
            if square == value {
                return Some(mid as i64);
            }
            if square < value {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        None
    }

    pub(crate) fn resolve_permissions_query_descriptor_name(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                for property in properties {
                    if !matches!(property.kind, ObjectPropertyKind::Init) {
                        continue;
                    }

                    let key_name = match &property.key {
                        PropertyName::Identifier(name) | PropertyName::String(name) => {
                            name.as_str()
                        }
                        PropertyName::Number(_) => continue,
                    };

                    if key_name != "name" {
                        continue;
                    }

                    return self.resolve_static_string_expression(&property.value);
                }

                None
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_threaded_runtime_member(&mut self, expr: &MemberExpression) {
        let Expression::Identifier(object_name) = &expr.object else {
            return;
        };

        if object_name != "globalThis" {
            return;
        }

        if !matches!(expr.property.as_str(), "SharedArrayBuffer" | "Atomics") {
            return;
        }

        if self.has_threaded_runtime_profile() {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "threaded runtime global 'globalThis.{}' is unavailable until the WASM-threaded profile is enabled",
                expr.property
            ),
        ));
    }

    pub(crate) fn resolve_late_host_control_member(&mut self, expr: &MemberExpression) {
        if !matches!(
            expr.property.as_str(),
            "pid" | "cwd" | "chdir" | "exit" | "kill"
        ) {
            return;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return;
        };

        if expr.property == "pid" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "exit" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "cwd" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "chdir" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "cwd" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "chdir" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "pid" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "exit" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "kill" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if !matches!(object_name.as_str(), "Deno" | "process") {
            return;
        }

        let dotted = Self::member_access_name(expr)
            .unwrap_or_else(|| format!("{}.{}", object_name, expr.property));
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());
        let extra_alias = if object_name == "Deno"
            && matches!(expr.property.as_str(), "cwd" | "chdir" | "exit")
        {
            Some(format!("globalThis[\"Deno\"].{}", expr.property))
        } else if object_name == "process"
            && matches!(
                expr.property.as_str(),
                "pid" | "cwd" | "chdir" | "exit" | "kill"
            )
        {
            let mut aliases = vec![
                format!("globalThis[\"process\"].{}", expr.property),
                format!("globalThis.process[\"{}\"]", expr.property),
                format!("globalThis[\"process\"][\"{}\"]", expr.property),
            ];
            if expr.property == "kill" {
                aliases.extend(
                    late_process_control_single_quoted_kill_aliases()
                        .iter()
                        .copied()
                        .map(String::from),
                );
                aliases.extend(
                    process_kill_zero_probe_wrapped_zero_aliases()
                        .iter()
                        .copied()
                        .map(String::from),
                );
            } else if expr.property == "exit" {
                aliases.extend(
                    late_process_control_single_quoted_exit_aliases()
                        .iter()
                        .copied()
                        .map(String::from),
                );
            }
            Some(aliases.join(", "))
        } else {
            None
        };

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late host-control API '{}' (aka {}{}) is unavailable until the later host-control compatibility path is enabled",
                dotted,
                bracketed,
                extra_alias
                    .as_deref()
                    .map(|alias| format!(", {alias}"))
                    .unwrap_or_default()
            ),
        ));
    }

    pub(crate) fn resolve_late_subprocess_member(&mut self, expr: &MemberExpression) -> bool {
        if self.sandbox_policy_attached {
            return false;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "Deno" || expr.property != "Command" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "subprocess spawning API '{}' (aka {}) is unavailable until the later subprocess compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_network_member(&mut self, expr: &MemberExpression) -> bool {
        if self.sandbox_policy_attached {
            return false;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "Deno"
            || !matches!(expr.property.as_str(), "connect" | "listen" | "serve")
        {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "socket/listener networking API '{}' (aka {}) is unavailable until the later network compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_permission_escalation_member(&mut self, expr: &MemberExpression) -> bool {
        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.permissions.request"
                | "Deno.permissions.revoke"
                | "globalThis.Deno.permissions.request"
                | "globalThis.Deno.permissions.revoke"
        ) && !matches!(
            bracketed.as_str(),
            r#"Deno["permissions"]["request"]"#
                | r#"Deno["permissions"]["revoke"]"#
                | r#"globalThis["Deno"]["permissions"]["request"]"#
                | r#"globalThis["Deno"]["permissions"]["revoke"]"#
        ) {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "permission escalation API '{}' (aka {}) is unavailable in the Phase-1 Deno permission facade",
                dotted, bracketed
            ),
        ));
        true
    }

    pub(crate) fn resolve_deno_args_member(&mut self, expr: &MemberExpression) -> bool {
        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "Deno" || expr.property != "args" {
            return false;
        }

        if self.api_surface == "deno" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "invocation arguments API '{}' (aka {}) is unavailable on the {} API surface until the Deno runtime surface is enabled",
                dotted, bracketed, self.api_surface
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_env_object_member(&mut self, expr: &MemberExpression) -> bool {
        if self.api_surface == "deno" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.env.toObject"
                | "Deno.env[\"toObject\"]"
                | "globalThis.Deno.env.toObject"
                | "globalThis.Deno.env[\"toObject\"]"
                | "Deno[\"env\"].toObject"
                | "Deno[\"env\"][\"toObject\"]"
                | "globalThis.Deno[\"env\"].toObject"
                | "globalThis.Deno[\"env\"][\"toObject\"]"
                | "globalThis[\"Deno\"].env.toObject"
                | "globalThis[\"Deno\"].env[\"toObject\"]"
                | "globalThis[\"Deno\"][\"env\"].toObject"
                | "globalThis[\"Deno\"][\"env\"][\"toObject\"]"
        ) && !matches!(
            bracketed.as_str(),
            r#"Deno["env"]["toObject"]"#
                | r#"globalThis["Deno"]["env"]["toObject"]"#
                | r#"globalThis.Deno["env"]["toObject"]"#
        ) {
            return false;
        }

        let aliases = [
            bracketed.as_str(),
            "Deno.env[\"toObject\"]",
            "Deno[\"env\"].toObject",
            "Deno[\"env\"][\"toObject\"]",
            "globalThis.Deno.env[\"toObject\"]",
            "globalThis.Deno[\"env\"].toObject",
            "globalThis.Deno[\"env\"][\"toObject\"]",
            "globalThis[\"Deno\"].env.toObject",
            "globalThis[\"Deno\"].env[\"toObject\"]",
            "globalThis[\"Deno\"][\"env\"].toObject",
            "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
            "globalThis[\"Deno\"].env[\"toObject\"]",
            "globalThis.Deno[\"env\"][\"toObject\"]",
            "globalThis[\"Deno\"].env[\"toObject\"]",
        ];

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "environment snapshot materialization API '{}' (aka {}) is unavailable until the later env-object materialization and object-aggregate lowering path is enabled",
                dotted,
                aliases.join(", "),
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_env_mutation_member(&mut self, expr: &MemberExpression) -> bool {
        if self.api_surface == "deno" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.env.set"
                | "Deno.env.delete"
                | "globalThis.Deno.env.set"
                | "globalThis.Deno.env.delete"
        ) {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                dotted, bracketed, self.api_surface
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_env_assignment_mutation(&mut self, expr: &AssignmentExpression) -> bool {
        let Expression::MemberExpression(member) = &expr.left else {
            return false;
        };

        let dotted = Self::member_access_name(member).unwrap_or_else(|| member.property.clone());
        let bracketed =
            Self::member_access_name_bracketed(member).unwrap_or_else(|| dotted.clone());

        if Self::is_process_env_root_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        if self.api_surface != "node" && Self::is_process_env_mutation_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        false
    }

    pub(crate) fn resolve_late_process_env_mutation_member(&mut self, member: &MemberExpression) -> bool {
        let dotted = Self::member_access_name(member).unwrap_or_else(|| member.property.clone());
        let bracketed =
            Self::member_access_name_bracketed(member).unwrap_or_else(|| dotted.clone());

        if Self::is_process_env_root_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        if self.api_surface != "node" && Self::is_process_env_mutation_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        false
    }

    pub(crate) fn is_process_env_root_path(path: &str) -> bool {
        matches!(path, "process.env" | "globalThis.process.env")
    }

    pub(crate) fn is_process_env_mutation_path(path: &str) -> bool {
        Self::is_process_env_root_path(path)
            || path.starts_with("process.env.")
            || path.starts_with("process.env[")
            || path.starts_with("globalThis.process.env.")
            || path.starts_with("globalThis.process.env[")
    }

    pub(crate) fn resolve_late_intl_member(&mut self, expr: &MemberExpression) -> bool {
        let is_intl_root = matches!(&expr.object, Expression::Identifier(name) if name == "Intl")
            || matches!(
                &expr.object,
                Expression::Identifier(name) if name == "globalThis" && expr.property == "Intl"
            )
            || matches!(
                &expr.object,
                Expression::MemberExpression(member)
                    if matches!(&member.object, Expression::Identifier(name) if name == "globalThis")
                        && member.property == "Intl"
            );

        if !is_intl_root {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr)
            .unwrap_or_else(|| format!("globalThis[\"{}\"]", expr.property));
        let single_quoted = Self::member_access_name_single_quoted(expr)
            .unwrap_or_else(|| format!("globalThis['{}']", expr.property));
        let single_quoted_root_dotted = Self::member_access_single_quoted_root_name(&expr.object)
            .map(|root| format!("{}.{}", root, expr.property))
            .unwrap_or_else(|| single_quoted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "broader Intl support via '{}' (aka {}, {}, {}) is unavailable until the later web/Intl compatibility path is enabled",
                dotted, bracketed, single_quoted, single_quoted_root_dotted
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_object_model_member(&mut self, expr: &MemberExpression) -> bool {
        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());
        let single_quoted = Self::member_access_name_single_quoted(expr).unwrap_or_else(|| {
            format!(
                "{}['{}']",
                dotted
                    .rsplit_once('.')
                    .map(|(root, _)| root)
                    .unwrap_or(&dotted),
                expr.property
            )
        });
        let single_quoted_root_dotted = Self::member_access_single_quoted_root_name(&expr.object)
            .map(|root| format!("{}.{}", root, expr.property))
            .unwrap_or_else(|| single_quoted.clone());

        if self.api_surface != "node"
            && self.is_supported_static_callable_member_name(&dotted, &bracketed)
        {
            return false;
        }

        if matches!(
            dotted.as_str(),
            "Proxy.revocable" | "globalThis.Proxy.revocable"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' (aka {}, {}, {}) is unavailable until the later object-model compatibility path is enabled",
                    dotted, bracketed, single_quoted, single_quoted_root_dotted
                ),
            ));
            return true;
        }

        if matches!(
            dotted.as_str(),
            "Object.hasOwn"
                | "globalThis.Object.hasOwn"
                | "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' (aka {}) is unavailable until the later object-model compatibility path is enabled",
                    dotted, bracketed
                ),
            ));
            return true;
        }

        if !matches!(
            expr.property.as_str(),
            "Proxy" | "WeakMap" | "WeakSet" | "WeakRef" | "FinalizationRegistry"
        ) {
            return false;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "globalThis" {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late object-model API '{}' (aka {}) is unavailable until the later object-model compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

}

#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

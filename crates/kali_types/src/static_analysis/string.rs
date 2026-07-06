//! String static-analysis helpers.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_static_string_iterable_expression(
        &self,
        expression: &Expression,
    ) -> Option<String> {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::Literal(LiteralValue::String(value)) => Some(value.clone()),
            Expression::Identifier(name) => self.resolve_static_string_binding(name),
            Expression::BinaryExpression(expr) if expr.operator == "+" => {
                let left = self.resolve_static_string_iterable_expression(&expr.left)?;
                let right = self.resolve_static_string_iterable_expression(&expr.right)?;
                Some(format!("{left}{right}"))
            }
            Expression::TemplateLiteral(_) => self.resolve_static_string_expression(expression),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_string_iterable_expression(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_string_iterable_expression(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_string_iterable_expression(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_string_iterable_expression(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_string_iterable_expression(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(|expression| self.resolve_static_string_iterable_expression(expression)),
            _ => None,
        }
    }
    pub(crate) fn resolve_static_string_expression(
        &self,
        expression: &Expression,
    ) -> Option<String> {
        match expression {
            Expression::Literal(LiteralValue::String(value)) => {
                if let Some(rendered) = resolve_interpolated_template_literal(value, |segment| {
                    self.resolve_static_string_from_source(segment)
                }) {
                    Some(rendered)
                } else {
                    Some(Self::normalize_import_segment(value))
                }
            }
            Expression::Literal(LiteralValue::Number(value)) => Some(value.to_string()),
            Expression::Literal(LiteralValue::Boolean(value)) => Some(value.to_string()),
            Expression::Literal(LiteralValue::Null) => Some("null".to_string()),
            Expression::BinaryExpression(expr) if expr.operator == "+" => {
                let left = self.resolve_static_string_expression(&expr.left)?;
                let right = self.resolve_static_string_expression(&expr.right)?;
                Some(format!("{}{}", left, right))
            }
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_string_expression(&expr.argument)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(|expression| self.resolve_static_string_expression(expression)),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .and_then(|argument| self.resolve_static_string_expression(argument)),
            Expression::CallExpression(call) => self
                .resolve_static_string_from_char_code_expression(call)
                .or_else(|| self.resolve_static_string_concat_expression(call))
                .or_else(|| self.resolve_static_string_normalize_expression(call)),
            Expression::LogicalExpression(expr) => {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                match expr.operator {
                    LogicalOperator::Coalesce => {
                        if left.is_nullish() {
                            self.resolve_static_string_expression(&expr.right)
                        } else {
                            self.resolve_static_string_expression(&expr.left)
                        }
                    }
                    LogicalOperator::And => match left.truthiness() {
                        Some(true) => self.resolve_static_string_expression(&expr.right),
                        Some(false) => self.resolve_static_string_expression(&expr.left),
                        None => None,
                    },
                    LogicalOperator::Or => match left.truthiness() {
                        Some(true) => self.resolve_static_string_expression(&expr.left),
                        Some(false) => self.resolve_static_string_expression(&expr.right),
                        None => None,
                    },
                }
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "??" | "&&" | "||") =>
            {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                match expr.operator.as_str() {
                    "??" => {
                        if left.is_nullish() {
                            self.resolve_static_string_expression(&expr.right)
                        } else {
                            self.resolve_static_string_expression(&expr.left)
                        }
                    }
                    "&&" => match left.truthiness() {
                        Some(true) => self.resolve_static_string_expression(&expr.right),
                        Some(false) => self.resolve_static_string_expression(&expr.left),
                        None => None,
                    },
                    "||" => match left.truthiness() {
                        Some(true) => self.resolve_static_string_expression(&expr.left),
                        Some(false) => self.resolve_static_string_expression(&expr.right),
                        None => None,
                    },
                    _ => unreachable!(),
                }
            }
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_string_expression(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_string_expression(&expr.alternate)
                    }
                    _ => {
                        let consequent = self.resolve_static_string_expression(&expr.consequent);
                        let alternate = self.resolve_static_string_expression(&expr.alternate);
                        match (consequent, alternate) {
                            (Some(consequent), Some(alternate)) if consequent == alternate => {
                                Some(consequent)
                            }
                            _ => None,
                        }
                    }
                }
            }
            Expression::TemplateLiteral(template) => {
                let mut rendered = String::new();
                for (idx, quasi) in template.quasis.iter().enumerate() {
                    rendered.push_str(&quasi.value);
                    if let Some(expr) = template.expressions.get(idx) {
                        rendered.push_str(&self.resolve_static_string_expression(expr)?);
                    }
                }
                Some(rendered)
            }
            Expression::Identifier(name) => self.resolve_static_string_binding(name),
            _ => None,
        }
    }
    pub(crate) fn resolve_static_string_from_char_code_expression(
        &self,
        expr: &CallExpression,
    ) -> Option<String> {
        let callee_name =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })?;

        if !Self::is_string_from_char_code_callable_name(&callee_name) {
            return None;
        }

        let mut rendered = String::new();
        for arg in &expr.args {
            let value = self.resolve_static_numeric_literal_value(arg)?;
            if !is_supported_static_ascii_char_code(value) {
                return None;
            }
            rendered.push(char::from_u32(value as u32)?);
        }
        Some(rendered)
    }
    pub(crate) fn resolve_static_string_normalize_expression(
        &self,
        expr: &CallExpression,
    ) -> Option<String> {
        let Expression::MemberExpression(member) = &expr.callee else {
            return None;
        };
        if member.property.as_str() != "normalize" {
            return None;
        }

        let source = self.resolve_static_string_expression(&member.object)?;
        if !source.is_ascii() || expr.args.len() > 1 {
            return None;
        }
        let form = expr
            .args
            .first()
            .map(|argument| self.resolve_static_string_expression(argument))
            .unwrap_or_else(|| Some("NFC".to_string()))?;
        matches!(form.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD").then_some(source)
    }
    pub(crate) fn resolve_static_string_binding(&self, name: &str) -> Option<String> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.static_values.get(name) {
                return Some(value.clone());
            }
            current = scope.parent;
        }

        self.global_scope.static_values.get(name).cloned()
    }
    pub(crate) fn resolve_static_string_from_source(&self, source: &str) -> Option<String> {
        let wrapped = format!("const __kali_template__ = ({source});");
        let lexer = Lexer::new(kali_common::FileId::new(0), wrapped);
        let tokens = lexer.lex_all().tokens;
        let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;
        let Statement::VariableDeclaration(declaration) = statements.first()? else {
            return None;
        };
        let initializer = declaration.declarations.first()?.init.as_ref()?;
        self.resolve_static_string_expression(initializer)
    }
    pub(crate) fn resolve_string_from_char_code_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
        else {
            return false;
        };

        let Some(method) = Self::static_ascii_string_constructor_method(&callee_name) else {
            return false;
        };

        let supported = expr.args.iter().all(|arg| {
            self.resolve_static_numeric_literal_value(arg)
                .is_some_and(is_supported_static_ascii_char_code)
        });

        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        if supported {
            return true;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "String.{method} is unavailable unless every argument is a statically-known ASCII integer code unit from 0 through 127 in the current direct-runtime path; use explicit ASCII integer literals or the later compatibility path"
            ),
        ));
        true
    }
    pub(crate) fn is_string_from_char_code_callable_name(name: &str) -> bool {
        Self::static_ascii_string_constructor_method(name) == Some("fromCharCode")
    }
    pub(crate) fn static_ascii_string_constructor_method(name: &str) -> Option<&'static str> {
        match name {
            "String.fromCharCode"
            | "globalThis.String.fromCharCode"
            | r#"String["fromCharCode"]"#
            | r#"String['fromCharCode']"#
            | r#"globalThis.String["fromCharCode"]"#
            | r#"globalThis.String['fromCharCode']"#
            | r#"globalThis["String"].fromCharCode"#
            | r#"globalThis['String'].fromCharCode"#
            | r#"globalThis["String"]["fromCharCode"]"#
            | r#"globalThis['String']['fromCharCode']"# => Some("fromCharCode"),
            "String.fromCodePoint"
            | "globalThis.String.fromCodePoint"
            | r#"String["fromCodePoint"]"#
            | r#"String['fromCodePoint']"#
            | r#"globalThis.String["fromCodePoint"]"#
            | r#"globalThis.String['fromCodePoint']"#
            | r#"globalThis["String"].fromCodePoint"#
            | r#"globalThis['String'].fromCodePoint"#
            | r#"globalThis["String"]["fromCodePoint"]"#
            | r#"globalThis['String']['fromCodePoint']"# => Some("fromCodePoint"),
            _ => None,
        }
    }
    pub(crate) fn resolve_string_search_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        let method = member.property.as_str();
        if !matches!(
            method,
            "includes" | "indexOf" | "lastIndexOf" | "startsWith" | "endsWith"
        ) {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        if source.is_none() {
            return;
        }
        let search = expr
            .args
            .first()
            .map(|argument| self.resolve_static_string_expression(argument))
            .unwrap_or_else(|| Some("undefined".to_string()));
        let has_ascii_source_and_search = source
            .as_ref()
            .zip(search.as_ref())
            .is_some_and(|(source, search)| source.is_ascii() && search.is_ascii());
        let supported_arg_count = matches!(expr.args.len(), 0..=2);
        let has_static_position = expr
            .args
            .get(1)
            .is_none_or(|argument| self.is_static_numeric_literal_expr(argument));

        if supported_arg_count && has_ascii_source_and_search && has_static_position {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "string search method '{method}' is unavailable unless the receiver, optional search value, and position/fromIndex are statically-known ASCII string/number literals in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
            ),
        ));
    }
    pub(crate) fn resolve_string_slice_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "slice" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        if source.is_none() {
            return;
        }

        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let supported_arg_count = matches!(expr.args.len(), 0..=2);
        let has_static_finite_bounds = expr.args.iter().all(|argument| {
            self.resolve_static_numeric_literal_value(argument)
                .is_some_and(|bound| bound.is_finite())
        });

        if supported_arg_count && has_ascii_source && has_static_finite_bounds {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.slice is unavailable unless the receiver is a statically-known ASCII string literal and the start/end bounds are statically-known finite numeric literals in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_string_substring_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "substring" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let supported_arg_count = matches!(expr.args.len(), 0..=2);
        let has_static_finite_bounds = expr.args.iter().all(|argument| {
            self.resolve_static_numeric_literal_value(argument)
                .is_some_and(|bound| bound.is_finite())
        });

        if supported_arg_count && has_ascii_source && has_static_finite_bounds {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        let receiver_is_runtime_ascii_string = self.expression_repr_is_ascii_string(&member.object);
        let bounds_are_int_repr = expr
            .args
            .iter()
            .all(|argument| self.expression_is_int_repr_bound(argument));
        if supported_arg_count && receiver_is_runtime_ascii_string && bounds_are_int_repr {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.substring is unavailable unless the receiver is a statically-known ASCII string literal with statically-known finite numeric bounds, or an ASCII-provable runtime string value with integer-typed bounds, in the current direct-runtime path; non-ASCII receivers and float-typed bounds are rejected".to_string(),
        ));
    }
    pub(crate) fn resolve_string_repeat_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "repeat" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        if source.is_none() {
            return;
        }

        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let repeat_count = expr
            .args
            .first()
            .and_then(|argument| self.resolve_static_numeric_literal_value(argument));
        let has_supported_count = repeat_count.is_some_and(|count| {
            count.is_finite() && count.fract() == 0.0 && (0.0..=1024.0).contains(&count)
        });

        if expr.args.len() == 1 && has_ascii_source && has_supported_count {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.repeat is unavailable unless the receiver is a statically-known ASCII string literal and the repeat count is a statically-known integer from 0 through 1024 in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_static_string_concat_expression(
        &self,
        expr: &CallExpression,
    ) -> Option<String> {
        let Expression::MemberExpression(member) = &expr.callee else {
            return None;
        };
        if member.property.as_str() != "concat" {
            return None;
        }

        let mut result = self.resolve_static_string_expression(&member.object)?;
        if !result.is_ascii() {
            return None;
        }
        for arg in &expr.args {
            let value = self.resolve_static_string_expression(arg)?;
            if !value.is_ascii() {
                return None;
            }
            result.push_str(&value);
        }
        Some(result)
    }
    pub(crate) fn resolve_string_concat_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "concat" {
            return;
        }

        if self.is_static_array_concat_receiver(&member.object) {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let has_ascii_args = expr.args.iter().all(|argument| {
            self.resolve_static_string_expression(argument)
                .is_some_and(|value| value.is_ascii())
        });

        if has_ascii_source && has_ascii_args {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.concat is unavailable unless the receiver and all operands are statically-known ASCII string literals in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_string_pad_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        let method = member.property.as_str();
        if !matches!(method, "padStart" | "padEnd") {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let target_length = expr
            .args
            .first()
            .and_then(|argument| self.resolve_static_numeric_literal_value(argument));
        let has_supported_target_length = target_length.is_some_and(|length| {
            length.is_finite() && length.fract() == 0.0 && (0.0..=1024.0).contains(&length)
        });
        let has_supported_pad_string = expr.args.get(1).is_none_or(|argument| {
            self.resolve_static_string_expression(argument)
                .is_some_and(|padding| padding.is_ascii())
        });

        if matches!(expr.args.len(), 1 | 2)
            && has_ascii_source
            && has_supported_target_length
            && has_supported_pad_string
        {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "String.prototype.{method} is unavailable unless the receiver is a statically-known ASCII string literal, the target length is a statically-known integer from 0 through 1024, and the optional pad string is statically-known ASCII in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
            ),
        ));
    }
    pub(crate) fn resolve_string_at_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "at" {
            return;
        }
        if matches!(member.object, Expression::ArrayExpression(_)) {
            return;
        }

        let source = match self.resolve_static_object_identity_literal_value(&member.object) {
            Some(StaticObjectIdentityValue::String(value)) => Some(value),
            _ => None,
        };
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let supported_arg_count = matches!(expr.args.len(), 0 | 1);
        let has_static_integer_index = expr.args.first().is_none_or(|argument| {
            self.resolve_static_numeric_literal_value(argument)
                .is_some_and(|index| index.is_finite() && index.fract() == 0.0)
        });

        if supported_arg_count && has_ascii_source && has_static_integer_index {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.at is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_string_char_at_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "charAt" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let supported_arg_count = matches!(expr.args.len(), 0 | 1);
        let has_static_integer_index = expr.args.first().is_none_or(|argument| {
            self.resolve_static_numeric_literal_value(argument)
                .is_some_and(|index| index.is_finite() && index.fract() == 0.0)
        });

        if supported_arg_count && has_ascii_source && has_static_integer_index {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.charAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_string_char_code_at_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "charCodeAt" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let supported_arg_count = matches!(expr.args.len(), 0 | 1);
        let has_static_integer_index = expr.args.first().is_none_or(|argument| {
            self.resolve_static_numeric_literal_value(argument)
                .is_some_and(|index| index.is_finite() && index.fract() == 0.0)
        });

        if supported_arg_count && has_ascii_source && has_static_integer_index {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.charCodeAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_string_code_point_at_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "codePointAt" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        let supported_arg_count = matches!(expr.args.len(), 0 | 1);
        let has_static_integer_index = expr.args.first().is_none_or(|argument| {
            self.resolve_static_numeric_literal_value(argument)
                .is_some_and(|index| index.is_finite() && index.fract() == 0.0)
        });

        if supported_arg_count && has_ascii_source && has_static_integer_index {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.codePointAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_string_trim_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        let method = member.property.as_str();
        if !matches!(
            method,
            "trim" | "trimStart" | "trimEnd" | "trimLeft" | "trimRight"
        ) {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        if source.is_none() {
            return;
        }

        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        if expr.args.is_empty() && has_ascii_source {
            self.resolve_expression(&member.object);
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "String.prototype.{method} is unavailable unless the receiver is a statically-known ASCII string literal and no arguments are supplied in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
            ),
        ));
    }
    pub(crate) fn resolve_string_case_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        let method = member.property.as_str();
        if !matches!(
            method,
            "toLowerCase" | "toUpperCase" | "toLocaleLowerCase" | "toLocaleUpperCase"
        ) {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let has_ascii_source = source.as_ref().is_some_and(|source| source.is_ascii());
        if expr.args.is_empty() && has_ascii_source {
            self.resolve_expression(&member.object);
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "String.prototype.{method} is unavailable unless the receiver is a statically-known ASCII string literal and no arguments are supplied in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
            ),
        ));
    }
    pub(crate) fn resolve_string_normalize_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "normalize" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let form = expr
            .args
            .first()
            .map(|argument| self.resolve_static_string_expression(argument))
            .unwrap_or_else(|| Some("NFC".to_string()));
        let has_supported_form = form
            .as_ref()
            .is_some_and(|form| matches!(form.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD"));

        if matches!(expr.args.len(), 0 | 1)
            && source.as_ref().is_some_and(|source| source.is_ascii())
            && has_supported_form
        {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.normalize is unavailable unless the receiver is a statically-known ASCII string literal and the optional normalization form is one of the statically-known strings NFC, NFD, NFKC, or NFKD in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_string_replace_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        let method = member.property.as_str();
        if !matches!(method, "replace" | "replaceAll") {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let search = expr
            .args
            .first()
            .and_then(|argument| self.resolve_static_string_expression(argument));
        let replacement = expr
            .args
            .get(1)
            .and_then(|argument| self.resolve_static_string_expression(argument));
        let has_ascii_operands = source
            .as_ref()
            .zip(search.as_ref())
            .zip(replacement.as_ref())
            .is_some_and(|((source, search), replacement)| {
                source.is_ascii()
                    && search.is_ascii()
                    && replacement.is_ascii()
                    && !replacement.contains('$')
            });

        if expr.args.len() == 2 && has_ascii_operands {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "String.prototype.{method} is unavailable unless the receiver, search value, and replacement are statically-known ASCII string literals and the replacement contains no substitution markers in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
            ),
        ));
    }
    pub(crate) fn resolve_string_split_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "split" {
            return;
        }

        let source = self.resolve_static_string_expression(&member.object);
        let separator = expr
            .args
            .first()
            .and_then(|argument| self.resolve_static_string_expression(argument));
        let limit = expr
            .args
            .get(1)
            .and_then(|argument| self.resolve_static_numeric_literal_value(argument));
        let has_ascii_operands = source.as_ref().is_some_and(|source| {
            source.is_ascii()
                && expr.args.first().is_none_or(|_| {
                    separator
                        .as_ref()
                        .is_some_and(|separator| separator.is_ascii())
                })
        });
        let has_supported_limit = expr.args.get(1).is_none_or(|_| {
            limit.is_some_and(|limit| {
                limit.is_finite() && limit.fract() == 0.0 && (0.0..=1024.0).contains(&limit)
            })
        });

        if matches!(expr.args.len(), 0..=2) && has_ascii_operands && has_supported_limit {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "String.prototype.split is unavailable unless the receiver is a statically-known ASCII string literal, the optional separator is a statically-known ASCII string literal, and the optional limit is a statically-known integer from 0 through 1024 in the current direct-runtime path; use explicit ASCII literals or the later compatibility path".to_string(),
        ));
    }
}

#[cfg(test)]
#[path = "string_tests.rs"]
mod string_tests;

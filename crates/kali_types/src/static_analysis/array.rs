//! Array static-analysis helpers.
use crate::*;

impl TypeContext {
    pub(crate) fn unwrap_for_of_wrapper_expression<'a>(&self, expression: &'a Expression) -> &'a Expression {
        let mut current = expression;
        loop {
            current = match current {
                Expression::ParenthesizedExpression(expr) => &expr.expression,
                Expression::TypeAssertion(expr) => &expr.expression,
                Expression::SatisfiesExpression(expr) => &expr.expression,
                Expression::ChainExpression(expr) => &expr.expression,
                Expression::DecoratedExpression(expr) => &expr.expression,
                Expression::AwaitExpression(expr) => &expr.argument,
                Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                    OptionalChainInner::NonNull { object, .. } => object,
                },
                Expression::CallExpression(call)
                    if Self::is_object_freeze_call(call) && !call.args.is_empty() =>
                {
                    &call.args[0]
                }
                Expression::LogicalExpression(expr) => match expr.operator {
                    LogicalOperator::Coalesce => {
                        let Some(left) =
                            self.resolve_static_object_identity_literal_value(&expr.left)
                        else {
                            return current;
                        };
                        if left.is_nullish() {
                            &expr.right
                        } else {
                            &expr.left
                        }
                    }
                    LogicalOperator::And => {
                        let Some(left) =
                            self.resolve_static_object_identity_literal_value(&expr.left)
                        else {
                            return current;
                        };
                        match left.truthiness() {
                            Some(true) => &expr.right,
                            Some(false) => &expr.left,
                            None => return current,
                        }
                    }
                    LogicalOperator::Or => {
                        let Some(left) =
                            self.resolve_static_object_identity_literal_value(&expr.left)
                        else {
                            return current;
                        };
                        match left.truthiness() {
                            Some(true) => &expr.left,
                            Some(false) => &expr.right,
                            None => return current,
                        }
                    }
                },
                Expression::BinaryExpression(expr)
                    if matches!(expr.operator.as_str(), "??" | "&&" | "||") =>
                {
                    let Some(left) = self.resolve_static_object_identity_literal_value(&expr.left)
                    else {
                        return current;
                    };
                    match expr.operator.as_str() {
                        "??" => {
                            if left.is_nullish() {
                                &expr.right
                            } else {
                                &expr.left
                            }
                        }
                        "&&" => match left.truthiness() {
                            Some(true) => &expr.right,
                            Some(false) => &expr.left,
                            None => return current,
                        },
                        "||" => match left.truthiness() {
                            Some(true) => &expr.left,
                            Some(false) => &expr.right,
                            None => return current,
                        },
                        _ => unreachable!(),
                    }
                }
                Expression::ConditionalExpression(expr) => {
                    match self.resolve_static_object_identity_literal_value(&expr.test) {
                        Some(StaticObjectIdentityValue::Boolean(true)) => &expr.consequent,
                        Some(StaticObjectIdentityValue::Boolean(false)) => &expr.alternate,
                        _ => return current,
                    }
                }
                Expression::SequenceExpression(expr) => match expr.expressions.last() {
                    Some(expression) => expression,
                    None => return current,
                },
                _ => return current,
            };
        }
    }
    pub(crate) fn is_static_array_iteration_target(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrayExpression(array) => {
                array.elements.iter().all(|element| match element {
                    Some(ExpressionOrSpread::Expression(expr)) => {
                        self.is_static_array_iteration_element(expr)
                    }
                    Some(ExpressionOrSpread::Spread(spread)) => {
                        self.is_static_array_iteration_target(&spread.argument)
                    }
                    Some(ExpressionOrSpread::Empty) | None => false,
                })
            }
            Expression::Identifier(name) => {
                self.resolve_static_array_binding_name(name)
                    || self.resolve_static_string_binding(name).is_some()
            }
            Expression::CallExpression(call) => {
                self.is_static_object_enumeration_iteration_target(call)
                    || Self::is_object_freeze_call(call)
                        && call
                            .args
                            .first()
                            .is_some_and(|arg| self.is_static_array_iteration_target(arg))
                    || self.is_static_array_from_call(call)
                        && call.args.len() == 1
                        && self.is_static_array_iteration_target(&call.args[0])
                    || self.is_static_identity_array_map_call(call)
                    || self.is_static_identity_array_filter_call(call)
                    || self.is_static_predicate_array_filter_call(call)
                    || self.is_static_identity_array_flat_map_call(call)
            }
            Expression::NewExpression(expr) => {
                self.is_static_set_constructor_iteration_target(expr)
                    || self.is_static_map_constructor_iteration_target(expr)
            }
            other => self
                .resolve_static_string_iterable_expression(other)
                .is_some(),
        }
    }
    pub(crate) fn is_static_literal_array_receiver(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrayExpression(array) => {
                array.elements.iter().all(|element| match element {
                    Some(ExpressionOrSpread::Expression(expr)) => {
                        self.is_static_array_iteration_element(expr)
                    }
                    Some(ExpressionOrSpread::Spread(spread)) => {
                        self.is_static_literal_array_receiver(&spread.argument)
                    }
                    Some(ExpressionOrSpread::Empty) | None => false,
                })
            }
            Expression::Identifier(name) => self.resolve_static_array_binding_name(name),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .is_some_and(|arg| self.is_static_literal_array_receiver(arg)),
            _ => false,
        }
    }
    pub(crate) fn is_static_truthy_array_literal(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrayExpression(array) => {
                array.elements.iter().all(|element| match element {
                    Some(ExpressionOrSpread::Expression(expr)) => {
                        self.resolve_static_object_identity_literal_value(expr)
                            .and_then(|value| value.truthiness())
                            == Some(true)
                    }
                    Some(ExpressionOrSpread::Spread(spread)) => {
                        self.is_static_truthy_array_literal(&spread.argument)
                    }
                    Some(ExpressionOrSpread::Empty) | None => false,
                })
            }
            _ => false,
        }
    }
    pub(crate) fn is_static_non_empty_numeric_array_iteration_target(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrayExpression(array) => {
                !array.elements.is_empty()
                    && array.elements.iter().all(|element| match element {
                        Some(ExpressionOrSpread::Expression(expr)) => {
                            self.is_static_numeric_literal_expr(expr)
                        }
                        Some(ExpressionOrSpread::Spread(_))
                        | Some(ExpressionOrSpread::Empty)
                        | None => false,
                    })
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().is_some_and(|argument| {
                    self.is_static_non_empty_numeric_array_iteration_target(argument)
                })
            }
            _ => false,
        }
    }
    pub(crate) fn is_static_identity_array_filter_call(&self, call: &CallExpression) -> bool {
        let Expression::MemberExpression(member) = &call.callee else {
            return false;
        };

        if member.property.as_str() != "filter" {
            return false;
        }

        call.args.len() == 1
            && self.is_static_truthy_array_literal(&member.object)
            && self.is_identity_array_callback(&call.args[0])
    }
    pub(crate) fn is_static_predicate_array_filter_call(&self, call: &CallExpression) -> bool {
        let Expression::MemberExpression(member) = &call.callee else {
            return false;
        };

        if member.property.as_str() != "filter" {
            return false;
        }

        call.args.len() == 1
            && self.is_static_array_iteration_target(&member.object)
            && self.is_some_every_array_callback(&call.args[0])
    }
    pub(crate) fn is_static_identity_array_flat_map_call(&self, call: &CallExpression) -> bool {
        let Expression::MemberExpression(member) = &call.callee else {
            return false;
        };

        if member.property.as_str() != "flatMap" {
            return false;
        }

        call.args.len() == 1
            && self.is_static_array_iteration_target(&member.object)
            && self.is_identity_array_flat_map_callback(&call.args[0])
    }
    pub(crate) fn is_static_array_from_call(&self, call: &CallExpression) -> bool {
        matches!(
            self.resolve_static_callable_name(&call.callee).as_deref(),
            Some(name) if kali_common::array_from_aliases().contains(&name)
        )
    }
    pub(crate) fn is_static_identity_array_map_call(&self, call: &CallExpression) -> bool {
        let Expression::MemberExpression(member) = &call.callee else {
            return false;
        };

        if member.property.as_str() != "map" {
            return false;
        }

        call.args.len() == 1
            && self.is_static_array_iteration_target(&member.object)
            && self.is_identity_array_callback(&call.args[0])
    }
    pub(crate) fn is_static_set_constructor_iteration_target(&self, expression: &NewExpression) -> bool {
        let (callee_expression, args) = match &expression.callee {
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                (call.args.first().unwrap_or(&call.callee), &expression.args)
            }
            Expression::CallExpression(call) if expression.args.is_empty() => {
                (&call.callee, &call.args)
            }
            Expression::CallExpression(call) => (&call.callee, &expression.args),
            other => (other, &expression.args),
        };

        let callee_name = self.resolve_static_callable_name(callee_expression);
        let Some(callee_name) = callee_name else {
            return false;
        };

        matches!(
            callee_name.as_str(),
            "Set" | "globalThis.Set" | r#"globalThis["Set"]"# | r#"globalThis['Set']"#
        ) && args.len() == 1
            && self.is_static_array_iteration_target(&args[0])
    }
    pub(crate) fn is_static_map_constructor_iteration_target(&self, expression: &NewExpression) -> bool {
        let (callee_expression, args) = match &expression.callee {
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                (call.args.first().unwrap_or(&call.callee), &expression.args)
            }
            Expression::CallExpression(call) if expression.args.is_empty() => {
                (&call.callee, &call.args)
            }
            Expression::CallExpression(call) => (&call.callee, &expression.args),
            other => (other, &expression.args),
        };

        let callee_name = self.resolve_static_callable_name(callee_expression);
        let Some(callee_name) = callee_name else {
            return false;
        };
        matches!(
            callee_name.as_str(),
            "Map" | "globalThis.Map" | r#"globalThis["Map"]"# | r#"globalThis['Map']"#
        ) && args.len() == 1
            && self.is_static_array_iteration_target(&args[0])
    }
    pub(crate) fn is_static_object_enumeration_iteration_target(&self, call: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&call.callee) else {
            return false;
        };
        if !matches!(
            callee_name.as_str(),
            "Object.keys"
                | "Object[\"keys\"]"
                | "Object['keys']"
                | "Object.values"
                | "Object[\"values\"]"
                | "Object['values']"
                | "Object.entries"
                | "Object[\"entries\"]"
                | "Object['entries']"
                | "globalThis.Object.keys"
                | "globalThis.Object[\"keys\"]"
                | "globalThis.Object['keys']"
                | "globalThis.Object.values"
                | "globalThis.Object[\"values\"]"
                | "globalThis.Object['values']"
                | "globalThis.Object.entries"
                | "globalThis.Object[\"entries\"]"
                | "globalThis.Object['entries']"
                | r#"globalThis["Object"].keys"#
                | r#"globalThis["Object"]["keys"]"#
                | r#"globalThis["Object"]['keys']"#
                | r#"globalThis['Object'].keys"#
                | r#"globalThis['Object']['keys']"#
                | r#"globalThis['Object']["keys"]"#
                | r#"globalThis["Object"].values"#
                | r#"globalThis["Object"]["values"]"#
                | r#"globalThis["Object"]['values']"#
                | r#"globalThis['Object'].values"#
                | r#"globalThis['Object']['values']"#
                | r#"globalThis['Object']["values"]"#
                | r#"globalThis["Object"].entries"#
                | r#"globalThis["Object"]["entries"]"#
                | r#"globalThis["Object"]['entries']"#
                | r#"globalThis['Object'].entries"#
                | r#"globalThis['Object']['entries']"#
                | r#"globalThis['Object']["entries"]"#
                | "Reflect.ownKeys"
                | "Reflect[\"ownKeys\"]"
                | "Reflect['ownKeys']"
                | "globalThis.Reflect.ownKeys"
                | "globalThis.Reflect[\"ownKeys\"]"
                | "globalThis.Reflect['ownKeys']"
                | r#"globalThis["Reflect"].ownKeys"#
                | r#"globalThis["Reflect"]["ownKeys"]"#
                | r#"globalThis['Reflect'].ownKeys"#
                | r#"globalThis['Reflect']['ownKeys']"#
        ) {
            return false;
        }

        let Some(object_arg) = call.args.first() else {
            return false;
        };
        if call.args.len() != 1 {
            return false;
        }

        self.resolve_static_object_keys_target(object_arg)
    }
    pub(crate) fn is_static_array_iteration_element(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::Literal(_) => true,
            Expression::Identifier(_) => {
                self.resolve_static_numeric_literal_value(expression)
                    .is_some()
                    || self.resolve_static_string_expression(expression).is_some()
                    || self
                        .resolve_static_object_identity_literal_value(expression)
                        .is_some()
            }
            Expression::ArrayExpression(array) => {
                array.elements.iter().all(|element| match element {
                    Some(ExpressionOrSpread::Expression(expr)) => {
                        self.is_static_array_iteration_element(expr)
                    }
                    Some(ExpressionOrSpread::Spread(spread)) => {
                        self.is_static_array_iteration_target(&spread.argument)
                    }
                    Some(ExpressionOrSpread::Empty) | None => false,
                })
            }
            _ => false,
        }
    }
    pub(crate) fn resolve_static_array_binding_name(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.static_arrays.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }

        self.global_scope.static_arrays.contains_key(name)
    }
    pub(crate) fn resolve_array_is_array_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "Array.isArray"
                | "globalThis.Array.isArray"
                | r#"Array["isArray"]"#
                | r#"Array['isArray']"#
                | r#"globalThis.Array["isArray"]"#
                | r#"globalThis.Array['isArray']"#
                | r#"globalThis["Array"].isArray"#
                | r#"globalThis['Array'].isArray"#
                | r#"globalThis["Array"]["isArray"]"#
                | r#"globalThis['Array']['isArray']"#
        ) {
            return false;
        }

        let Some(argument) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.isArray requires at least one statically-known argument in the current phase; use an explicit literal or the later compatibility path",
            ));
            return true;
        };

        if self
            .resolve_static_array_is_array_argument(argument)
            .is_some()
        {
            self.resolve_expression(argument);
            for arg in expr.args.iter().skip(1) {
                self.resolve_expression(arg);
            }
            return true;
        }

        self.resolve_expression(argument);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "Array.isArray is unavailable unless the argument is a statically-known array, object, or primitive literal in the current phase; use explicit literals or the later compatibility path",
        ));
        true
    }
    pub(crate) fn resolve_static_array_is_array_argument(&self, expression: &Expression) -> Option<bool> {
        let unwrapped = self.unwrap_for_of_wrapper_expression(expression);
        match unwrapped {
            Expression::ArrayExpression(_) => Some(true),
            Expression::ObjectExpression(_) => Some(false),
            Expression::Identifier(name) => {
                if self.resolve_static_array_binding_name(name) {
                    Some(true)
                } else if self.resolve_static_object_binding_name(name)
                    || self.resolve_static_string_binding(name).is_some()
                    || self.resolve_static_object_identity_binding(name).is_some()
                {
                    Some(false)
                } else {
                    None
                }
            }
            Expression::CallExpression(call) => {
                if Self::is_object_freeze_call(call) {
                    call.args
                        .first()
                        .and_then(|arg| self.resolve_static_array_is_array_argument(arg))
                } else if self.is_static_array_from_call(call)
                    || self.is_static_identity_array_map_call(call)
                    || self.is_static_identity_array_filter_call(call)
                    || self.is_static_predicate_array_filter_call(call)
                    || self.is_static_identity_array_flat_map_call(call)
                {
                    Some(true)
                } else if self.resolve_static_object_model_call_target(call) {
                    Some(false)
                } else {
                    None
                }
            }
            Expression::NewExpression(expr)
                if self.is_static_set_constructor_iteration_target(expr)
                    || self.is_static_map_constructor_iteration_target(expr) =>
            {
                Some(false)
            }
            _ => self
                .resolve_static_object_identity_literal_value(unwrapped)
                .map(|_| false),
        }
    }
    pub(crate) fn resolve_array_callback_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        let method = member.property.as_str();
        if !matches!(
            method,
            "find"
                | "findIndex"
                | "findLast"
                | "findLastIndex"
                | "map"
                | "filter"
                | "flatMap"
                | "some"
                | "every"
                | "reduce"
                | "reduceRight"
        ) {
            return;
        }

        if !self.is_static_array_iteration_target(&member.object) {
            return;
        }

        if method == "map"
            && expr
                .args
                .first()
                .is_some_and(|callback| self.is_identity_array_callback(callback))
        {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        if method == "filter"
            && ((expr
                .args
                .first()
                .is_some_and(|callback| self.is_identity_array_callback(callback))
                && self.is_static_truthy_array_literal(&member.object))
                || expr
                    .args
                    .first()
                    .is_some_and(|callback| self.is_some_every_array_callback(callback)))
        {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        if method == "flatMap"
            && expr
                .args
                .first()
                .is_some_and(|callback| self.is_identity_array_flat_map_callback(callback))
        {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        if matches!(method, "find" | "findIndex" | "findLast" | "findLastIndex")
            && expr
                .args
                .first()
                .is_some_and(|callback| self.is_some_every_array_callback(callback))
            && self.is_static_array_iteration_target(&member.object)
        {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        if matches!(method, "some" | "every")
            && expr.args.first().is_some_and(|callback| {
                self.is_identity_array_callback(callback)
                    || self.is_some_every_array_callback(callback)
            })
            && self.is_static_array_iteration_target(&member.object)
        {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        if matches!(method, "reduce" | "reduceRight")
            && ((expr.args.len() == 2
                && self.is_static_array_iteration_target(&member.object)
                && self.is_static_numeric_literal_expr(&expr.args[1]))
                || (expr.args.len() == 1
                    && self.is_static_non_empty_numeric_array_iteration_target(&member.object)))
            && self.is_numeric_reducer_callback(&expr.args[0])
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
                "array callback method '{method}' is unavailable in the current direct-runtime path; use a supported iterator slice or the later compatibility path"
            ),
        ));
    }
    pub(crate) fn resolve_array_slice_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "slice" {
            return;
        }

        if self
            .resolve_static_string_expression(&member.object)
            .is_some()
        {
            return;
        }

        if Self::is_runtime_args_slice_member(member) {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }

        let has_static_receiver = self.is_static_array_iteration_target(&member.object);
        let supported_arg_count = expr.args.len() <= 2;
        let has_static_bounds = expr.args.iter().all(|argument| {
            self.resolve_static_numeric_literal_value(argument)
                .is_some_and(|value| value.is_finite())
        });

        if has_static_receiver && supported_arg_count && has_static_bounds {
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
            "Array.prototype.slice is unavailable unless the receiver is a statically-known array literal and the optional start/end bounds are statically-known finite numeric literals in the current direct-runtime path; use explicit literals or the later compatibility path",
        ));
    }
    pub(crate) fn resolve_array_concat_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "concat" {
            return;
        }

        if self
            .resolve_static_string_expression(&member.object)
            .is_some()
        {
            return;
        }

        if !self.is_static_array_concat_receiver(&member.object)
            && !expr
                .args
                .iter()
                .any(|argument| self.is_static_array_concat_receiver(argument))
        {
            return;
        }

        let has_static_receiver = self.is_static_array_concat_receiver(&member.object);
        let has_static_operands = expr.args.iter().all(|argument| {
            self.is_static_array_concat_receiver(argument)
                || self
                    .resolve_static_object_identity_literal_value(argument)
                    .is_some_and(|value| !matches!(value, StaticObjectIdentityValue::Reference(_)))
        });

        if has_static_receiver && has_static_operands {
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
            "Array.prototype.concat is unavailable unless the receiver is a statically-known array literal and each argument is a statically-known array or primitive literal in the current direct-runtime path; use explicit literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_array_at_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "at" {
            return;
        }

        if matches!(
            self.resolve_static_object_identity_literal_value(&member.object),
            Some(StaticObjectIdentityValue::String(_))
        ) {
            return;
        }

        let has_static_receiver = self.is_static_array_iteration_target(&member.object);
        let static_index = expr
            .args
            .first()
            .and_then(|argument| self.resolve_static_numeric_literal_value(argument));

        if has_static_receiver && expr.args.len() == 1 && static_index.is_some() {
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
            "Array.prototype.at is unavailable unless the receiver is a statically-known array literal and the index is a statically-known integer in the current direct-runtime path; use explicit literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_array_join_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "join" {
            return;
        }

        let has_static_receiver = self.is_static_array_iteration_target(&member.object);
        if !has_static_receiver {
            return;
        }

        let supported_arg_count = matches!(expr.args.len(), 0 | 1);
        let has_static_separator = expr
            .args
            .first()
            .is_none_or(|argument| self.resolve_static_string_expression(argument).is_some());

        if supported_arg_count && has_static_separator {
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
            "Array.prototype.join is unavailable for static literal-array receivers unless the optional separator is a statically-known string in the current direct-runtime path; use explicit literals or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_array_to_string_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "toString" {
            return;
        }

        let has_static_receiver = self.is_static_literal_array_receiver(&member.object);
        if !has_static_receiver {
            return;
        }

        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }

        if expr.args.is_empty() {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "Array.prototype.toString is unavailable for static literal-array receivers when arguments are supplied in the current direct-runtime path; use a no-argument call or the later compatibility path".to_string(),
        ));
    }
    pub(crate) fn resolve_array_search_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        let method = member.property.as_str();
        if !matches!(method, "includes" | "indexOf" | "lastIndexOf") {
            return;
        }

        if self
            .resolve_static_string_expression(&member.object)
            .is_some()
        {
            return;
        }
        let has_static_receiver = self.is_static_array_iteration_target(&member.object);
        let supported_arg_count = matches!(expr.args.len(), 1 | 2);
        let has_static_search_value = expr
            .args
            .first()
            .and_then(|argument| self.resolve_static_object_identity_literal_value(argument))
            .is_some();
        let has_static_from_index = expr
            .args
            .get(1)
            .is_none_or(|argument| self.is_static_numeric_literal_expr(argument));

        if has_static_receiver
            && supported_arg_count
            && has_static_search_value
            && has_static_from_index
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
                "array search method '{method}' is unavailable unless the receiver, search value, and fromIndex are statically known in the current direct-runtime path; use explicit literals or the later compatibility path"
            ),
        ));
    }
    pub(crate) fn is_static_array_concat_receiver(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrayExpression(_) => true,
            Expression::Identifier(name) => self.resolve_static_array_binding_name(name),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .is_some_and(|arg| self.is_static_array_concat_receiver(arg)),
            _ => false,
        }
    }
    pub(crate) fn is_identity_array_callback(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrowFunctionExpression(arrow) => {
                arrow.params.len() == 1
                    && self
                        .is_identity_array_callback_expression(&arrow.body, &arrow.params[0].name)
            }
            Expression::FunctionExpression(function) => {
                let Some(body) = &function.body else {
                    return false;
                };
                function.params.len() == 1
                    && body.body.len() == 1
                    && matches!(
                        &body.body[0],
                        Statement::ReturnStatement(ReturnStatement {
                            argument: Some(argument),
                        }) if self.is_identity_array_callback_expression(argument, &function.params[0].name)
                    )
            }
            _ => false,
        }
    }
    pub(crate) fn is_identity_array_callback_expression(
        &self,
        expression: &Expression,
        param_name: &str,
    ) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::Identifier(name) => name == param_name,
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|last| self.is_identity_array_callback_expression(last, param_name)),
            _ => false,
        }
    }
    pub(crate) fn is_some_every_array_callback(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrowFunctionExpression(arrow) => {
                arrow.params.len() == 1
                    && self
                        .is_some_every_array_callback_expression(&arrow.body, &arrow.params[0].name)
            }
            Expression::FunctionExpression(function) => {
                let Some(body) = &function.body else {
                    return false;
                };
                function.params.len() == 1
                    && body.body.len() == 1
                    && matches!(
                        &body.body[0],
                        Statement::ReturnStatement(ReturnStatement {
                            argument: Some(argument),
                        }) if self.is_some_every_array_callback_expression(
                            argument,
                            &function.params[0].name,
                        )
                    )
            }
            _ => false,
        }
    }
    pub(crate) fn is_some_every_array_callback_expression(
        &self,
        expression: &Expression,
        param_name: &str,
    ) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::Identifier(name) => name == param_name,
            Expression::Literal(LiteralValue::Boolean(_))
            | Expression::Literal(LiteralValue::Number(_))
            | Expression::Literal(LiteralValue::String(_))
            | Expression::Literal(LiteralValue::Null) => true,
            Expression::UnaryExpression(expr)
                if matches!(expr.operator.as_str(), "!" | "+" | "-") =>
            {
                self.is_some_every_array_callback_expression(&expr.argument, param_name)
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "===" | "!==") =>
            {
                (self.is_some_every_array_callback_identity_operand(&expr.left, param_name)
                    && self
                        .resolve_static_object_identity_literal_value(&expr.right)
                        .is_some())
                    || (self
                        .resolve_static_object_identity_literal_value(&expr.left)
                        .is_some()
                        && self
                            .is_some_every_array_callback_identity_operand(&expr.right, param_name))
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), ">" | ">=" | "<" | "<=") =>
            {
                (self.is_some_every_array_callback_operand(&expr.left, param_name)
                    && self
                        .resolve_static_numeric_literal_value(&expr.right)
                        .is_some())
                    || (self
                        .resolve_static_numeric_literal_value(&expr.left)
                        .is_some()
                        && self.is_some_every_array_callback_operand(&expr.right, param_name))
            }
            Expression::LogicalExpression(expr)
                if matches!(expr.operator, LogicalOperator::And | LogicalOperator::Or) =>
            {
                self.is_some_every_array_callback_expression(&expr.left, param_name)
                    && self.is_some_every_array_callback_expression(&expr.right, param_name)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|last| self.is_some_every_array_callback_expression(last, param_name)),
            Expression::ParenthesizedExpression(expr) => {
                self.is_some_every_array_callback_expression(&expr.expression, param_name)
            }
            Expression::TypeAssertion(expr) => {
                self.is_some_every_array_callback_expression(&expr.expression, param_name)
            }
            Expression::SatisfiesExpression(expr) => {
                self.is_some_every_array_callback_expression(&expr.expression, param_name)
            }
            Expression::AwaitExpression(expr) => {
                self.is_some_every_array_callback_expression(&expr.argument, param_name)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.is_some_every_array_callback_expression(object, param_name)
                }
            },
            Expression::ChainExpression(expr) => {
                self.is_some_every_array_callback_expression(&expr.expression, param_name)
            }
            Expression::DecoratedExpression(expr) => {
                self.is_some_every_array_callback_expression(&expr.expression, param_name)
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().is_some_and(|argument| {
                    self.is_some_every_array_callback_expression(argument, param_name)
                })
            }
            _ => false,
        }
    }
    pub(crate) fn is_some_every_array_callback_identity_operand(
        &self,
        expression: &Expression,
        param_name: &str,
    ) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::Identifier(name) => name == param_name,
            Expression::SequenceExpression(expr) => expr.expressions.last().is_some_and(|last| {
                self.is_some_every_array_callback_identity_operand(last, param_name)
            }),
            Expression::ParenthesizedExpression(expr) => {
                self.is_some_every_array_callback_identity_operand(&expr.expression, param_name)
            }
            Expression::TypeAssertion(expr) => {
                self.is_some_every_array_callback_identity_operand(&expr.expression, param_name)
            }
            Expression::SatisfiesExpression(expr) => {
                self.is_some_every_array_callback_identity_operand(&expr.expression, param_name)
            }
            Expression::AwaitExpression(expr) => {
                self.is_some_every_array_callback_identity_operand(&expr.argument, param_name)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.is_some_every_array_callback_identity_operand(object, param_name)
                }
            },
            Expression::ChainExpression(expr) => {
                self.is_some_every_array_callback_identity_operand(&expr.expression, param_name)
            }
            Expression::DecoratedExpression(expr) => {
                self.is_some_every_array_callback_identity_operand(&expr.expression, param_name)
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().is_some_and(|argument| {
                    self.is_some_every_array_callback_identity_operand(argument, param_name)
                })
            }
            _ => self
                .resolve_static_object_identity_literal_value(expression)
                .is_some(),
        }
    }
    pub(crate) fn is_some_every_array_callback_operand(
        &self,
        expression: &Expression,
        param_name: &str,
    ) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::Identifier(name) => name == param_name,
            Expression::UnaryExpression(expr) if matches!(expr.operator.as_str(), "+" | "-") => {
                self.is_some_every_array_callback_operand(&expr.argument, param_name)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|last| self.is_some_every_array_callback_operand(last, param_name)),
            Expression::ParenthesizedExpression(expr) => {
                self.is_some_every_array_callback_operand(&expr.expression, param_name)
            }
            Expression::TypeAssertion(expr) => {
                self.is_some_every_array_callback_operand(&expr.expression, param_name)
            }
            Expression::SatisfiesExpression(expr) => {
                self.is_some_every_array_callback_operand(&expr.expression, param_name)
            }
            Expression::AwaitExpression(expr) => {
                self.is_some_every_array_callback_operand(&expr.argument, param_name)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.is_some_every_array_callback_operand(object, param_name)
                }
            },
            Expression::ChainExpression(expr) => {
                self.is_some_every_array_callback_operand(&expr.expression, param_name)
            }
            Expression::DecoratedExpression(expr) => {
                self.is_some_every_array_callback_operand(&expr.expression, param_name)
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().is_some_and(|argument| {
                    self.is_some_every_array_callback_operand(argument, param_name)
                })
            }
            _ => self
                .resolve_static_numeric_literal_value(expression)
                .is_some(),
        }
    }
    pub(crate) fn is_numeric_reducer_callback(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrowFunctionExpression(arrow) => {
                arrow.params.len() >= 2
                    && self.is_numeric_reducer_callback_expression(
                        &arrow.body,
                        &arrow.params[0].name,
                        &arrow.params[1].name,
                    )
            }
            Expression::FunctionExpression(function) => {
                let Some(body) = &function.body else {
                    return false;
                };
                function.params.len() >= 2
                    && body.body.len() == 1
                    && matches!(
                        &body.body[0],
                        Statement::ReturnStatement(ReturnStatement {
                            argument: Some(argument),
                        }) if self.is_numeric_reducer_callback_expression(
                            argument,
                            &function.params[0].name,
                            &function.params[1].name,
                        )
                    )
            }
            _ => false,
        }
    }
    pub(crate) fn is_numeric_reducer_callback_expression(
        &self,
        expression: &Expression,
        accumulator_name: &str,
        value_name: &str,
    ) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::Identifier(name) => name == accumulator_name || name == value_name,
            Expression::Literal(LiteralValue::Number(_)) => true,
            Expression::UnaryExpression(expr) if matches!(expr.operator.as_str(), "+" | "-") => {
                self.is_numeric_reducer_callback_expression(
                    &expr.argument,
                    accumulator_name,
                    value_name,
                )
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "+" | "-" | "*") =>
            {
                self.is_numeric_reducer_callback_expression(
                    &expr.left,
                    accumulator_name,
                    value_name,
                ) && self.is_numeric_reducer_callback_expression(
                    &expr.right,
                    accumulator_name,
                    value_name,
                )
            }
            Expression::SequenceExpression(expr) => expr.expressions.last().is_some_and(|last| {
                self.is_numeric_reducer_callback_expression(last, accumulator_name, value_name)
            }),
            Expression::ParenthesizedExpression(expr) => self
                .is_numeric_reducer_callback_expression(
                    &expr.expression,
                    accumulator_name,
                    value_name,
                ),
            Expression::TypeAssertion(expr) => self.is_numeric_reducer_callback_expression(
                &expr.expression,
                accumulator_name,
                value_name,
            ),
            Expression::SatisfiesExpression(expr) => self.is_numeric_reducer_callback_expression(
                &expr.expression,
                accumulator_name,
                value_name,
            ),
            Expression::DecoratedExpression(expr) => self.is_numeric_reducer_callback_expression(
                &expr.expression,
                accumulator_name,
                value_name,
            ),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().is_some_and(|argument| {
                    self.is_numeric_reducer_callback_expression(
                        argument,
                        accumulator_name,
                        value_name,
                    )
                })
            }
            _ => self
                .resolve_static_numeric_literal_value(expression)
                .is_some(),
        }
    }
    pub(crate) fn is_static_numeric_literal_expr(&self, expression: &Expression) -> bool {
        self.resolve_static_numeric_literal_value(expression)
            .is_some()
    }
    pub(crate) fn is_identity_array_flat_map_callback(&self, expression: &Expression) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrowFunctionExpression(arrow) => {
                arrow.params.len() == 1
                    && self.is_identity_array_flat_map_callback_expression(
                        &arrow.body,
                        &arrow.params[0].name,
                    )
            }
            Expression::FunctionExpression(function) => {
                let Some(body) = &function.body else {
                    return false;
                };
                function.params.len() == 1
                    && body.body.len() == 1
                    && matches!(
                        &body.body[0],
                        Statement::ReturnStatement(ReturnStatement {
                            argument: Some(argument),
                        }) if self.is_identity_array_flat_map_callback_expression(
                            argument,
                            &function.params[0].name,
                        )
                    )
            }
            _ => false,
        }
    }
    pub(crate) fn is_identity_array_flat_map_callback_expression(
        &self,
        expression: &Expression,
        param_name: &str,
    ) -> bool {
        match self.unwrap_for_of_wrapper_expression(expression) {
            Expression::ArrayExpression(array) => {
                array.elements.len() == 1
                    && matches!(
                        &array.elements[0],
                        Some(ExpressionOrSpread::Expression(expr))
                            if self.is_identity_array_callback_expression(expr, param_name)
                    )
            }
            Expression::SequenceExpression(expr) => expr.expressions.last().is_some_and(|last| {
                self.is_identity_array_flat_map_callback_expression(last, param_name)
            }),
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "array_tests.rs"]
mod array_tests;

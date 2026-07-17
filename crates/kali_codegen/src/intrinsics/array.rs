//! Array intrinsic call recognition, iteration, and constant-folding.
use crate::*;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn is_array_literal(&self, node: &LirNode) -> bool {
        node.kind == LirNodeKind::Value && node.text.is_none() && !self.is_object_literal(node)
    }

    pub(crate) fn is_truthy_array_literal(&self, node: &LirNode) -> bool {
        self.is_array_literal(node)
            && node.children.iter().all(|child| {
                self.resolve_static_object_identity_value(*child)
                    .and_then(|value| value.truthiness())
                    == Some(true)
            })
    }

    pub(crate) fn is_array_object(&self, callee_node: &LirNode) -> bool {
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Array")
                | Some("globalThis.Array")
                | Some(r#"globalThis["Array"]"#)
                | Some(r#"globalThis['Array']"#)
        )
    }

    pub(crate) fn is_array_is_array_call(&self, callee_node: &LirNode) -> bool {
        let Some(text) = callee_node.text.as_deref() else {
            return false;
        };
        (text == "isArray" || text.ends_with(r#"["isArray"]"#) || text.ends_with(r#"['isArray']"#))
            && self.is_array_object(callee_node)
    }

    pub(crate) fn static_array_is_array_result(&self, id: LirNodeId) -> Option<bool> {
        let node = self.node(id);
        if self.resolve_set_constructor_call(node).is_some()
            || self.resolve_map_constructor_call(node).is_some()
        {
            return Some(false);
        }

        if let Some(aggregate_id) = self.resolve_literal_aggregate(id) {
            let aggregate = self.node(aggregate_id);
            if self.is_array_literal(aggregate) {
                return Some(true);
            }
            if self.is_object_literal(aggregate) {
                return Some(false);
            }
        }

        self.resolve_static_object_identity_value(id).map(|_| false)
    }

    pub(crate) fn is_array_from_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let callee_node = self.node(callee);
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let object = self.resolve_literal_aggregate(object).unwrap_or(object);

        matches!(
            callee_node.text.as_deref(),
            Some(text)
                if text == "from"
                    || text.ends_with(".from")
                    || text.ends_with(r#"["from"]"#)
                    || text.ends_with(r#"['from']"#)
        ) && matches!(
            self.node(object).text.as_deref(),
            Some("Array")
                | Some("globalThis.Array")
                | Some(r#"globalThis["Array"]"#)
                | Some(r#"globalThis['Array']"#)
        )
    }

    pub(crate) fn is_array_callback_iteration_call(&self, node: &LirNode) -> bool {
        let mut current = node;
        while current.kind == LirNodeKind::Value
            && current.children.len() == 1
            && current
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
        {
            let Some(child) = current.children.first().copied() else {
                return false;
            };
            current = self.node(child);
        }

        if current.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = current.children.first().copied() else {
            return false;
        };
        let Some(raw_callee_text) = self.node(callee).text.as_deref() else {
            return false;
        };

        let matches_callback_iterable_method = |callee_text: &str| {
            [
                "map",
                "filter",
                "find",
                "findLast",
                "findIndex",
                "findLastIndex",
                "flatMap",
                "some",
                "every",
                "reduce",
                "reduceRight",
            ]
            .iter()
            .any(|method| {
                callee_text == *method
                    || callee_text.ends_with(&format!(".{method}"))
                    || callee_text.ends_with(&format!("[\"{method}\"]"))
                    || callee_text.ends_with(&format!("['{method}']"))
            })
        };

        if matches_callback_iterable_method(raw_callee_text) {
            return true;
        }

        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let Some(callee_text) = self.node(callee).text.as_deref() else {
            return false;
        };

        matches_callback_iterable_method(callee_text)
    }

    pub(crate) fn is_identity_array_map_callback(&self, id: LirNodeId) -> bool {
        let id = self.resolve_transparent_callable_node(id).unwrap_or(id);
        let node = self.node(id);
        if node.function_flavor != Some(FunctionFlavor::Sync)
            || node.kind != LirNodeKind::Instruction
            || node.children.len() < 2
        {
            return false;
        }

        let Some(param_name) = self.node(node.children[0]).text.as_deref() else {
            return false;
        };
        let Some(body_expr) = self.node(node.children[1]).children.first().copied() else {
            return false;
        };
        let Some(body_expr) = self.resolve_transparent_callable_node(body_expr) else {
            return false;
        };

        self.node(body_expr).text.as_deref() == Some(param_name)
    }

    pub(crate) fn is_identity_array_flat_map_callback(&self, id: LirNodeId) -> bool {
        let id = self.resolve_transparent_callable_node(id).unwrap_or(id);
        let node = self.node(id);
        if node.function_flavor != Some(FunctionFlavor::Sync)
            || node.kind != LirNodeKind::Instruction
            || node.children.len() < 2
        {
            return false;
        }

        let Some(param_name) = self.node(node.children[0]).text.as_deref() else {
            return false;
        };
        let Some(body_expr) = self.node(node.children[1]).children.first().copied() else {
            return false;
        };

        self.is_identity_array_flat_map_expression(body_expr, param_name)
    }

    pub(crate) fn is_identity_array_flat_map_expression(
        &self,
        id: LirNodeId,
        param_name: &str,
    ) -> bool {
        let node = self.node(id);
        if node.kind != LirNodeKind::Value
            || node.text.as_deref().is_some_and(|text| !text.is_empty())
            || node.children.len() != 1
        {
            return false;
        }

        let child = node.children[0];
        if self.node(child).text.as_deref() == Some(param_name) {
            return true;
        }

        self.is_identity_array_flat_map_expression(child, param_name)
    }

    pub(crate) fn resolve_identity_array_callback_source(
        &self,
        node: &LirNode,
    ) -> Option<LirNodeId> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        match callee_node.text.as_deref() {
            Some("map") => Some(callee_node.children.first().copied()?),
            Some("flatMap") => {
                let callback = node.children.get(1).copied()?;
                let callback = self.resolve_transparent_callable_node(callback)?;
                if self.is_identity_array_flat_map_callback(callback) {
                    Some(callee_node.children.first().copied()?)
                } else {
                    None
                }
            }
            Some("filter") => self.resolve_truthy_identity_array_filter_source(node),
            _ => None,
        }
    }

    pub(crate) fn resolve_static_array_callback_truthiness(
        &self,
        callback: LirNodeId,
        value: LirNodeId,
    ) -> Option<bool> {
        let callback = self.resolve_transparent_callable_node(callback)?;
        let callback_node = self.node(callback);
        if callback_node.function_flavor != Some(FunctionFlavor::Sync)
            || callback_node.kind != LirNodeKind::Instruction
            || callback_node.children.len() < 2
        {
            return None;
        }

        let param_name = self.node(callback_node.children[0]).text.as_deref()?;
        let body_node = self.node(callback_node.children[1]);
        let body_expr = body_node.children.first().copied()?;
        self.resolve_static_array_callback_truthiness_expr(body_expr, param_name, value)
    }

    pub(crate) fn resolve_static_array_callback_truthiness_expr(
        &self,
        id: LirNodeId,
        param_name: &str,
        value: LirNodeId,
    ) -> Option<bool> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
        {
            return self.resolve_static_array_callback_truthiness_expr(
                node.children[0],
                param_name,
                value,
            );
        }

        match node.kind {
            LirNodeKind::Literal => self
                .resolve_static_object_identity_value(id)
                .and_then(|value| value.truthiness()),
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if text == param_name {
                    return self
                        .resolve_static_object_identity_value(value)
                        .and_then(|value| value.truthiness());
                }

                self.resolve_static_object_identity_value(id)
                    .and_then(|value| value.truthiness())
            }
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                Some("!") => self
                    .resolve_static_array_callback_truthiness_expr(
                        node.children[0],
                        param_name,
                        value,
                    )
                    .map(|truthy| !truthy),
                Some("+") => self.resolve_static_array_callback_truthiness_expr(
                    node.children[0],
                    param_name,
                    value,
                ),
                Some("-") => self
                    .resolve_static_array_callback_numeric_operand(
                        node.children[0],
                        param_name,
                        value,
                    )
                    .map(|number| !number.is_nan() && number != 0.0),
                _ => None,
            },
            LirNodeKind::Value if node.children.len() == 2 => match node.text.as_deref() {
                Some("&&") => {
                    let left = self.resolve_static_array_callback_truthiness_expr(
                        node.children[0],
                        param_name,
                        value,
                    )?;
                    if left {
                        self.resolve_static_array_callback_truthiness_expr(
                            node.children[1],
                            param_name,
                            value,
                        )
                    } else {
                        Some(false)
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_array_callback_truthiness_expr(
                        node.children[0],
                        param_name,
                        value,
                    )?;
                    if left {
                        Some(true)
                    } else {
                        self.resolve_static_array_callback_truthiness_expr(
                            node.children[1],
                            param_name,
                            value,
                        )
                    }
                }
                Some("===") | Some("!==") => {
                    let left = self.resolve_static_array_callback_identity_operand(
                        node.children[0],
                        param_name,
                        value,
                    )?;
                    let right = self.resolve_static_array_callback_identity_operand(
                        node.children[1],
                        param_name,
                        value,
                    )?;
                    let strict_eq = left.strict_eq(&right);
                    Some(if node.text.as_deref() == Some("===") {
                        strict_eq
                    } else {
                        !strict_eq
                    })
                }
                Some(">") | Some(">=") | Some("<") | Some("<=") => {
                    let left = self.resolve_static_array_callback_numeric_operand(
                        node.children[0],
                        param_name,
                        value,
                    )?;
                    let right = self.resolve_static_array_callback_numeric_operand(
                        node.children[1],
                        param_name,
                        value,
                    )?;
                    Some(match node.text.as_deref() {
                        Some(">") => left > right,
                        Some(">=") => left >= right,
                        Some("<") => left < right,
                        Some("<=") => left <= right,
                        _ => unreachable!(),
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn resolve_static_array_callback_identity_operand(
        &self,
        id: LirNodeId,
        param_name: &str,
        value: LirNodeId,
    ) -> Option<StaticObjectIdentityValue> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
        {
            return self.resolve_static_array_callback_identity_operand(
                node.children[0],
                param_name,
                value,
            );
        }

        if node.kind == LirNodeKind::Value
            && node.children.is_empty()
            && node.text.as_deref() == Some(param_name)
        {
            return self.resolve_static_object_identity_value(value);
        }

        self.resolve_static_object_identity_value(id)
    }

    pub(crate) fn resolve_static_array_callback_numeric_operand(
        &self,
        id: LirNodeId,
        param_name: &str,
        value: LirNodeId,
    ) -> Option<f64> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.is_empty()
            && node.text.as_deref() == Some(param_name)
        {
            return self.resolve_static_numeric_value(value);
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 1 {
            match node.text.as_deref() {
                Some("+") => {
                    return self.resolve_static_array_callback_numeric_operand(
                        node.children[0],
                        param_name,
                        value,
                    );
                }
                Some("-") => {
                    return self
                        .resolve_static_array_callback_numeric_operand(
                            node.children[0],
                            param_name,
                            value,
                        )
                        .map(|number| -number);
                }
                None | Some("") | Some("await") => {
                    return self.resolve_static_array_callback_numeric_operand(
                        node.children[0],
                        param_name,
                        value,
                    );
                }
                _ => {}
            }
        }

        self.resolve_static_numeric_value(id)
    }

    pub(crate) fn resolve_static_array_some_every_call(
        &self,
        node: &LirNode,
        method: &str,
    ) -> Option<bool> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some(method) {
            return None;
        }

        let callback = node.children.get(1).copied()?;
        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0])
        } else {
            source_node
        };
        if !self.is_array_literal(source_node) {
            return None;
        }

        match method {
            "some" => {
                for child in &source_node.children {
                    if self.resolve_static_array_callback_truthiness(callback, *child)? {
                        return Some(true);
                    }
                }
                Some(false)
            }
            "every" => {
                for child in &source_node.children {
                    if !self.resolve_static_array_callback_truthiness(callback, *child)? {
                        return Some(false);
                    }
                }
                Some(true)
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_static_array_find_call(
        &mut self,
        node: &LirNode,
        method: &str,
    ) -> Option<StaticArraySearchResult> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some(method) {
            return None;
        }

        let callback = node.children.get(1).copied()?;
        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source).clone();
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0]).clone()
        } else {
            source_node
        };
        if !self.is_array_literal(&source_node) {
            return None;
        }

        match method {
            "find" => {
                for child in &source_node.children {
                    if self.resolve_static_array_callback_truthiness(callback, *child)? {
                        return Some(StaticArraySearchResult::Value(*child));
                    }
                }
                Some(StaticArraySearchResult::Value(self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some("undefined".to_string()),
                    vec![],
                )))
            }
            "findIndex" => {
                for (index, child) in source_node.children.iter().enumerate() {
                    if self.resolve_static_array_callback_truthiness(callback, *child)? {
                        return Some(StaticArraySearchResult::Index(index as i64));
                    }
                }
                Some(StaticArraySearchResult::Index(-1))
            }
            "findLast" => {
                for child in source_node.children.iter().rev() {
                    if self.resolve_static_array_callback_truthiness(callback, *child)? {
                        return Some(StaticArraySearchResult::Value(*child));
                    }
                }
                Some(StaticArraySearchResult::Value(self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some("undefined".to_string()),
                    vec![],
                )))
            }
            "findLastIndex" => {
                for (index, child) in source_node.children.iter().enumerate().rev() {
                    if self.resolve_static_array_callback_truthiness(callback, *child)? {
                        return Some(StaticArraySearchResult::Index(index as i64));
                    }
                }
                Some(StaticArraySearchResult::Index(-1))
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_static_array_search_call(
        &self,
        node: &LirNode,
        method: &str,
    ) -> Option<i64> {
        if node.kind != LirNodeKind::Call || !(2..=3).contains(&node.children.len()) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some(method) {
            return None;
        }

        let search_value = self.resolve_static_object_identity_value(*node.children.get(1)?)?;
        let explicit_from_index = match node.children.get(2) {
            Some(id) => Some(self.resolve_static_numeric_value(*id)?.trunc() as i64),
            None => None,
        };
        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0])
        } else {
            source_node
        };
        if !self.is_array_literal(source_node) {
            return None;
        }

        let length = source_node.children.len() as i64;
        if length == 0 {
            return Some(-1);
        }

        match method {
            "includes" | "indexOf" => {
                let from_index = explicit_from_index.unwrap_or(0);
                let start = if from_index >= 0 {
                    from_index.min(length)
                } else {
                    (length + from_index).max(0)
                } as usize;
                for (index, child) in source_node.children.iter().enumerate().skip(start) {
                    let candidate = self.resolve_static_object_identity_value(*child)?;
                    let matches = if method == "includes" {
                        candidate.same_value_zero(&search_value)
                    } else {
                        candidate.strict_eq(&search_value)
                    };
                    if matches {
                        return Some(index as i64);
                    }
                }
                Some(-1)
            }
            "lastIndexOf" => {
                let from_index = explicit_from_index.unwrap_or(length - 1);
                let start = if from_index >= 0 {
                    from_index.min(length - 1)
                } else {
                    length + from_index
                };
                if start < 0 {
                    return Some(-1);
                }
                for (index, child) in source_node
                    .children
                    .iter()
                    .enumerate()
                    .take(start as usize + 1)
                    .rev()
                {
                    let candidate = self.resolve_static_object_identity_value(*child)?;
                    if candidate.strict_eq(&search_value) {
                        return Some(index as i64);
                    }
                }
                Some(-1)
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_static_array_slice_bounds(
        &self,
        node: &LirNode,
    ) -> Option<(LirNodeId, usize, usize)> {
        if node.kind != LirNodeKind::Call || !(1..=3).contains(&node.children.len()) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("slice") {
            return None;
        }

        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        if !self.is_array_literal(source_node) {
            return None;
        }

        let length = source_node.children.len() as i64;
        let normalize = |value: Option<f64>, default: i64| -> Option<i64> {
            let value = value.map(|value| {
                if value < 0.0 {
                    (length as f64 + value.trunc()).max(0.0)
                } else {
                    value.trunc().min(length as f64)
                }
            });
            let normalized = value.unwrap_or(default as f64);
            if normalized.is_finite() {
                Some(normalized as i64)
            } else {
                None
            }
        };

        let start_value = match node.children.get(1) {
            Some(id) => Some(self.resolve_static_numeric_value(*id)?),
            None => None,
        };
        let end_value = match node.children.get(2) {
            Some(id) => Some(self.resolve_static_numeric_value(*id)?),
            None => None,
        };
        let start = normalize(start_value, 0)?;
        let end = normalize(end_value, length)?;
        let end = end.max(start);

        Some((source, start as usize, end as usize))
    }

    pub(crate) fn resolve_static_array_slice_element(
        &self,
        id: LirNodeId,
        index: usize,
    ) -> Option<LirNodeId> {
        let (source, start, end) = self.resolve_static_array_slice_bounds(self.node(id))?;
        let absolute_index = start.checked_add(index)?;
        if absolute_index >= end {
            return None;
        }
        self.node(source).children.get(absolute_index).copied()
    }

    pub(crate) fn resolve_static_array_concat_element(
        &self,
        id: LirNodeId,
        index: usize,
    ) -> Option<StaticIndexMemberResult> {
        let node = self.node(id);
        if node.kind != LirNodeKind::Call || node.children.is_empty() {
            return None;
        }

        let callee = self.resolve_transparent_callable_node(*node.children.first()?)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("concat") {
            return None;
        }

        let receiver = *callee_node.children.first()?;
        let mut elements = Vec::new();
        self.collect_static_array_concat_operand(receiver, true, &mut elements)?;
        for arg in node.children.iter().skip(1).copied() {
            self.collect_static_array_concat_operand(arg, false, &mut elements)?;
        }

        Some(
            elements
                .get(index)
                .copied()
                .map(StaticIndexMemberResult::Node)
                .unwrap_or(StaticIndexMemberResult::Undefined),
        )
    }

    pub(crate) fn collect_static_array_concat_operand(
        &self,
        id: LirNodeId,
        require_array: bool,
        elements: &mut Vec<LirNodeId>,
    ) -> Option<()> {
        if let Some(aggregate_id) = self.resolve_literal_aggregate(id) {
            let aggregate = self.node(aggregate_id);
            if self.is_array_literal(aggregate) {
                elements.extend(aggregate.children.iter().copied());
                return Some(());
            }
        }

        if require_array {
            return None;
        }

        match self.resolve_static_object_identity_value(id)? {
            StaticObjectIdentityValue::Boolean(_)
            | StaticObjectIdentityValue::Number(_)
            | StaticObjectIdentityValue::String(_)
            | StaticObjectIdentityValue::BigInt(_)
            | StaticObjectIdentityValue::Null
            | StaticObjectIdentityValue::Undefined => {
                elements.push(id);
                Some(())
            }
        }
    }

    pub(crate) fn resolve_static_array_at_call(
        &self,
        node: &LirNode,
    ) -> Option<StaticArrayAtResult> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let source_node = self.static_array_at_literal_receiver(node)?;
        let index = self
            .resolve_static_numeric_value(*node.children.get(1)?)?
            .trunc() as i64;
        let length = source_node.children.len() as i64;
        let resolved_index = if index >= 0 { index } else { length + index };
        if resolved_index < 0 || resolved_index >= length {
            return Some(StaticArrayAtResult::OutOfRange);
        }

        source_node
            .children
            .get(resolved_index as usize)
            .copied()
            .map(StaticArrayAtResult::Value)
    }

    pub(crate) fn is_array_at_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        self.static_array_at_literal_receiver(node).is_some()
    }

    pub(crate) fn resolve_static_array_join_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || !(1..=2).contains(&node.children.len()) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("join") {
            return None;
        }

        let separator = match node.children.get(1) {
            Some(id) => match self.resolve_static_object_identity_value(*id)? {
                StaticObjectIdentityValue::String(value) => value,
                _ => return None,
            },
            None => ",".to_string(),
        };

        self.resolve_static_array_join_receiver(callee_node, &separator)
    }

    pub(crate) fn resolve_static_array_join_receiver(
        &self,
        callee_node: &LirNode,
        separator: &str,
    ) -> Option<String> {
        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0])
        } else {
            source_node
        };
        if !self.is_array_literal(source_node) {
            return None;
        }

        let mut rendered = Vec::with_capacity(source_node.children.len());
        for child in &source_node.children {
            rendered.push(self.static_array_join_element_to_string(*child)?);
        }
        Some(rendered.join(separator))
    }

    pub(crate) fn static_array_at_literal_receiver(&self, node: &LirNode) -> Option<&LirNode> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("at") {
            return None;
        }

        let source = callee_node.children.first().copied()?;
        if matches!(
            self.resolve_static_object_identity_value(source),
            Some(StaticObjectIdentityValue::String(_))
        ) {
            return None;
        }
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0])
        } else {
            source_node
        };
        self.is_array_literal(source_node).then_some(source_node)
    }

    pub(crate) fn resolve_static_array_filter_items(
        &self,
        node: &LirNode,
    ) -> Option<Vec<LirNodeId>> {
        let mut current = node;
        while current.kind == LirNodeKind::Value
            && current.children.len() == 1
            && current
                .text
                .as_deref()
                .is_none_or(|text| text.is_empty() || text == "await")
        {
            current = self.node(current.children[0]);
        }

        if current.kind != LirNodeKind::Call || current.children.len() != 2 {
            return None;
        }

        let callee = current.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if !callee_node.text.as_deref().is_some_and(|text| {
            text == "filter"
                || text.ends_with(".filter")
                || text.ends_with("[\"filter\"]")
                || text.ends_with("['filter']")
        }) {
            return None;
        }

        let callback = current.children.get(1).copied()?;
        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source).clone();
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0]).clone()
        } else {
            source_node
        };
        if !self.is_array_literal(&source_node) {
            return None;
        }

        let mut items = Vec::new();
        for child in &source_node.children {
            if self.resolve_static_array_callback_truthiness(callback, *child)? {
                items.push(*child);
            }
        }
        Some(items)
    }

    pub(crate) fn resolve_static_array_reduce_call(
        &self,
        node: &LirNode,
        method: &str,
    ) -> Option<i64> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 2 | 3) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some(method) {
            return None;
        }

        let callback = node.children.get(1).copied()?;
        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source).clone();
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0]).clone()
        } else {
            source_node
        };
        if !self.is_array_literal(&source_node) {
            return None;
        }

        let mut ordered_values: Vec<LirNodeId> = if method == "reduceRight" {
            source_node.children.iter().rev().copied().collect()
        } else {
            source_node.children.clone()
        };

        let mut accumulator = match node.children.get(2).copied() {
            Some(initial) => self.resolve_static_numeric_value(initial)?,
            None => {
                let first = ordered_values.first().copied()?;
                ordered_values.remove(0);
                self.resolve_static_numeric_value(first)?
            }
        };

        for child in ordered_values {
            let current = self.resolve_static_numeric_value(child)?;
            accumulator =
                self.resolve_static_numeric_reducer_callback(callback, accumulator, current)?;
        }

        if accumulator.is_finite() && accumulator.fract() == 0.0 {
            Some(accumulator as i64)
        } else {
            None
        }
    }

    pub(crate) fn resolve_truthy_identity_array_filter_source(
        &self,
        node: &LirNode,
    ) -> Option<LirNodeId> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("filter") {
            return None;
        }

        let callback = node.children.get(1).copied()?;
        if !self.is_identity_array_map_callback(callback) {
            return None;
        }

        let source = callee_node.children.first().copied()?;
        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        let source_node = if source_node.kind == LirNodeKind::Value
            && source_node.text.is_none()
            && source_node.children.len() == 1
        {
            self.node(source_node.children[0])
        } else {
            source_node
        };
        if !self.is_truthy_array_literal(source_node) {
            return None;
        }

        Some(source)
    }

    pub(crate) fn is_frozen_array_from_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(resolved_callee) = self.resolve_literal_aggregate(callee) else {
            return false;
        };

        self.is_array_from_callable_node(self.node(resolved_callee))
    }

    pub(crate) fn is_array_from_callable_node(&self, node: &LirNode) -> bool {
        match node.text.as_deref() {
            Some(text) if kali_common::array_from_aliases().contains(&text) => true,
            Some("from") => {
                let Some(object) = node.children.first().copied() else {
                    return false;
                };

                matches!(
                    self.node(object).text.as_deref(),
                    Some("Array")
                        | Some("globalThis.Array")
                        | Some(r#"globalThis["Array"]"#)
                        | Some(r#"globalThis['Array']"#)
                )
            }
            _ => false,
        }
    }

    /// Runtime counted `for..of` over a growable (push-accumulated) array
    /// (throw-fallout Stage 4 Task 4): `i = 0; n = len(handle); loop { if i >= n
    /// break; v = data[i]; i += 1; body }`. `i` and `n` are shared per-function
    /// i64 scratch locals; the loop var `v` is a real wasm local so the body's
    /// reads resolve to it (via `emit_value`'s `locals` lookup) — unlike the
    /// static-unroll lane, which substitutes a compile-time node per iteration.
    ///
    /// The increment is emitted BEFORE the body (not after), so an unlabeled
    /// `continue` — which `emit_break_or_continue` lowers to a `Br` back to the
    /// `Loop` top, exactly like the plain `for`-loop lane — has already advanced
    /// the index and therefore visits the NEXT element, not the same one. `break`
    /// exits the enclosing `Block`. Both reuse the standard control-frame /
    /// `LoopFrame` scaffolding, so they resolve identically to every other loop.
    fn emit_for_of_growable_runtime_loop(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        iterable_id: LirNodeId,
        handle_name: String,
        field_receiver: bool,
    ) -> EmittedValue {
        // Fail closed: a growable `for..of` lexically NESTED inside another
        // would share the single index/length scratch pair, clobbering the
        // outer loop's counter — a silent miscompile. (Codegen-side guard; the
        // resolve gate admits the shape, so this is a fail-CLOSED both-sides
        // asymmetry, listed in the report — never a fail-open.)
        if self.growable_for_of_active.is_some() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "a for-of over a growable array nested inside another for-of over a growable array is unavailable in the current phase; use an index loop over `.length` or the later compatibility path".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        // Defensive both-sides mirror of the resolve i64-element gate: a
        // String-element growable `for..of` never reaches codegen (the resolve
        // gate aborts the compile), but never emit a loop that would store a raw
        // string handle into an i64-printed loop var if that gate ever regresses.
        // A FIELD receiver (`handle_name` is a `base.field` key, not a binding)
        // is provably `GrowableArrayI64` — i64 elements only (Task 3 conflicts
        // string array fields to E5506) — so the name-keyed elem lookup does not
        // apply; skip it (the field key is not an array binding name).
        if !field_receiver && self.array_elem_repr(&handle_name) != kali_common::Repr::I64 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "for-of over a growable array of non-integer elements is unavailable in the current phase".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let Some(loop_name) = self.for_of_binding_name(node) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "for-of array iteration lowering is unavailable unless the loop target is a single variable declaration or identifier binding; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };
        let Some(loop_local) = self.locals.get(&loop_name).copied() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "growable for-of loop variable `{loop_name}` has no local slot; iteration lowering is unavailable"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };
        // Reserved by `collect_function_locals`' `for_of_growable_loop_var_names`
        // walk (the structural twin of this lane's guard). A miss is a
        // reserve/resolve twin desync — fail closed (E5506), never panic.
        let (Some(index_local), Some(len_local)) = (
            self.locals
                .get(&crate::lower::growable_foreach_index_local_name())
                .copied(),
            self.locals
                .get(&crate::lower::growable_foreach_len_local_name())
                .copied(),
        ) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "growable for-of index/length scratch locals were not reserved; iteration lowering is unavailable".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };
        let body = node.children.get(2).copied();

        // i = 0
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        // n = len(handle)  — snapshotted ONCE (a body that pushes to a
        // different array is unaffected; pushing to the array being iterated
        // does not extend this loop, matching the design's fixed count).
        self.emit_growable_length(function, iterable_id);
        function.instruction(&Instruction::LocalSet(len_local));

        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index,
        });

        // if i >= n break
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeS);
        let break_depth = self.control_frame_depth(break_index);
        function.instruction(&Instruction::BrIf(break_depth));

        // v = data[i]
        self.emit_growable_index_read_at_local(function, iterable_id, index_local);
        function.instruction(&Instruction::LocalSet(loop_local));

        // i += 1  (BEFORE the body — makes `continue` visit the next element)
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));

        // body — with the iterated binding name active, so the nesting guard
        // above and `emit_growable_push_call`'s same-binding self-push guard
        // both see it.
        let previous_active = self.growable_for_of_active.replace(handle_name.clone());
        if let Some(body) = body {
            let _ = self.emit_node(function, body, false);
        }
        self.growable_for_of_active = previous_active;

        // back-edge to the loop top
        let continue_depth = self.control_frame_depth(continue_index);
        function.instruction(&Instruction::Br(continue_depth));

        function.instruction(&Instruction::End); // end loop
        self.loop_frames.pop();
        self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::End); // end block
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    pub(crate) fn emit_for_of_array_iteration(
        &mut self,
        function: &mut Function,
        node: &LirNode,
    ) -> EmittedValue {
        let Some(array_id) = node.children.get(1).copied() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "for-of array iteration lowering is unavailable in the current phase; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };

        // Runtime growable lane (throw-fallout Stage 4 Task 4): a bare-identifier
        // iterable that names a GROWABLE array binding runs a REAL wasm counted
        // loop over the live handle, NOT the compile-time static unroll below
        // (which would fold the stale declarator literal). Keyed on the SAME
        // predicate the types-side resolve gate admits (bare identifier +
        // growable binding), so the two never desync. Every other iterable —
        // literal arrays, map/filter/Array.from/spread/flatMap over literals,
        // Object.keys/values/entries, string iterables — keeps the static lane
        // byte-identically.
        if let Some(handle_name) = self.bare_identifier_name(array_id) {
            if self.is_growable_array(&handle_name) {
                return self.emit_for_of_growable_runtime_loop(
                    function,
                    node,
                    array_id,
                    handle_name,
                    false,
                );
            }
        }

        // Growable-array FIELD iterable `for (const x of o.values)` (Stage P2
        // Lane 1 Task 5): the field slot holds the tagged growable handle (Task
        // 3 + object-field alloc), so the same counted growable loop runs, with
        // the handle materialized from the field read (`array_id` is the
        // `o.values` node). Admitted only through the positive
        // `object_field_is_growable_array` proof; the loop identity is the
        // `base.field` key so the self-push guard can key on it.
        if self.object_field_is_growable_array(array_id) {
            let key = self
                .growable_field_receiver_key(array_id)
                .unwrap_or_default();
            return self.emit_for_of_growable_runtime_loop(function, node, array_id, key, true);
        }

        let mut array_id = array_id;
        if let Some(resolved) = self.resolve_literal_aggregate(array_id) {
            array_id = resolved;
        }
        if let Some(resolved) = self.resolve_identity_array_callback_source(self.node(array_id)) {
            array_id = resolved;
        }
        let Some(array_id) = self.resolve_literal_aggregate(array_id) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "for-of array iteration lowering is unavailable unless the iterable is a literal array with literal elements; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };

        let mut array = self.node(array_id).clone();
        if array.kind == LirNodeKind::Value
            && (array.text.is_none() || array.text.as_deref() == Some("await"))
            && array.children.len() == 1
        {
            let child_id = array.children[0];
            let child = self.node(child_id).clone();
            if self.is_array_literal(&child) {
                array = child;
            }
        }

        let Some(loop_name) = self.for_of_binding_name(node) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "for-of array iteration lowering is unavailable unless the loop target is a single variable declaration or identifier binding; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };

        let body = node.children.get(2).copied();
        if let Some(string_text) = self.render_static_string_value(&array) {
            let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
            function.instruction(&Instruction::Block(BlockType::Empty));
            for value in string_text.chars() {
                let literal = self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(format!("{value:?}")),
                    vec![],
                );
                let previous_binding = self.bindings.insert(loop_name.clone(), literal);
                let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.push(LoopFrame {
                    break_index,
                    continue_index,
                });
                function.instruction(&Instruction::Block(BlockType::Empty));
                if let Some(body) = body {
                    let _ = self.emit_node(function, body, false);
                }
                if let Some(previous_binding) = previous_binding {
                    self.bindings.insert(loop_name.clone(), previous_binding);
                } else {
                    self.bindings.remove(&loop_name);
                }
                function.instruction(&Instruction::End);
                self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.pop();
            }
            function.instruction(&Instruction::End);
            self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(set_call) = self.resolve_set_constructor_call(&array) {
            let Some(source_arg) = set_call.children.get(1).copied() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a literal array or supported string iterable with literal elements; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(source_id) = self.resolve_literal_aggregate(source_arg) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a literal array or supported string iterable with literal elements; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let source = self.node(source_id).clone();
            let mut items = Vec::new();
            if !self.collect_set_constructor_iteration_items(&source, &mut items) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a literal array or supported string iterable with literal elements; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
            function.instruction(&Instruction::Block(BlockType::Empty));
            for child in items {
                let previous_binding = self.bindings.insert(loop_name.clone(), child);
                let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.push(LoopFrame {
                    break_index,
                    continue_index,
                });
                function.instruction(&Instruction::Block(BlockType::Empty));
                if let Some(body) = body {
                    let _ = self.emit_node(function, body, false);
                }
                if let Some(previous_binding) = previous_binding {
                    self.bindings.insert(loop_name.clone(), previous_binding);
                } else {
                    self.bindings.remove(&loop_name);
                }
                function.instruction(&Instruction::End);
                self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.pop();
            }
            function.instruction(&Instruction::End);
            self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(map_call) = self.resolve_map_constructor_call(&array) {
            let Some(source_arg) = map_call.children.get(1).copied() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a literal array of supported Map entry tuples; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(source_id) = self.resolve_literal_aggregate(source_arg) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a literal array of supported Map entry tuples; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let source = self.node(source_id).clone();
            let mut items = Vec::new();
            if !self.collect_map_constructor_iteration_items(&source, &mut items) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a literal array of supported Map entry tuples; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
            function.instruction(&Instruction::Block(BlockType::Empty));
            for child in items {
                let previous_binding = self.bindings.insert(loop_name.clone(), child);
                let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.push(LoopFrame {
                    break_index,
                    continue_index,
                });
                function.instruction(&Instruction::Block(BlockType::Empty));
                if let Some(body) = body {
                    let _ = self.emit_node(function, body, false);
                }
                if let Some(previous_binding) = previous_binding {
                    self.bindings.insert(loop_name.clone(), previous_binding);
                } else {
                    self.bindings.remove(&loop_name);
                }
                function.instruction(&Instruction::End);
                self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.pop();
            }
            function.instruction(&Instruction::End);
            self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(object_enumeration_mode) = self.is_object_enumeration_call(&array) {
            let Some(object_arg) = array.children.get(1).copied() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a supported Object.keys(...), Object.values(...), Object.entries(...), or Reflect.ownKeys(...) slice; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(object_id) = self.resolve_literal_aggregate(object_arg) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a supported Object.keys(...), Object.values(...), Object.entries(...), or Reflect.ownKeys(...) slice; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let object = self.node(object_id).clone();
            let mut items = Vec::with_capacity(object.children.len());
            if !self.collect_object_enumeration_iteration_items(
                &object,
                object_enumeration_mode,
                &mut items,
            ) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a supported Object.keys(...), Object.values(...), Object.entries(...), or Reflect.ownKeys(...) slice with string literal keys; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            if !self.is_object_from_entries_call(&object) {
                let produced = self.emit_node(function, object_arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }

            let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
            function.instruction(&Instruction::Block(BlockType::Empty));
            for child in items {
                let previous_binding = self.bindings.insert(loop_name.clone(), child);
                let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.push(LoopFrame {
                    break_index,
                    continue_index,
                });
                function.instruction(&Instruction::Block(BlockType::Empty));
                if let Some(body) = body {
                    let _ = self.emit_node(function, body, false);
                }
                if let Some(previous_binding) = previous_binding {
                    self.bindings.insert(loop_name.clone(), previous_binding);
                } else {
                    self.bindings.remove(&loop_name);
                }
                function.instruction(&Instruction::End);
                self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.pop();
            }
            function.instruction(&Instruction::End);
            self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(items) = self.resolve_static_array_filter_items(&array) {
            let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
            function.instruction(&Instruction::Block(BlockType::Empty));
            for child in items {
                let previous_binding = self.bindings.insert(loop_name.clone(), child);
                let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.push(LoopFrame {
                    break_index,
                    continue_index,
                });
                function.instruction(&Instruction::Block(BlockType::Empty));
                if let Some(body) = body {
                    let _ = self.emit_node(function, body, false);
                }
                if let Some(previous_binding) = previous_binding {
                    self.bindings.insert(loop_name.clone(), previous_binding);
                } else {
                    self.bindings.remove(&loop_name);
                }
                function.instruction(&Instruction::End);
                self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
                self.loop_frames.pop();
            }
            function.instruction(&Instruction::End);
            self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_array_callback_iteration_call(&array)
            && self
                .resolve_identity_array_callback_source(&array)
                .is_none()
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "for-of array iteration lowering is unavailable for array callback-produced iterables in the current phase; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if !self.is_array_literal(&array) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "for-of array iteration lowering is unavailable unless the iterable is a literal array with literal elements; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let mut items = Vec::with_capacity(array.children.len());
        for child in &array.children {
            if !self.collect_for_of_array_iteration_items(*child, &mut items) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for-of array iteration lowering is unavailable unless the iterable is a literal array with literal elements; use a supported loop form or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
        }

        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        for child in items {
            let previous_binding = self.bindings.insert(loop_name.clone(), child);
            let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
            self.loop_frames.push(LoopFrame {
                break_index,
                continue_index,
            });
            function.instruction(&Instruction::Block(BlockType::Empty));
            if let Some(body) = body {
                let _ = self.emit_node(function, body, false);
            }
            if let Some(previous_binding) = previous_binding {
                self.bindings.insert(loop_name.clone(), previous_binding);
            } else {
                self.bindings.remove(&loop_name);
            }
            function.instruction(&Instruction::End);
            self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
            self.loop_frames.pop();
        }
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    pub(crate) fn collect_for_of_array_iteration_items(
        &mut self,
        id: LirNodeId,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        let Some(resolved_id) = self.resolve_literal_aggregate(id) else {
            return false;
        };

        let node = self.node(resolved_id);
        if matches!(node.kind, LirNodeKind::Literal) {
            let original = self.node(id);
            if original.kind == LirNodeKind::Value
                && original.children.is_empty()
                && original
                    .text
                    .as_deref()
                    .is_some_and(|text| self.bindings.contains_key(text))
            {
                items.push(id);
            } else {
                items.push(resolved_id);
            }
            return true;
        }

        if self.is_array_literal(node) {
            let children = node.children.clone();
            if children
                .iter()
                .all(|child| self.is_supported_for_of_array_iteration_item(*child))
            {
                items.push(resolved_id);
                return true;
            }
            return false;
        }

        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("spread")
            && node.children.len() == 1
        {
            let Some(argument_id) = node.children.first().copied() else {
                return false;
            };
            let Some(array_id) = self.resolve_literal_aggregate(argument_id) else {
                return false;
            };
            let array = self.node(array_id).clone();
            if self.is_array_literal(&array) {
                for child in &array.children {
                    if !self.collect_for_of_array_iteration_items(*child, items) {
                        return false;
                    }
                }

                return true;
            }

            let Some(object_enumeration_mode) = self.is_object_enumeration_call(&array) else {
                return false;
            };
            let Some(object_arg) = array.children.get(1).copied() else {
                return false;
            };
            let Some(object_id) = self.resolve_literal_aggregate(object_arg) else {
                return false;
            };
            let object = self.node(object_id).clone();
            if !self.collect_object_enumeration_iteration_items(
                &object,
                object_enumeration_mode,
                items,
            ) {
                return false;
            }

            return true;
        }

        false
    }

    pub(crate) fn is_supported_for_of_array_iteration_item(&mut self, id: LirNodeId) -> bool {
        let Some(resolved_id) = self.resolve_literal_aggregate(id) else {
            return false;
        };

        let node = self.node(resolved_id);
        if matches!(node.kind, LirNodeKind::Literal) {
            return true;
        }

        if self.is_array_literal(node) {
            let children = node.children.clone();
            return children
                .iter()
                .all(|child| self.is_supported_for_of_array_iteration_item(*child));
        }

        false
    }
}

#[cfg(test)]
#[path = "array_tests.rs"]
mod array_tests;

//! String intrinsic call recognition and constant-folding.
use crate::*;

pub(crate) fn quote_string_literal(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

pub(crate) fn strip_string_delimiters(text: &str) -> &str {
    let trimmed = text.trim();
    let Some(first) = trimmed.chars().next() else {
        return trimmed;
    };
    let Some(last) = trimmed.chars().last() else {
        return trimmed;
    };

    if (first == '"' && last == '"')
        || (first == '\'' && last == '\'')
        || (first == '`' && last == '`')
    {
        &trimmed[1..trimmed.len().saturating_sub(1)]
    } else {
        trimmed
    }
}

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn static_ascii_string_relational_result(
        &self,
        left: LirNodeId,
        right: LirNodeId,
        op: &str,
    ) -> Option<bool> {
        let left = match self.resolve_static_object_identity_value(left)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let right = match self.resolve_static_object_identity_value(right)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };

        Some(match op {
            "<" => left < right,
            "<=" => left <= right,
            ">" => left > right,
            ">=" => left >= right,
            _ => return None,
        })
    }

    pub(crate) fn resolve_static_string_from_char_code_call(
        &self,
        node: &LirNode,
        callee_node: &LirNode,
    ) -> Option<String> {
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee_id = *node.children.first()?;
        let callee_node = self
            .resolve_transparent_callable_node(callee_id)
            .map(|id| self.node(id))
            .unwrap_or(callee_node);
        if !self.is_string_from_char_code_callable(callee_node) {
            return None;
        }

        let mut rendered = String::new();
        for id in node.children.iter().skip(1) {
            let value = self.resolve_static_numeric_value(*id)?;
            if !is_supported_static_ascii_char_code(value) {
                return None;
            }
            rendered.push(char::from_u32(value as u32)?);
        }
        Some(rendered)
    }

    pub(crate) fn is_string_from_char_code_callable(&self, callee_node: &LirNode) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };

        if method != "fromCharCode" && method != "fromCodePoint" {
            return false;
        }

        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("String")
                | Some("globalThis.String")
                | Some(r#"globalThis["String"]"#)
                | Some(r#"globalThis['String']"#)
        )
    }

    pub(crate) fn resolve_static_string_search_call(
        &self,
        node: &LirNode,
        method: &str,
    ) -> Option<i64> {
        if node.kind != LirNodeKind::Call || !(1..=3).contains(&node.children.len()) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some(method) {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let search = match node.children.get(1) {
            Some(id) => match self.resolve_static_object_identity_value(*id)? {
                StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
                _ => return None,
            },
            None => "undefined".to_string(),
        };
        let explicit_from_index = match node.children.get(2) {
            Some(id) => Some(self.resolve_static_numeric_value(*id)?.trunc() as i64),
            None => None,
        };

        let length = source.len() as i64;
        match method {
            "includes" | "indexOf" => {
                let from_index = explicit_from_index.unwrap_or(0);
                let start = if method == "includes" && from_index < 0 {
                    (length + from_index).max(0)
                } else {
                    from_index.max(0).min(length)
                } as usize;
                let haystack = source.get(start..)?;
                haystack
                    .find(&search)
                    .map_or(Some(-1), |index| Some((start + index) as i64))
            }
            "lastIndexOf" => {
                let from_index = explicit_from_index.unwrap_or(length);
                let position = from_index.clamp(0, length) as usize;
                if search.is_empty() {
                    return Some(position as i64);
                }
                let end = position.saturating_add(search.len()).min(source.len());
                let haystack = source.get(..end)?;
                haystack
                    .rfind(&search)
                    .filter(|index| *index <= position)
                    .map_or(Some(-1), |index| Some(index as i64))
            }
            "startsWith" => {
                let position = explicit_from_index.unwrap_or(0).clamp(0, length) as usize;
                source
                    .get(position..)
                    .filter(|suffix| suffix.starts_with(&search))
                    .map(|_| position as i64)
                    .or(Some(-1))
            }
            "endsWith" => {
                let end = explicit_from_index.unwrap_or(length).clamp(0, length) as usize;
                source
                    .get(..end)
                    .filter(|prefix| prefix.ends_with(&search))
                    .map(|_| 0)
                    .or(Some(-1))
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_static_string_identity_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.len() != 1 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if !matches!(callee_node.text.as_deref(), Some("toString" | "valueOf")) {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn string_identity_call_method_with_literal_receiver(
        &self,
        node: &LirNode,
    ) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.len() <= 1 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        let method = callee_node.text.as_deref()?;
        if !matches!(method, "toString" | "valueOf") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        self.resolve_static_object_identity_value(receiver)
            .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
            .then(|| method.to_string())
    }

    pub(crate) fn resolve_static_string_slice_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || !(1..=3).contains(&node.children.len()) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("slice") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let start = match node.children.get(1) {
            Some(id) => {
                let start = self.resolve_static_numeric_value(*id)?;
                if !start.is_finite() {
                    return None;
                }
                start.trunc() as i64
            }
            None => 0,
        };
        let end = match node.children.get(2) {
            Some(id) => {
                let end = self.resolve_static_numeric_value(*id)?;
                if !end.is_finite() {
                    return None;
                }
                end.trunc() as i64
            }
            None => source.len() as i64,
        };

        let length = source.len() as i64;
        let from = if start < 0 {
            (length + start).max(0)
        } else {
            start.min(length)
        };
        let to = if end < 0 {
            (length + end).max(0)
        } else {
            end.min(length)
        };
        let to = to.max(from);

        source
            .get(from as usize..to as usize)
            .map(ToString::to_string)
    }

    pub(crate) fn resolve_static_string_substring_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || !(1..=3).contains(&node.children.len()) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("substring") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let start = match node.children.get(1) {
            Some(id) => {
                let start = self.resolve_static_numeric_value(*id)?;
                if !start.is_finite() {
                    return None;
                }
                start.trunc() as i64
            }
            None => 0,
        };
        let end = match node.children.get(2) {
            Some(id) => {
                let end = self.resolve_static_numeric_value(*id)?;
                if !end.is_finite() {
                    return None;
                }
                end.trunc() as i64
            }
            None => source.len() as i64,
        };

        let length = source.len() as i64;
        let mut from = start.clamp(0, length);
        let mut to = end.clamp(0, length);
        if from > to {
            std::mem::swap(&mut from, &mut to);
        }

        source
            .get(from as usize..to as usize)
            .map(ToString::to_string)
    }

    pub(crate) fn resolve_static_string_repeat_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("repeat") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let count = self.resolve_static_numeric_value(*node.children.get(1)?)?;
        if !count.is_finite() || count.fract() != 0.0 || !(0.0..=1024.0).contains(&count) {
            return None;
        }

        Some(source.repeat(count as usize))
    }

    pub(crate) fn is_string_repeat_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("repeat")
            && callee_node
                .children
                .first()
                .copied()
                .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
                .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
    }

    pub(crate) fn resolve_static_string_concat_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.is_empty() {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("concat") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let mut result = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };

        for argument in node.children.iter().skip(1) {
            match self.resolve_static_object_identity_value(*argument)? {
                StaticObjectIdentityValue::String(value) if value.is_ascii() => {
                    result.push_str(&value);
                }
                _ => return None,
            }
        }

        Some(result)
    }

    pub(crate) fn is_string_concat_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("concat")
            && callee_node
                .children
                .first()
                .copied()
                .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
                .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
    }

    pub(crate) fn resolve_static_string_pad_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 2 | 3) {
            return None;
        }

        let method = self.string_pad_call_method(node)?;
        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let target_length = self.resolve_static_numeric_value(*node.children.get(1)?)?;
        if !target_length.is_finite()
            || target_length.fract() != 0.0
            || !(0.0..=1024.0).contains(&target_length)
        {
            return None;
        }
        let padding = match node.children.get(2) {
            Some(id) => match self.resolve_static_object_identity_value(*id)? {
                StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
                _ => return None,
            },
            None => " ".to_string(),
        };

        let target_length = target_length as usize;
        if source.len() >= target_length || padding.is_empty() {
            return Some(source);
        }

        let needed = target_length - source.len();
        let mut fill = String::new();
        while fill.len() < needed {
            fill.push_str(&padding);
        }
        fill.truncate(needed);

        match method.as_str() {
            "padStart" => Some(format!("{fill}{source}")),
            "padEnd" => Some(format!("{source}{fill}")),
            _ => None,
        }
    }

    pub(crate) fn string_pad_call_method(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let method = self.node(callee).text.as_deref()?;
        matches!(method, "padStart" | "padEnd").then(|| method.to_string())
    }

    pub(crate) fn resolve_static_string_at_call(
        &self,
        node: &LirNode,
    ) -> Option<StaticStringAtResult> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 1 | 2) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("at") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = self
            .resolve_static_object_identity_value(receiver)
            .and_then(|value| match value {
                StaticObjectIdentityValue::String(value) => Some(value),
                _ => None,
            })?;
        if !source.is_ascii() {
            return None;
        }
        let index = match node.children.get(1) {
            Some(id) => {
                let index = self.resolve_static_numeric_value(*id)?;
                if !index.is_finite() || index.fract() != 0.0 {
                    return None;
                }
                index as i64
            }
            None => 0,
        };

        let length = source.len() as i64;
        let index = if index < 0 { length + index } else { index };
        if index < 0 || index >= length {
            return Some(StaticStringAtResult::OutOfRange);
        }

        Some(StaticStringAtResult::Value(
            source
                .as_bytes()
                .get(index as usize)
                .map(|byte| (*byte as char).to_string())?,
        ))
    }

    pub(crate) fn is_string_at_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("at")
            && callee_node
                .children
                .first()
                .copied()
                .is_some_and(|receiver| {
                    self.resolve_static_object_identity_value(receiver)
                        .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
                })
    }

    pub(crate) fn resolve_static_string_char_at_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 1 | 2) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("charAt") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let index = match node.children.get(1) {
            Some(id) => {
                let index = self.resolve_static_numeric_value(*id)?;
                if !index.is_finite() || index.fract() != 0.0 {
                    return None;
                }
                index as i64
            }
            None => 0,
        };

        if index < 0 {
            return Some(String::new());
        }

        Some(
            source
                .as_bytes()
                .get(index as usize)
                .map(|byte| (*byte as char).to_string())
                .unwrap_or_default(),
        )
    }

    pub(crate) fn is_string_char_at_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("charAt")
            && callee_node
                .children
                .first()
                .copied()
                .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
                .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
    }

    pub(crate) fn resolve_static_string_char_code_at_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 1 | 2) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("charCodeAt") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let index = match node.children.get(1) {
            Some(id) => {
                let index = self.resolve_static_numeric_value(*id)?;
                if !index.is_finite() || index.fract() != 0.0 {
                    return None;
                }
                index as i64
            }
            None => 0,
        };

        if index < 0 {
            return Some("NaN".to_string());
        }

        Some(
            source
                .as_bytes()
                .get(index as usize)
                .map(|byte| byte.to_string())
                .unwrap_or_else(|| "NaN".to_string()),
        )
    }

    pub(crate) fn is_string_char_code_at_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("charCodeAt")
            && callee_node
                .children
                .first()
                .copied()
                .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
                .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
    }

    pub(crate) fn resolve_static_string_code_point_at_call(
        &self,
        node: &LirNode,
    ) -> Option<StaticStringAtResult> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 1 | 2) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("codePointAt") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let index = match node.children.get(1) {
            Some(id) => {
                let index = self.resolve_static_numeric_value(*id)?;
                if !index.is_finite() || index.fract() != 0.0 {
                    return None;
                }
                index as i64
            }
            None => 0,
        };

        if index < 0 {
            return Some(StaticStringAtResult::OutOfRange);
        }

        Some(match source.as_bytes().get(index as usize) {
            Some(byte) => StaticStringAtResult::Value(byte.to_string()),
            None => StaticStringAtResult::OutOfRange,
        })
    }

    pub(crate) fn is_string_code_point_at_call_with_literal_receiver(
        &self,
        node: &LirNode,
    ) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("codePointAt")
            && callee_node
                .children
                .first()
                .copied()
                .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
                .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
    }

    pub(crate) fn resolve_static_string_trim_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.len() != 1 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        let method = callee_node.text.as_deref()?;
        if !matches!(
            method,
            "trim" | "trimStart" | "trimEnd" | "trimLeft" | "trimRight"
        ) {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };

        let is_ascii_trim = |ch: char| ch.is_ascii_whitespace();
        match method {
            "trim" => Some(source.trim_matches(is_ascii_trim).to_string()),
            "trimStart" | "trimLeft" => Some(source.trim_start_matches(is_ascii_trim).to_string()),
            "trimEnd" | "trimRight" => Some(source.trim_end_matches(is_ascii_trim).to_string()),
            _ => None,
        }
    }

    pub(crate) fn resolve_static_string_case_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.len() != 1 {
            return None;
        }

        let method = self.string_case_call_method(node)?;
        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };

        match method.as_str() {
            "toLowerCase" | "toLocaleLowerCase" => Some(source.to_ascii_lowercase()),
            "toUpperCase" | "toLocaleUpperCase" => Some(source.to_ascii_uppercase()),
            _ => None,
        }
    }

    pub(crate) fn resolve_static_string_normalize_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 1 | 2) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("normalize") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let form = match node.children.get(1) {
            Some(id) => match self.resolve_static_object_identity_value(*id)? {
                StaticObjectIdentityValue::String(value)
                    if matches!(value.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD") =>
                {
                    value
                }
                _ => return None,
            },
            None => "NFC".to_string(),
        };

        matches!(form.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD").then_some(source)
    }

    pub(crate) fn is_string_normalize_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("normalize")
            && callee_node
                .children
                .first()
                .copied()
                .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
                .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
    }

    pub(crate) fn resolve_static_string_replace_call(
        &self,
        node: &LirNode,
        method: &str,
    ) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.len() != 3 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some(method) {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let search = match self.resolve_static_object_identity_value(*node.children.get(1)?)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let replacement = match self.resolve_static_object_identity_value(*node.children.get(2)?)? {
            StaticObjectIdentityValue::String(value)
                if value.is_ascii() && !value.contains('$') =>
            {
                value
            }
            _ => return None,
        };

        match method {
            "replace" => Some(source.replacen(&search, &replacement, 1)),
            "replaceAll" => Some(source.replace(&search, &replacement)),
            _ => None,
        }
    }

    pub(crate) fn string_replace_call_method_with_literal_receiver(
        &self,
        node: &LirNode,
    ) -> Option<String> {
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        let method = callee_node.text.as_deref()?;
        if !matches!(method, "replace" | "replaceAll") {
            return None;
        }
        callee_node
            .children
            .first()
            .copied()
            .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
            .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
            .then(|| method.to_string())
    }

    pub(crate) fn resolve_static_string_split_parts_from_id(
        &self,
        mut id: LirNodeId,
    ) -> Option<Vec<String>> {
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = self.node(id);
            if let Some(parts) = self.resolve_static_string_split_call(node) {
                return Some(parts);
            }

            if node.kind == LirNodeKind::Value
                && node.text.as_deref().is_some_and(|text| text.is_empty())
                && !node.children.is_empty()
            {
                id = *node.children.last().expect("sequence wrapper has a child");
                continue;
            }

            if node.kind == LirNodeKind::Value
                && node.children.is_empty()
                && node.text.as_deref().is_some()
            {
                let name = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(name).copied() {
                    id = bound;
                    continue;
                }
            }

            if self.is_object_freeze_call(node) || self.is_frozen_array_from_call(node) {
                id = node.children.get(1).copied()?;
                continue;
            }

            return None;
        }
    }

    pub(crate) fn resolve_static_string_split_call(&self, node: &LirNode) -> Option<Vec<String>> {
        if node.kind != LirNodeKind::Call || !matches!(node.children.len(), 1..=3) {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("split") {
            return None;
        }

        let receiver = callee_node.children.first().copied()?;
        let source = match self.resolve_static_object_identity_value(receiver)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let separator = match node.children.get(1) {
            Some(id) => match self.resolve_static_object_identity_value(*id)? {
                StaticObjectIdentityValue::String(value) if value.is_ascii() => Some(value),
                _ => return None,
            },
            None => None,
        };
        let limit = match node.children.get(2) {
            Some(id) => {
                let limit = self.resolve_static_numeric_value(*id)?;
                if !limit.is_finite() || limit.fract() != 0.0 || !(0.0..=1024.0).contains(&limit) {
                    return None;
                }
                Some(limit as usize)
            }
            None => None,
        };

        let mut parts = match separator {
            Some(separator) if separator.is_empty() => {
                source.chars().map(|ch| ch.to_string()).collect::<Vec<_>>()
            }
            Some(separator) => source
                .split(&separator)
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            None => vec![source],
        };
        if let Some(limit) = limit {
            parts.truncate(limit);
        }
        Some(parts)
    }

    pub(crate) fn is_string_split_call_with_literal_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        callee_node.text.as_deref() == Some("split")
            && callee_node
                .children
                .first()
                .copied()
                .and_then(|receiver| self.resolve_static_object_identity_value(receiver))
                .is_some_and(|value| matches!(value, StaticObjectIdentityValue::String(_)))
    }

    pub(crate) fn string_case_call_method(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let method = self.node(callee).text.as_deref()?;
        matches!(
            method,
            "toLowerCase" | "toUpperCase" | "toLocaleLowerCase" | "toLocaleUpperCase"
        )
        .then(|| method.to_string())
    }

    pub(crate) fn resolve_static_array_to_string_call(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.len() != 1 {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("toString") {
            return None;
        }

        self.resolve_static_array_join_receiver(callee_node, ",")
    }

    pub(crate) fn static_array_join_element_to_string(&self, id: LirNodeId) -> Option<String> {
        match self.resolve_static_object_identity_value(id)? {
            StaticObjectIdentityValue::String(value) => Some(value),
            StaticObjectIdentityValue::Boolean(value) => Some(value.to_string()),
            StaticObjectIdentityValue::Null | StaticObjectIdentityValue::Undefined => {
                Some(String::new())
            }
            StaticObjectIdentityValue::BigInt(value) => Some(value.to_string()),
            StaticObjectIdentityValue::Number(value) => {
                if value.is_nan() {
                    Some("NaN".to_string())
                } else if value == f64::INFINITY {
                    Some("Infinity".to_string())
                } else if value == f64::NEG_INFINITY {
                    Some("-Infinity".to_string())
                } else if value == 0.0 {
                    Some("0".to_string())
                } else if value.is_finite() && value.fract() == 0.0 {
                    Some((value as i64).to_string())
                } else if value.is_finite() {
                    Some(value.to_string())
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn render_static_string_value(&self, node: &LirNode) -> Option<String> {
        let node = if node.kind == LirNodeKind::Value
            && node.text.as_deref().is_some_and(|text| text.is_empty())
            && node.children.len() == 1
        {
            self.node(node.children[0])
        } else {
            node
        };

        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("+")
            && node.children.len() == 2
        {
            let left = self.render_static_string_value(self.node(node.children[0]))?;
            let right = self.render_static_string_value(self.node(node.children[1]))?;
            return Some(format!("{left}{right}"));
        }

        let text = match node.kind {
            LirNodeKind::Literal if node.children.is_empty() => node.text.as_deref()?,
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.render_static_string_value(self.node(bound));
                }
                text
            }
            _ => return None,
        };

        if text == "true"
            || text == "false"
            || text == "null"
            || text == "undefined"
            || text == "Infinity"
            || text == "NaN"
            || parse_number_literal(text).is_some()
            || parse_numeric_literal_value(text).is_some()
        {
            return None;
        }

        Some(strip_string_delimiters(text).to_string())
    }
}

#[cfg(test)]
#[path = "string_tests.rs"]
mod string_tests;

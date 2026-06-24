//! WASM code generation for the Kali compiler.

mod ctx;
mod emit;
mod emitter;
pub use ctx::{CodegenCtx, CodegenResult, TargetConfig};
use emitter::{
    ControlFlowLabelKind, EmittedValue, FunctionEmitter, FunctionPlan, LoopFrame,
    ObjectEnumerationMode, ValueShape,
};
use ctx::{
    StaticArrayAtResult, StaticArraySearchResult, StaticIndexMemberResult,
    StaticObjectIdentityValue, StaticStringAtResult, StringPool,
};

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

use kali_common::generator_function_yield_lowering_unavailable_message;
use kali_error::{
    _error_codes::{e3, e5, e8},
    Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_lir::{FunctionFlavor, LirNode, LirNodeId, LirNodeKind, LirProgram};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, CustomSection, DataSection, EntityType, ExportKind,
    ExportSection, Function, FunctionSection, ImportSection, Instruction, MemorySection,
    MemoryType, Module, TypeSection, ValType,
};

const TEST_REGISTER_IMPORT_INDEX: u32 = 0;
const CONSOLE_LOG_IMPORT_INDEX: u32 = 1;
const CONSOLE_ERROR_IMPORT_INDEX: u32 = 2;
const CONSOLE_WARN_IMPORT_INDEX: u32 = 3;
const CONSOLE_INFO_IMPORT_INDEX: u32 = 4;
const CONSOLE_DEBUG_IMPORT_INDEX: u32 = 5;
const ARGS_LEN_IMPORT_INDEX: u32 = 6;
const MATH_MAX_IMPORT_INDEX: u32 = 7;
const MATH_MIN_IMPORT_INDEX: u32 = 8;
const MATH_ABS_IMPORT_INDEX: u32 = 9;
const MATH_SIGN_IMPORT_INDEX: u32 = 10;
const MATH_IMUL_IMPORT_INDEX: u32 = 11;
const MATH_ROUND_IMPORT_INDEX: u32 = 12;
const PROCESS_PID_IMPORT_INDEX: u32 = 13;
const CWD_IMPORT_INDEX: u32 = 14;
const MATH_CLZ32_IMPORT_INDEX: u32 = 15;
const MATH_POW_IMPORT_INDEX: u32 = 16;
const COVERAGE_HIT_IMPORT_INDEX: u32 = 17;
const FUNCTION_INDEX_OFFSET: u32 = 17;
const ENV_GET_BUFFER_RESERVED: u32 = 4096;
const STRING_HANDLE_TAG: u64 = 0x8000_0000_0000_0000;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn emit_coverage_hit(&mut self, function: &mut Function, coverage_id: Option<u32>) {
        if let Some(coverage_id) = coverage_id {
            function.instruction(&Instruction::I32Const(coverage_id as i32));
            function.instruction(&Instruction::Call(COVERAGE_HIT_IMPORT_INDEX));
        }
    }

    pub(crate) fn is_object_literal(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.is_empty() {
            return false;
        }

        node.children.iter().all(|child| {
            self.node(*child).children.len() == 2
                && self
                    .node(*child)
                    .text
                    .as_deref()
                    .is_some_and(|kind| matches!(kind, "init" | "get" | "set"))
                && self.node(self.node(*child).children[0]).kind == LirNodeKind::Literal
        })
    }

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

    pub(crate) fn object_literal_field(&self, node: &LirNode, field: &str) -> Option<LirNodeId> {
        if !self.is_object_literal(node) {
            return None;
        }

        let field = field.trim_matches('"');
        for child in &node.children {
            let property = self.node(*child);
            if property.children.len() != 2 {
                continue;
            }
            let key = self
                .node(property.children[0])
                .text
                .as_deref()
                .map(|value| value.trim_matches('"'))?;
            if key == field {
                return property.children.get(1).copied();
            }
        }

        None
    }

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

    pub(crate) fn console_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        let object = callee_node.children.first().copied()?;
        let object_name = self.node(object).text.as_deref()?;
        if object_name != "console" {
            return None;
        }

        match method {
            "log" => Some(CONSOLE_LOG_IMPORT_INDEX),
            "error" => Some(CONSOLE_ERROR_IMPORT_INDEX),
            "warn" => Some(CONSOLE_WARN_IMPORT_INDEX),
            "info" => Some(CONSOLE_INFO_IMPORT_INDEX),
            "debug" => Some(CONSOLE_DEBUG_IMPORT_INDEX),
            _ => None,
        }
    }

    pub(crate) fn is_console_assert(&self, callee_node: &LirNode) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("console") && method == "assert"
    }

    pub(crate) fn resolve_transparent_object_root_node(&self, id: LirNodeId) -> Option<LirNodeId> {
        let mut id = self.resolve_bound_node(id);
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = self.node(id);
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
            {
                id = node.children[0];
                continue;
            }

            if self.is_object_freeze_call(node) {
                id = node.children.get(1).copied()?;
                continue;
            }

            return Some(id);
        }
    }

    pub(crate) fn is_math_object(&self, callee_node: &LirNode) -> bool {
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Math")
                | Some("globalThis.Math")
                | Some(r#"globalThis["Math"]"#)
                | Some(r#"globalThis['Math']"#)
        )
    }

    pub(crate) fn is_object_identity_object(&self, callee_node: &LirNode) -> bool {
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Object")
                | Some("globalThis.Object")
                | Some(r#"globalThis["Object"]"#)
                | Some(r#"globalThis['Object']"#)
        )
    }

    pub(crate) fn is_number_object(&self, callee_node: &LirNode) -> bool {
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Number")
                | Some("globalThis.Number")
                | Some(r#"globalThis["Number"]"#)
                | Some(r#"globalThis['Number']"#)
        )
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

    pub(crate) fn resolve_static_global_number_predicate_call(
        &self,
        node: &LirNode,
        callee_node: &LirNode,
    ) -> Option<bool> {
        if node.kind != LirNodeKind::Call || node.children.len() < 2 {
            return None;
        }

        let callee_id = *node.children.first()?;
        let callee_node = self
            .resolve_transparent_callable_node(callee_id)
            .map(|id| self.node(id))
            .unwrap_or(callee_node);
        let method = self.global_number_predicate_callable_method(callee_node)?;
        let StaticObjectIdentityValue::Number(number) =
            self.resolve_static_object_identity_value(*node.children.get(1)?)?
        else {
            return None;
        };

        match method {
            "isFinite" => Some(number.is_finite()),
            "isNaN" => Some(number.is_nan()),
            _ => None,
        }
    }

    pub(crate) fn resolve_static_parse_int_call(&self, node: &LirNode, callee_node: &LirNode) -> Option<i64> {
        if node.kind != LirNodeKind::Call || !(2..=3).contains(&node.children.len()) {
            return None;
        }

        let callee_id = *node.children.first()?;
        let callee_node = self
            .resolve_transparent_callable_node(callee_id)
            .map(|id| self.node(id))
            .unwrap_or(callee_node);
        if !self.is_parse_int_callable(callee_node) {
            return None;
        }

        let source = match self.resolve_static_object_identity_value(*node.children.get(1)?)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let radix = match node.children.get(2) {
            Some(id) => {
                let radix = self.resolve_static_numeric_value(*id)?;
                if !radix.is_finite() || radix.fract() != 0.0 {
                    return None;
                }
                radix as u32
            }
            None => 0,
        };
        static_parse_int_ascii(&source, radix)
    }

    pub(crate) fn resolve_static_parse_float_call(
        &self,
        node: &LirNode,
        callee_node: &LirNode,
    ) -> Option<i64> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee_id = *node.children.first()?;
        let callee_node = self
            .resolve_transparent_callable_node(callee_id)
            .map(|id| self.node(id))
            .unwrap_or(callee_node);
        if !self.is_parse_float_callable(callee_node) {
            return None;
        }

        let source = match self.resolve_static_object_identity_value(*node.children.get(1)?)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        static_parse_float_ascii_integer(&source)
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

    pub(crate) fn global_number_predicate_callable_method<'b>(
        &self,
        callee_node: &'b LirNode,
    ) -> Option<&'b str> {
        let method = callee_node.text.as_deref()?;
        if method == "isFinite" || method == "isNaN" {
            if callee_node.children.is_empty() {
                return Some(method);
            }

            let object = callee_node.children.first().copied()?;
            let object = self.resolve_transparent_object_root_node(object)?;
            if matches!(self.node(object).text.as_deref(), Some("globalThis")) {
                return Some(method);
            }
        }

        None
    }

    pub(crate) fn is_parse_int_callable(&self, callee_node: &LirNode) -> bool {
        self.is_number_parse_callable(callee_node, "parseInt")
    }

    pub(crate) fn is_parse_float_callable(&self, callee_node: &LirNode) -> bool {
        self.is_number_parse_callable(callee_node, "parseFloat")
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

    pub(crate) fn is_number_parse_callable(&self, callee_node: &LirNode, expected: &str) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };

        if method == expected && callee_node.children.is_empty() {
            return true;
        }

        if method != expected {
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
            Some("Number")
                | Some("globalThis.Number")
                | Some(r#"globalThis["Number"]"#)
                | Some(r#"globalThis['Number']"#)
                | Some("globalThis")
        )
    }

    pub(crate) fn is_object_freeze_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let callee_node = self.node(callee);
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };

        matches!(
            callee_node.text.as_deref(),
            Some(text)
                if text == "freeze"
                    || text.ends_with(".freeze")
                    || text.ends_with(r#"["freeze"]"#)
                    || text.ends_with(r#"['freeze']"#)
        ) && matches!(
            self.node(object).text.as_deref(),
            Some("Object")
                | Some("globalThis.Object")
                | Some(r#"globalThis["Object"]"#)
                | Some(r#"globalThis['Object']"#)
        )
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
            && current.text.as_deref().is_none_or(|text| text.is_empty())
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

    pub(crate) fn is_identity_array_flat_map_expression(&self, id: LirNodeId, param_name: &str) -> bool {
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

    pub(crate) fn resolve_identity_array_callback_source(&self, node: &LirNode) -> Option<LirNodeId> {
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
            && node.text.as_deref().is_none_or(|text| text.is_empty())
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
            && node.text.as_deref().is_none_or(|text| text.is_empty())
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
                None | Some("") => {
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

    pub(crate) fn resolve_static_array_some_every_call(&self, node: &LirNode, method: &str) -> Option<bool> {
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

    pub(crate) fn resolve_static_array_search_call(&self, node: &LirNode, method: &str) -> Option<i64> {
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

    pub(crate) fn resolve_static_array_slice_element(&self, id: LirNodeId, index: usize) -> Option<LirNodeId> {
        let (source, start, end) = self.resolve_static_array_slice_bounds(self.node(id))?;
        let absolute_index = start.checked_add(index)?;
        if absolute_index >= end {
            return None;
        }
        self.node(source).children.get(absolute_index).copied()
    }

    pub(crate) fn resolve_static_string_search_call(&self, node: &LirNode, method: &str) -> Option<i64> {
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

    pub(crate) fn string_identity_call_method_with_literal_receiver(&self, node: &LirNode) -> Option<String> {
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

    pub(crate) fn resolve_static_string_at_call(&self, node: &LirNode) -> Option<StaticStringAtResult> {
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

    pub(crate) fn is_string_code_point_at_call_with_literal_receiver(&self, node: &LirNode) -> bool {
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

    pub(crate) fn resolve_static_string_replace_call(&self, node: &LirNode, method: &str) -> Option<String> {
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

    pub(crate) fn string_replace_call_method_with_literal_receiver(&self, node: &LirNode) -> Option<String> {
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

    pub(crate) fn resolve_static_string_split_parts_from_id(&self, mut id: LirNodeId) -> Option<Vec<String>> {
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

    pub(crate) fn resolve_static_array_at_call(&self, node: &LirNode) -> Option<StaticArrayAtResult> {
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

    pub(crate) fn resolve_static_array_filter_items(&self, node: &LirNode) -> Option<Vec<LirNodeId>> {
        let mut current = node;
        while current.kind == LirNodeKind::Value
            && current.children.len() == 1
            && current.text.as_deref().is_none_or(|text| text.is_empty())
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

    pub(crate) fn resolve_static_array_reduce_call(&self, node: &LirNode, method: &str) -> Option<i64> {
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

    pub(crate) fn resolve_static_numeric_reducer_callback(
        &self,
        callback: LirNodeId,
        accumulator: f64,
        current: f64,
    ) -> Option<f64> {
        let callback = self.resolve_transparent_callable_node(callback)?;
        let callback_node = self.node(callback);
        if callback_node.function_flavor != Some(FunctionFlavor::Sync)
            || callback_node.kind != LirNodeKind::Instruction
            || callback_node.children.len() < 3
        {
            return None;
        }

        let accumulator_name = self.node(callback_node.children[0]).text.as_deref()?;
        let current_name = self.node(callback_node.children[1]).text.as_deref()?;
        let body_node = self.node(*callback_node.children.last()?);
        let body_expr = body_node.children.first().copied()?;
        self.resolve_static_numeric_reducer_expr(
            body_expr,
            accumulator_name,
            current_name,
            accumulator,
            current,
        )
    }

    pub(crate) fn resolve_static_numeric_reducer_expr(
        &self,
        id: LirNodeId,
        accumulator_name: &str,
        current_name: &str,
        accumulator: f64,
        current: f64,
    ) -> Option<f64> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref().is_none_or(|text| text.is_empty())
        {
            return self.resolve_static_numeric_reducer_expr(
                node.children[0],
                accumulator_name,
                current_name,
                accumulator,
                current,
            );
        }

        match node.kind {
            LirNodeKind::Literal => node.text.as_deref().and_then(parse_numeric_literal_value),
            LirNodeKind::Value if node.children.is_empty() => match node.text.as_deref()? {
                name if name == accumulator_name => Some(accumulator),
                name if name == current_name => Some(current),
                _ => self.resolve_static_numeric_value(id),
            },
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                None | Some("") | Some("+") => self.resolve_static_numeric_reducer_expr(
                    node.children[0],
                    accumulator_name,
                    current_name,
                    accumulator,
                    current,
                ),
                Some("-") => self
                    .resolve_static_numeric_reducer_expr(
                        node.children[0],
                        accumulator_name,
                        current_name,
                        accumulator,
                        current,
                    )
                    .map(|value| -value),
                _ => None,
            },
            LirNodeKind::Value if node.children.len() == 2 => {
                let left = self.resolve_static_numeric_reducer_expr(
                    node.children[0],
                    accumulator_name,
                    current_name,
                    accumulator,
                    current,
                )?;
                let right = self.resolve_static_numeric_reducer_expr(
                    node.children[1],
                    accumulator_name,
                    current_name,
                    accumulator,
                    current,
                )?;
                match node.text.as_deref() {
                    Some("+") => Some(left + right),
                    Some("-") => Some(left - right),
                    Some("*") => Some(left * right),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_truthy_identity_array_filter_source(&self, node: &LirNode) -> Option<LirNodeId> {
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

    pub(crate) fn resolve_static_object_identity_value(
        &self,
        id: LirNodeId,
    ) -> Option<StaticObjectIdentityValue> {
        let node = self.node(id);
        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_static_object_identity_value(child));
        }
        match node.kind {
            LirNodeKind::Literal => match node.text.as_deref() {
                Some("true") => Some(StaticObjectIdentityValue::Boolean(true)),
                Some("false") => Some(StaticObjectIdentityValue::Boolean(false)),
                Some("null") => Some(StaticObjectIdentityValue::Null),
                Some("Infinity") => Some(StaticObjectIdentityValue::Number(f64::INFINITY)),
                Some("NaN") => Some(StaticObjectIdentityValue::Number(f64::NAN)),
                Some("void") => Some(StaticObjectIdentityValue::Undefined),
                Some(text) => text
                    .strip_suffix('n')
                    .and_then(|value| value.parse::<i64>().ok())
                    .map(StaticObjectIdentityValue::BigInt)
                    .or_else(|| {
                        parse_numeric_literal_value(text).map(StaticObjectIdentityValue::Number)
                    })
                    .or_else(|| {
                        Some(StaticObjectIdentityValue::String(
                            strip_string_delimiters(text).to_string(),
                        ))
                    }),
                None => None,
            },
            LirNodeKind::Value if node.children.len() == 2 => match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    if left.is_nullish() {
                        self.resolve_static_object_identity_value(node.children[1])
                    } else {
                        Some(left)
                    }
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => self.resolve_static_object_identity_value(node.children[1]),
                        Some(false) => Some(left),
                        None => {
                            let right =
                                self.resolve_static_object_identity_value(node.children[1])?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => Some(left),
                        Some(false) => self.resolve_static_object_identity_value(node.children[1]),
                        None => {
                            let right =
                                self.resolve_static_object_identity_value(node.children[1])?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.resolve_static_object_identity_value(bound);
                }
                match text {
                    "Infinity" => Some(StaticObjectIdentityValue::Number(f64::INFINITY)),
                    "NaN" => Some(StaticObjectIdentityValue::Number(f64::NAN)),
                    _ => parse_numeric_literal_value(text).map(StaticObjectIdentityValue::Number),
                }
            }
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                None | Some("") => self.resolve_static_object_identity_value(node.children[0]),
                Some("+") => match self.resolve_static_object_identity_value(node.children[0]) {
                    Some(StaticObjectIdentityValue::BigInt(_)) => None,
                    other => other,
                },
                Some("void") => Some(StaticObjectIdentityValue::Undefined),
                Some("-") => self
                    .resolve_static_object_identity_value(node.children[0])
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
                _ => None,
            },
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_static_numeric_value(&self, id: LirNodeId) -> Option<f64> {
        let node = self.node(id);
        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_static_numeric_value(child));
        }
        match node.kind {
            LirNodeKind::Literal => node.text.as_deref().and_then(parse_numeric_literal_value),
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.resolve_static_numeric_value(bound);
                }
                parse_numeric_literal_value(text)
            }
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                None | Some("") => self.resolve_static_numeric_value(node.children[0]),
                Some("+") => self.resolve_static_numeric_value(node.children[0]),
                Some("-") => self
                    .resolve_static_numeric_value(node.children[0])
                    .map(|value| if value == 0.0 { -0.0 } else { -value }),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn math_max_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "max" {
            Some(MATH_MAX_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_min_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "min" {
            Some(MATH_MIN_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_abs_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "abs" {
            Some(MATH_ABS_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_sign_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "sign" {
            Some(MATH_SIGN_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_imul_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "imul" {
            Some(MATH_IMUL_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_round_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "round" {
            Some(MATH_ROUND_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_clz32_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "clz32" {
            Some(MATH_CLZ32_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_pow_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "pow" {
            Some(MATH_POW_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_member_method<'b>(&self, callee_node: &'b LirNode) -> Option<&'b str> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) {
            Some(method)
        } else {
            None
        }
    }

    pub(crate) fn emit_integer_math_arg(
        &mut self,
        function: &mut Function,
        arg: LirNodeId,
        method: &str,
    ) -> bool {
        if self.contains_non_integer_numeric_literal(arg) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return false;
        }

        let _ = self.emit_node(function, arg, true);
        true
    }

    pub(crate) fn math_exp_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(1)
        } else {
            None
        }
    }

    pub(crate) fn math_log_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 1 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_exp2_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if !(0..=62).contains(&value) {
            return None;
        }

        Some(1_i64 << (value as u32))
    }

    pub(crate) fn math_expm1_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_log1p_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_fround_zero_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value == 0.0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_sin_cos_zero_constant_value(&self, method: &str, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cos" { 1 } else { 0 })
    }

    pub(crate) fn math_hyperbolic_zero_constant_value(&self, method: &str, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cosh" { 1 } else { 0 })
    }

    pub(crate) fn math_inverse_trig_constant_value(&self, method: &str, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;

        match method {
            "asin" | "atan" if value == 0.0 => Some(0),
            "acos" if value == 1.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn math_atan2_zero_slice_value(&self, y: LirNodeId, x: LirNodeId) -> Option<i64> {
        let y = self.render_static_value(y)?;
        let x = self.render_static_value(x)?;
        let y = parse_numeric_literal_value(&y)?;
        let x = parse_numeric_literal_value(&x)?;
        if y == 0.0 && x.is_finite() && x >= 0.0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_inverse_hyperbolic_constant_value(&self, method: &str, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;

        match method {
            "acosh" if value == 1.0 => Some(0),
            "asinh" | "atanh" if value == 0.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn math_sqrt_constant_root(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value < 0 {
            return None;
        }

        let root = (value as f64).sqrt() as i64;
        if root.checked_mul(root) == Some(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn math_cbrt_constant_root(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        let root = (value as f64).cbrt().round() as i64;
        if i128::from(root).pow(3) == i128::from(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn math_log2_constant_exponent(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value <= 0 {
            return None;
        }

        let value = value as u64;
        if value.is_power_of_two() {
            Some(i64::from(value.trailing_zeros()))
        } else {
            None
        }
    }

    pub(crate) fn math_log10_constant_exponent(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let mut value = parse_number_literal(&rendered)?;
        if value <= 0 {
            return None;
        }

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

    pub(crate) fn math_hypot_constant_root(&self, args: &[LirNodeId]) -> Option<i64> {
        if args.is_empty() {
            return Some(0);
        }

        let mut sum = 0_i128;
        for arg in args {
            let rendered = self.render_static_value(*arg)?;
            let value = parse_numeric_literal_value(&rendered)?;
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

        self.perfect_square_root_i128(sum)
    }

    pub(crate) fn math_round_like_static_literal_value(&self, method: &str, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        let folded = match method {
            "round" => {
                if value.fract() == 0.0 {
                    value
                } else {
                    (value + 0.5).floor()
                }
            }
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

    pub(crate) fn math_extrema_static_literal_value(&self, method: &str, args: &[LirNodeId]) -> Option<i64> {
        let mut values = args.iter().map(|arg| {
            let rendered = self.render_static_value(*arg)?;
            let value = parse_numeric_literal_value(&rendered)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }
            Some(value as i64)
        });

        let mut folded = values.next().flatten()?;
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

    pub(crate) fn math_abs_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        parse_number_literal(&rendered)?.checked_abs()
    }

    pub(crate) fn math_imul_static_literal_value(&self, left: LirNodeId, right: LirNodeId) -> Option<i64> {
        let rendered_left = self.render_static_value(left)?;
        let rendered_right = self.render_static_value(right)?;
        let left = parse_number_literal(&rendered_left)? as i32;
        let right = parse_number_literal(&rendered_right)? as i32;
        Some(i64::from(left.wrapping_mul(right)))
    }

    pub(crate) fn math_clz32_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        let uint32 = self.to_uint32_literal_value(value)?;
        Some(i64::from(uint32.leading_zeros()))
    }

    pub(crate) fn static_bigint_literal_value(&self, id: LirNodeId) -> Option<i64> {
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Literal {
            return None;
        }

        parse_number_literal(node.text.as_deref()?)
    }

    pub(crate) fn math_sign_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        Some(if value == 0.0 {
            0
        } else if value.is_sign_negative() {
            -1
        } else {
            1
        })
    }

    pub(crate) fn contains_negative_numeric_literal(&self, id: LirNodeId) -> bool {
        self.render_static_value(id)
            .and_then(|rendered| parse_number_literal(&rendered))
            .is_some_and(|value| value < 0)
    }

    pub(crate) fn to_uint32_literal_value(&self, value: f64) -> Option<u32> {
        if !value.is_finite() {
            return Some(0);
        }

        let truncated = value.trunc();
        let modulo = truncated.rem_euclid(4_294_967_296.0);
        Some(modulo as u32)
    }

    pub(crate) fn contains_non_integer_numeric_literal(&self, arg: LirNodeId) -> bool {
        self.render_static_value(arg)
            .and_then(|rendered| parse_numeric_literal_value(&rendered))
            .is_some_and(|value| value.fract() != 0.0)
    }

    pub(crate) fn env_set_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "set" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_set_import_index
    }

    pub(crate) fn env_delete_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "delete" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_delete_import_index
    }

    pub(crate) fn env_get_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "get" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_get_import_index
    }

    pub(crate) fn env_has_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "has" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_has_import_index
    }

    pub(crate) fn cwd_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "cwd" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !self.is_deno_pid(object) && !self.is_process_cwd(object) {
            return None;
        }

        Some(CWD_IMPORT_INDEX)
    }

    pub(crate) fn cwd_set_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "chdir" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !self.is_deno_pid(object) && !self.is_process_cwd(object) {
            return None;
        }

        self.cwd_set_import_index
    }

    pub(crate) fn process_exit_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "exit" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !self.is_process_exit(object) && !self.is_deno_exit(object) {
            return None;
        }

        self.process_exit_import_index
    }

    pub(crate) fn render_console_call(&self, node: &LirNode) -> Option<String> {
        let args = node.children.iter().skip(1).copied().collect::<Vec<_>>();
        self.render_console_arguments(&args)
    }

    pub(crate) fn render_console_arguments(&self, args: &[LirNodeId]) -> Option<String> {
        let mut rendered = Vec::new();
        for arg in args {
            rendered.push(self.render_static_value(*arg)?);
        }
        Some(rendered.join(" "))
    }

    pub(crate) fn render_static_value(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        match node.kind {
            LirNodeKind::Literal => match node.text.as_deref() {
                Some("true") => Some("true".to_string()),
                Some("false") => Some("false".to_string()),
                Some("null") => Some("null".to_string()),
                Some("undefined") => Some("undefined".to_string()),
                Some(text) => {
                    if parse_number_literal(text).is_some() {
                        Some(text.to_string())
                    } else {
                        Some(strip_string_delimiters(text).to_string())
                    }
                }
                None => Some("0".to_string()),
            },
            LirNodeKind::Call => {
                if self.is_object_freeze_call(node) {
                    return self.render_static_value(*node.children.get(1)?);
                }

                if let Some(result) = self.resolve_static_array_at_call(node) {
                    return match result {
                        StaticArrayAtResult::Value(value) => self.render_static_value(value),
                        StaticArrayAtResult::OutOfRange => Some("undefined".to_string()),
                    };
                }

                if let Some(result) = self.resolve_static_string_at_call(node) {
                    return match result {
                        StaticStringAtResult::Value(value) => Some(value),
                        StaticStringAtResult::OutOfRange => Some("undefined".to_string()),
                    };
                }

                if let Some(result) = self.resolve_static_string_code_point_at_call(node) {
                    return match result {
                        StaticStringAtResult::Value(value) => Some(value),
                        StaticStringAtResult::OutOfRange => Some("undefined".to_string()),
                    };
                }

                let callee = node.children.first().copied()?;
                let callee_node = self.node(callee);
                let callee_name = callee_node.text.as_deref()?;
                if callee_name == "require" {
                    if let Some(specifier) = self.render_static_value(*node.children.get(1)?) {
                        if let Some(version) = self.render_package_json_version(&specifier) {
                            return Some(version);
                        }
                    }
                }
                self.render_semver_intrinsic(callee_name, node)
            }
            LirNodeKind::Value => {
                if node.children.is_empty() {
                    let text = node.text.as_deref()?;
                    if let Some(bound) = self.bindings.get(text).copied() {
                        return self.render_static_value(bound);
                    }
                    if let Some(index) = self.locals.get(text).copied() {
                        return Some(index.to_string());
                    }
                    if let Some(number) = parse_number_literal(text) {
                        return Some(number.to_string());
                    }
                    if parse_numeric_literal_value(text).is_some() {
                        return Some(text.to_string());
                    }
                    match text {
                        "true" | "false" | "null" | "undefined" => Some(text.to_string()),
                        _ => None,
                    }
                } else if node.children.len() == 1
                    && matches!(node.text.as_deref(), Some("+") | Some("-"))
                {
                    let rendered = self.render_static_value(node.children[0])?;
                    if let Some(value) = parse_number_literal(&rendered) {
                        Some(if node.text.as_deref() == Some("-") {
                            (-value).to_string()
                        } else {
                            value.to_string()
                        })
                    } else {
                        let value = parse_numeric_literal_value(&rendered)?;
                        Some(if node.text.as_deref() == Some("-") {
                            (-value).to_string()
                        } else {
                            value.to_string()
                        })
                    }
                } else if node.text.as_deref().is_some_and(|text| text == "length") {
                    if self.is_process_argv(node.children[0]) {
                        None
                    } else {
                        self.render_length(&node.children[0])
                    }
                } else if let Some(result) = self.resolve_static_index_member(node) {
                    match result {
                        StaticIndexMemberResult::Node(value) => self.render_static_value(value),
                        StaticIndexMemberResult::String(value) => Some(value),
                        StaticIndexMemberResult::Undefined => Some("undefined".to_string()),
                    }
                } else if node.text.is_none() {
                    if node.children.len() == 1 {
                        self.render_static_value(node.children[0])
                    } else {
                        Some(node.children.len().to_string())
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn has_semver_import(&self) -> bool {
        self.program
            .nodes
            .iter()
            .any(|node| node.text.as_deref() == Some("semver"))
    }

    pub(crate) fn render_semver_intrinsic(&self, callee_name: &str, node: &LirNode) -> Option<String> {
        if !self.has_semver_import() {
            return None;
        }

        match callee_name {
            "valid" => {
                let arg = *node.children.get(1)?;
                let version = self.render_static_value(arg)?;
                Version::parse(&version)
                    .ok()
                    .map(|parsed| parsed.to_string())
            }
            "satisfies" => {
                let version = self.render_static_value(*node.children.get(1)?)?;
                let range = self.render_static_value(*node.children.get(2)?)?;
                let version = Version::parse(&version).ok()?;
                let range = VersionReq::parse(&range).ok()?;
                Some(if range.matches(&version) { "1" } else { "0" }.to_string())
            }
            "minVersion" => {
                let range = self.render_static_value(*node.children.get(1)?)?;
                semver_min_version(&range)
            }
            _ => None,
        }
    }

    pub(crate) fn render_package_json_version(&self, specifier: &str) -> Option<String> {
        let source_path = self.source_path.as_ref()?;
        let package_json_path = source_path
            .parent()?
            .join(strip_string_delimiters(specifier));
        if package_json_path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
            return None;
        }

        let raw = std::fs::read_to_string(package_json_path).ok()?;
        let package_json: serde_json::Value = serde_json::from_str(&raw).ok()?;
        package_json
            .get("version")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
    }

    pub(crate) fn render_package_json_version_access(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        if self.node(callee).text.as_deref() != Some("require") {
            return None;
        }

        let specifier = self.render_static_value(*node.children.get(1)?)?;
        self.render_package_json_version(&specifier)
    }

    pub(crate) fn is_deno_pid(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.text.as_deref() == Some("Deno") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("Deno"))
    }

    pub(crate) fn is_deno_exit(&self, id: LirNodeId) -> bool {
        self.is_deno_pid(id)
    }

    pub(crate) fn is_process_pid(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.text.as_deref() == Some("process") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    pub(crate) fn is_process_cwd(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if node.text.as_deref() == Some("process") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    pub(crate) fn is_process_exit(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .is_some_and(|child| self.is_process_exit(child));
        }

        if node.text.as_deref() == Some("process") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    pub(crate) fn is_process_kill(&self, callee_node: &LirNode) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };
        if method != "kill" {
            return false;
        }

        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        if !self.is_process_exit(object) {
            return false;
        }

        true
    }

    pub(crate) fn is_process_argv(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if node.text.as_deref() != Some("argv") || node.children.len() != 1 {
            return false;
        }

        let object = self.unwrap_transparent_value_node(node.children[0]);
        let object = self.node(object);
        if object.text.as_deref() == Some("process") {
            return true;
        }

        object.text.as_deref() == Some("globalThis")
            && object
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    pub(crate) fn is_deno_args(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if node.text.as_deref() != Some("args") || node.children.len() != 1 {
            return false;
        }

        let object = self.unwrap_transparent_value_node(node.children[0]);
        let object = self.node(object);
        if object.text.as_deref() == Some("Deno") {
            return true;
        }

        object.text.as_deref() == Some("globalThis")
            && object
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("Deno"))
    }

    pub(crate) fn process_argv_slice_start(&self, id: LirNodeId) -> Option<i64> {
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("slice") {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !(self.is_process_argv(object) || self.is_deno_args(object)) {
            return None;
        }

        let start = *node.children.get(1)?;
        let start_node = self.node(start);
        parse_number_literal(start_node.text.as_deref()?)
    }

    pub(crate) fn render_length(&self, id: &LirNodeId) -> Option<String> {
        if self.process_argv_slice_start(*id).is_some() {
            return None;
        }

        if let Some(parts) = self.resolve_static_string_split_parts_from_id(*id) {
            return Some(parts.len().to_string());
        }

        if let Some(StaticObjectIdentityValue::String(value)) =
            self.resolve_static_object_identity_value(*id)
        {
            return Some(value.encode_utf16().count().to_string());
        }

        let node = self.node(*id);
        if node.text.is_none() {
            return Some(node.children.len().to_string());
        }

        if node.children.is_empty() {
            if let Some(text) = node.text.as_deref() {
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.render_length(&bound);
                }
                return Some("0".to_string());
            }
        }

        if node.children.len() == 1 {
            self.render_length(&node.children[0])
        } else {
            Some(node.children.len().to_string())
        }
    }

    pub(crate) fn is_kali_test_call(&self, callee_node: &LirNode) -> bool {
        if callee_node.text.as_deref() != Some("test") {
            return false;
        }

        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("Kali")
    }

    pub(crate) fn kali_test_callback_index(&self, node: &LirNode) -> Option<u32> {
        let callback_node = node.children.get(2).copied()?;
        let callback_name = self.node(callback_node).text.as_deref()?;
        self.functions.get(callback_name).copied()
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
        if array.kind == LirNodeKind::Value && array.text.is_none() && array.children.len() == 1 {
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

    pub(crate) fn is_object_has_own_call(&self, node: &LirNode, callee_node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let receiver_text = callee_node
            .children
            .first()
            .and_then(|receiver| self.node(*receiver).text.as_deref())
            .unwrap_or_default();
        match callee_node.text.as_deref() {
            Some(text)
                if text == "hasOwn"
                    || text.ends_with(".hasOwn")
                    || text.ends_with("[\"hasOwn\"]")
                    || text.ends_with("['hasOwn']")
                    || text == "Object.hasOwn"
                    || text == "Object[\"hasOwn\"]"
                    || text == "Object['hasOwn']"
                    || text == "globalThis.Object.hasOwn"
                    || text == "globalThis.Object[\"hasOwn\"]"
                    || text == "globalThis.Object['hasOwn']"
                    || text == r#"globalThis["Object"].hasOwn"#
                    || text == r#"globalThis["Object"]["hasOwn"]"#
                    || text == r#"globalThis["Object"]['hasOwn']"#
                    || text == r#"globalThis['Object'].hasOwn"#
                    || text == r#"globalThis['Object']['hasOwn']"#
                    || text == r#"globalThis['Object']["hasOwn"]"# =>
            {
                true
            }
            Some("call") if receiver_text.contains("hasOwnProperty") => true,
            _ => false,
        }
    }

    pub(crate) fn is_object_from_entries_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let callee_node = self.node(callee);
        let _receiver_text = callee_node
            .children
            .first()
            .and_then(|receiver| self.node(*receiver).text.as_deref())
            .unwrap_or_default();
        matches!(
            callee_node.text.as_deref(),
            Some(text)
                if text == "fromEntries"
                    || text.ends_with(".fromEntries")
                    || text.ends_with("[\"fromEntries\"]")
                    || text.ends_with("['fromEntries']")
                    || text == r#"globalThis["Object"]["fromEntries"]"#
                    || text == r#"globalThis["Object"]['fromEntries']"#
                    || text == r#"globalThis['Object']["fromEntries"]"#
                    || text == r#"globalThis['Object']['fromEntries']"#
        )
    }

    pub(crate) fn static_object_has_own(&self, object_id: LirNodeId, key: &str) -> Option<bool> {
        let object_id = self.resolve_literal_aggregate(object_id)?;
        let object = self.node(object_id);
        if self.is_object_literal(object) {
            return Some(self.object_literal_field(object, key).is_some());
        }

        if self.is_object_from_entries_call(object) {
            return self.static_object_from_entries_has_key(object, key);
        }

        None
    }

    pub(crate) fn static_object_from_entries_has_key(&self, call: &LirNode, key: &str) -> Option<bool> {
        let entries_id = call.children.get(1).copied()?;
        let entries_id = self.resolve_literal_aggregate(entries_id)?;
        let entries_node = self.node(entries_id);
        if !self.is_array_literal(entries_node) {
            return None;
        }

        for entry_id in &entries_node.children {
            let entry_id = self.resolve_literal_aggregate(*entry_id)?;
            let entry_node = self.node(entry_id);
            if !self.is_array_literal(entry_node) || entry_node.children.len() != 2 {
                return None;
            }

            let rendered_key = self.render_static_value(entry_node.children[0])?;
            if rendered_key == key {
                return Some(true);
            }
        }

        Some(false)
    }

    pub(crate) fn is_object_enumeration_call(&self, node: &LirNode) -> Option<ObjectEnumerationMode> {
        let node = if node.kind == LirNodeKind::Value && node.children.len() == 1 {
            self.node(node.children[0])
        } else {
            node
        };

        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee = self.resolve_literal_aggregate(callee).unwrap_or(callee);
        let callee_node = self.node(callee);
        let mode = match callee_node.text.as_deref() {
            Some(text)
                if text == "keys"
                    || text.ends_with(".keys")
                    || text.ends_with("[\"keys\"]")
                    || text.ends_with("['keys']")
                    || text == "Object.keys"
                    || text == "Object[\"keys\"]"
                    || text == "Object['keys']"
                    || text == "globalThis.Object.keys"
                    || text == "globalThis.Object[\"keys\"]"
                    || text == "globalThis.Object['keys']"
                    || text == r#"globalThis["Object"].keys"#
                    || text == r#"globalThis["Object"]["keys"]"#
                    || text == r#"globalThis["Object"]['keys']"#
                    || text == r#"globalThis['Object'].keys"#
                    || text == r#"globalThis['Object']['keys']"#
                    || text == r#"globalThis['Object']["keys"]"# =>
            {
                ObjectEnumerationMode::Keys
            }
            Some(text)
                if text == "ownKeys"
                    || text.ends_with(".ownKeys")
                    || text.ends_with("[\"ownKeys\"]")
                    || text.ends_with("['ownKeys']")
                    || text == "Reflect.ownKeys"
                    || text == "Reflect[\"ownKeys\"]"
                    || text == "Reflect['ownKeys']"
                    || text == "globalThis.Reflect.ownKeys"
                    || text == "globalThis.Reflect[\"ownKeys\"]"
                    || text == "globalThis.Reflect['ownKeys']"
                    || text == r#"globalThis["Reflect"].ownKeys"#
                    || text == r#"globalThis["Reflect"]["ownKeys"]"#
                    || text == r#"globalThis["Reflect"]['ownKeys']"#
                    || text == r#"globalThis['Reflect'].ownKeys"#
                    || text == r#"globalThis['Reflect']['ownKeys']"#
                    || text == r#"globalThis['Reflect']["ownKeys"]"# =>
            {
                ObjectEnumerationMode::ReflectOwnKeys
            }
            Some(text)
                if text == "values"
                    || text.ends_with(".values")
                    || text.ends_with("[\"values\"]")
                    || text.ends_with("['values']")
                    || text == "Object.values"
                    || text == "Object[\"values\"]"
                    || text == "Object['values']"
                    || text == "globalThis.Object.values"
                    || text == "globalThis.Object[\"values\"]"
                    || text == "globalThis.Object['values']"
                    || text == r#"globalThis["Object"].values"#
                    || text == r#"globalThis["Object"]["values"]"#
                    || text == r#"globalThis["Object"]['values']"#
                    || text == r#"globalThis['Object'].values"#
                    || text == r#"globalThis['Object']['values']"#
                    || text == r#"globalThis['Object']["values"]"# =>
            {
                ObjectEnumerationMode::Values
            }
            Some(text)
                if text == "entries"
                    || text.ends_with(".entries")
                    || text.ends_with("[\"entries\"]")
                    || text.ends_with("['entries']")
                    || text == "Object.entries"
                    || text == "Object[\"entries\"]"
                    || text == "Object['entries']"
                    || text == "globalThis.Object.entries"
                    || text == "globalThis.Object[\"entries\"]"
                    || text == "globalThis.Object['entries']"
                    || text == r#"globalThis["Object"].entries"#
                    || text == r#"globalThis["Object"]["entries"]"#
                    || text == r#"globalThis["Object"]['entries']"#
                    || text == r#"globalThis['Object'].entries"#
                    || text == r#"globalThis['Object']['entries']"#
                    || text == r#"globalThis['Object']["entries"]"#
                    || text == r#"globalThis["Object"]['entries']"# =>
            {
                ObjectEnumerationMode::Entries
            }
            _ => return None,
        };

        let object = callee_node.children.first().copied()?;
        let object = self.resolve_transparent_object_root_node(object)?;
        let object_text = self.node(object).text.as_deref().unwrap_or_default();
        if object_text.contains("Object") || object_text.contains("Reflect") {
            Some(mode)
        } else {
            None
        }
    }

    pub(crate) fn collect_object_enumeration_iteration_items(
        &mut self,
        node: &LirNode,
        mode: ObjectEnumerationMode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        if let Some(string_text) = self.render_static_string_value(node) {
            if matches!(mode, ObjectEnumerationMode::ReflectOwnKeys) {
                return false;
            }
            for (index, value) in string_text.chars().enumerate() {
                let key = self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(format!("{index:?}")),
                    vec![],
                );
                let value = self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(format!("{value:?}")),
                    vec![],
                );
                match mode {
                    ObjectEnumerationMode::Keys | ObjectEnumerationMode::ReflectOwnKeys => {
                        items.push(key)
                    }
                    ObjectEnumerationMode::Values => items.push(value),
                    ObjectEnumerationMode::Entries => {
                        let pair =
                            self.alloc_scratch_node(LirNodeKind::Value, None, vec![key, value]);
                        items.push(pair);
                    }
                }
            }

            return true;
        }

        if self.is_object_literal(node) {
            for child in &node.children {
                let property = self.node(*child);
                if property.children.len() != 2 {
                    return false;
                }

                let key = property.children[0];
                let key_node = self.node(key);
                if key_node.kind != LirNodeKind::Literal || key_node.text.is_none() {
                    return false;
                }

                match mode {
                    ObjectEnumerationMode::Keys | ObjectEnumerationMode::ReflectOwnKeys => {
                        items.push(key)
                    }
                    ObjectEnumerationMode::Values => items.push(property.children[1]),
                    ObjectEnumerationMode::Entries => {
                        let pair = self.alloc_scratch_node(
                            LirNodeKind::Value,
                            None,
                            vec![key, property.children[1]],
                        );
                        items.push(pair);
                    }
                }
            }

            return true;
        }

        if self.is_object_from_entries_call(node) {
            return self.collect_object_from_entries_iteration_items(node, mode, items);
        }

        false
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

    pub(crate) fn collect_object_from_entries_iteration_items(
        &mut self,
        node: &LirNode,
        mode: ObjectEnumerationMode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        let Some(entries_id) = node.children.get(1).copied() else {
            return false;
        };
        let Some(entries_id) = self.resolve_literal_aggregate(entries_id) else {
            return false;
        };
        let entries_node = self.node(entries_id).clone();
        if !self.is_array_literal(&entries_node) {
            return false;
        }

        let mut ordered = Vec::with_capacity(entries_node.children.len());
        for entry_id in &entries_node.children {
            let Some(entry_id) = self.resolve_literal_aggregate(*entry_id) else {
                return false;
            };
            let entry_node = self.node(entry_id).clone();
            if !self.is_array_literal(&entry_node) || entry_node.children.len() != 2 {
                return false;
            }

            let Some(key_text) = self.render_static_value(entry_node.children[0]) else {
                return false;
            };
            let value_id = entry_node.children[1];
            if let Some((_, existing_value)) = ordered
                .iter_mut()
                .find(|(existing_key, _)| existing_key == &key_text)
            {
                *existing_value = value_id;
            } else {
                ordered.push((key_text, value_id));
            }
        }

        for (key_text, value_id) in ordered {
            match mode {
                ObjectEnumerationMode::Keys | ObjectEnumerationMode::ReflectOwnKeys => {
                    items.push(self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some(format!("{key_text:?}")),
                        vec![],
                    ))
                }
                ObjectEnumerationMode::Values => items.push(value_id),
                ObjectEnumerationMode::Entries => {
                    let key = self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some(format!("{key_text:?}")),
                        vec![],
                    );
                    let pair =
                        self.alloc_scratch_node(LirNodeKind::Value, None, vec![key, value_id]);
                    items.push(pair);
                }
            }
        }

        true
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

    pub(crate) fn is_set_constructor_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        matches!(
            self.node(callee).text.as_deref(),
            Some("Set")
                | Some("globalThis.Set")
                | Some(r#"globalThis["Set"]"#)
                | Some(r#"globalThis['Set']"#)
        )
    }

    pub(crate) fn resolve_set_constructor_call<'b>(&'b self, node: &'b LirNode) -> Option<&'b LirNode> {
        if self.is_set_constructor_call(node) {
            return Some(node);
        }

        if self.is_object_freeze_call(node) {
            let argument = node.children.get(1).copied()?;
            return self.resolve_set_constructor_call(self.node(argument));
        }

        if node.kind == LirNodeKind::Value && node.text.is_none() && node.children.len() == 1 {
            return self.resolve_set_constructor_call(self.node(node.children[0]));
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    let selected = if left.is_nullish() {
                        node.children[1]
                    } else {
                        node.children[0]
                    };
                    return self.resolve_set_constructor_call(self.node(selected));
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_set_constructor_call(self.node(node.children[1]));
                        }
                        Some(false) => {
                            return self.resolve_set_constructor_call(self.node(node.children[0]));
                        }
                        None => {
                            let consequent =
                                self.resolve_set_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_set_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_set_constructor_call(self.node(node.children[0]));
                        }
                        Some(false) => {
                            return self.resolve_set_constructor_call(self.node(node.children[1]));
                        }
                        None => {
                            let consequent =
                                self.resolve_set_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_set_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 3 {
            if let Some(test) = self.resolve_static_object_identity_value(node.children[0]) {
                let selected = if test.truthiness() == Some(true) {
                    node.children[1]
                } else {
                    node.children[2]
                };
                return self.resolve_set_constructor_call(self.node(selected));
            }

            let consequent = self.resolve_set_constructor_call(self.node(node.children[1]));
            let alternate = self.resolve_set_constructor_call(self.node(node.children[2]));
            if consequent.is_some() && consequent == alternate {
                return consequent;
            }
        }

        None
    }

    pub(crate) fn collect_set_constructor_iteration_items(
        &mut self,
        node: &LirNode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        let mut seen = HashSet::new();
        if let Some(set_call) = self.resolve_set_constructor_call(node) {
            let Some(source_arg) = set_call.children.get(1).copied() else {
                return false;
            };
            let Some(source_id) = self.resolve_literal_aggregate(source_arg) else {
                return false;
            };
            let source = self.node(source_id).clone();
            return self.collect_set_constructor_iteration_items(&source, items);
        }

        if let Some(string_text) = self.render_static_string_value(node) {
            for value in string_text.chars() {
                let item = self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(format!("{value:?}")),
                    vec![],
                );
                let Some(key) = self.static_set_item_key(item) else {
                    return false;
                };
                if seen.insert(key) {
                    items.push(item);
                }
            }
            return true;
        }

        if self.is_array_literal(node) {
            let mut collected = Vec::new();
            for child in &node.children {
                if !self.collect_for_of_array_iteration_items(*child, &mut collected) {
                    return false;
                }
            }

            for item in collected {
                let Some(key) = self.static_set_item_key(item) else {
                    return false;
                };
                if seen.insert(key) {
                    items.push(item);
                }
            }
            return true;
        }

        if self.resolve_set_constructor_call(node).is_some() {
            return self.collect_set_constructor_iteration_items(node, items);
        }

        if self.resolve_map_constructor_call(node).is_some() {
            return self.collect_map_constructor_iteration_items(node, items);
        }

        if let Some(object_enumeration_mode) = self.is_object_enumeration_call(node) {
            if matches!(object_enumeration_mode, ObjectEnumerationMode::Entries) {
                return false;
            }

            let Some(object_arg) = node.children.get(1).copied() else {
                return false;
            };
            let Some(object_id) = self.resolve_literal_aggregate(object_arg) else {
                return false;
            };
            let object = self.node(object_id).clone();
            let mut collected = Vec::new();
            if !self.collect_object_enumeration_iteration_items(
                &object,
                object_enumeration_mode,
                &mut collected,
            ) {
                return false;
            }

            for item in collected {
                let Some(key) = self.static_set_item_key(item) else {
                    return false;
                };
                if seen.insert(key) {
                    items.push(item);
                }
            }
            return true;
        }

        false
    }

    pub(crate) fn is_map_constructor_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        matches!(
            self.node(callee).text.as_deref(),
            Some("Map")
                | Some("globalThis.Map")
                | Some(r#"globalThis["Map"]"#)
                | Some(r#"globalThis['Map']"#)
        )
    }

    pub(crate) fn resolve_map_constructor_call<'b>(&'b self, node: &'b LirNode) -> Option<&'b LirNode> {
        if self.is_map_constructor_call(node) {
            return Some(node);
        }

        if self.is_object_freeze_call(node) {
            let argument = node.children.get(1).copied()?;
            return self.resolve_map_constructor_call(self.node(argument));
        }

        if node.kind == LirNodeKind::Value && node.text.is_none() && node.children.len() == 1 {
            return self.resolve_map_constructor_call(self.node(node.children[0]));
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    let selected = if left.is_nullish() {
                        node.children[1]
                    } else {
                        node.children[0]
                    };
                    return self.resolve_map_constructor_call(self.node(selected));
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_map_constructor_call(self.node(node.children[1]));
                        }
                        Some(false) => {
                            return self.resolve_map_constructor_call(self.node(node.children[0]));
                        }
                        None => {
                            let consequent =
                                self.resolve_map_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_map_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_map_constructor_call(self.node(node.children[0]));
                        }
                        Some(false) => {
                            return self.resolve_map_constructor_call(self.node(node.children[1]));
                        }
                        None => {
                            let consequent =
                                self.resolve_map_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_map_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 3 {
            if let Some(test) = self.resolve_static_object_identity_value(node.children[0]) {
                let selected = if test.truthiness() == Some(true) {
                    node.children[1]
                } else {
                    node.children[2]
                };
                return self.resolve_map_constructor_call(self.node(selected));
            }

            let consequent = self.resolve_map_constructor_call(self.node(node.children[1]));
            let alternate = self.resolve_map_constructor_call(self.node(node.children[2]));
            if consequent.is_some() && consequent == alternate {
                return consequent;
            }
        }

        None
    }

    pub(crate) fn collect_map_constructor_iteration_items(
        &mut self,
        node: &LirNode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        if let Some(map_call) = self.resolve_map_constructor_call(node) {
            let Some(source_arg) = map_call.children.get(1).copied() else {
                return false;
            };
            let Some(source_id) = self.resolve_literal_aggregate(source_arg) else {
                return false;
            };
            let source = self.node(source_id).clone();
            return self.collect_map_constructor_iteration_items(&source, items);
        }

        let mut collected = Vec::<(String, LirNodeId)>::new();
        if self.is_array_literal(node) {
            for child in &node.children {
                let Some(resolved_child) = self.resolve_literal_aggregate(*child) else {
                    return false;
                };
                let entry = self.node(resolved_child).clone();
                if !self.is_array_literal(&entry) || entry.children.len() < 2 {
                    return false;
                }
                if !entry
                    .children
                    .iter()
                    .take(2)
                    .all(|child| self.is_supported_for_of_array_iteration_item(*child))
                {
                    return false;
                }
                let Some(key) = self.static_map_entry_key(resolved_child) else {
                    return false;
                };
                if let Some((_, existing_entry)) = collected
                    .iter_mut()
                    .find(|(existing_key, _)| existing_key == &key)
                {
                    *existing_entry = resolved_child;
                } else {
                    collected.push((key, resolved_child));
                }
            }

            items.extend(collected.into_iter().map(|(_, id)| id));
            return true;
        }

        false
    }

    pub(crate) fn static_map_entry_key(&self, id: LirNodeId) -> Option<String> {
        let resolved_id = self.resolve_literal_aggregate(id)?;
        let node = self.node(resolved_id);
        if !self.is_array_literal(node) {
            return None;
        }
        let key = node.children.first().copied()?;
        let resolved_key = self.resolve_literal_aggregate(key)?;
        self.render_static_value(resolved_key)
    }

    pub(crate) fn static_set_item_key(&self, id: LirNodeId) -> Option<String> {
        let resolved_id = self.resolve_literal_aggregate(id)?;
        let node = self.node(resolved_id);
        if node.kind == LirNodeKind::Value && !node.children.is_empty() {
            return None;
        }
        self.render_static_value(resolved_id)
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

pub(crate) fn generator_lowering_unavailable_message(function_plans: &[FunctionPlan]) -> &'static str {
    let has_generator = function_plans
        .iter()
        .any(|plan| matches!(plan.flavor, Some(FunctionFlavor::Generator)));
    let has_async_generator = function_plans
        .iter()
        .any(|plan| matches!(plan.flavor, Some(FunctionFlavor::AsyncGenerator)));

    kali_common::generator_function_lowering_unavailable_message_for_flavors(
        has_generator,
        has_async_generator,
    )
}

/// Generate WASM from LIR.
pub fn lower_lir_to_wasm(ctx: &mut CodegenCtx, lir: &LirProgram) -> CodegenResult {
    let mut diagnostics = Vec::new();
    let function_plans = collect_functions(lir);
    if function_plans.iter().any(|plan| {
        matches!(
            plan.flavor,
            Some(FunctionFlavor::Generator | FunctionFlavor::AsyncGenerator)
        )
    }) {
        diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            generator_lowering_unavailable_message(&function_plans),
        ));
        return CodegenResult {
            wasm_bytes: Vec::new(),
            diagnostics,
        };
    }
    let mut function_name_to_index = BTreeMap::new();
    let mut string_pool = StringPool::new(ENV_GET_BUFFER_RESERVED);
    let uses_env_get = program_uses_env_get(lir);
    let uses_env_has = program_uses_env_has(lir);
    let uses_env_set = program_uses_env_set(lir);
    let uses_env_delete = program_uses_env_delete(lir);
    let uses_cwd_set = program_uses_cwd_set(lir);
    let uses_process_exit = program_uses_process_exit(lir);
    let uses_env_access = uses_env_get || uses_env_has || uses_env_set || uses_env_delete;
    let function_index_offset = FUNCTION_INDEX_OFFSET
        + if ctx.target.coverage { 1 } else { 0 }
        + if uses_env_set { 1 } else { 0 }
        + if uses_env_delete { 1 } else { 0 }
        + if uses_env_get { 1 } else { 0 }
        + if uses_env_has { 1 } else { 0 }
        + if uses_cwd_set { 1 } else { 0 }
        + if uses_process_exit { 1 } else { 0 };
    let env_get_type_index = if uses_env_access { Some(6) } else { None };
    let env_has_type_index = if uses_env_has { Some(7) } else { None };
    let cwd_set_type_index = if uses_cwd_set { Some(5) } else { None };
    let env_set_import_index = if uses_env_set {
        Some(COVERAGE_HIT_IMPORT_INDEX + if ctx.target.coverage { 1 } else { 0 })
    } else {
        None
    };
    let env_delete_import_index = if uses_env_delete {
        Some(
            COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 },
        )
    } else {
        None
    };
    let env_get_import_index = if uses_env_get {
        Some(
            COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 },
        )
    } else {
        None
    };
    let env_has_import_index = if uses_env_has {
        Some(
            COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 },
        )
    } else {
        None
    };
    let cwd_set_import_index = if uses_cwd_set {
        Some(
            COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 },
        )
    } else {
        None
    };
    let process_exit_import_index = if uses_process_exit {
        Some(
            COVERAGE_HIT_IMPORT_INDEX
                + if ctx.target.coverage { 1 } else { 0 }
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 },
        )
    } else {
        None
    };

    // Keep the emitted order deterministic: imported registration hook first, synthetic entry
    // second, then named functions in source order.
    let mut all_functions = Vec::new();
    all_functions.push(FunctionPlan {
        name: "_start".to_string(),
        params: Vec::new(),
        locals: collect_function_locals(&lir.nodes, lir.root),
        body: lir.root,
        result: false,
        is_entry: true,
        flavor: None,
    });
    all_functions.extend(function_plans);

    for (idx, function) in all_functions.iter().enumerate() {
        function_name_to_index.insert(function.name.clone(), idx as u32 + function_index_offset);
    }

    let mut type_section = TypeSection::new();
    type_section.ty().function(vec![ValType::I32], Vec::new());
    type_section.ty().function(vec![ValType::I64], Vec::new());
    type_section.ty().function(Vec::new(), vec![ValType::I32]);
    type_section
        .ty()
        .function(vec![ValType::I64, ValType::I64], vec![ValType::I64]);
    type_section
        .ty()
        .function(vec![ValType::I64], vec![ValType::I64]);
    type_section
        .ty()
        .function(vec![ValType::I64], vec![ValType::I32]);
    type_section.ty().function(
        vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        vec![ValType::I32],
    );
    type_section
        .ty()
        .function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);
    let mut import_section = ImportSection::new();
    import_section.import("kali:rt", "test_register", EntityType::Function(0));
    import_section.import("kali:rt", "console_log", EntityType::Function(1));
    import_section.import("kali:rt", "console_error", EntityType::Function(1));
    import_section.import("kali:rt", "console_warn", EntityType::Function(1));
    import_section.import("kali:rt", "console_info", EntityType::Function(1));
    import_section.import("kali:rt", "console_debug", EntityType::Function(1));
    import_section.import("kali:rt", "args_len", EntityType::Function(2));
    import_section.import("kali:rt", "math_max", EntityType::Function(3));
    import_section.import("kali:rt", "math_min", EntityType::Function(3));
    import_section.import("kali:rt", "math_abs", EntityType::Function(4));
    import_section.import("kali:rt", "math_sign", EntityType::Function(4));
    import_section.import("kali:rt", "math_imul", EntityType::Function(3));
    import_section.import("kali:rt", "math_round", EntityType::Function(4));
    import_section.import("kali:rt", "process_pid", EntityType::Function(2));
    import_section.import("kali:rt", "cwd", EntityType::Function(6));
    import_section.import("kali:rt", "math_clz32", EntityType::Function(4));
    import_section.import("kali:rt", "math_pow", EntityType::Function(3));
    if ctx.target.coverage {
        import_section.import("kali:rt", "coverage_hit", EntityType::Function(0));
    }
    if env_set_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_set",
            EntityType::Function(env_get_type_index.unwrap()),
        );
    }
    if env_delete_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_delete",
            EntityType::Function(env_get_type_index.unwrap()),
        );
    }
    if env_get_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_get",
            EntityType::Function(env_get_type_index.unwrap()),
        );
    }
    if env_has_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "env_has",
            EntityType::Function(env_has_type_index.unwrap()),
        );
    }
    if cwd_set_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "cwd_set",
            EntityType::Function(cwd_set_type_index.unwrap()),
        );
    }
    if process_exit_import_index.is_some() {
        import_section.import("kali:rt", "process_exit", EntityType::Function(1));
    }
    let mut function_types = BTreeMap::<(usize, bool), u32>::new();
    let mut type_for_function = Vec::with_capacity(all_functions.len());

    for function in &all_functions {
        let key = (function.params.len(), function.result);
        let type_index = if let Some(&idx) = function_types.get(&key) {
            idx
        } else {
            let idx = function_types.len() as u32 + 8;
            let params = vec![ValType::I64; function.params.len()];
            let results = if function.result {
                vec![ValType::I64]
            } else {
                Vec::new()
            };
            type_section.ty().function(params, results);
            function_types.insert(key, idx);
            idx
        };
        type_for_function.push(type_index);
    }

    let mut function_section = FunctionSection::new();
    for type_index in &type_for_function {
        function_section.function(*type_index);
    }

    let mut memory_section = MemorySection::new();
    memory_section.memory(MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });

    let mut export_section = ExportSection::new();
    export_section.export("memory", ExportKind::Memory, 0);
    for function in &all_functions {
        if function.is_entry {
            export_section.export("_start", ExportKind::Func, function_name_to_index["_start"]);
        } else {
            export_section.export(
                &function.name,
                ExportKind::Func,
                function_name_to_index[&function.name],
            );
        }
    }

    let mut code_section = CodeSection::new();
    for (coverage_id, function) in all_functions.iter().enumerate() {
        let mut body = Function::new(vec![((function.locals.len() + 1) as u32, ValType::I64)]);
        let mut emitter = FunctionEmitter::new(
            lir,
            &function_name_to_index,
            env_set_import_index,
            env_delete_import_index,
            env_get_import_index,
            env_has_import_index,
            cwd_set_import_index,
            process_exit_import_index,
            &mut diagnostics,
            &mut string_pool,
            ctx.source_path.clone(),
            function.flavor,
            &function.params,
            &function.locals,
        );
        let coverage_id = ctx.target.coverage.then_some(coverage_id as u32);
        if function.is_entry {
            emitter.emit_coverage_hit(&mut body, coverage_id);
            emitter.emit_sequence(&mut body, &top_level_children(lir), false);
        } else {
            emitter.emit_function_body(&mut body, function.body, function.result, coverage_id);
        }
        body.instruction(&Instruction::End);
        code_section.function(&body);
    }

    let mut data_section = DataSection::new();
    for (offset, text) in &string_pool.entries {
        data_section.active(
            0,
            &ConstExpr::i32_const(*offset as i32),
            text.as_bytes().iter().copied(),
        );
    }

    let mut module = Module::new();
    module.section(&type_section);
    module.section(&import_section);
    module.section(&function_section);
    module.section(&memory_section);
    module.section(&export_section);
    module.section(&code_section);
    if ctx.target.coverage {
        module.section(&CustomSection {
            name: Cow::Borrowed("kali:coverage"),
            data: Cow::Owned((all_functions.len() as u32).to_le_bytes().to_vec()),
        });
    }
    if !data_section.is_empty() {
        module.section(&data_section);
    }

    let wasm_bytes = module.finish();

    CodegenResult {
        wasm_bytes,
        diagnostics,
    }
}

pub(crate) fn collect_functions(lir: &LirProgram) -> Vec<FunctionPlan> {
    let mut plans = Vec::new();
    let mut visited = HashSet::new();
    collect_functions_from_node(lir, lir.root, &mut visited, &mut plans);
    plans
}

pub(crate) fn program_uses_env_get(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("get") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_env_has(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("has") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn is_process_root(nodes: &[LirNode], id: LirNodeId) -> bool {
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };

    if node.text.as_deref() == Some("process") {
        return true;
    }

    node.text.as_deref() == Some("globalThis")
        && node.children.first().is_some_and(|child| {
            nodes
                .get(child.0 as usize)
                .is_some_and(|process| process.text.as_deref() == Some("process"))
        })
}

pub(crate) fn process_env_property_key(nodes: &[LirNode], id: LirNodeId) -> Option<String> {
    let node = nodes.get(id.0 as usize)?;
    let key = node.text.as_deref()?;
    let object = node.children.first().copied()?;
    let object_node = nodes.get(object.0 as usize)?;
    if object_node.text.as_deref() != Some("env") {
        return None;
    }
    let root = object_node.children.first().copied()?;
    if !is_process_root(nodes, root) {
        return None;
    }

    Some(key.to_string())
}

pub(crate) fn program_uses_env_set(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("=")
            && node.children.len() == 2
            && process_env_property_key(&lir.nodes, node.children[0]).is_some()
        {
            return true;
        }

        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("set") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_env_delete(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("delete")
            && node.children.len() == 1
            && process_env_property_key(&lir.nodes, node.children[0]).is_some()
        {
            return true;
        }

        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("delete") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };
        if object_node.text.as_deref() != Some("env") {
            return false;
        }

        let Some(root) = object_node.children.first() else {
            return false;
        };
        let Some(root_node) = lir.nodes.get(root.0 as usize) else {
            return false;
        };

        root_node.text.as_deref() == Some("Deno")
            || (root_node.text.as_deref() == Some("globalThis")
                && root_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_cwd_set(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("chdir") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };

        object_node.text.as_deref() == Some("Deno")
            || (object_node.text.as_deref() == Some("globalThis")
                && object_node.children.first().is_some_and(|child| {
                    lir.nodes
                        .get(child.0 as usize)
                        .is_some_and(|deno| deno.text.as_deref() == Some("Deno"))
                }))
    })
}

pub(crate) fn program_uses_process_exit(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        if callee_node.text.as_deref() != Some("exit") {
            return false;
        }

        let Some(object) = callee_node.children.first() else {
            return false;
        };
        let Some(object_node) = lir.nodes.get(object.0 as usize) else {
            return false;
        };

        object_node.text.as_deref() == Some("process")
            || object_node.text.as_deref() == Some("Deno")
            || (object_node.text.as_deref() == Some("globalThis")
                && object_node.children.first().is_some_and(|child| {
                    lir.nodes.get(child.0 as usize).is_some_and(|host| {
                        matches!(host.text.as_deref(), Some("process") | Some("Deno"))
                    })
                }))
    })
}

pub(crate) fn collect_functions_from_node(
    lir: &LirProgram,
    id: LirNodeId,
    visited: &mut HashSet<LirNodeId>,
    plans: &mut Vec<FunctionPlan>,
) {
    if !visited.insert(id) {
        return;
    }

    if let Some(plan) = function_plan(&lir.nodes, id) {
        plans.push(plan);
    }

    let Some(node) = lir.nodes.get(id.0 as usize) else {
        return;
    };

    for child in &node.children {
        collect_functions_from_node(lir, *child, visited, plans);
    }
}

pub(crate) fn function_plan(nodes: &[LirNode], id: LirNodeId) -> Option<FunctionPlan> {
    let node = nodes.get(id.0 as usize)?;
    if node.kind != LirNodeKind::Instruction {
        return None;
    }
    let name = node.text.clone()?;
    let flavor = node.function_flavor;
    if node.children.is_empty() {
        return None;
    }
    let body_id = *node.children.last()?;
    if nodes.get(body_id.0 as usize)?.kind != LirNodeKind::Block {
        return None;
    }

    let mut params = Vec::new();
    for child in node.children.iter().take(node.children.len() - 1) {
        let child_node = nodes.get(child.0 as usize)?;
        if child_node.kind == LirNodeKind::Value {
            params.push(child_node.text.clone().unwrap_or_default());
        }
    }

    let locals = collect_function_locals(nodes, body_id);

    Some(FunctionPlan {
        name,
        params,
        locals,
        body: body_id,
        result: true,
        is_entry: false,
        flavor,
    })
}

pub(crate) fn is_function_like(nodes: &[LirNode], id: LirNodeId) -> bool {
    function_plan(nodes, id).is_some()
}

pub(crate) fn collect_function_locals(nodes: &[LirNode], body_id: LirNodeId) -> Vec<String> {
    let mut locals = Vec::new();
    let mut seen = HashSet::new();
    collect_function_locals_from_node(nodes, body_id, &mut seen, &mut locals);
    locals
}

pub(crate) fn collect_function_locals_from_node(
    nodes: &[LirNode],
    id: LirNodeId,
    seen: &mut HashSet<LirNodeId>,
    locals: &mut Vec<String>,
) {
    if !seen.insert(id) {
        return;
    }

    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };

    if node.kind == LirNodeKind::Instruction && matches!(node.text.as_deref(), Some("let" | "var"))
    {
        for declarator in &node.children {
            let Some(declarator_node) = nodes.get(declarator.0 as usize) else {
                continue;
            };
            if let Some(name) = declarator_node.text.clone() {
                if !locals.contains(&name) {
                    locals.push(name);
                }
            }
        }
    }

    for child in &node.children {
        if is_function_like(nodes, *child) {
            continue;
        }
        collect_function_locals_from_node(nodes, *child, seen, locals);
    }
}

pub(crate) fn top_level_children(lir: &LirProgram) -> Vec<LirNodeId> {
    let mut children = Vec::new();
    if let Some(root) = lir.nodes.get(lir.root.0 as usize) {
        for child in &root.children {
            if !is_function_like(&lir.nodes, *child) {
                children.push(*child);
            }
        }
    }
    children
}

pub(crate) fn emit_literal(
    function: &mut Function,
    text: Option<&str>,
    strings: &mut StringPool,
) -> EmittedValue {
    match text {
        Some("true") => {
            function.instruction(&Instruction::I64Const(1));
            EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            }
        }
        Some("false") => {
            function.instruction(&Instruction::I64Const(0));
            EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            }
        }
        Some("null") | Some("undefined") => {
            function.instruction(&Instruction::I64Const(0));
            EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            }
        }
        Some(text) => {
            if let Some(number) = parse_number_literal(text) {
                function.instruction(&Instruction::I64Const(number));
            } else {
                let normalized = strip_string_delimiters(text);
                let (offset, len) = strings.intern(normalized);
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
            }
            EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            }
        }
        None => {
            function.instruction(&Instruction::I64Const(0));
            EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            }
        }
    }
}

pub(crate) fn encode_string_handle(offset: u32, len: u32) -> i64 {
    (STRING_HANDLE_TAG | ((offset as u64) << 32) | u64::from(len)) as i64
}

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

pub(crate) fn semver_min_version(range: &str) -> Option<String> {
    let trimmed = range.trim();
    let candidate = trimmed
        .trim_start_matches(|c: char| {
            c.is_whitespace() || matches!(c, '^' | '~' | '=' | 'v' | '>' | '<')
        })
        .split(|c: char| c.is_whitespace() || c == ',' || c == '|')
        .next()?;
    Version::parse(candidate)
        .ok()
        .map(|version| version.to_string())
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

pub(crate) fn parse_number_literal(text: &str) -> Option<i64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<i64>().ok();
    }
    text.parse::<i64>().ok()
}

pub(crate) fn parse_numeric_literal_value(text: &str) -> Option<f64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<f64>().ok();
    }
    text.parse::<f64>().ok()
}

pub(crate) fn is_supported_static_ascii_char_code(value: f64) -> bool {
    value.is_finite() && value.fract() == 0.0 && (0.0..=127.0).contains(&value)
}

pub(crate) fn static_parse_float_ascii_integer(source: &str) -> Option<i64> {
    if !source.is_ascii() {
        return None;
    }

    let trimmed = source.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    let mut end = 0;
    let bytes = trimmed.as_bytes();
    if matches!(bytes.get(end), Some(b'+' | b'-')) {
        end += 1;
    }

    let digits_before = bytes[end..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    end += digits_before;

    let mut digits_after = 0;
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        digits_after = bytes[end..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        end += digits_after;
    }

    if digits_before + digits_after == 0 {
        return None;
    }

    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let exponent_marker = end;
        end += 1;
        if matches!(bytes.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let exponent_digits = bytes[end..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if exponent_digits == 0 {
            end = exponent_marker;
        } else {
            end += exponent_digits;
        }
    }

    let value = trimmed[..end].parse::<f64>().ok()?;
    if !value.is_finite()
        || value.fract() != 0.0
        || value < i64::MIN as f64
        || value > i64::MAX as f64
    {
        return None;
    }

    Some(value as i64)
}

pub(crate) fn static_parse_int_ascii(source: &str, radix: u32) -> Option<i64> {
    if !source.is_ascii() || !(radix == 0 || (2..=36).contains(&radix)) {
        return None;
    }

    let trimmed = source.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    let (negative, rest) = if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        (false, rest)
    } else {
        (false, trimmed)
    };

    let (radix, digits) = if radix == 0 {
        if let Some(rest) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
            (16, rest)
        } else {
            (10, rest)
        }
    } else if radix == 16 {
        (
            16,
            rest.strip_prefix("0x")
                .or_else(|| rest.strip_prefix("0X"))
                .unwrap_or(rest),
        )
    } else {
        (radix, rest)
    };

    let mut value: i64 = 0;
    let mut consumed = false;
    for ch in digits.chars() {
        let Some(digit) = ch.to_digit(radix) else {
            break;
        };
        consumed = true;
        value = value.checked_mul(radix as i64)?.checked_add(digit as i64)?;
    }

    if !consumed {
        return None;
    }

    if negative {
        value.checked_neg()
    } else {
        Some(value)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

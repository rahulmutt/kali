//! WASM code generation for the Kali compiler.

mod ctx;
mod emit;
mod emitter;
mod intrinsics;
pub use ctx::{CodegenCtx, CodegenResult, TargetConfig};
pub(crate) use intrinsics::{quote_string_literal, strip_string_delimiters, parse_number_literal, parse_numeric_literal_value, is_supported_static_ascii_char_code, static_parse_float_ascii_integer, static_parse_int_ascii};
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


#[cfg(test)]
#[path = "tests.rs"]
mod tests;

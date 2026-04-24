//! WASM code generation for the Kali compiler.

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

use kali_error::{
    _error_codes::{e3, e8},
    Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
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
const COVERAGE_HIT_IMPORT_INDEX: u32 = 11;
const FUNCTION_INDEX_OFFSET: u32 = 11;
const STRING_HANDLE_TAG: u64 = 0x8000_0000_0000_0000;

/// WASM code generator context.
pub struct CodegenCtx {
    /// Target configuration.
    pub target: TargetConfig,
    /// Source file path for context-sensitive static lowering.
    pub source_path: Option<PathBuf>,
}

impl CodegenCtx {
    pub fn new(target: TargetConfig) -> Self {
        Self {
            target,
            source_path: None,
        }
    }
}

/// Target configuration for code generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Upper bound on specialization fan-out.
    pub max_specializations: usize,
    /// Whether compatibility eval source stubs were pre-resolved earlier in the pipeline.
    pub compat_eval: bool,
    /// Whether coverage instrumentation is enabled for this compilation.
    pub coverage: bool,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        }
    }
}

/// Code generation result containing the WASM output.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodegenResult {
    /// WASM bytes.
    pub wasm_bytes: Vec<u8>,
    /// Diagnostics collected during codegen.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug)]
struct FunctionPlan {
    name: String,
    params: Vec<String>,
    body: LirNodeId,
    result: bool,
    is_entry: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueShape {
    Unknown,
    Scalar,
    Boolean,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EmittedValue {
    produced: bool,
    shape: ValueShape,
}

struct StringPool {
    entries: Vec<(u32, String)>,
    offsets: BTreeMap<String, u32>,
    next_offset: u32,
}

impl StringPool {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            offsets: BTreeMap::new(),
            next_offset: 0,
        }
    }

    fn intern(&mut self, text: &str) -> (u32, u32) {
        if let Some(&offset) = self.offsets.get(text) {
            return (offset, text.len() as u32);
        }

        let offset = self.next_offset;
        let len = text.len() as u32;
        self.entries.push((offset, text.to_owned()));
        self.offsets.insert(text.to_owned(), offset);
        self.next_offset = self.next_offset.saturating_add(len);
        (offset, len)
    }
}

struct FunctionEmitter<'a> {
    program: &'a LirProgram,
    node_lookup: &'a [LirNode],
    functions: &'a BTreeMap<String, u32>,
    diagnostics: &'a mut Vec<Diagnostic>,
    strings: &'a mut StringPool,
    source_path: Option<PathBuf>,
    locals: BTreeMap<String, u32>,
    bindings: BTreeMap<String, LirNodeId>,
    reported_placeholder_fallbacks: HashSet<String>,
}

impl<'a> FunctionEmitter<'a> {
    fn new(
        program: &'a LirProgram,
        functions: &'a BTreeMap<String, u32>,
        diagnostics: &'a mut Vec<Diagnostic>,
        strings: &'a mut StringPool,
        source_path: Option<PathBuf>,
        params: &[String],
    ) -> Self {
        let mut locals = BTreeMap::new();
        for (idx, name) in params.iter().enumerate() {
            locals.insert(name.clone(), idx as u32);
        }

        Self {
            program,
            node_lookup: &program.nodes,
            functions,
            diagnostics,
            strings,
            source_path,
            locals,
            bindings: BTreeMap::new(),
            reported_placeholder_fallbacks: HashSet::new(),
        }
    }

    fn node(&self, id: LirNodeId) -> &LirNode {
        &self.node_lookup[id.0 as usize]
    }

    fn push_placeholder_fallback_diagnostic(&mut self, kind: &str, name: &str) {
        let fallback_key = format!("{kind}:{name}");
        if !self.reported_placeholder_fallbacks.insert(fallback_key) {
            return;
        }

        let message = match kind {
            "identifier" => format!(
                "undefined identifier '{}' reached codegen and was lowered through a zero placeholder compatibility fallback",
                name
            ),
            "call target" => format!(
                "undefined call target '{}' reached codegen and was lowered through a zero placeholder compatibility fallback",
                name
            ),
            _ => format!(
                "undefined {} '{}' reached codegen and was lowered through a zero placeholder compatibility fallback",
                kind, name
            ),
        };

        let mut diagnostic = Diagnostic::warning(e3::UNDEFINED_IDENTIFIER as u32, message)
            .with_context(
                DiagnosticContext::new(DiagnosticContextOrigin::Source)
                    .with_requested_value(name)
                    .with_effective_value("zero placeholder compatibility fallback"),
            )
            .note(
                "name resolution should resolve this before codegen; the fallback emits a zero placeholder and should remain a compatibility escape hatch only",
            );
        if let Some(source_path) = &self.source_path {
            diagnostic = diagnostic.note(format!("source path: {}", source_path.display()));
        }
        self.diagnostics.push(diagnostic);
    }

    fn emit_coverage_hit(&mut self, function: &mut Function, coverage_id: Option<u32>) {
        if let Some(coverage_id) = coverage_id {
            function.instruction(&Instruction::I32Const(coverage_id as i32));
            function.instruction(&Instruction::Call(COVERAGE_HIT_IMPORT_INDEX));
        }
    }

    fn emit_function_body(
        &mut self,
        function: &mut Function,
        body: LirNodeId,
        returns_value: bool,
        coverage_id: Option<u32>,
    ) {
        self.emit_coverage_hit(function, coverage_id);
        let produced = self.emit_node(function, body, returns_value);
        if returns_value && !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
    }

    fn emit_sequence(
        &mut self,
        function: &mut Function,
        children: &[LirNodeId],
        want_value: bool,
    ) -> EmittedValue {
        let mut final_value = EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        };
        for (idx, child) in children.iter().enumerate() {
            let child_want_value = want_value && idx + 1 == children.len();
            let child_result = self.emit_node(function, *child, child_want_value);
            if child_result.produced && !child_want_value {
                function.instruction(&Instruction::Drop);
            }
            if child_want_value {
                final_value = child_result;
            }
        }

        if want_value {
            final_value
        } else {
            EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            }
        }
    }

    fn emit_node(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        want_value: bool,
    ) -> EmittedValue {
        let node = self.node(id).clone();
        match node.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                self.emit_sequence(function, &node.children, want_value)
            }
            LirNodeKind::Instruction => {
                if matches!(node.text.as_deref(), Some("const" | "let" | "var")) {
                    for declarator_id in &node.children {
                        let declarator = self.node(*declarator_id).clone();
                        if declarator.children.len() < 2 {
                            continue;
                        }
                        if let Some(name) = declarator.text.clone() {
                            self.bindings.insert(name, declarator.children[1]);
                        }
                        let init = declarator.children[1];
                        let init_result = self.emit_node(function, init, false);
                        if init_result.produced {
                            function.instruction(&Instruction::Drop);
                        }
                    }
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                if is_function_like(&self.program.nodes, id) {
                    // Function declarations are emitted separately from the body scan.
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                self.emit_sequence(function, &node.children, false)
            }
            LirNodeKind::Literal => emit_literal(function, node.text.as_deref(), self.strings),
            LirNodeKind::Value => self.emit_value(function, &node, want_value),
            LirNodeKind::Call => self.emit_call(function, &node),
            LirNodeKind::Branch => self.emit_branch(function, &node, want_value),
            LirNodeKind::Unknown => {
                function.instruction(&Instruction::Unreachable);
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    fn emit_value(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        match node.children.len() {
            0 => {
                if let Some(text) = node.text.as_deref() {
                    if let Some(index) = self.locals.get(text).copied() {
                        function.instruction(&Instruction::LocalGet(index));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Unknown,
                        };
                    }

                    if let Some(bound) = self.bindings.get(text).copied() {
                        return self.emit_node(function, bound, want_value);
                    }

                    if let Some(constant) = parse_number_literal(text) {
                        function.instruction(&Instruction::I64Const(constant));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }

                    self.push_placeholder_fallback_diagnostic("identifier", text);
                    function.instruction(&Instruction::I64Const(0));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    }
                } else {
                    function.instruction(&Instruction::I64Const(0));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    }
                }
            }
            1 => {
                if node.text.as_deref().unwrap_or_default().is_empty() {
                    self.emit_node(function, node.children[0], want_value)
                } else {
                    self.emit_unary(function, node)
                }
            }
            2 => self.emit_binary(function, node),
            _ => EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            },
        }
    }

    fn emit_unary(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let op = node.text.as_deref().unwrap_or_default();
        let arg = node.children[0];
        match op {
            "-" => {
                function.instruction(&Instruction::I64Const(0));
                let _ = self.emit_node(function, arg, true);
                function.instruction(&Instruction::I64Sub);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "!" => {
                let _ = self.emit_node(function, arg, true);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "length" => {
                if self.is_process_argv(arg) {
                    function.instruction(&Instruction::Call(ARGS_LEN_IMPORT_INDEX));
                    function.instruction(&Instruction::I64ExtendI32U);
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                if let Some(slice_start) = self.process_argv_slice_start(arg) {
                    function.instruction(&Instruction::Call(ARGS_LEN_IMPORT_INDEX));
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::I64Const(slice_start));
                    function.instruction(&Instruction::I64Sub);
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "version" => {
                if let Some(rendered) = self.render_package_json_version_access(arg) {
                    let (offset, len) = self.strings.intern(&rendered);
                    function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                if self.has_semver_import() {
                    if let Some(rendered) = self.render_static_value(arg) {
                        let (offset, len) = self.strings.intern(&rendered);
                        function
                            .instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }
                }

                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported unary operator '{}'", op),
                ));
                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    fn emit_binary(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let op = node.text.as_deref().unwrap_or_default();
        let left = node.children[0];
        let right = node.children[1];
        let _ = self.emit_node(function, left, true);
        let _ = self.emit_node(function, right, true);

        match op {
            "+" => {
                function.instruction(&Instruction::I64Add);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "-" => {
                function.instruction(&Instruction::I64Sub);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "*" => {
                function.instruction(&Instruction::I64Mul);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "/" => {
                function.instruction(&Instruction::I64DivS);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "==" => {
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "&&" => {
                function.instruction(&Instruction::I64And);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "||" => {
                function.instruction(&Instruction::I64Or);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported binary operator '{}'", op),
                ));
                function.instruction(&Instruction::I64Add);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    fn emit_call(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let Some(callee) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };

        let callee_node = self.node(callee).clone();
        if self.is_kali_test_call(&callee_node) {
            if let Some(callback_index) = self.kali_test_callback_index(node) {
                function.instruction(&Instruction::I32Const(callback_index as i32));
                function.instruction(&Instruction::Call(TEST_REGISTER_IMPORT_INDEX));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            self.diagnostics.push(Diagnostic::warning(
                e8::IR_UNREADABLE as u32,
                "`Kali.test(...)` requires a function callback lowered as an exported function",
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let callee_name = callee_node.text.as_deref().unwrap_or_default();
        let resolved = self.functions.get(callee_name).copied();

        if self.is_console_assert(&callee_node) {
            let message_args: Vec<LirNodeId> = node.children.iter().skip(2).copied().collect();
            let Some(condition) = node.children.get(1).copied() else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                let (offset, len) = self.strings.intern("Assertion failed");
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                function.instruction(&Instruction::End);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let condition_result = self.emit_node(function, condition, true);
            if !condition_result.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            match condition_result.shape {
                ValueShape::Boolean => {
                    function.instruction(&Instruction::I32WrapI64);
                }
                ValueShape::Scalar | ValueShape::Unknown => {
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                }
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            if !message_args.is_empty() {
                if let Some(rendered) = self.render_console_arguments(&message_args) {
                    let (offset, len) = self.strings.intern(&rendered);
                    let handle = encode_string_handle(offset, len);
                    function.instruction(&Instruction::I64Const(handle));
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                } else if let Some(first_arg) = message_args.first().copied() {
                    let _ = self.emit_node(function, first_arg, true);
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                    for arg in message_args.iter().skip(1) {
                        let _ = self.emit_node(function, *arg, true);
                        function.instruction(&Instruction::Drop);
                    }
                }
            } else {
                let (offset, len) = self.strings.intern("Assertion failed");
                let handle = encode_string_handle(offset, len);
                function.instruction(&Instruction::I64Const(handle));
                function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
            }
            function.instruction(&Instruction::End);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.console_import_index(&callee_node) {
            if let Some(rendered) = self.render_console_call(node) {
                let (offset, len) = self.strings.intern(&rendered);
                let handle = encode_string_handle(offset, len);
                function.instruction(&Instruction::I64Const(handle));
                function.instruction(&Instruction::Call(import_index));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            let mut args = node.children.iter().skip(1);
            if let Some(first_arg) = args.next() {
                let _ = self.emit_node(function, *first_arg, true);
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.math_max_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            let _ = self.emit_node(function, *first_arg, true);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Call(import_index));
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_min_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            let _ = self.emit_node(function, *first_arg, true);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Call(import_index));
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_abs_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            let _ = self.emit_node(function, *first_arg, true);
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_sign_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            let _ = self.emit_node(function, *first_arg, true);
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        for arg in node.children.iter().skip(1) {
            let _ = self.emit_node(function, *arg, true);
        }

        if let Some(index) = resolved {
            function.instruction(&Instruction::Call(index));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if self.has_semver_import() {
            if let Some(rendered) = self.render_semver_intrinsic(callee_name, node) {
                for _ in node.children.iter().skip(1) {
                    function.instruction(&Instruction::Drop);
                }
                if rendered == "0" || rendered == "1" {
                    let value = rendered.parse::<i64>().unwrap_or(0);
                    function.instruction(&Instruction::I64Const(value));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Boolean,
                    };
                }
                let (offset, len) = self.strings.intern(&rendered);
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }
        }

        self.push_placeholder_fallback_diagnostic("call target", callee_name);
        for _ in node.children.iter().skip(1) {
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::I64Const(0));
        EmittedValue {
            produced: true,
            shape: ValueShape::Unknown,
        }
    }

    fn console_import_index(&self, callee_node: &LirNode) -> Option<u32> {
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

    fn is_console_assert(&self, callee_node: &LirNode) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("console") && method == "assert"
    }

    fn math_max_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        let object = callee_node.children.first().copied()?;
        let object_name = self.node(object).text.as_deref()?;
        if object_name == "Math" && method == "max" {
            Some(MATH_MAX_IMPORT_INDEX)
        } else {
            None
        }
    }

    fn math_min_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        let object = callee_node.children.first().copied()?;
        let object_name = self.node(object).text.as_deref()?;
        if object_name == "Math" && method == "min" {
            Some(MATH_MIN_IMPORT_INDEX)
        } else {
            None
        }
    }

    fn math_abs_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        let object = callee_node.children.first().copied()?;
        let object_name = self.node(object).text.as_deref()?;
        if object_name == "Math" && method == "abs" {
            Some(MATH_ABS_IMPORT_INDEX)
        } else {
            None
        }
    }

    fn math_sign_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        let object = callee_node.children.first().copied()?;
        let object_name = self.node(object).text.as_deref()?;
        if object_name == "Math" && method == "sign" {
            Some(MATH_SIGN_IMPORT_INDEX)
        } else {
            None
        }
    }

    fn render_console_call(&self, node: &LirNode) -> Option<String> {
        let args = node.children.iter().skip(1).copied().collect::<Vec<_>>();
        self.render_console_arguments(&args)
    }

    fn render_console_arguments(&self, args: &[LirNodeId]) -> Option<String> {
        let mut rendered = Vec::new();
        for arg in args {
            rendered.push(self.render_static_value(*arg)?);
        }
        Some(rendered.join(" "))
    }

    fn render_static_value(&self, id: LirNodeId) -> Option<String> {
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
                    match text {
                        "true" | "false" | "null" | "undefined" => Some(text.to_string()),
                        _ => None,
                    }
                } else if node.text.as_deref().is_some_and(|text| text == "length") {
                    if self.is_process_argv(node.children[0]) {
                        None
                    } else {
                        self.render_length(&node.children[0])
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

    fn has_semver_import(&self) -> bool {
        self.program
            .nodes
            .iter()
            .any(|node| node.text.as_deref() == Some("semver"))
    }

    fn render_semver_intrinsic(&self, callee_name: &str, node: &LirNode) -> Option<String> {
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

    fn render_package_json_version(&self, specifier: &str) -> Option<String> {
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

    fn render_package_json_version_access(&self, id: LirNodeId) -> Option<String> {
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

    fn is_process_argv(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.text.as_deref() != Some("argv") || node.children.len() != 1 {
            return false;
        }

        let object = self.node(node.children[0]);
        if object.text.as_deref() == Some("process") {
            return true;
        }

        object.text.as_deref() == Some("globalThis")
            && object
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    fn resolve_bound_node(&self, mut id: LirNodeId) -> LirNodeId {
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(id) {
                return id;
            }

            let node = self.node(id);
            if node.kind == LirNodeKind::Value && node.children.is_empty() {
                if let Some(text) = node.text.as_deref() {
                    if let Some(bound) = self.bindings.get(text).copied() {
                        id = bound;
                        continue;
                    }
                }
            }

            return id;
        }
    }

    fn process_argv_slice_start(&self, id: LirNodeId) -> Option<i64> {
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("slice") {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !self.is_process_argv(object) {
            return None;
        }

        let start = *node.children.get(1)?;
        let start_node = self.node(start);
        parse_number_literal(start_node.text.as_deref()?)
    }

    fn render_length(&self, id: &LirNodeId) -> Option<String> {
        if self.process_argv_slice_start(*id).is_some() {
            return None;
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

    fn is_kali_test_call(&self, callee_node: &LirNode) -> bool {
        if callee_node.text.as_deref() != Some("test") {
            return false;
        }

        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("Kali")
    }

    fn kali_test_callback_index(&self, node: &LirNode) -> Option<u32> {
        let callback_node = node.children.get(2).copied()?;
        let callback_name = self.node(callback_node).text.as_deref()?;
        self.functions.get(callback_name).copied()
    }

    fn emit_branch(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        let Some(cond) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };

        let then_branch = node.children.get(1).copied();
        let else_branch = node.children.get(2).copied();

        let condition = self.emit_node(function, cond, true);
        match condition.shape {
            ValueShape::Boolean => {
                function.instruction(&Instruction::I32WrapI64);
            }
            ValueShape::Scalar | ValueShape::Unknown => {
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
            }
        }
        function.instruction(&Instruction::If(if want_value {
            BlockType::Result(ValType::I64)
        } else {
            BlockType::Empty
        }));

        if let Some(then_branch) = then_branch {
            let produced = self.emit_node(function, then_branch, want_value);
            if want_value && !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else if want_value {
            function.instruction(&Instruction::I64Const(0));
        }

        if let Some(else_branch) = else_branch {
            function.instruction(&Instruction::Else);
            let produced = self.emit_node(function, else_branch, want_value);
            if want_value && !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else if want_value {
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
        }

        function.instruction(&Instruction::End);
        EmittedValue {
            produced: want_value,
            shape: ValueShape::Unknown,
        }
    }
}

/// Generate WASM from LIR.
pub fn lower_lir_to_wasm(ctx: &mut CodegenCtx, lir: &LirProgram) -> CodegenResult {
    let mut diagnostics = Vec::new();
    let function_plans = collect_functions(lir);
    let mut function_name_to_index = BTreeMap::new();
    let mut string_pool = StringPool::new();
    let function_index_offset = FUNCTION_INDEX_OFFSET + if ctx.target.coverage { 1 } else { 0 };

    // Keep the emitted order deterministic: imported registration hook first, synthetic entry
    // second, then named functions in source order.
    let mut all_functions = Vec::new();
    all_functions.push(FunctionPlan {
        name: "_start".to_string(),
        params: Vec::new(),
        body: lir.root,
        result: false,
        is_entry: true,
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
    if ctx.target.coverage {
        import_section.import("kali:rt", "coverage_hit", EntityType::Function(0));
    }
    let mut function_types = BTreeMap::<(usize, bool), u32>::new();
    let mut type_for_function = Vec::with_capacity(all_functions.len());

    for function in &all_functions {
        let key = (function.params.len(), function.result);
        let type_index = if let Some(&idx) = function_types.get(&key) {
            idx
        } else {
            let idx = function_types.len() as u32 + 5;
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
        let mut body = Function::new(Vec::new());
        let mut emitter = FunctionEmitter::new(
            lir,
            &function_name_to_index,
            &mut diagnostics,
            &mut string_pool,
            ctx.source_path.clone(),
            &function.params,
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

    let validation_result = wasmparser::Validator::new().validate_all(&wasm_bytes);
    if let Err(error) = validation_result {
        diagnostics.push(Diagnostic::error(
            e8::CODEGEN_UNEXPECTED as u32,
            format!("emitted WASM failed validation: {}", error),
        ));
        return CodegenResult {
            wasm_bytes: Vec::new(),
            diagnostics,
        };
    }

    CodegenResult {
        wasm_bytes,
        diagnostics,
    }
}

fn collect_functions(lir: &LirProgram) -> Vec<FunctionPlan> {
    let mut plans = Vec::new();
    let mut visited = HashSet::new();
    collect_functions_from_node(lir, lir.root, &mut visited, &mut plans);
    plans
}

fn collect_functions_from_node(
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

fn function_plan(nodes: &[LirNode], id: LirNodeId) -> Option<FunctionPlan> {
    let node = nodes.get(id.0 as usize)?;
    if node.kind != LirNodeKind::Instruction {
        return None;
    }
    let name = node.text.clone()?;
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

    Some(FunctionPlan {
        name,
        params,
        body: body_id,
        result: true,
        is_entry: false,
    })
}

fn is_function_like(nodes: &[LirNode], id: LirNodeId) -> bool {
    function_plan(nodes, id).is_some()
}

fn top_level_children(lir: &LirProgram) -> Vec<LirNodeId> {
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

fn emit_literal(
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

fn encode_string_handle(offset: u32, len: u32) -> i64 {
    (STRING_HANDLE_TAG | ((offset as u64) << 32) | u64::from(len)) as i64
}

fn semver_min_version(range: &str) -> Option<String> {
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

fn strip_string_delimiters(text: &str) -> &str {
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

fn parse_number_literal(text: &str) -> Option<i64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<i64>().ok();
    }
    text.parse::<i64>().ok()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

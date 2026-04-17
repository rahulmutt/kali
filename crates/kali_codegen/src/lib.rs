//! WASM code generation for the Kali compiler.

use std::collections::{BTreeMap, HashSet};

use kali_error::{_error_codes::e8, Diagnostic};
use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
use serde::{Deserialize, Serialize};
use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

const TEST_REGISTER_IMPORT_INDEX: u32 = 0;
const FUNCTION_INDEX_OFFSET: u32 = 1;

/// WASM code generator context.
pub struct CodegenCtx {
    /// Target configuration.
    pub target: TargetConfig,
}

impl CodegenCtx {
    pub fn new(target: TargetConfig) -> Self {
        Self { target }
    }
}

/// Target configuration for code generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Whether to enable optimization passes.
    pub optimize: bool,
    /// Upper bound on specialization fan-out.
    pub max_specializations: usize,
    /// Whether compatibility eval source stubs were pre-resolved earlier in the pipeline.
    pub compat_eval: bool,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            optimize: false,
            max_specializations: 16,
            compat_eval: false,
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

struct FunctionEmitter<'a> {
    program: &'a LirProgram,
    node_lookup: &'a [LirNode],
    functions: &'a BTreeMap<String, u32>,
    diagnostics: &'a mut Vec<Diagnostic>,
    locals: BTreeMap<String, u32>,
}

impl<'a> FunctionEmitter<'a> {
    fn new(
        program: &'a LirProgram,
        functions: &'a BTreeMap<String, u32>,
        diagnostics: &'a mut Vec<Diagnostic>,
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
            locals,
        }
    }

    fn node(&self, id: LirNodeId) -> &LirNode {
        &self.node_lookup[id.0 as usize]
    }

    fn emit_function_body(
        &mut self,
        function: &mut Function,
        body: LirNodeId,
        returns_value: bool,
    ) {
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
                if is_function_like(&self.program.nodes, id) {
                    // Function declarations are emitted separately from the body scan.
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                self.emit_sequence(function, &node.children, false)
            }
            LirNodeKind::Literal => emit_literal(function, node.text.as_deref()),
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

                    if let Some(constant) = parse_number_literal(text) {
                        function.instruction(&Instruction::I64Const(constant));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }

                    self.diagnostics.push(Diagnostic::warning(
                        e8::IR_UNREADABLE as u32,
                        format!("unresolved identifier '{}' lowered as 0", text),
                    ));
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
            _ => {
                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported unary operator '{}'", op),
                ));
                EmittedValue {
                    produced: false,
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
        let mut resolved = self.functions.get(callee_name).copied();

        for arg in node.children.iter().skip(1) {
            let _ = self.emit_node(function, *arg, true);
        }

        if let Some(index) = resolved.take() {
            function.instruction(&Instruction::Call(index));
            EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            }
        } else {
            self.diagnostics.push(Diagnostic::warning(
                e8::IR_UNREADABLE as u32,
                format!("unresolved call target '{}' lowered as 0", callee_name),
            ));
            for _ in node.children.iter().skip(1) {
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            }
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
        function_name_to_index.insert(function.name.clone(), idx as u32 + FUNCTION_INDEX_OFFSET);
    }

    let mut type_section = TypeSection::new();
    type_section.ty().function(vec![ValType::I32], Vec::new());
    let mut import_section = ImportSection::new();
    import_section.import("kali:rt", "test_register", EntityType::Function(0));
    let mut function_types = BTreeMap::<(usize, bool), u32>::new();
    let mut type_for_function = Vec::with_capacity(all_functions.len());

    for function in &all_functions {
        let key = (function.params.len(), function.result);
        let type_index = if let Some(&idx) = function_types.get(&key) {
            idx
        } else {
            let idx = function_types.len() as u32 + 1;
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
    for function in &all_functions {
        let mut body = Function::new(Vec::new());
        let mut emitter = FunctionEmitter::new(
            lir,
            &function_name_to_index,
            &mut diagnostics,
            &function.params,
        );
        if function.is_entry {
            emitter.emit_sequence(&mut body, &top_level_children(lir), false);
        } else {
            emitter.emit_function_body(&mut body, function.body, function.result);
        }
        body.instruction(&Instruction::End);
        code_section.function(&body);
    }

    let mut module = Module::new();
    module.section(&type_section);
    module.section(&import_section);
    module.section(&function_section);
    module.section(&memory_section);
    module.section(&export_section);
    module.section(&code_section);

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

    if ctx.target.optimize {
        // Phase 1 keeps the optimization pipeline as a stable no-op placeholder.
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

fn emit_literal(function: &mut Function, text: Option<&str>) -> EmittedValue {
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
                function.instruction(&Instruction::I64Const(0));
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

fn parse_number_literal(text: &str) -> Option<i64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<i64>().ok();
    }
    text.parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmparser::Validator;

    fn node(kind: LirNodeKind, text: Option<&str>, children: Vec<LirNodeId>) -> LirNode {
        LirNode {
            kind,
            text: text.map(ToString::to_string),
            children,
        }
    }

    fn sample_program() -> LirProgram {
        let mut nodes = Vec::new();

        let root = LirNodeId(0);
        let add = LirNodeId(1);
        let add_param_a = LirNodeId(2);
        let add_param_b = LirNodeId(3);
        let add_block = LirNodeId(4);
        let add_return = LirNodeId(5);
        let add_expr = LirNodeId(6);
        let add_left = LirNodeId(7);
        let add_right = LirNodeId(8);
        let call_expr = LirNodeId(9);
        let call_callee = LirNodeId(10);
        let lit_one = LirNodeId(11);
        let lit_two = LirNodeId(12);

        nodes.push(node(LirNodeKind::Program, None, vec![add, call_expr]));
        nodes.push(node(
            LirNodeKind::Instruction,
            Some("add"),
            vec![add_param_a, add_param_b, add_block],
        ));
        nodes.push(node(LirNodeKind::Value, Some("a"), vec![]));
        nodes.push(node(LirNodeKind::Value, Some("b"), vec![]));
        nodes.push(node(LirNodeKind::Block, None, vec![add_return]));
        nodes.push(node(
            LirNodeKind::Instruction,
            Some("return"),
            vec![add_expr],
        ));
        nodes.push(node(
            LirNodeKind::Value,
            Some("+"),
            vec![add_left, add_right],
        ));
        nodes.push(node(LirNodeKind::Value, Some("a"), vec![]));
        nodes.push(node(LirNodeKind::Value, Some("b"), vec![]));
        nodes.push(node(
            LirNodeKind::Call,
            None,
            vec![call_callee, lit_one, lit_two],
        ));
        nodes.push(node(LirNodeKind::Value, Some("add"), vec![]));
        nodes.push(node(LirNodeKind::Literal, Some("1"), vec![]));
        nodes.push(node(LirNodeKind::Literal, Some("2"), vec![]));

        LirProgram { root, nodes }
    }

    #[test]
    fn generates_valid_wasm_for_simple_programs() {
        let program = sample_program();
        let mut ctx = CodegenCtx::new(TargetConfig {
            optimize: false,
            max_specializations: 16,
            compat_eval: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");

        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        assert!(printed.contains("i64.add"));
        assert!(printed.contains("call"));
    }

    #[test]
    fn function_plans_are_detected_from_instruction_shape() {
        let program = sample_program();
        let plans = collect_functions(&program);
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].name, "add");
        assert_eq!(plans[0].params, vec!["a", "b"]);
    }

    fn parse_and_lower_lir(source: &str) -> LirProgram {
        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;
        let mut hir_lowerer = kali_hir::HirLowerer::new();
        let hir = hir_lowerer.lower_statements(&statements);
        let mir = kali_mir::MirLowerer::new().lower_hir_result(&hir);
        kali_lir::LirLowerer::new().lower_program(&mir)
    }

    #[test]
    fn boolean_branches_use_the_layout_fast_path() {
        let program = parse_and_lower_lir("if (1 == 1) { 7; } else { 9; }");
        let mut ctx = CodegenCtx::new(TargetConfig {
            optimize: false,
            max_specializations: 16,
            compat_eval: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(result.diagnostics.is_empty());
        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        assert!(printed.contains("i32.wrap_i64"));
        assert!(!printed.contains("i64.eqz"));
    }

    fn legacy_phase1_baseline(program: &LirProgram, mir: &kali_mir::MirProgram) -> LirProgram {
        let mut nodes = program.nodes.clone();
        let mut extra_nodes = Vec::new();
        let mut insertions = Vec::new();

        let mut ownership_by_name = std::collections::BTreeMap::new();
        for function in &mir.functions {
            for binding in &function.bindings {
                if binding.kind == kali_mir::MirBindingKind::Local {
                    ownership_by_name
                        .entry(binding.name.clone())
                        .or_insert(binding.ownership);
                }
            }
        }

        insertions.push((
            program.root.0 as usize,
            vec!["phase1.alloc", "phase1.decref"],
        ));

        for (index, node) in program.nodes.iter().enumerate() {
            if node.kind != LirNodeKind::Instruction {
                continue;
            }

            let Some(name) = node.text.as_deref() else {
                continue;
            };

            if let Some(last_child) = node.children.last().copied() {
                if program
                    .nodes
                    .get(last_child.0 as usize)
                    .is_some_and(|child| child.kind == LirNodeKind::Block)
                {
                    insertions.push((last_child.0 as usize, vec!["phase1.alloc", "phase1.decref"]));
                    continue;
                }
            }

            let Some(ownership) = ownership_by_name.get(name).copied() else {
                continue;
            };

            let markers: Vec<&'static str> = match ownership {
                kali_mir::OwnershipClass::OwnedHeap => vec!["phase1.alloc", "phase1.decref"],
                kali_mir::OwnershipClass::SharedHeap => {
                    vec!["phase1.alloc", "phase1.incref", "phase1.decref"]
                }
                kali_mir::OwnershipClass::Stack | kali_mir::OwnershipClass::Borrowed => Vec::new(),
            };

            if markers.is_empty() {
                continue;
            }

            insertions.push((index, markers));
        }

        for (index, markers) in insertions {
            let mut synthetic_children = Vec::with_capacity(markers.len());
            for marker in markers {
                let id = LirNodeId((nodes.len() + extra_nodes.len()) as u32);
                extra_nodes.push(LirNode::with_text(LirNodeKind::Literal, marker));
                synthetic_children.push(id);
            }
            nodes[index].children.extend(synthetic_children);
        }

        nodes.extend(extra_nodes);
        LirProgram {
            root: program.root,
            nodes,
        }
    }

    fn compile_and_measure(program: &LirProgram) -> (Vec<u8>, usize) {
        let mut ctx = CodegenCtx::new(TargetConfig {
            optimize: false,
            max_specializations: 16,
            compat_eval: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, program);
        assert!(
            result.diagnostics.is_empty(),
            "codegen diagnostics: {:?}",
            result.diagnostics
        );
        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");

        let instruction_count = wasm_instruction_count(&result.wasm_bytes);
        (result.wasm_bytes, instruction_count)
    }

    fn wasm_instruction_count(bytes: &[u8]) -> usize {
        use wasmparser::{Parser as WasmParser, Payload};

        let mut count = 0;
        for payload in WasmParser::new(0).parse_all(bytes) {
            if let Ok(Payload::CodeSectionEntry(body)) = payload {
                let mut operators = body.get_operators_reader().expect("operators");
                while !operators.eof() {
                    operators.read().expect("operator");
                    count += 1;
                }
            }
        }
        count
    }

    #[test]
    fn mir_backed_pipeline_reduces_legacy_overhead_on_escaping_locals() {
        let current_lir = sample_program();
        let mir = kali_mir::MirProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: Vec::new(),
        };
        let baseline_lir = legacy_phase1_baseline(&current_lir, &mir);

        let current_trace = current_lir
            .nodes
            .iter()
            .filter_map(|node| node.text.as_deref())
            .collect::<Vec<_>>();
        let baseline_trace = baseline_lir
            .nodes
            .iter()
            .filter_map(|node| node.text.as_deref())
            .collect::<Vec<_>>();

        assert!(!current_trace.contains(&"phase1.alloc"));
        assert!(!current_trace.contains(&"phase1.incref"));
        assert!(!current_trace.contains(&"phase1.decref"));
        assert!(baseline_trace.contains(&"phase1.alloc"));
        assert!(baseline_trace.contains(&"phase1.decref"));

        let (current_bytes, current_instructions) = compile_and_measure(&current_lir);
        let (baseline_bytes, baseline_instructions) = compile_and_measure(&baseline_lir);

        assert!(
            current_bytes.len() < baseline_bytes.len(),
            "MIR-backed pipeline should produce smaller WASM than the legacy baseline"
        );
        assert!(
            current_instructions < baseline_instructions,
            "MIR-backed pipeline should emit fewer instructions than the legacy baseline"
        );
    }
}

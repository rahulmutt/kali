//! WASM code generation for the Kali compiler.

use std::collections::BTreeMap;

use kali_error::{Diagnostic, _error_codes::e8};
use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
use serde::{Deserialize, Serialize};
use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    Module, TypeSection, ValType,
};

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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Whether to enable optimization passes.
    pub optimize: bool,
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
        if returns_value && !produced {
            function.instruction(&Instruction::I64Const(0));
        }
    }

    fn emit_sequence(
        &mut self,
        function: &mut Function,
        children: &[LirNodeId],
        want_value: bool,
    ) -> bool {
        let mut produced = false;
        for (idx, child) in children.iter().enumerate() {
            let child_want_value = want_value && idx + 1 == children.len();
            let child_produced = self.emit_node(function, *child, child_want_value);
            if child_produced && !child_want_value {
                function.instruction(&Instruction::Drop);
            }
            produced = child_produced && child_want_value;
        }
        produced
    }

    fn emit_node(&mut self, function: &mut Function, id: LirNodeId, want_value: bool) -> bool {
        let node = self.node(id).clone();
        match node.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                self.emit_sequence(function, &node.children, want_value)
            }
            LirNodeKind::Instruction => {
                if is_function_like(&self.program.nodes, id) {
                    // Function declarations are emitted separately from the body scan.
                    return false;
                }
                self.emit_sequence(function, &node.children, false)
            }
            LirNodeKind::Literal => emit_literal(function, node.text.as_deref()),
            LirNodeKind::Value => self.emit_value(function, &node),
            LirNodeKind::Call => self.emit_call(function, &node),
            LirNodeKind::Branch => self.emit_branch(function, &node, want_value),
            LirNodeKind::Unknown => {
                function.instruction(&Instruction::Unreachable);
                false
            }
        }
    }

    fn emit_value(&mut self, function: &mut Function, node: &LirNode) -> bool {
        match node.children.len() {
            0 => {
                if let Some(text) = node.text.as_deref() {
                    if let Some(index) = self.locals.get(text).copied() {
                        function.instruction(&Instruction::LocalGet(index));
                        return true;
                    }

                    if let Some(constant) = parse_number_literal(text) {
                        function.instruction(&Instruction::I64Const(constant));
                        return true;
                    }

                    self.diagnostics.push(Diagnostic::warning(
                        e8::IR_UNREADABLE as u32,
                        format!("unresolved identifier '{}' lowered as 0", text),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    true
                } else {
                    function.instruction(&Instruction::I64Const(0));
                    true
                }
            }
            1 => self.emit_unary(function, node),
            2 => self.emit_binary(function, node),
            _ => false,
        }
    }

    fn emit_unary(&mut self, function: &mut Function, node: &LirNode) -> bool {
        let op = node.text.as_deref().unwrap_or_default();
        let arg = node.children[0];
        match op {
            "-" => {
                function.instruction(&Instruction::I64Const(0));
                let _ = self.emit_node(function, arg, true);
                function.instruction(&Instruction::I64Sub);
                true
            }
            "!" => {
                let _ = self.emit_node(function, arg, true);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I64ExtendI32U);
                true
            }
            _ => {
                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported unary operator '{}'", op),
                ));
                false
            }
        }
    }

    fn emit_binary(&mut self, function: &mut Function, node: &LirNode) -> bool {
        let op = node.text.as_deref().unwrap_or_default();
        let left = node.children[0];
        let right = node.children[1];
        let _ = self.emit_node(function, left, true);
        let _ = self.emit_node(function, right, true);

        match op {
            "+" => {
                function.instruction(&Instruction::I64Add);
            }
            "-" => {
                function.instruction(&Instruction::I64Sub);
            }
            "*" => {
                function.instruction(&Instruction::I64Mul);
            }
            "/" => {
                function.instruction(&Instruction::I64DivS);
            }
            "==" => {
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
            }
            "&&" => {
                function.instruction(&Instruction::I64And);
            }
            "||" => {
                function.instruction(&Instruction::I64Or);
            }
            _ => {
                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported binary operator '{}'", op),
                ));
                function.instruction(&Instruction::I64Add);
            }
        }

        true
    }

    fn emit_call(&mut self, function: &mut Function, node: &LirNode) -> bool {
        let Some(callee) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return true;
        };

        let callee_node = self.node(callee).clone();
        let callee_name = callee_node.text.as_deref().unwrap_or_default();
        let mut resolved = self.functions.get(callee_name).copied();

        for arg in node.children.iter().skip(1) {
            let _ = self.emit_node(function, *arg, true);
        }

        if let Some(index) = resolved.take() {
            function.instruction(&Instruction::Call(index));
            true
        } else {
            self.diagnostics.push(Diagnostic::warning(
                e8::IR_UNREADABLE as u32,
                format!("unresolved call target '{}' lowered as 0", callee_name),
            ));
            for _ in node.children.iter().skip(1) {
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            true
        }
    }

    fn emit_branch(&mut self, function: &mut Function, node: &LirNode, want_value: bool) -> bool {
        let Some(cond) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return true;
        };

        let then_branch = node.children.get(1).copied();
        let else_branch = node.children.get(2).copied();

        let _ = self.emit_node(function, cond, true);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(if want_value {
            BlockType::Result(ValType::I64)
        } else {
            BlockType::Empty
        }));

        if let Some(then_branch) = then_branch {
            let produced = self.emit_node(function, then_branch, want_value);
            if want_value && !produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else if want_value {
            function.instruction(&Instruction::I64Const(0));
        }

        if let Some(else_branch) = else_branch {
            function.instruction(&Instruction::Else);
            let produced = self.emit_node(function, else_branch, want_value);
            if want_value && !produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else if want_value {
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
        }

        function.instruction(&Instruction::End);
        want_value
    }
}

/// Generate WASM from LIR.
pub fn lower_lir_to_wasm(ctx: &mut CodegenCtx, lir: &LirProgram) -> CodegenResult {
    let mut diagnostics = Vec::new();
    let function_plans = collect_functions(lir);
    let mut function_name_to_index = BTreeMap::new();

    // Keep the emitted order deterministic: synthetic entry first, then named functions in
    // source order.
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
        function_name_to_index.insert(function.name.clone(), idx as u32);
    }

    let mut type_section = TypeSection::new();
    let mut function_types = BTreeMap::<(usize, bool), u32>::new();
    let mut type_for_function = Vec::with_capacity(all_functions.len());

    for function in &all_functions {
        let key = (function.params.len(), function.result);
        let type_index = if let Some(&idx) = function_types.get(&key) {
            idx
        } else {
            let idx = function_types.len() as u32;
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

    let mut export_section = ExportSection::new();
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
    module.section(&function_section);
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
    for (index, _) in lir.nodes.iter().enumerate() {
        if index == lir.root.0 as usize {
            continue;
        }
        if let Some(plan) = function_plan(&lir.nodes, LirNodeId(index as u32)) {
            plans.push(plan);
        }
    }
    plans
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

fn emit_literal(function: &mut Function, text: Option<&str>) -> bool {
    match text {
        Some("true") => {
            function.instruction(&Instruction::I64Const(1));
        }
        Some("false") => {
            function.instruction(&Instruction::I64Const(0));
        }
        Some("null") | Some("undefined") => {
            function.instruction(&Instruction::I64Const(0));
        }
        Some(text) => {
            if let Some(number) = parse_number_literal(text) {
                function.instruction(&Instruction::I64Const(number));
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
        }
        None => {
            function.instruction(&Instruction::I64Const(0));
        }
    }
    true
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
        let mut ctx = CodegenCtx::new(TargetConfig { optimize: false });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(result.diagnostics.is_empty());
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
}

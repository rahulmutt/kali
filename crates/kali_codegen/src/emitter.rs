//! `FunctionEmitter` struct, support types, and lifecycle methods.

use super::*;

#[derive(Clone, Debug)]
pub(crate) struct FunctionPlan {
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
    pub(crate) locals: Vec<String>,
    pub(crate) body: LirNodeId,
    pub(crate) result: bool,
    pub(crate) is_entry: bool,
    pub(crate) flavor: Option<FunctionFlavor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueShape {
    Unknown,
    Scalar,
    Boolean,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObjectEnumerationMode {
    Keys,
    Values,
    Entries,
    ReflectOwnKeys,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlFlowLabelKind {
    If,
    LoopBreak,
    LoopContinue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LoopFrame {
    pub(crate) break_index: usize,
    pub(crate) continue_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EmittedValue {
    pub(crate) produced: bool,
    pub(crate) shape: ValueShape,
}

pub(crate) struct FunctionEmitter<'a> {
    pub(crate) program: &'a LirProgram,
    pub(crate) node_lookup: &'a [LirNode],
    pub(crate) scratch_nodes: Vec<LirNode>,
    pub(crate) functions: &'a BTreeMap<String, u32>,
    pub(crate) env_set_import_index: Option<u32>,
    pub(crate) env_delete_import_index: Option<u32>,
    pub(crate) env_get_import_index: Option<u32>,
    pub(crate) env_has_import_index: Option<u32>,
    pub(crate) cwd_set_import_index: Option<u32>,
    pub(crate) process_exit_import_index: Option<u32>,
    pub(crate) diagnostics: &'a mut Vec<Diagnostic>,
    pub(crate) strings: &'a mut StringPool,
    pub(crate) source_path: Option<PathBuf>,
    pub(crate) current_function_flavor: Option<FunctionFlavor>,
    pub(crate) locals: BTreeMap<String, u32>,
    pub(crate) bindings: BTreeMap<String, LirNodeId>,
    pub(crate) reported_placeholder_fallbacks: HashSet<String>,
    pub(crate) control_frames: Vec<ControlFlowLabelKind>,
    pub(crate) loop_frames: Vec<LoopFrame>,
}

impl<'a> FunctionEmitter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        program: &'a LirProgram,
        functions: &'a BTreeMap<String, u32>,
        env_set_import_index: Option<u32>,
        env_delete_import_index: Option<u32>,
        env_get_import_index: Option<u32>,
        env_has_import_index: Option<u32>,
        cwd_set_import_index: Option<u32>,
        process_exit_import_index: Option<u32>,
        diagnostics: &'a mut Vec<Diagnostic>,
        strings: &'a mut StringPool,
        source_path: Option<PathBuf>,
        current_function_flavor: Option<FunctionFlavor>,
        params: &[String],
        local_names: &[String],
    ) -> Self {
        let mut locals = BTreeMap::new();
        for (idx, name) in params.iter().enumerate() {
            locals.insert(name.clone(), idx as u32);
        }
        for (offset, name) in local_names.iter().enumerate() {
            locals.insert(name.clone(), (params.len() + offset) as u32);
        }

        Self {
            program,
            node_lookup: &program.nodes,
            scratch_nodes: Vec::new(),
            functions,
            env_set_import_index,
            env_delete_import_index,
            env_get_import_index,
            env_has_import_index,
            cwd_set_import_index,
            process_exit_import_index,
            diagnostics,
            strings,
            source_path,
            current_function_flavor,
            locals,
            bindings: BTreeMap::new(),
            reported_placeholder_fallbacks: HashSet::new(),
            control_frames: Vec::new(),
            loop_frames: Vec::new(),
        }
    }

    pub(crate) fn push_control_frame(&mut self, kind: ControlFlowLabelKind) -> usize {
        self.control_frames.push(kind);
        self.control_frames.len() - 1
    }

    pub(crate) fn pop_control_frame(&mut self, kind: ControlFlowLabelKind) {
        let popped = self.control_frames.pop();
        debug_assert_eq!(popped, Some(kind));
    }

    pub(crate) fn control_frame_depth(&self, target_index: usize) -> u32 {
        debug_assert!(target_index < self.control_frames.len());
        (self.control_frames.len() - 1 - target_index) as u32
    }

    pub(crate) fn node(&self, id: LirNodeId) -> &LirNode {
        let index = id.0 as usize;
        if index < self.node_lookup.len() {
            &self.node_lookup[index]
        } else {
            &self.scratch_nodes[index - self.node_lookup.len()]
        }
    }

    pub(crate) fn alloc_scratch_node(
        &mut self,
        kind: LirNodeKind,
        text: Option<String>,
        children: Vec<LirNodeId>,
    ) -> LirNodeId {
        let id = LirNodeId((self.node_lookup.len() + self.scratch_nodes.len()) as u32);
        self.scratch_nodes.push(LirNode {
            kind,
            text,
            children,
            function_flavor: None,
        });
        id
    }

    pub(crate) fn push_placeholder_fallback_diagnostic(&mut self, kind: &str, name: &str) {
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
}

//! Mid-level IR (MIR) for the Kali compiler.
//!
//! MIR is a conservative structural lowering of HIR that preserves the source
//! shape while providing a stable bridge for later memory/ownership analysis.

use std::collections::{BTreeMap, BTreeSet};

use kali_hir::{HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};

/// Canonical ownership classes used by MIR analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwnershipClass {
    Stack,
    OwnedHeap,
    SharedHeap,
    Borrowed,
}

/// Canonical layout descriptor used by MIR analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LayoutDescriptor {
    Scalar(String),
    Struct {
        fields: Vec<(String, Box<LayoutDescriptor>)>,
    },
    Array {
        element: Box<LayoutDescriptor>,
        length: Option<usize>,
    },
    Closure {
        captures: Vec<String>,
    },
    TaggedVal,
}

impl LayoutDescriptor {
    fn scalar(name: impl Into<String>) -> Self {
        Self::Scalar(name.into())
    }
}

/// MIR binding classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirBindingKind {
    Parameter,
    Local,
    Function,
    Import,
}

/// MIR binding metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirBinding {
    pub name: String,
    pub kind: MirBindingKind,
    pub ownership: OwnershipClass,
    pub layout: LayoutDescriptor,
    pub escapes: bool,
    pub captured_by: Vec<String>,
}

/// MIR function/module scope metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirFunctionKind {
    Module,
    Function,
    Closure,
}

/// MIR function/module scope analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFunction {
    pub name: Option<String>,
    pub kind: MirFunctionKind,
    pub bindings: Vec<MirBinding>,
}

impl MirFunction {
    pub fn binding(&self, name: &str) -> Option<&MirBinding> {
        self.bindings.iter().find(|binding| binding.name == name)
    }
}

/// MIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirNodeKind {
    Program,
    Block,
    Function,
    Decl,
    Expr,
    Call,
    Literal,
    ControlFlow,
    Unknown,
}

/// MIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MirNodeId(pub u32);

impl MirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// MIR place reference (an addressable location).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlaceRef(pub MirNodeId);

impl PlaceRef {
    pub fn new(id: MirNodeId) -> Self {
        Self(id)
    }
}

/// MIR place value (the loaded value from a place).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlaceValue(pub MirNodeId);

impl PlaceValue {
    pub fn new(id: MirNodeId) -> Self {
        Self(id)
    }
}

/// MIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirNode {
    pub kind: MirNodeKind,
    pub text: Option<String>,
    pub children: Vec<MirNodeId>,
}

impl MirNode {
    pub fn new(kind: MirNodeKind) -> Self {
        Self {
            kind,
            text: None,
            children: Vec::new(),
        }
    }

    pub fn with_text(kind: MirNodeKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: Some(text.into()),
            children: Vec::new(),
        }
    }
}

/// MIR builder.
#[derive(Default)]
pub struct MirBuilder {
    nodes: Vec<MirNode>,
}

impl MirBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, kind: MirNodeKind) -> MirNodeId {
        let id = MirNodeId(self.nodes.len() as u32);
        self.nodes.push(MirNode::new(kind));
        id
    }

    pub fn alloc_text(&mut self, kind: MirNodeKind, text: impl Into<String>) -> MirNodeId {
        let id = MirNodeId(self.nodes.len() as u32);
        self.nodes.push(MirNode::with_text(kind, text));
        id
    }

    pub fn node_mut(&mut self, id: MirNodeId) -> Option<&mut MirNode> {
        self.nodes.get_mut(id.0 as usize)
    }
}

/// MIR lowering result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirProgram {
    pub root: MirNodeId,
    pub nodes: Vec<MirNode>,
    pub functions: Vec<MirFunction>,
}

impl MirProgram {
    pub fn module_scope(&self) -> Option<&MirFunction> {
        self.functions
            .iter()
            .find(|function| function.kind == MirFunctionKind::Module)
    }

    pub fn function(&self, name: &str) -> Option<&MirFunction> {
        self.functions
            .iter()
            .find(|function| function.name.as_deref() == Some(name))
    }
}

/// MIR lowering from HIR.
#[derive(Default)]
pub struct MirLowerer;

impl MirLowerer {
    pub fn new() -> Self {
        Self
    }

    /// Preserve the old shape-oriented API.
    pub fn lower_hir(&self, _hir: HirNodeId) -> MirNodeId {
        MirNodeId::new(0)
    }

    pub fn lower_hir_result(&self, hir: &HirLoweringResult) -> MirProgram {
        let mut builder = MirBuilder::new();
        let root = self.lower_hir_node(&mut builder, &hir.nodes, hir.root);
        let functions = OwnershipAnalyzer::new(&hir.nodes).analyze_program(hir.root);
        MirProgram {
            root,
            nodes: builder.nodes,
            functions,
        }
    }

    fn lower_hir_node(
        &self,
        builder: &mut MirBuilder,
        nodes: &[HirNode],
        id: HirNodeId,
    ) -> MirNodeId {
        let node = &nodes[id.0 as usize];
        let kind = map_kind(&node.kind);
        let mir_id = match node.text.as_ref() {
            Some(text) => builder.alloc_text(kind, text.clone()),
            None => builder.alloc(kind),
        };
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(self.lower_hir_node(builder, nodes, *child));
        }
        if let Some(mir_node) = builder.node_mut(mir_id) {
            mir_node.children = children;
        }
        mir_id
    }
}

fn map_kind(kind: &HirNodeKind) -> MirNodeKind {
    match kind {
        HirNodeKind::Program => MirNodeKind::Program,
        HirNodeKind::Block => MirNodeKind::Block,
        HirNodeKind::FunctionDecl
        | HirNodeKind::FunctionExpr
        | HirNodeKind::ClassDecl
        | HirNodeKind::ClassExpr => MirNodeKind::Function,
        HirNodeKind::VarDecl
        | HirNodeKind::VarDeclarator
        | HirNodeKind::ImportDecl
        | HirNodeKind::ExportDecl
        | HirNodeKind::TypeDecl
        | HirNodeKind::InterfaceDecl
        | HirNodeKind::EnumDecl => MirNodeKind::Decl,
        HirNodeKind::IfStmt
        | HirNodeKind::ForStmt
        | HirNodeKind::ForInStmt
        | HirNodeKind::ForOfStmt
        | HirNodeKind::WhileStmt
        | HirNodeKind::DoWhileStmt
        | HirNodeKind::SwitchStmt
        | HirNodeKind::TryStmt
        | HirNodeKind::ReturnStmt
        | HirNodeKind::BreakStmt
        | HirNodeKind::ContinueStmt
        | HirNodeKind::ThrowStmt
        | HirNodeKind::DebuggerStmt
        | HirNodeKind::LabeledStmt
        | HirNodeKind::WithStmt => MirNodeKind::ControlFlow,
        HirNodeKind::Literal => MirNodeKind::Literal,
        HirNodeKind::CallExpr => MirNodeKind::Call,
        HirNodeKind::MemberExpr
        | HirNodeKind::NewExpr
        | HirNodeKind::BinaryExpr
        | HirNodeKind::LogicalExpr
        | HirNodeKind::UnaryExpr
        | HirNodeKind::UpdateExpr
        | HirNodeKind::AssignmentExpr
        | HirNodeKind::ConditionalExpr
        | HirNodeKind::SequenceExpr
        | HirNodeKind::ArrayExpr
        | HirNodeKind::ObjectExpr
        | HirNodeKind::ObjectProperty
        | HirNodeKind::OptionalChain
        | HirNodeKind::ChainExpr
        | HirNodeKind::Spread
        | HirNodeKind::Rest
        | HirNodeKind::ImportExpr
        | HirNodeKind::JsxElement
        | HirNodeKind::JsxFragment
        | HirNodeKind::TypeAssertion
        | HirNodeKind::SatisfiesExpr
        | HirNodeKind::MetaProperty
        | HirNodeKind::YieldExpr
        | HirNodeKind::AwaitExpr
        | HirNodeKind::ThisExpr
        | HirNodeKind::Ident
        | HirNodeKind::ExprStmt
        | HirNodeKind::TemplateLiteral => MirNodeKind::Expr,
        HirNodeKind::Unknown => MirNodeKind::Unknown,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UseContext {
    Normal,
    Return,
    Escape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BindingState {
    name: String,
    kind: MirBindingKind,
    ownership: OwnershipClass,
    layout: LayoutDescriptor,
    returned: bool,
    escaped_via_flow: bool,
    captured_by: BTreeSet<String>,
}

impl BindingState {
    fn new(name: impl Into<String>, kind: MirBindingKind, layout: LayoutDescriptor) -> Self {
        Self {
            name: name.into(),
            kind,
            ownership: default_ownership(kind),
            layout,
            returned: false,
            escaped_via_flow: false,
            captured_by: BTreeSet::new(),
        }
    }

    fn finalize(self) -> MirBinding {
        let escapes = self.returned || self.escaped_via_flow || !self.captured_by.is_empty();
        let ownership = if !self.captured_by.is_empty() {
            OwnershipClass::SharedHeap
        } else if self.returned || self.escaped_via_flow {
            match self.kind {
                MirBindingKind::Local | MirBindingKind::Function => OwnershipClass::OwnedHeap,
                MirBindingKind::Parameter | MirBindingKind::Import => self.ownership,
            }
        } else {
            self.ownership
        };
        MirBinding {
            name: self.name,
            kind: self.kind,
            ownership,
            layout: self.layout,
            escapes,
            captured_by: self.captured_by.into_iter().collect(),
        }
    }
}

fn default_ownership(kind: MirBindingKind) -> OwnershipClass {
    match kind {
        MirBindingKind::Parameter => OwnershipClass::Borrowed,
        MirBindingKind::Function => OwnershipClass::Stack,
        MirBindingKind::Import => OwnershipClass::Borrowed,
        MirBindingKind::Local => OwnershipClass::Stack,
    }
}

fn parameter_escape_flags(function: &MirFunction) -> Vec<bool> {
    function
        .bindings
        .iter()
        .filter(|binding| binding.kind == MirBindingKind::Parameter)
        .map(|binding| binding.escapes)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeState {
    label: String,
    kind: MirFunctionKind,
    bindings: Vec<BindingState>,
    binding_index: BTreeMap<String, usize>,
    function_aliases: BTreeMap<String, String>,
}

impl ScopeState {
    fn new(label: impl Into<String>, kind: MirFunctionKind) -> Self {
        Self {
            label: label.into(),
            kind,
            bindings: Vec::new(),
            binding_index: BTreeMap::new(),
            function_aliases: BTreeMap::new(),
        }
    }

    fn define(&mut self, name: impl Into<String>, kind: MirBindingKind, layout: LayoutDescriptor) {
        let name = name.into();
        let binding = BindingState::new(name.clone(), kind, layout);
        if let Some(index) = self.binding_index.get(&name).copied() {
            self.bindings[index] = binding;
        } else {
            let index = self.bindings.len();
            self.bindings.push(binding);
            self.binding_index.insert(name, index);
        }
    }

    fn get_binding_index(&self, name: &str) -> Option<usize> {
        self.binding_index.get(name).copied()
    }

    fn get_binding_mut(&mut self, name: &str) -> Option<&mut BindingState> {
        let index = self.get_binding_index(name)?;
        self.bindings.get_mut(index)
    }

    fn alias_function(
        &mut self,
        binding_name: impl Into<String>,
        function_name: impl Into<String>,
    ) {
        self.function_aliases
            .insert(binding_name.into(), function_name.into());
    }

    fn finalize(self) -> MirFunction {
        MirFunction {
            name: if self.kind == MirFunctionKind::Module {
                None
            } else {
                Some(self.label)
            },
            kind: self.kind,
            bindings: self
                .bindings
                .into_iter()
                .map(BindingState::finalize)
                .collect(),
        }
    }
}

struct OwnershipAnalyzer<'a> {
    nodes: &'a [HirNode],
    functions: Vec<MirFunction>,
    scope_stack: Vec<ScopeState>,
    synthetic_function_counter: usize,
}

impl<'a> OwnershipAnalyzer<'a> {
    fn new(nodes: &'a [HirNode]) -> Self {
        Self {
            nodes,
            functions: Vec::new(),
            scope_stack: Vec::new(),
            synthetic_function_counter: 0,
        }
    }

    fn analyze_program(mut self, root: HirNodeId) -> Vec<MirFunction> {
        self.push_scope("<module>", MirFunctionKind::Module);
        self.precollect_scope_bindings(root);
        self.walk_scope_node(root, UseContext::Normal);
        self.pop_scope_and_record();
        self.functions
    }

    fn push_scope(&mut self, label: impl Into<String>, kind: MirFunctionKind) {
        self.scope_stack.push(ScopeState::new(label, kind));
    }

    fn pop_scope_and_record(&mut self) {
        if let Some(scope) = self.scope_stack.pop() {
            self.functions.push(scope.finalize());
        }
    }

    fn current_scope_label(&self) -> String {
        self.scope_stack
            .last()
            .map(|scope| scope.label.clone())
            .unwrap_or_else(|| "<module>".to_string())
    }

    fn current_scope_index(&self) -> usize {
        self.scope_stack.len().saturating_sub(1)
    }

    fn current_scope_mut(&mut self) -> Option<&mut ScopeState> {
        self.scope_stack.last_mut()
    }

    fn precollect_scope_bindings(&mut self, node_id: HirNodeId) {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Program | HirNodeKind::Block => {
                for child in &node.children {
                    self.precollect_scope_bindings(*child);
                }
            }
            HirNodeKind::VarDecl => {
                for child in &node.children {
                    self.precollect_scope_bindings(*child);
                }
            }
            HirNodeKind::VarDeclarator => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Local,
                        LayoutDescriptor::TaggedVal,
                    );
                }
            }
            HirNodeKind::ImportDecl => {
                for child in &node.children {
                    self.collect_import_bindings(*child);
                }
            }
            HirNodeKind::FunctionDecl => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
            }
            HirNodeKind::FunctionExpr => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
            }
            HirNodeKind::ClassDecl => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Local,
                        LayoutDescriptor::TaggedVal,
                    );
                }
            }
            _ => {
                for child in &node.children {
                    self.precollect_scope_bindings(*child);
                }
            }
        }
    }

    fn define_binding(&mut self, name: String, kind: MirBindingKind, layout: LayoutDescriptor) {
        if let Some(scope) = self.current_scope_mut() {
            scope.define(name, kind, layout);
        }
    }

    fn collect_import_bindings(&mut self, node_id: HirNodeId) {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Ident => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Import,
                        LayoutDescriptor::TaggedVal,
                    );
                }
            }
            HirNodeKind::ImportDecl => {
                for child in &node.children {
                    self.collect_import_bindings(*child);
                }
            }
            _ => {}
        }
    }

    fn walk_scope_node(&mut self, node_id: HirNodeId, context: UseContext) {
        let node = &self.nodes[node_id.0 as usize];
        let kind = node.kind.clone();
        let text = node.text.clone();
        let children = node.children.clone();

        match kind {
            HirNodeKind::Program | HirNodeKind::Block => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
            HirNodeKind::VarDecl => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
            HirNodeKind::VarDeclarator => {
                if let Some(name) = text.as_ref() {
                    if let Some(init) = children.get(1).copied() {
                        let init_node = &self.nodes[init.0 as usize];
                        let layout = self.infer_layout(init);
                        if let Some(scope) = self.current_scope_mut() {
                            if let Some(binding) = scope.get_binding_mut(name) {
                                binding.layout = layout;
                                if matches!(init_node.kind, HirNodeKind::FunctionExpr) {
                                    binding.kind = MirBindingKind::Function;
                                    binding.layout = LayoutDescriptor::Closure {
                                        captures: Vec::new(),
                                    };
                                }
                            }
                        }
                        self.walk_scope_node(init, UseContext::Normal);
                        if matches!(init_node.kind, HirNodeKind::FunctionExpr) {
                            if let Some(scope) = self.current_scope_mut() {
                                if let Some(function_name) = init_node.text.as_ref() {
                                    scope.alias_function(name.clone(), function_name.clone());
                                }
                            }
                        }
                    }
                }
            }
            HirNodeKind::FunctionDecl => {
                let function_name = text.unwrap_or_else(|| self.next_function_name());
                let body = children.last().copied();
                let params_end = body.map_or(children.len(), |_| children.len().saturating_sub(1));
                self.push_scope(function_name.clone(), MirFunctionKind::Function);
                if let Some(scope) = self.current_scope_mut() {
                    scope.define(
                        function_name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
                for child in children.iter().take(params_end) {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
                        if let Some(scope) = self.current_scope_mut() {
                            scope.define(
                                param_name.clone(),
                                MirBindingKind::Parameter,
                                LayoutDescriptor::TaggedVal,
                            );
                        }
                    }
                }
                if let Some(body) = body {
                    self.precollect_scope_bindings(body);
                    self.walk_scope_node(body, UseContext::Normal);
                }
                self.pop_scope_and_record();
            }
            HirNodeKind::FunctionExpr => {
                let function_name = text.unwrap_or_else(|| self.next_function_name());
                let body = children.last().copied();
                let params_end = body.map_or(children.len(), |_| children.len().saturating_sub(1));
                self.push_scope(function_name.clone(), MirFunctionKind::Closure);
                if let Some(scope) = self.current_scope_mut() {
                    scope.define(
                        function_name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
                for child in children.iter().take(params_end) {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
                        if let Some(scope) = self.current_scope_mut() {
                            scope.define(
                                param_name.clone(),
                                MirBindingKind::Parameter,
                                LayoutDescriptor::TaggedVal,
                            );
                        }
                    }
                }
                if let Some(body) = body {
                    self.precollect_scope_bindings(body);
                    self.walk_scope_node(body, UseContext::Normal);
                }
                self.pop_scope_and_record();
            }
            HirNodeKind::ImportDecl => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
            HirNodeKind::ReturnStmt => {
                for child in children {
                    self.walk_scope_node(child, UseContext::Return);
                }
            }
            HirNodeKind::CallExpr => {
                let mut direct_call_escape_flags = None;
                if let Some(callee) = children.first().copied() {
                    let callee_node = &self.nodes[callee.0 as usize];
                    match callee_node.kind {
                        HirNodeKind::FunctionExpr => {
                            let functions_before = self.functions.len();
                            self.walk_scope_node(callee, UseContext::Normal);
                            direct_call_escape_flags = self
                                .functions
                                .get(functions_before..)
                                .and_then(|functions| functions.last())
                                .map(parameter_escape_flags);
                        }
                        HirNodeKind::Ident => {
                            self.walk_scope_node(callee, UseContext::Normal);
                            if let Some(name) = callee_node.text.as_deref() {
                                if let Some((scope_index, binding_index)) =
                                    self.resolve_binding(name)
                                {
                                    let function_name =
                                        self.scope_stack.get(scope_index).and_then(|scope| {
                                            scope.function_aliases.get(name).cloned().or_else(
                                                || {
                                                    scope.bindings.get(binding_index).and_then(
                                                        |binding| {
                                                            (binding.kind
                                                                == MirBindingKind::Function)
                                                                .then(|| name.to_string())
                                                        },
                                                    )
                                                },
                                            )
                                        });
                                    if let Some(function_name) = function_name {
                                        direct_call_escape_flags =
                                            self.function_parameter_escape_flags(&function_name);
                                    }
                                }
                            }
                        }
                        _ => {
                            self.walk_scope_node(callee, UseContext::Normal);
                        }
                    }
                }

                for (index, child) in children.into_iter().enumerate().skip(1) {
                    let should_escape = direct_call_escape_flags
                        .as_ref()
                        .and_then(|flags| flags.get(index - 1).copied())
                        .unwrap_or(true);
                    self.walk_scope_node(
                        child,
                        if should_escape {
                            UseContext::Escape
                        } else {
                            UseContext::Normal
                        },
                    );
                }
            }
            HirNodeKind::NewExpr => {
                for (index, child) in children.into_iter().enumerate() {
                    let child_context = if index == 0 {
                        UseContext::Normal
                    } else {
                        UseContext::Escape
                    };
                    self.walk_scope_node(child, child_context);
                }
            }
            HirNodeKind::ArrayExpr => {
                for child in children {
                    self.walk_scope_node(child, UseContext::Escape);
                }
            }
            HirNodeKind::ObjectExpr => {
                for child in children {
                    self.walk_scope_node(child, UseContext::Escape);
                }
            }
            HirNodeKind::ObjectProperty => {
                if let Some(key) = children.first().copied() {
                    self.walk_scope_node(key, UseContext::Normal);
                }
                if let Some(value) = children.get(1).copied() {
                    self.walk_scope_node(value, UseContext::Escape);
                }
            }
            HirNodeKind::Unknown if text.as_deref() == Some("unknown") && !children.is_empty() => {
                for child in children {
                    self.walk_scope_node(child, UseContext::Escape);
                }
            }
            HirNodeKind::AssignmentExpr => {
                let left = children.first().copied();
                let right = children.get(1).copied();
                if let Some(left) = left {
                    self.walk_scope_node(left, UseContext::Normal);
                }
                if let Some(right) = right {
                    let rhs_context = left.is_some_and(|left| self.is_heap_store_target(left));
                    self.walk_scope_node(
                        right,
                        if rhs_context {
                            UseContext::Escape
                        } else {
                            context
                        },
                    );
                }
            }
            HirNodeKind::SequenceExpr => {
                for child in children.iter().take(children.len().saturating_sub(1)) {
                    self.walk_scope_node(*child, UseContext::Normal);
                }
                if let Some(last) = children.last().copied() {
                    self.walk_scope_node(last, context);
                }
            }
            HirNodeKind::Ident => {
                if let Some(name) = text.as_ref() {
                    self.resolve_use(name, context);
                }
            }
            _ => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
        }
    }

    fn resolve_use(&mut self, name: &str, context: UseContext) {
        let Some((scope_index, binding_index)) = self.resolve_binding(name) else {
            return;
        };

        let current_index = self.current_scope_index();
        let current_label = self.current_scope_label();
        if scope_index < current_index {
            let captured_by = current_label.clone();
            if let Some(binding) = self
                .scope_stack
                .get_mut(scope_index)
                .and_then(|scope| scope.bindings.get_mut(binding_index))
            {
                binding.captured_by.insert(captured_by);
            }
        }

        if matches!(context, UseContext::Return) {
            if let Some(binding) = self
                .scope_stack
                .get_mut(scope_index)
                .and_then(|scope| scope.bindings.get_mut(binding_index))
            {
                binding.returned = true;
            }
        }

        if matches!(context, UseContext::Escape) {
            if let Some(binding) = self
                .scope_stack
                .get_mut(scope_index)
                .and_then(|scope| scope.bindings.get_mut(binding_index))
            {
                binding.escaped_via_flow = true;
            }
        }
    }

    fn resolve_binding(&self, name: &str) -> Option<(usize, usize)> {
        for (scope_index, scope) in self.scope_stack.iter().enumerate().rev() {
            if let Some(binding_index) = scope.get_binding_index(name) {
                return Some((scope_index, binding_index));
            }
        }
        None
    }

    fn infer_layout(&self, node_id: HirNodeId) -> LayoutDescriptor {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Literal => match node.text.as_deref() {
                Some("true") | Some("false") => LayoutDescriptor::scalar("bool"),
                Some("null") | Some("undefined") => LayoutDescriptor::scalar("unknown"),
                Some(text) if text.parse::<i64>().is_ok() || text.parse::<f64>().is_ok() => {
                    LayoutDescriptor::scalar("number")
                }
                Some(_) => LayoutDescriptor::scalar("string"),
                None => LayoutDescriptor::TaggedVal,
            },
            HirNodeKind::ArrayExpr => {
                let element = node
                    .children
                    .first()
                    .copied()
                    .map(|child| Box::new(self.infer_layout(child)))
                    .unwrap_or_else(|| Box::new(LayoutDescriptor::TaggedVal));
                LayoutDescriptor::Array {
                    element,
                    length: Some(node.children.len()),
                }
            }
            HirNodeKind::ObjectExpr => {
                let mut fields = Vec::new();
                for child in &node.children {
                    let property = &self.nodes[child.0 as usize];
                    if matches!(property.kind, HirNodeKind::ObjectProperty)
                        && property.children.len() >= 2
                    {
                        let key = self.layout_field_name(property);
                        let value = property.children[1];
                        fields.push((key, Box::new(self.infer_layout(value))));
                    }
                }
                if fields.is_empty() {
                    LayoutDescriptor::TaggedVal
                } else {
                    LayoutDescriptor::Struct { fields }
                }
            }
            HirNodeKind::FunctionExpr | HirNodeKind::FunctionDecl => LayoutDescriptor::Closure {
                captures: Vec::new(),
            },
            HirNodeKind::Ident => self
                .resolve_binding_layout(node.text.as_deref().unwrap_or_default())
                .unwrap_or(LayoutDescriptor::TaggedVal),
            HirNodeKind::CallExpr | HirNodeKind::NewExpr | HirNodeKind::ImportExpr => {
                LayoutDescriptor::TaggedVal
            }
            HirNodeKind::BinaryExpr => self.infer_binary_layout(node),
            HirNodeKind::UnaryExpr => self.infer_unary_layout(node),
            HirNodeKind::ConditionalExpr | HirNodeKind::SequenceExpr => node
                .children
                .last()
                .copied()
                .map(|child| self.infer_layout(child))
                .unwrap_or(LayoutDescriptor::TaggedVal),
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    fn infer_binary_layout(&self, node: &HirNode) -> LayoutDescriptor {
        let op = node.text.as_deref().unwrap_or_default();
        match op {
            "+" | "-" | "*" | "/" | "%" | "**" => LayoutDescriptor::scalar("number"),
            "==" | "!=" | "===" | "!==" | "<" | "<=" | ">" | ">=" | "&&" | "||" => {
                LayoutDescriptor::scalar("bool")
            }
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    fn infer_unary_layout(&self, node: &HirNode) -> LayoutDescriptor {
        match node.text.as_deref().unwrap_or_default() {
            "!" => LayoutDescriptor::scalar("bool"),
            "-" | "+" | "~" => LayoutDescriptor::scalar("number"),
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    fn resolve_binding_layout(&self, name: &str) -> Option<LayoutDescriptor> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(index) = scope.get_binding_index(name) {
                return scope
                    .bindings
                    .get(index)
                    .map(|binding| binding.layout.clone());
            }
        }
        None
    }

    fn function_parameter_escape_flags(&self, name: &str) -> Option<Vec<bool>> {
        self.functions
            .iter()
            .rev()
            .find(|function| function.name.as_deref() == Some(name))
            .map(parameter_escape_flags)
    }

    fn layout_field_name(&self, node: &HirNode) -> String {
        if let Some(key) = node.children.first() {
            let key_node = &self.nodes[key.0 as usize];
            if let Some(text) = key_node.text.as_ref() {
                return text.clone();
            }
        }

        node.text
            .clone()
            .unwrap_or_else(|| format!("field_{}", node.children.len()))
    }

    fn next_function_name(&mut self) -> String {
        let name = format!("__kali_fn_{}", self.synthetic_function_counter);
        self.synthetic_function_counter += 1;
        name
    }

    fn is_heap_store_target(&self, node_id: HirNodeId) -> bool {
        matches!(
            self.nodes[node_id.0 as usize].kind,
            HirNodeKind::MemberExpr | HirNodeKind::OptionalChain | HirNodeKind::ChainExpr
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kali_common::FileId;
    use kali_hir::{
        HirLowerer, HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult,
    };
    use kali_lexer::Lexer;
    use kali_parser::Parser;

    fn parse_and_lower_hir(source: &str) -> HirLoweringResult {
        let lexer = Lexer::new(FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = Parser::new(FileId::new(0), tokens);
        let statements = parser.parse(None).statements;
        let mut lowerer = HirLowerer::new();
        lowerer.lower_statements(&statements)
    }

    fn analyze(source: &str) -> MirProgram {
        let hir = parse_and_lower_hir(source);
        MirLowerer::new().lower_hir_result(&hir)
    }

    #[test]
    fn test_mir_lowering_preserves_program_shape() {
        let hir = parse_and_lower_hir("const answer = 40 + 2;");
        let mir = MirLowerer::new().lower_hir_result(&hir);

        assert_eq!(mir.nodes[mir.root.0 as usize].kind, MirNodeKind::Program);
        assert_eq!(mir.nodes[mir.root.0 as usize].children.len(), 1);
        assert_eq!(
            mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize].kind,
            MirNodeKind::Decl
        );
    }

    #[test]
    fn test_call_expressions_lower_to_call_nodes() {
        let hir = parse_and_lower_hir("foo(bar, 1);");
        let mir = MirLowerer::new().lower_hir_result(&hir);
        let expr_stmt = &mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize];
        let call = expr_stmt
            .children
            .iter()
            .map(|child| &mir.nodes[child.0 as usize])
            .find(|node| node.kind == MirNodeKind::Call)
            .expect("call node");
        assert_eq!(call.children.len(), 3);
    }

    #[test]
    fn test_stack_local_bindings_stay_stack_allocated() {
        let mir = analyze("const answer = 40 + 2;");
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.kind, MirBindingKind::Local);
        assert_eq!(binding.ownership, OwnershipClass::Stack);
        assert!(!binding.escapes);
        assert_eq!(binding.layout, LayoutDescriptor::scalar("number"));
    }

    #[test]
    fn test_returned_bindings_become_owned_heap() {
        let mir = analyze("function make() { const answer = 40 + 2; return answer; }");
        let function = mir.function("make").expect("make function");
        let binding = function.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
        assert!(binding.escapes);
        assert_eq!(binding.layout, LayoutDescriptor::scalar("number"));
    }

    #[test]
    fn test_captured_bindings_become_shared_heap() {
        let mir = analyze(
            "function outer() { const answer = 1; function inner() { return answer; } return inner; }",
        );
        let function = mir.function("outer").expect("outer function");
        let binding = function.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::SharedHeap);
        assert!(binding.escapes);
        assert_eq!(binding.captured_by, vec!["inner".to_string()]);
    }

    #[test]
    fn test_call_arguments_escape_to_unknown_callees() {
        let mir = analyze("const answer = 1; sink(answer);");
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
        assert!(binding.escapes);
    }

    #[test]
    fn test_inline_pure_function_calls_do_not_force_argument_escape() {
        let mir = analyze("const answer = 1; (function identity(x) { return 0; })(answer);");
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::Stack);
        assert!(!binding.escapes);
    }

    #[test]
    fn test_inline_leaking_function_calls_still_escape_arguments() {
        let mir = analyze("const answer = 1; (function leak(x) { return x; })(answer);");
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
        assert!(binding.escapes);
    }

    #[test]
    fn test_aliased_function_expressions_preserve_direct_call_precision() {
        let mir = analyze(
            "const identity = function(x) { return 0; }; const answer = 1; identity(answer);",
        );
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::Stack);
        assert!(!binding.escapes);
    }

    #[test]
    fn test_aliased_function_expressions_still_track_nested_closure_escapes() {
        let mir = analyze(
            "const leak = function outer(x) { function inner() { return x; } return inner; }; const answer = 1; leak(answer);",
        );
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
        assert!(binding.escapes);
    }

    #[test]
    fn test_object_literal_values_escape_without_treating_keys_as_identifiers() {
        let hir = HirLoweringResult {
            root: HirNodeId::new(0),
            nodes: vec![
                HirNode {
                    kind: HirNodeKind::Program,
                    span: None,
                    text: None,
                    children: vec![HirNodeId::new(1), HirNodeId::new(5)],
                },
                HirNode {
                    kind: HirNodeKind::VarDecl,
                    span: None,
                    text: Some("const".to_string()),
                    children: vec![HirNodeId::new(2)],
                },
                HirNode {
                    kind: HirNodeKind::VarDeclarator,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![HirNodeId::new(3), HirNodeId::new(4)],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::Literal,
                    span: None,
                    text: Some("1".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::VarDecl,
                    span: None,
                    text: Some("const".to_string()),
                    children: vec![HirNodeId::new(7)],
                },
                HirNode {
                    kind: HirNodeKind::ObjectExpr,
                    span: None,
                    text: None,
                    children: vec![HirNodeId::new(8)],
                },
                HirNode {
                    kind: HirNodeKind::VarDeclarator,
                    span: None,
                    text: Some("bag".to_string()),
                    children: vec![HirNodeId::new(9), HirNodeId::new(6)],
                },
                HirNode {
                    kind: HirNodeKind::ObjectProperty,
                    span: None,
                    text: Some("init".to_string()),
                    children: vec![HirNodeId::new(10), HirNodeId::new(11)],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("bag".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::Literal,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![],
                },
            ],
            diagnostics: vec![],
        };

        let mir = MirLowerer::new().lower_hir_result(&hir);
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
        assert!(binding.escapes);
        assert_eq!(binding.captured_by, Vec::<String>::new());
    }

    #[test]
    fn test_array_element_values_escape_to_heap_storage() {
        let hir = HirLoweringResult {
            root: HirNodeId::new(0),
            nodes: vec![
                HirNode {
                    kind: HirNodeKind::Program,
                    span: None,
                    text: None,
                    children: vec![HirNodeId::new(1), HirNodeId::new(5)],
                },
                HirNode {
                    kind: HirNodeKind::VarDecl,
                    span: None,
                    text: Some("const".to_string()),
                    children: vec![HirNodeId::new(2)],
                },
                HirNode {
                    kind: HirNodeKind::VarDeclarator,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![HirNodeId::new(3), HirNodeId::new(4)],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::Literal,
                    span: None,
                    text: Some("1".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::VarDecl,
                    span: None,
                    text: Some("const".to_string()),
                    children: vec![HirNodeId::new(6)],
                },
                HirNode {
                    kind: HirNodeKind::VarDeclarator,
                    span: None,
                    text: Some("bag".to_string()),
                    children: vec![HirNodeId::new(7), HirNodeId::new(8)],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("bag".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::ArrayExpr,
                    span: None,
                    text: None,
                    children: vec![HirNodeId::new(9)],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![],
                },
            ],
            diagnostics: vec![],
        };

        let mir = MirLowerer::new().lower_hir_result(&hir);
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
        assert!(binding.escapes);
        assert!(binding.captured_by.is_empty());
    }

    #[test]
    fn test_assignment_into_member_expressions_marks_rhs_escape() {
        let hir = HirLoweringResult {
            root: HirNodeId::new(0),
            nodes: vec![
                HirNode {
                    kind: HirNodeKind::Program,
                    span: None,
                    text: None,
                    children: vec![HirNodeId::new(1), HirNodeId::new(5)],
                },
                HirNode {
                    kind: HirNodeKind::VarDecl,
                    span: None,
                    text: Some("const".to_string()),
                    children: vec![HirNodeId::new(2)],
                },
                HirNode {
                    kind: HirNodeKind::VarDeclarator,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![HirNodeId::new(3), HirNodeId::new(4)],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::Literal,
                    span: None,
                    text: Some("1".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::AssignmentExpr,
                    span: None,
                    text: Some("=".to_string()),
                    children: vec![HirNodeId::new(6), HirNodeId::new(8)],
                },
                HirNode {
                    kind: HirNodeKind::MemberExpr,
                    span: None,
                    text: Some("value".to_string()),
                    children: vec![HirNodeId::new(7)],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("box".to_string()),
                    children: vec![],
                },
                HirNode {
                    kind: HirNodeKind::Ident,
                    span: None,
                    text: Some("answer".to_string()),
                    children: vec![],
                },
            ],
            diagnostics: vec![],
        };

        let mir = MirLowerer::new().lower_hir_result(&hir);
        let module = mir.module_scope().expect("module scope");
        let binding = module.binding("answer").expect("answer binding");

        assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
        assert!(binding.escapes);
        assert!(binding.captured_by.is_empty());
    }
}

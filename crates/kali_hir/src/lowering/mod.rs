//! AST → HIR lowering passes (one `impl HirLowerer` per responsibility).

use crate::builder::HirBuilder;
use crate::node::{HirNodeId, HirNodeKind};
use crate::result::{FunctionFlavor, LoweringResult};
use kali_ast::{NodeId, Statement, AST};
use kali_error::diagnostic::Diagnostic;

/// HIR lowering from AST.
pub struct HirLowerer {
    pub(crate) builder: HirBuilder,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) function_flavors: Vec<(HirNodeId, FunctionFlavor)>,
    pub(crate) synthetic_function_counter: usize,
}

macro_rules! push_child {
    ($this:expr, $parent:expr, $child:expr) => {{
        let child = $child;
        $this.push_child($parent, child);
    }};
}

impl HirLowerer {
    pub fn new() -> Self {
        Self {
            builder: HirBuilder::new(),
            diagnostics: Vec::new(),
            function_flavors: Vec::new(),
            synthetic_function_counter: 0,
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    /// Lower a complete program from parsed statements.
    pub fn lower_statements(&mut self, statements: &[Statement]) -> LoweringResult {
        self.clear_diagnostics();
        self.builder = HirBuilder::new();
        self.function_flavors.clear();
        self.synthetic_function_counter = 0;

        let root = self.builder.alloc(HirNodeKind::Program, None);
        let mut children = Vec::with_capacity(statements.len());
        for statement in statements {
            children.push(self.lower_statement(statement));
        }
        if let Some(node) = self.builder.node_mut(root) {
            node.children = children;
        }

        LoweringResult {
            root,
            nodes: self.builder.nodes.clone(),
            function_flavors: self.function_flavors.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    /// Lower an AST root by using its statements as the source of truth.
    ///
    /// The current parser already returns the statement list alongside an empty
    /// AST container, so this helper keeps the lowering API flexible while the
    /// frontend tree ownership model evolves.
    pub fn lower_program_from_ast(
        &mut self,
        ast: &AST,
        statements: &[Statement],
    ) -> LoweringResult {
        let _ = ast;
        self.lower_statements(statements)
    }

    pub fn lower_node(&mut self, node_id: NodeId) -> HirNodeId {
        self.builder.alloc_text(
            HirNodeKind::Unknown,
            None,
            format!("ast:{}", node_id.as_u32()),
        )
    }

    pub(crate) fn next_synthetic_function_name(&mut self) -> String {
        let name = format!("__kali_fn_{}", self.synthetic_function_counter);
        self.synthetic_function_counter += 1;
        name
    }

    pub(crate) fn record_function_flavor(&mut self, node_id: HirNodeId, flavor: FunctionFlavor) {
        self.function_flavors.push((node_id, flavor));
    }

    pub(crate) fn push_child(&mut self, parent: HirNodeId, child: HirNodeId) {
        if let Some(node) = self.builder.node_mut(parent) {
            node.children.push(child);
        }
    }
}

impl Default for HirLowerer {
    fn default() -> Self {
        Self::new()
    }
}

mod expression;
mod function;
mod module;
mod object;
mod statement;

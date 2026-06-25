//! Lowering output: function-flavor metadata, the lowering result, and tree validation.

use crate::node::{HirNode, HirNodeId};
use kali_error::diagnostic::Diagnostic;

/// Function-flavor metadata preserved through HIR lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionFlavor {
    Sync,
    Async,
    Generator,
    AsyncGenerator,
}

impl FunctionFlavor {
    pub fn from_flags(is_async: bool, generator: bool) -> Self {
        match (is_async, generator) {
            (false, false) => Self::Sync,
            (true, false) => Self::Async,
            (false, true) => Self::Generator,
            (true, true) => Self::AsyncGenerator,
        }
    }
}

/// Lowering result from AST to HIR.
pub struct LoweringResult {
    /// Root node of the HIR.
    pub root: HirNodeId,
    /// All HIR nodes.
    pub nodes: Vec<HirNode>,
    /// Function-flavor metadata keyed by lowered HIR node id.
    pub function_flavors: Vec<(HirNodeId, FunctionFlavor)>,
    /// Diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl LoweringResult {
    /// Validate the structural consistency of the lowered HIR tree.
    pub fn validate(&self) -> Result<(), String> {
        validate_tree(
            "HIR",
            self.root,
            &self.nodes,
            |node| &node.children,
            |id| id.0 as usize,
        )
    }

    /// Return the preserved flavor for a lowered function node.
    pub fn function_flavor(&self, node_id: HirNodeId) -> Option<FunctionFlavor> {
        self.function_flavors
            .iter()
            .find(|(id, _)| *id == node_id)
            .map(|(_, flavor)| *flavor)
    }
}

fn validate_tree<Node, Id>(
    label: &str,
    root: Id,
    nodes: &[Node],
    children: impl Fn(&Node) -> &[Id],
    to_index: impl Fn(Id) -> usize,
) -> Result<(), String>
where
    Id: Copy,
{
    if nodes.is_empty() {
        return Err(format!("{label} tree contains no nodes"));
    }

    let root_index = to_index(root);
    if root_index >= nodes.len() {
        return Err(format!(
            "{label} root node id {root_index} is out of bounds for {} nodes",
            nodes.len()
        ));
    }

    for (index, node) in nodes.iter().enumerate() {
        for child in children(node) {
            let child_index = to_index(*child);
            if child_index >= nodes.len() {
                return Err(format!(
                    "{label} node {index} references child node id {child_index} outside the node table of {} nodes",
                    nodes.len()
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "result_tests.rs"]
mod result_tests;

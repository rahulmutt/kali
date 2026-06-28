//! Assembled LIR program and its structural validation.

use crate::{LirNode, LirNodeId};

/// LIR lowering result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirProgram {
    pub root: LirNodeId,
    pub nodes: Vec<LirNode>,
}

impl LirProgram {
    /// Validate the structural consistency of the lowered LIR tree.
    pub fn validate(&self) -> Result<(), String> {
        validate_tree(
            "LIR",
            self.root,
            &self.nodes,
            |node| &node.children,
            |id| id.0 as usize,
        )
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

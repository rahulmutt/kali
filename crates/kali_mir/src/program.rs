//! Assembled MIR program and its query/summary API.

use std::collections::BTreeSet;

use crate::{
    BorrowedLifetime, MirFunction, MirFunctionKind, MirNode, MirNodeId, ThreadBoundaryProfile,
};

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

    /// Validate the structural consistency of the lowered MIR tree.
    pub fn validate(&self) -> Result<(), String> {
        validate_tree(
            "MIR",
            self.root,
            &self.nodes,
            |node| &node.children,
            |id| id.0 as usize,
        )
    }

    pub fn function(&self, name: &str) -> Option<&MirFunction> {
        self.functions
            .iter()
            .find(|function| function.name.as_deref() == Some(name))
    }

    /// Return borrowed-lifetime summaries for the module scope.
    pub fn module_borrowed_lifetimes(&self) -> Vec<BorrowedLifetime> {
        self.borrowed_lifetimes_in_scope("module")
    }

    /// Return the thread-boundary profile for the module scope.
    pub fn module_thread_boundary_profile(&self) -> ThreadBoundaryProfile {
        self.thread_boundary_profile_in_scope("module")
    }

    /// Return borrowed-lifetime summaries for the whole MIR program.
    pub fn borrowed_lifetimes(&self) -> Vec<BorrowedLifetime> {
        let mut borrowed = BTreeSet::new();
        for function in &self.functions {
            let scope = function_scope_name(function);
            borrowed.extend(function.borrowed_lifetimes(scope));
        }
        borrowed.into_iter().collect()
    }

    /// Return borrowed-lifetime summaries for a specific scope.
    pub fn borrowed_lifetimes_in_scope(&self, scope: impl AsRef<str>) -> Vec<BorrowedLifetime> {
        let scope = scope.as_ref();
        let mut borrowed = BTreeSet::new();
        for function in self
            .functions
            .iter()
            .filter(|function| function_scope_name(function) == scope)
        {
            borrowed.extend(function.borrowed_lifetimes(scope.to_string()));
        }
        borrowed.into_iter().collect()
    }

    /// Return the thread-boundary profile for the whole MIR program.
    pub fn thread_boundary_profile(&self) -> ThreadBoundaryProfile {
        let mut profile = ThreadBoundaryProfile::default();
        for function in &self.functions {
            let scope = function_scope_name(function);
            for binding in &function.bindings {
                profile.push_binding(scope.clone(), binding);
            }
        }
        profile.finalize()
    }

    /// Return the thread-boundary profile for a specific scope.
    pub fn thread_boundary_profile_in_scope(
        &self,
        scope: impl AsRef<str>,
    ) -> ThreadBoundaryProfile {
        let scope = scope.as_ref();
        let mut profile = ThreadBoundaryProfile::default();
        for function in &self.functions {
            if function_scope_name(function) != scope {
                continue;
            }
            for binding in &function.bindings {
                profile.push_binding(scope.to_string(), binding);
            }
        }
        profile.finalize()
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

fn function_scope_name(function: &MirFunction) -> String {
    function
        .name
        .clone()
        .unwrap_or_else(|| "module".to_string())
}

#[cfg(test)]
#[path = "program_tests.rs"]
mod program_tests;

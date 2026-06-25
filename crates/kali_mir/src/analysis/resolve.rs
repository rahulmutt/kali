//! Function-target and escape-flag resolution for the analyzer.

use std::collections::BTreeSet;

use kali_hir::{HirNodeId, HirNodeKind};

use crate::{parameter_escape_flags, MirBindingKind, OwnershipAnalyzer};

impl<'a> OwnershipAnalyzer<'a> {
    pub(crate) fn function_parameter_escape_flags(&self, name: &str) -> Option<Vec<bool>> {
        self.functions
            .iter()
            .rev()
            .find(|function| function.name.as_deref() == Some(name))
            .map(parameter_escape_flags)
    }

    pub(crate) fn resolve_function_target(&self, name: &str) -> Option<String> {
        let mut current = name.to_string();
        let mut seen = BTreeSet::new();

        loop {
            if !seen.insert(current.clone()) {
                return None;
            }

            if self
                .functions
                .iter()
                .any(|function| function.name.as_deref() == Some(current.as_str()))
            {
                return Some(current);
            }

            let (scope_index, binding_index) = self.resolve_binding(&current)?;
            let scope = self.scope_stack.get(scope_index)?;
            let binding = scope.bindings.get(binding_index)?;
            if let Some(next) = scope.function_aliases.get(&current) {
                current = next.clone();
                continue;
            }

            if binding.kind == MirBindingKind::Function {
                return Some(current);
            }

            return None;
        }
    }

    pub(crate) fn function_target_from_node(&self, node_id: HirNodeId) -> Option<String> {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Ident => node
                .text
                .as_deref()
                .and_then(|name| self.resolve_function_target(name)),
            _ => None,
        }
    }

    pub(crate) fn function_name_from_recent_functions(
        &self,
        functions_before: usize,
    ) -> Option<String> {
        self.functions
            .get(functions_before..)
            .and_then(|functions| functions.last())
            .and_then(|function| function.name.clone())
    }

    pub(crate) fn next_function_name(&mut self) -> String {
        let name = format!("__kali_fn_{}", self.synthetic_function_counter);
        self.synthetic_function_counter += 1;
        name
    }
}

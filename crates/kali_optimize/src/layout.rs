use crate::*;

impl Optimizer {
    pub(crate) fn fold_layout_member_access(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
        env: &BindingEnv,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        let Some(property) = snapshot.text.as_deref() else {
            return false;
        };
        if snapshot.kind != LirNodeKind::Value || snapshot.children.len() != 1 {
            return false;
        }

        let Some(object_id) = snapshot.children.first().copied() else {
            return false;
        };

        if let Some(field_value) = self.object_literal_field(program, object_id, property) {
            let key = format!(
                "layout-member:{}:{}",
                property,
                node_signature(program, object_id)
            );
            if !tracker.allow(owner, key) {
                return false;
            }

            program.nodes[id.0 as usize] = program.nodes[field_value.0 as usize].clone();
            return true;
        }

        if property == "length" {
            if let Some(length) = self.array_literal_length(program, object_id) {
                let key = format!(
                    "layout-array-length:{}:{}",
                    property,
                    node_signature(program, object_id)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }

                program.nodes[id.0 as usize] = LirNode {
                    kind: LirNodeKind::Literal,
                    text: Some(length.to_string()),
                    children: Vec::new(),
                    function_flavor: None,
                };
                return true;
            }
        }

        let Some(index) = self.constant_array_index(program, env, property) else {
            return false;
        };
        let Some(element_value) = self.array_literal_element(program, object_id, index) else {
            return false;
        };

        let key = format!(
            "layout-array:{}:{}:{}",
            index,
            property,
            node_signature(program, object_id)
        );
        if !tracker.allow(owner, key) {
            return false;
        }

        program.nodes[id.0 as usize] = program.nodes[element_value.0 as usize].clone();
        true
    }

    pub(crate) fn object_literal_field(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        field: &str,
    ) -> Option<LirNodeId> {
        if !self.is_object_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        for property in &node.children {
            let property_node = program.nodes.get(property.0 as usize)?;
            if property_node.children.len() != 2 {
                continue;
            }
            let key_node = program.nodes.get(property_node.children[0].0 as usize)?;
            let key = key_node.text.as_deref()?;
            if key == field {
                return property_node.children.get(1).copied();
            }
        }

        None
    }

    pub(crate) fn array_literal_element(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        index: usize,
    ) -> Option<LirNodeId> {
        if !self.is_array_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        node.children.get(index).copied()
    }

    pub(crate) fn array_literal_length(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<usize> {
        if !self.is_array_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        Some(node.children.len())
    }

    pub(crate) fn constant_array_index(
        &self,
        program: &LirProgram,
        env: &BindingEnv,
        property: &str,
    ) -> Option<usize> {
        property.parse::<usize>().ok().or_else(|| {
            env.bindings
                .get(property)
                .and_then(|bound| literal_value(program, *bound))
                .and_then(|value| match value {
                    ConstantValue::Number(value) if value >= 0 => Some(value as usize),
                    _ => None,
                })
        })
    }

    pub(crate) fn is_object_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.is_empty() {
            return false;
        }

        node.children.iter().all(|child| {
            program
                .nodes
                .get(child.0 as usize)
                .is_some_and(|child_node| {
                    matches!(child_node.kind, LirNodeKind::Value)
                        && matches!(
                            child_node.text.as_deref(),
                            Some("init") | Some("get") | Some("set")
                        )
                        && child_node.children.len() == 2
                        && program
                            .nodes
                            .get(child_node.children[0].0 as usize)
                            .is_some_and(|key| key.kind == LirNodeKind::Literal)
                })
        })
    }

    pub(crate) fn is_array_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind != LirNodeKind::Value || node.text.is_some() {
            return false;
        }

        !self.is_object_literal(program, id)
    }
}

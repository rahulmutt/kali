use crate::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct BindingEnv {
    pub(crate) bindings: BTreeMap<String, LirNodeId>,
}

impl Optimizer {
    pub(crate) fn fold_object_has_own_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        if !matches!(
            callee_name.as_str(),
            "Object.hasOwn"
                | "globalThis.Object.hasOwn"
                | "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
        ) {
            return None;
        }

        let object_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if !self.is_object_literal(program, object_id) {
            return None;
        }

        let key_id =
            self.resolve_constant_binding(program, *snapshot.children.get(2)?, bindings)?;
        let key = self.constant_property_key(program, key_id)?;
        let has_own = self
            .object_literal_field(program, object_id, &key)
            .is_some();
        Some(self.clone_boolean_literal(program, has_own))
    }

    pub(crate) fn fold_object_enumeration_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        let string_mode = match callee_name.as_str() {
            "Object.keys" | "globalThis.Object.keys" => Some("keys"),
            "Object.values" | "globalThis.Object.values" => Some("values"),
            "Object.entries" | "globalThis.Object.entries" => Some("entries"),
            _ => None,
        };
        let is_reflect_own_keys = matches!(
            callee_name.as_str(),
            "Reflect.ownKeys" | "globalThis.Reflect.ownKeys"
        );
        if string_mode.is_none() && !is_reflect_own_keys {
            return None;
        }

        let object_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if let Some(ConstantValue::String(string_text)) = literal_value(program, object_id) {
            if let Some(mode) = string_mode {
                let mut elements = Vec::with_capacity(string_text.chars().count());
                match mode {
                    "keys" => {
                        for (index, _) in string_text.chars().enumerate() {
                            elements.push(
                                self.clone_string_literal(
                                    program,
                                    format!("{:?}", index.to_string()),
                                ),
                            );
                        }
                    }
                    "values" => {
                        for value in string_text.chars() {
                            elements.push(
                                self.clone_string_literal(
                                    program,
                                    format!("{:?}", value.to_string()),
                                ),
                            );
                        }
                    }
                    "entries" => {
                        for (index, value) in string_text.chars().enumerate() {
                            let key_id = self
                                .clone_string_literal(program, format!("{:?}", index.to_string()));
                            let value_id = self
                                .clone_string_literal(program, format!("{:?}", value.to_string()));
                            elements.push(self.push_array_literal(program, vec![key_id, value_id]));
                        }
                    }
                    _ => unreachable!(),
                }
                return Some(self.push_array_literal(program, elements));
            }
        }
        if !self.is_object_literal(program, object_id) {
            return None;
        }

        let properties = self.ordered_object_literal_properties(program, object_id)?;
        match callee_name.as_str() {
            "Object.keys" | "globalThis.Object.keys" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, _) in properties {
                    elements.push(self.clone_string_literal(program, key));
                }
                Some(self.push_array_literal(program, elements))
            }
            "Reflect.ownKeys" | "globalThis.Reflect.ownKeys" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, _) in properties {
                    elements.push(self.clone_string_literal(program, key));
                }
                Some(self.push_array_literal(program, elements))
            }
            "Object.values" | "globalThis.Object.values" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (_, value) in properties {
                    elements.push(self.clone_subtree_with_substitution(
                        program,
                        value,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    ));
                }
                Some(self.push_array_literal(program, elements))
            }
            "Object.entries" | "globalThis.Object.entries" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, value) in properties {
                    let key_id = self.clone_string_literal(program, key);
                    let value_id = self.clone_subtree_with_substitution(
                        program,
                        value,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    );
                    let pair = self.push_array_literal(program, vec![key_id, value_id]);
                    elements.push(pair);
                }
                Some(self.push_array_literal(program, elements))
            }
            _ => None,
        }
    }

    pub(crate) fn fold_object_from_entries_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        if !matches!(
            callee_name.as_str(),
            "Object.fromEntries" | "globalThis.Object.fromEntries"
        ) {
            return None;
        }

        let entries_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if !self.is_array_literal(program, entries_id) {
            return None;
        }

        let entries_node = program.nodes.get(entries_id.0 as usize)?;
        let mut properties: Vec<(String, usize, LirNodeId)> = Vec::new();
        let mut key_positions: HashMap<String, usize> = HashMap::new();
        for (entry_index, entry_id) in entries_node.children.iter().copied().enumerate() {
            let entry_id = self.resolve_constant_binding(program, entry_id, bindings)?;
            if !self.is_array_literal(program, entry_id) {
                return None;
            }

            let entry_node = program.nodes.get(entry_id.0 as usize)?;
            if entry_node.children.len() != 2 {
                return None;
            }

            let key_id =
                self.resolve_constant_binding(program, entry_node.children[0], bindings)?;
            let key = self.constant_property_key(program, key_id)?;
            let value_id =
                self.resolve_constant_binding(program, entry_node.children[1], bindings)?;

            if let Some(position) = key_positions.get(&key).copied() {
                properties[position].2 = value_id;
                continue;
            }

            key_positions.insert(key.clone(), properties.len());
            properties.push((key, entry_index, value_id));
        }

        let object_properties = properties
            .into_iter()
            .map(|(key, _, value)| (key, value))
            .collect::<Vec<_>>();
        Some(self.push_object_literal(program, object_properties))
    }

    pub(crate) fn fold_object_enumeration_calls(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        bindings: &BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        for child in snapshot.children.iter().copied() {
            self.fold_object_enumeration_calls(program, child, bindings);
        }

        if snapshot.kind != LirNodeKind::Call {
            return;
        }

        let Some(callee_id) = snapshot.children.first().copied() else {
            return;
        };
        let Some(callee_node) = program.nodes.get(callee_id.0 as usize).cloned() else {
            return;
        };
        if let Some(folded) =
            self.fold_object_enumeration_call(program, &snapshot, &callee_node, bindings)
        {
            program.nodes[id.0 as usize] = program.nodes[folded.0 as usize].clone();
        }
    }

    pub(crate) fn ordered_object_literal_properties(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<Vec<(String, LirNodeId)>> {
        if !self.is_object_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        let mut properties = Vec::new();
        for (source_index, property) in node.children.iter().copied().enumerate() {
            let property_node = program.nodes.get(property.0 as usize)?;
            if property_node.children.len() != 2 {
                continue;
            }
            let key_node = program.nodes.get(property_node.children[0].0 as usize)?;
            let key = key_node.text.as_deref()?.to_string();
            properties.push((key, source_index, property_node.children[1]));
        }

        properties.sort_by(|(left_key, left_index, _), (right_key, right_index, _)| {
            match (
                Self::object_property_order_key(left_key),
                Self::object_property_order_key(right_key),
            ) {
                (Some(left_order), Some(right_order)) => left_order
                    .cmp(&right_order)
                    .then_with(|| left_index.cmp(right_index)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => left_index.cmp(right_index),
            }
        });

        Some(
            properties
                .into_iter()
                .map(|(key, _, value)| (key, value))
                .collect(),
        )
    }

    pub(crate) fn resolve_constant_binding(
        &self,
        program: &LirProgram,
        mut id: LirNodeId,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = program.nodes.get(id.0 as usize)?;
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
            {
                id = node.children[0];
                continue;
            }

            if self.is_object_freeze_call(program, node) {
                id = node.children[1];
                continue;
            }

            if node.kind == LirNodeKind::Value
                && node.children.is_empty()
                && node.text.as_deref().is_some()
            {
                let name = node.text.as_deref()?;
                if let Some(bound) = bindings.bindings.get(name).copied() {
                    id = bound;
                    continue;
                }
            }

            return Some(id);
        }
    }

    pub(crate) fn is_object_freeze_call(&self, program: &LirProgram, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() < 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee_node) = program.nodes.get(callee.0 as usize) else {
            return false;
        };
        matches!(
            self.normalized_member_access_name(program, callee_node)
                .as_deref(),
            Some("Object.freeze") | Some("globalThis.Object.freeze")
        )
    }

    pub(crate) fn collect_constant_bindings(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> BindingEnv {
        let mut env = BindingEnv::default();
        self.collect_constant_bindings_into(program, id, &mut env);
        env
    }

    pub(crate) fn collect_constant_bindings_into(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        env: &mut BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        if let Some((name, init)) = self.extract_const_binding(program, id) {
            let resolved = self
                .resolve_constant_binding(program, init, env)
                .unwrap_or(init);
            if self.is_specializable_binding(program, resolved) {
                env.bindings.insert(name, resolved);
            }
        }

        for child in snapshot.children {
            self.collect_constant_bindings_into(program, child, env);
        }
    }
}

#[cfg(test)]
#[path = "object_fold_tests.rs"]
mod object_fold_tests;

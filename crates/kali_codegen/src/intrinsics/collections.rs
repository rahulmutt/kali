//! Map and Set constructor intrinsic recognition and iteration-item collection.
use crate::*;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn is_set_constructor_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        matches!(
            self.node(callee).text.as_deref(),
            Some("Set")
                | Some("globalThis.Set")
                | Some(r#"globalThis["Set"]"#)
                | Some(r#"globalThis['Set']"#)
        )
    }

    pub(crate) fn resolve_set_constructor_call<'b>(
        &'b self,
        node: &'b LirNode,
    ) -> Option<&'b LirNode> {
        if self.is_set_constructor_call(node) {
            return Some(node);
        }

        if self.is_object_freeze_call(node) {
            let argument = node.children.get(1).copied()?;
            return self.resolve_set_constructor_call(self.node(argument));
        }

        if node.kind == LirNodeKind::Value
            && (node.text.is_none() || node.text.as_deref() == Some("await"))
            && node.children.len() == 1
        {
            return self.resolve_set_constructor_call(self.node(node.children[0]));
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    let selected = if left.is_nullish() {
                        node.children[1]
                    } else {
                        node.children[0]
                    };
                    return self.resolve_set_constructor_call(self.node(selected));
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_set_constructor_call(self.node(node.children[1]));
                        }
                        Some(false) => {
                            return self.resolve_set_constructor_call(self.node(node.children[0]));
                        }
                        None => {
                            let consequent =
                                self.resolve_set_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_set_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_set_constructor_call(self.node(node.children[0]));
                        }
                        Some(false) => {
                            return self.resolve_set_constructor_call(self.node(node.children[1]));
                        }
                        None => {
                            let consequent =
                                self.resolve_set_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_set_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 3 {
            if let Some(test) = self.resolve_static_object_identity_value(node.children[0]) {
                let selected = if test.truthiness() == Some(true) {
                    node.children[1]
                } else {
                    node.children[2]
                };
                return self.resolve_set_constructor_call(self.node(selected));
            }

            let consequent = self.resolve_set_constructor_call(self.node(node.children[1]));
            let alternate = self.resolve_set_constructor_call(self.node(node.children[2]));
            if consequent.is_some() && consequent == alternate {
                return consequent;
            }
        }

        None
    }

    pub(crate) fn collect_set_constructor_iteration_items(
        &mut self,
        node: &LirNode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        let mut seen = HashSet::new();
        if let Some(set_call) = self.resolve_set_constructor_call(node) {
            let Some(source_arg) = set_call.children.get(1).copied() else {
                return false;
            };
            let Some(source_id) = self.resolve_literal_aggregate(source_arg) else {
                return false;
            };
            let source = self.node(source_id).clone();
            return self.collect_set_constructor_iteration_items(&source, items);
        }

        if let Some(string_text) = self.render_static_string_value(node) {
            for value in string_text.chars() {
                let item = self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(format!("{value:?}")),
                    vec![],
                );
                let Some(key) = self.static_set_item_key(item) else {
                    return false;
                };
                if seen.insert(key) {
                    items.push(item);
                }
            }
            return true;
        }

        if self.is_array_literal(node) {
            let mut collected = Vec::new();
            for child in &node.children {
                if !self.collect_for_of_array_iteration_items(*child, &mut collected) {
                    return false;
                }
            }

            for item in collected {
                let Some(key) = self.static_set_item_key(item) else {
                    return false;
                };
                if seen.insert(key) {
                    items.push(item);
                }
            }
            return true;
        }

        if self.resolve_set_constructor_call(node).is_some() {
            return self.collect_set_constructor_iteration_items(node, items);
        }

        if self.resolve_map_constructor_call(node).is_some() {
            return self.collect_map_constructor_iteration_items(node, items);
        }

        if let Some(object_enumeration_mode) = self.is_object_enumeration_call(node) {
            if matches!(object_enumeration_mode, ObjectEnumerationMode::Entries) {
                return false;
            }

            let Some(object_arg) = node.children.get(1).copied() else {
                return false;
            };
            let Some(object_id) = self.resolve_literal_aggregate(object_arg) else {
                return false;
            };
            let object = self.node(object_id).clone();
            let mut collected = Vec::new();
            if !self.collect_object_enumeration_iteration_items(
                &object,
                object_enumeration_mode,
                &mut collected,
            ) {
                return false;
            }

            for item in collected {
                let Some(key) = self.static_set_item_key(item) else {
                    return false;
                };
                if seen.insert(key) {
                    items.push(item);
                }
            }
            return true;
        }

        false
    }

    pub(crate) fn is_map_constructor_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        matches!(
            self.node(callee).text.as_deref(),
            Some("Map")
                | Some("globalThis.Map")
                | Some(r#"globalThis["Map"]"#)
                | Some(r#"globalThis['Map']"#)
        )
    }

    pub(crate) fn resolve_map_constructor_call<'b>(
        &'b self,
        node: &'b LirNode,
    ) -> Option<&'b LirNode> {
        if self.is_map_constructor_call(node) {
            return Some(node);
        }

        if self.is_object_freeze_call(node) {
            let argument = node.children.get(1).copied()?;
            return self.resolve_map_constructor_call(self.node(argument));
        }

        if node.kind == LirNodeKind::Value
            && (node.text.is_none() || node.text.as_deref() == Some("await"))
            && node.children.len() == 1
        {
            return self.resolve_map_constructor_call(self.node(node.children[0]));
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    let selected = if left.is_nullish() {
                        node.children[1]
                    } else {
                        node.children[0]
                    };
                    return self.resolve_map_constructor_call(self.node(selected));
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_map_constructor_call(self.node(node.children[1]));
                        }
                        Some(false) => {
                            return self.resolve_map_constructor_call(self.node(node.children[0]));
                        }
                        None => {
                            let consequent =
                                self.resolve_map_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_map_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_map_constructor_call(self.node(node.children[0]));
                        }
                        Some(false) => {
                            return self.resolve_map_constructor_call(self.node(node.children[1]));
                        }
                        None => {
                            let consequent =
                                self.resolve_map_constructor_call(self.node(node.children[0]));
                            let alternate =
                                self.resolve_map_constructor_call(self.node(node.children[1]));
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 3 {
            if let Some(test) = self.resolve_static_object_identity_value(node.children[0]) {
                let selected = if test.truthiness() == Some(true) {
                    node.children[1]
                } else {
                    node.children[2]
                };
                return self.resolve_map_constructor_call(self.node(selected));
            }

            let consequent = self.resolve_map_constructor_call(self.node(node.children[1]));
            let alternate = self.resolve_map_constructor_call(self.node(node.children[2]));
            if consequent.is_some() && consequent == alternate {
                return consequent;
            }
        }

        None
    }

    pub(crate) fn collect_map_constructor_iteration_items(
        &mut self,
        node: &LirNode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        if let Some(map_call) = self.resolve_map_constructor_call(node) {
            let Some(source_arg) = map_call.children.get(1).copied() else {
                return false;
            };
            let Some(source_id) = self.resolve_literal_aggregate(source_arg) else {
                return false;
            };
            let source = self.node(source_id).clone();
            return self.collect_map_constructor_iteration_items(&source, items);
        }

        let mut collected = Vec::<(String, LirNodeId)>::new();
        if self.is_array_literal(node) {
            for child in &node.children {
                let Some(resolved_child) = self.resolve_literal_aggregate(*child) else {
                    return false;
                };
                let entry = self.node(resolved_child).clone();
                if !self.is_array_literal(&entry) || entry.children.len() < 2 {
                    return false;
                }
                if !entry
                    .children
                    .iter()
                    .take(2)
                    .all(|child| self.is_supported_for_of_array_iteration_item(*child))
                {
                    return false;
                }
                let Some(key) = self.static_map_entry_key(resolved_child) else {
                    return false;
                };
                if let Some((_, existing_entry)) = collected
                    .iter_mut()
                    .find(|(existing_key, _)| existing_key == &key)
                {
                    *existing_entry = resolved_child;
                } else {
                    collected.push((key, resolved_child));
                }
            }

            items.extend(collected.into_iter().map(|(_, id)| id));
            return true;
        }

        false
    }

    pub(crate) fn static_map_entry_key(&self, id: LirNodeId) -> Option<String> {
        let resolved_id = self.resolve_literal_aggregate(id)?;
        let node = self.node(resolved_id);
        if !self.is_array_literal(node) {
            return None;
        }
        let key = node.children.first().copied()?;
        let resolved_key = self.resolve_literal_aggregate(key)?;
        self.render_static_value(resolved_key)
    }

    pub(crate) fn static_set_item_key(&self, id: LirNodeId) -> Option<String> {
        let resolved_id = self.resolve_literal_aggregate(id)?;
        let node = self.node(resolved_id);
        if node.kind == LirNodeKind::Value && !node.children.is_empty() {
            return None;
        }
        self.render_static_value(resolved_id)
    }
}

#[cfg(test)]
#[path = "collections_tests.rs"]
mod collections_tests;

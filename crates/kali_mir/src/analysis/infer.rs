//! Layout inference for analyzer bindings.

use kali_hir::{HirNode, HirNodeId, HirNodeKind};

use crate::{LayoutDescriptor, OwnershipAnalyzer};

impl<'a> OwnershipAnalyzer<'a> {
    pub(crate) fn infer_layout(&self, node_id: HirNodeId) -> LayoutDescriptor {
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
                for (source_index, child) in node.children.iter().copied().enumerate() {
                    let property = &self.nodes[child.0 as usize];
                    if matches!(property.kind, HirNodeKind::ObjectProperty)
                        && property.children.len() >= 2
                    {
                        let key = self.layout_field_name(property);
                        let value = property.children[1];
                        fields.push((key, source_index, Box::new(self.infer_layout(value))));
                    }
                }
                if fields.is_empty() {
                    LayoutDescriptor::TaggedVal
                } else {
                    fields.sort_by(
                        |(left_key, left_index, _), (right_key, right_index, _)| match (
                            Self::object_property_order_key(left_key),
                            Self::object_property_order_key(right_key),
                        ) {
                            (Some(left_order), Some(right_order)) => left_order
                                .cmp(&right_order)
                                .then_with(|| left_index.cmp(right_index)),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => left_index.cmp(right_index),
                        },
                    );
                    LayoutDescriptor::Struct {
                        fields: fields
                            .into_iter()
                            .map(|(key, _, layout)| (key, layout))
                            .collect(),
                    }
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

    pub(crate) fn infer_binary_layout(&self, node: &HirNode) -> LayoutDescriptor {
        let op = node.text.as_deref().unwrap_or_default();
        match op {
            "+" | "-" | "*" | "/" | "%" | "**" => LayoutDescriptor::scalar("number"),
            "==" | "!=" | "===" | "!==" | "<" | "<=" | ">" | ">=" | "&&" | "||" => {
                LayoutDescriptor::scalar("bool")
            }
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    pub(crate) fn infer_unary_layout(&self, node: &HirNode) -> LayoutDescriptor {
        match node.text.as_deref().unwrap_or_default() {
            "!" => LayoutDescriptor::scalar("bool"),
            "-" | "+" | "~" => LayoutDescriptor::scalar("number"),
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    pub(crate) fn resolve_binding_layout(&self, name: &str) -> Option<LayoutDescriptor> {
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

    pub(crate) fn layout_field_name(&self, node: &HirNode) -> String {
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

    pub(crate) fn object_property_order_key(key: &str) -> Option<u64> {
        let normalized = key.trim_matches('"');
        if normalized.is_empty() || (normalized.len() > 1 && normalized.starts_with('0')) {
            return None;
        }

        let value = normalized.parse::<u64>().ok()?;
        (value < u32::MAX as u64).then_some(value)
    }
}

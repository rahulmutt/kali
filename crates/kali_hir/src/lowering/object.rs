//! Object-literal lowering: property and property-name handling.

use crate::helpers::object_property_kind_text;
use crate::node::{HirNodeId, HirNodeKind};
use crate::HirLowerer;
use kali_ast::{ObjectProperty, PropertyName};

impl HirLowerer {
    pub(crate) fn lower_object_property(&mut self, property: &ObjectProperty) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::ObjectProperty,
            None,
            object_property_kind_text(&property.kind),
        );
        push_child!(self, id, self.lower_property_name(&property.key));
        push_child!(self, id, self.lower_expression(&property.value));
        id
    }

    pub(crate) fn lower_property_name(&mut self, name: &PropertyName) -> HirNodeId {
        match name {
            PropertyName::Identifier(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
            PropertyName::Number(value) => {
                // Preserve numeric property names as strings so object-literal lowering
                // keeps JavaScript's property-key semantics instead of treating them as
                // arithmetic literals during code generation.
                let key = if *value == 0.0 {
                    "0".to_string()
                } else {
                    value.to_string()
                };
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, format!("\"{}\"", key))
            }
            PropertyName::String(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
        }
    }
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;

//! Object-literal lowering: property and property-name handling.

use crate::helpers::object_property_kind_text;
use crate::node::{HirNodeId, HirNodeKind};
use crate::HirLowerer;
use kali_ast::{ObjectProperty, PropertyName};
use kali_common::js_number::format_js_number;

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

    /// A property key lowers to its JAVASCRIPT PROPERTY NAME -- `String(key)`
    /// -- and to nothing else. No quoting marker, in either direction.
    ///
    /// The marker this replaces encoded "was a number" as a leading `"`, which
    /// a string key's own content can also carry: `{'"5"': 1}` and `{5: 1}`
    /// reached codegen as the same text, so `Object.hasOwn` answered both
    /// wrongly at exit 0 while the member read still worked (register R-56).
    /// No downstream predicate could recover the difference, which is why the
    /// discriminator is spent HERE, where the type is still known, rather than
    /// re-derived there.
    ///
    /// `format_js_number` is the same function every other lane renders a
    /// number with, so a key's name and a number's rendering cannot drift.
    pub(crate) fn lower_property_name(&mut self, name: &PropertyName) -> HirNodeId {
        let text = match name {
            PropertyName::Identifier(value) | PropertyName::String(value) => value.clone(),
            PropertyName::Number(value) => format_js_number(*value),
            PropertyName::BigInt(digits) => digits.clone(),
        };
        self.builder.alloc_text(HirNodeKind::Literal, None, text)
    }
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;

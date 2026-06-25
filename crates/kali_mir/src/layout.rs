//! MIR memory layout descriptors.

/// Canonical layout descriptor used by MIR analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LayoutDescriptor {
    Scalar(String),
    Struct {
        fields: Vec<(String, Box<LayoutDescriptor>)>,
    },
    Array {
        element: Box<LayoutDescriptor>,
        length: Option<usize>,
    },
    Closure {
        captures: Vec<String>,
    },
    TaggedVal,
}

impl LayoutDescriptor {
    pub(crate) fn scalar(name: impl Into<String>) -> Self {
        Self::Scalar(name.into())
    }

    /// Return the canonical layout/representation fingerprint for this descriptor.
    ///
    /// The fingerprint is intentionally deterministic and only includes properties
    /// that materially change generated code shape or correctness.
    pub fn fingerprint(&self) -> String {
        match self {
            LayoutDescriptor::Scalar(name) => format!("Scalar({name})"),
            LayoutDescriptor::Struct { fields } => {
                let mut parts = Vec::with_capacity(fields.len());
                for (field, layout) in fields {
                    parts.push(format!("{}:{}", field, layout.fingerprint()));
                }
                format!("Struct({})", parts.join(","))
            }
            LayoutDescriptor::Array { element, length } => format!(
                "Array(length={:?},element={})",
                length,
                element.fingerprint()
            ),
            LayoutDescriptor::Closure { captures } => {
                let mut captures = captures.clone();
                captures.sort();
                format!("Closure(captures={})", captures.join("|"))
            }
            LayoutDescriptor::TaggedVal => "TaggedVal".to_string(),
        }
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;

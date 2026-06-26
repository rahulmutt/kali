//! Host argument view for the Deno compatibility layer.

/// Light-weight `Deno.args` projection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DenoArgs(Vec<String>);

impl DenoArgs {
    /// Create an argument view from a host-provided vector.
    pub fn new(values: Vec<String>) -> Self {
        Self(values)
    }

    /// Return the recorded arguments.
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Return the recorded arguments as an owned vector.
    pub fn to_vec(&self) -> Vec<String> {
        self.0.clone()
    }
}

#[cfg(test)]
#[path = "args_tests.rs"]
mod args_tests;

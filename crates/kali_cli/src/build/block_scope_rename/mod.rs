//! Block-scope rename (block-scoping spec §3.1–§3.2). See `rename_block_scoped_bindings`.

// Task 3 wires the planner and applier into the build; until then the walk and table are
// only reached from tests. Remove this allow when `rename_block_scoped_bindings` lands.
#![allow(dead_code)]

pub(crate) mod apply;
pub(crate) mod plan;
pub(crate) mod table;
pub(crate) mod walk;

#[cfg(test)]
pub(crate) mod test_support;

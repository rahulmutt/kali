use crate::test_support::*;
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};

#[path = "specialize_tests/mir_layout.rs"]
mod mir_layout;

#[path = "specialize_tests/tagged_budget.rs"]
mod tagged_budget;

#[path = "specialize_tests/generic_reuse.rs"]
mod generic_reuse;

#[path = "specialize_tests/literal_args.rs"]
mod literal_args;

#[path = "specialize_tests/layout_bindings.rs"]
mod layout_bindings;

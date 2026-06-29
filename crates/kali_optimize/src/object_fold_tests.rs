use crate::test_support::*;
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};

#[path = "object_fold_tests/enumeration.rs"]
mod enumeration;

#[path = "object_fold_tests/reflect_own_keys.rs"]
mod reflect_own_keys;

#[path = "object_fold_tests/object_has_own.rs"]
mod object_has_own;

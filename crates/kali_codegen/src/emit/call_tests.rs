use crate::test_support::*;
use crate::*;
use kali_test_support::fixtures::{tempdir, write_file};
use wasmparser::Validator;

#[path = "call_tests/reflect_own_keys.rs"]
mod reflect_own_keys;

#[path = "call_tests/diagnostics.rs"]
mod diagnostics;

#[path = "call_tests/array_iteration.rs"]
mod array_iteration;

#[path = "call_tests/object_enumeration.rs"]
mod object_enumeration;

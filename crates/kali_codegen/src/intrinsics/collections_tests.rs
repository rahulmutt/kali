use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[path = "collections_tests/combined.rs"]
mod combined;

#[path = "collections_tests/set.rs"]
mod set;

#[path = "collections_tests/map.rs"]
mod map;

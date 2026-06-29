use crate::test_support::lex;
use crate::*;
use kali_ast::{ExportSpecifier, ImportSpecifier, Statement};

#[path = "module_tests/import.rs"]
mod import;

#[path = "module_tests/export.rs"]
mod export;

use crate::test_support::*;
use crate::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[path = "execute_tests/execution.rs"]
mod execution;

#[path = "execute_tests/harness.rs"]
mod harness;

#[path = "execute_tests/diagnostic.rs"]
mod diagnostic;

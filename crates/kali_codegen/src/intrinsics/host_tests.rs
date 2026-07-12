use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[path = "host_tests/env.rs"]
mod env;

#[path = "host_tests/console.rs"]
mod console;

#[path = "host_tests/deno.rs"]
mod deno;

#[path = "host_tests/process.rs"]
mod process;

#[path = "host_tests/perf.rs"]
mod perf;

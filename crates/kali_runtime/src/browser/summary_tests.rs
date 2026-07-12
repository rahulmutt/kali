use crate::test_support::*;
use crate::*;
use std::fs;

#[path = "summary_tests/runtime_summary.rs"]
mod runtime_summary;

#[path = "summary_tests/bundle.rs"]
mod bundle;

#[path = "summary_tests/requested.rs"]
mod requested;

#[path = "summary_tests/coverage.rs"]
mod coverage;

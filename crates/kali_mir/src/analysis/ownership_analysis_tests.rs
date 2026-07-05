use crate::test_support::*;
use crate::*;
use kali_hir::{HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};

#[path = "ownership_analysis_tests/allocation.rs"]
mod allocation;

#[path = "ownership_analysis_tests/call_escape.rs"]
mod call_escape;

#[path = "ownership_analysis_tests/alias_precision.rs"]
mod alias_precision;

#[path = "ownership_analysis_tests/aggregate_escape.rs"]
mod aggregate_escape;

#[path = "ownership_analysis_tests/plain_ident_escape.rs"]
mod plain_ident_escape;

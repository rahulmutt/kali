use crate::*;
use crate::test_support::*;
use crate::LOCK_VERSION;
use std::fs;
use std::sync::atomic::Ordering;

use serde_json::json;

#[path = "install_tests/rejections.rs"]
mod rejections;

#[path = "install_tests/reconciliation.rs"]
mod reconciliation;

#[path = "install_tests/lifecycle.rs"]
mod lifecycle;

#[path = "install_tests/traversal.rs"]
mod traversal;

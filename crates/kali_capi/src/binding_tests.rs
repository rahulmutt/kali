use crate::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "binding_tests/python.rs"]
mod python;

#[path = "binding_tests/javascript.rs"]
mod javascript;

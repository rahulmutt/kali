//! Host-import registration and guest-memory plumbing for the wasmtime linker.
use crate::*;
pub(crate) mod memory;
pub(crate) mod io;
pub(crate) mod diagnostics;
pub(crate) mod enforce;

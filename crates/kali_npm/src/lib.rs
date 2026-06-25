//! Package management for Kali (npm/JSR registry support).

use base64::Engine;
use flate2::read::GzDecoder;
use kali_error::{_error_codes::e5, _error_codes::e6, Diagnostic};
use reqwest::blocking::Client;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
};
use tar::Archive;

const MANIFEST_SCHEMA: u32 = 1;
const LOCK_VERSION: u32 = 1;
const DEFAULT_NPM_REGISTRY: &str = "https://registry.npmjs.org";
const NODE_ONLY_HOST_APIS: &[&str] = &[
    "fs",
    "fs/promises",
    "path",
    "os",
    "url",
    "crypto",
    "events",
    "stream",
    "http",
    "https",
    "process",
    "buffer",
    "util",
    "assert",
    "child_process",
    "timers/promises",
    "timers",
];

static REGISTRY_METADATA_CACHE: OnceLock<Mutex<BTreeMap<String, serde_json::Value>>> =
    OnceLock::new();

mod install;
pub use install::*;

mod manifest;
pub use manifest::*;

mod registry;
pub use registry::*;

mod resolve;
pub use resolve::*;

mod tarball;
pub(crate) use tarball::*;

mod target;
pub use target::*;

mod validate;
pub use validate::*;

#[cfg(test)]
mod test_support;

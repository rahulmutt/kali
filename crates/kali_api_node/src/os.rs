//! Node.js `os` module compatibility surface.

use std::{env, path::PathBuf};

/// Lightweight OS view for Node-style environment helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeOs;

impl NodeOs {
    pub fn platform(&self) -> &'static str {
        env::consts::OS
    }

    pub fn arch(&self) -> &'static str {
        env::consts::ARCH
    }

    pub fn eol(&self) -> &'static str {
        if cfg!(windows) {
            "\r\n"
        } else {
            "\n"
        }
    }

    pub fn home_dir(&self) -> Option<PathBuf> {
        env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
    }

    pub fn tmpdir(&self) -> PathBuf {
        env::temp_dir()
    }

    pub fn cpus(&self) -> usize {
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    }
}

#[cfg(test)]
#[path = "os_tests.rs"]
mod os_tests;

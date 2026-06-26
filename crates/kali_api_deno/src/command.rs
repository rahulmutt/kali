//! Deterministic subprocess command model for the Deno compatibility layer.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::path::normalize_path;

/// Result of a Deno-style command invocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoCommandOutput {
    status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl DenoCommandOutput {
    pub fn status(&self) -> i32 {
        self.status
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn text_stdout(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.stdout.clone())
    }

    pub fn text_stderr(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.stderr.clone())
    }
}

/// Error produced by the Deno command helper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoCommandError {
    message: String,
}

impl DenoCommandError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for DenoCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DenoCommandError {}

/// Minimal Deno-style process command helper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoCommand {
    command: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: Option<PathBuf>,
}

impl DenoCommand {
    /// Create a command builder for one executable.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
        }
    }

    /// Append one argument.
    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Append multiple arguments.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set or replace one environment variable for the child process.
    pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the child process working directory.
    pub fn current_dir(&mut self, cwd: impl Into<PathBuf>) -> &mut Self {
        self.cwd = Some(normalize_path(cwd.into()));
        self
    }

    /// Run the command to completion, capturing stdout/stderr.
    pub fn output(&self) -> Result<DenoCommandOutput, DenoCommandError> {
        let mut command = Command::new(&self.command);
        command.args(&self.args);
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        for (key, value) in &self.env {
            command.env(key, value);
        }
        let output = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| {
                DenoCommandError::new(format!("failed to run '{}': {}", self.command, error))
            })?;
        Ok(DenoCommandOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    /// Synonym for [`output`](Self::output) to match the builder-style API.
    pub fn spawn(&self) -> Result<DenoCommandOutput, DenoCommandError> {
        self.output()
    }
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;

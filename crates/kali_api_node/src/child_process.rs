//! Node.js child_process module compatibility surface.

use std::process::{Command, Stdio};

/// Lightweight child-process helper used by the Node compatibility layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeChildProcess;

/// Result of a synchronous child-process run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeChildProcessOutput {
    status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl NodeChildProcessOutput {
    pub fn status(&self) -> i32 {
        self.status
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeChildProcessError {
    message: String,
}

impl NodeChildProcessError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for NodeChildProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NodeChildProcessError {}

impl NodeChildProcess {
    pub fn spawn_sync(
        command: impl AsRef<str>,
        args: &[impl AsRef<str>],
    ) -> Result<NodeChildProcessOutput, NodeChildProcessError> {
        let mut command = Command::new(command.as_ref());
        for arg in args {
            command.arg(arg.as_ref());
        }

        let program = command.get_program().to_string_lossy().into_owned();
        let output = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| {
                NodeChildProcessError::new(format!("failed to spawn '{}': {}", program, error))
            })?;

        Ok(NodeChildProcessOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    pub fn spawn(
        &self,
        command: impl AsRef<str>,
        args: &[impl AsRef<str>],
    ) -> Result<NodeChildProcessOutput, NodeChildProcessError> {
        Self::spawn_sync(command, args)
    }
}

#[cfg(test)]
#[path = "child_process_tests.rs"]
mod child_process_tests;

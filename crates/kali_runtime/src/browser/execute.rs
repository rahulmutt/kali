//! Browser harness execution: outcome types, invocation plan, and checked-execution helpers.
use crate::*;

/// Result of executing a browser-harnessed WASM module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserRuntimeExecutionOutcome {
    /// The fully resolved command line used to launch the harness, including the script path and
    /// any trailing entrypoint arguments.
    pub command: Vec<String>,
    /// The harness process exit status.
    pub status: std::process::ExitStatus,
    /// Captured harness stdout.
    pub stdout: String,
    /// Captured harness stderr.
    pub stderr: String,
    /// The high-level host contract selected for the browser harness request.
    pub host_contract: RuntimeHostContract,
    /// The browser backend reported by the harness summary.
    pub runtime_backend: RuntimeBackend,
    /// Runtime arguments reported by the harness summary.
    pub reported_args: Vec<String>,
    /// Test callbacks registered by the guest and reported by the browser harness summary.
    pub registered_tests: Vec<String>,
    /// Test callbacks that failed inside the browser harness summary.
    pub tests_failed: usize,
    /// Deterministic worker/thread shutdown snapshot reported by the harness summary.
    pub thread_topology: ThreadRuntimeShutdownReport,
}

impl BrowserRuntimeExecutionOutcome {
    /// Return the number of registered guest tests reported by the harness summary.
    pub fn tests_run(&self) -> usize {
        self.registered_tests.len()
    }
}

/// A deterministic browser-harness launch plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserHarnessInvocation {
    /// The executable used to launch the harness.
    pub executable: String,
    /// Arguments passed to the harness before the browser script path.
    pub harness_args: Vec<String>,
    /// The script or entrypoint that will be executed by the harness.
    pub script: PathBuf,
    /// Trailing arguments forwarded to the browser script.
    pub args: Vec<String>,
    /// Current working directory for the harness process.
    pub current_dir: PathBuf,
    /// The fully resolved command line used to launch the harness, including the script path and
    /// any trailing entrypoint arguments.
    pub command: Vec<String>,
}

impl BrowserHarnessInvocation {
    /// Launch the browser harness and capture stdout/stderr and exit status.
    pub fn launch(self) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
        self.launch_with_env(&[])
    }

    /// Launch the browser harness with additional environment variables.
    pub fn launch_with_env(
        self,
        extra_env: &[(&str, &std::ffi::OsStr)],
    ) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
        let BrowserHarnessInvocation {
            executable,
            harness_args,
            script,
            args,
            current_dir,
            command,
        } = self;

        let mut harness = Command::new(&executable);
        harness.args(&harness_args);
        let script_arg = if browser_harness_uses_html_entrypoint(&executable) {
            Url::from_file_path(&script)
                .map_err(|_| BrowserHarnessError::PreparationFailed {
                    message: format!(
                        "failed to convert browser harness script path {:?} into a file URL",
                        script
                    ),
                })?
                .to_string()
        } else {
            script.to_string_lossy().into_owned()
        };
        harness.arg(&script_arg);
        harness.args(&args);
        harness.current_dir(current_dir);
        for &(key, value) in extra_env {
            harness.env(key, value);
        }

        let output = harness
            .output()
            .map_err(|error| BrowserHarnessError::LaunchFailed {
                executable,
                script: script.clone(),
                command: command.clone(),
                message: error.to_string(),
            })?;

        Ok(BrowserHarnessOutcome {
            command,
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// A deterministic browser-harness execution result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserHarnessOutcome {
    /// The fully resolved command line used to launch the harness, including the script path and
    /// any trailing entrypoint arguments.
    pub command: Vec<String>,
    /// The harness process exit status.
    pub status: std::process::ExitStatus,
    /// Captured harness stdout.
    pub stdout: String,
    /// Captured harness stderr.
    pub stderr: String,
}

/// Error returned when launching a browser harness command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrowserHarnessError {
    /// The configured command override was malformed.
    MalformedOverride {
        /// The environment variable that carried the malformed override.
        env_var: &'static str,
        /// The malformed override value.
        value: String,
    },
    /// Browser-runtime harness preparation failed before launch.
    PreparationFailed {
        /// The preparation error message.
        message: String,
    },
    /// The harness command could not be launched.
    LaunchFailed {
        /// The executable that failed to launch.
        executable: String,
        /// The script or entrypoint that was being executed.
        script: PathBuf,
        /// The fully resolved command line that was being launched.
        command: Vec<String>,
        /// The launch error message.
        message: String,
    },
}

impl std::fmt::Display for BrowserHarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedOverride { env_var, value } => {
                write!(f, "malformed {env_var} override: {value:?}")
            }
            Self::PreparationFailed { message } => {
                write!(f, "failed to prepare browser harness execution: {message}")
            }
            Self::LaunchFailed {
                executable,
                script,
                command,
                message,
            } => write!(
                f,
                "failed to launch browser harness command {executable:?} for {script:?} with resolved command {command:?}: {message}"
            ),
        }
    }
}

impl std::error::Error for BrowserHarnessError {}

/// Execute an emitted browser-targeted bundle through the browser harness.
///
/// The bundle harness is written next to the emitted bundle directory so the shared prelude can
/// resolve the bundle glue with the expected relative layout.
pub fn browser_bundle_runtime_execute_checked(
    command: Option<&str>,
    bundle_root: impl AsRef<Path>,
    args: &[String],
    allow_subpaths: bool,
    run_registered_tests: bool,
) -> Result<BrowserRuntimeExecutionOutcome, BrowserHarnessError> {
    let bundle_root = bundle_root.as_ref();
    let bundle_dir = bundle_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| BrowserHarnessError::PreparationFailed {
            message: format!(
                "bundle root {:?} does not have a valid directory name",
                bundle_root
            ),
        })?;
    let current_dir =
        bundle_root
            .parent()
            .ok_or_else(|| BrowserHarnessError::PreparationFailed {
                message: format!(
                    "bundle root {:?} does not have a parent directory",
                    bundle_root
                ),
            })?;
    let browser_command = browser_harness_command_parts_checked(command)
        .map_err(|message| BrowserHarnessError::PreparationFailed { message })?;
    let use_html_entrypoint = browser_command
        .first()
        .is_some_and(|executable| browser_harness_uses_html_entrypoint(executable));
    let script_name = if use_html_entrypoint {
        "browser-bundle-runtime.html"
    } else {
        "browser-bundle-runtime.mjs"
    };
    let script_path = current_dir.join(script_name);
    let summary_path = current_dir.join("browser-bundle-runtime-summary.json");
    let script_contents = if use_html_entrypoint {
        browser_bundle_runtime_harness_page(bundle_dir, allow_subpaths, args, run_registered_tests)
    } else {
        browser_bundle_runtime_harness_script(
            bundle_dir,
            allow_subpaths,
            args,
            run_registered_tests,
        )
    };
    fs::write(&script_path, script_contents).map_err(|error| {
        BrowserHarnessError::PreparationFailed {
            message: error.to_string(),
        }
    })?;

    let outcome = browser_harness_run_checked_with_env(
        command,
        &script_path,
        &[],
        current_dir,
        &[(BROWSER_HARNESS_SUMMARY_FILE_ENV, summary_path.as_os_str())],
    )?;
    let summary = browser_runtime_summary_for_outcome(&summary_path, &outcome);

    Ok(BrowserRuntimeExecutionOutcome {
        command: outcome.command,
        status: outcome.status,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        host_contract: summary
            .host_contract
            .unwrap_or(RuntimeHostContract::BrowserRequested),
        runtime_backend: summary
            .runtime_backend
            .unwrap_or(RuntimeBackend::BrowserHarness),
        reported_args: summary.args,
        registered_tests: summary.tests,
        tests_failed: summary.tests_failed.unwrap_or(0),
        thread_topology: summary.thread_topology.unwrap_or_default(),
    })
}

/// Execute a WASM module through the browser harness and capture the resulting summary.
pub fn browser_runtime_execute_checked(
    command: Option<&str>,
    wasm_bytes: &[u8],
    args: &[String],
    current_dir: impl AsRef<Path>,
    run_registered_tests: bool,
) -> Result<BrowserRuntimeExecutionOutcome, BrowserHarnessError> {
    let tempdir = tempdir().map_err(|error| BrowserHarnessError::PreparationFailed {
        message: error.to_string(),
    })?;
    let browser_command = browser_harness_command_parts_checked(command)
        .map_err(|message| BrowserHarnessError::PreparationFailed { message })?;
    let use_html_entrypoint = browser_command
        .first()
        .is_some_and(|executable| browser_harness_uses_html_entrypoint(executable));
    let script_name = if use_html_entrypoint {
        "browser-runtime.html"
    } else {
        "browser-runtime.mjs"
    };
    let script_path = tempdir.path().join(script_name);
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    let script_contents = if use_html_entrypoint {
        browser_runtime_harness_page(wasm_bytes, args, run_registered_tests)
    } else {
        browser_runtime_harness_script(wasm_bytes, args, run_registered_tests)
    };
    fs::write(&script_path, script_contents).map_err(|error| {
        BrowserHarnessError::PreparationFailed {
            message: error.to_string(),
        }
    })?;

    let outcome = browser_harness_run_checked_with_env(
        command,
        &script_path,
        &[],
        current_dir,
        &[(BROWSER_HARNESS_SUMMARY_FILE_ENV, summary_path.as_os_str())],
    )?;
    let summary = browser_runtime_summary_for_outcome(&summary_path, &outcome);

    Ok(BrowserRuntimeExecutionOutcome {
        command: outcome.command,
        status: outcome.status,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        host_contract: summary
            .host_contract
            .unwrap_or(RuntimeHostContract::BrowserRequested),
        runtime_backend: summary
            .runtime_backend
            .unwrap_or(RuntimeBackend::BrowserHarness),
        reported_args: summary.args,
        registered_tests: summary.tests,
        tests_failed: summary.tests_failed.unwrap_or(0),
        thread_topology: summary.thread_topology.unwrap_or_default(),
    })
}

/// Build a browser harness launch plan from the configured environment override.
pub fn browser_harness_invocation_checked(
    command: Option<&str>,
    script: impl AsRef<Path>,
    args: &[String],
    current_dir: impl AsRef<Path>,
) -> Result<BrowserHarnessInvocation, BrowserHarnessError> {
    let mut parts = browser_harness_command_parts_checked(command).map_err(|value| {
        BrowserHarnessError::MalformedOverride {
            env_var: BROWSER_HARNESS_COMMAND_ENV,
            value,
        }
    })?;

    let executable = parts.remove(0);
    let script = script.as_ref().to_path_buf();
    let current_dir = current_dir.as_ref().to_path_buf();
    let mut command = Vec::with_capacity(2 + parts.len() + args.len());
    command.push(executable.clone());
    command.extend(parts.iter().cloned());
    let script_arg = if browser_harness_uses_html_entrypoint(&executable) {
        Url::from_file_path(&script)
            .map_err(|_| BrowserHarnessError::PreparationFailed {
                message: format!(
                    "failed to convert browser harness script path {:?} into a file URL",
                    script
                ),
            })?
            .to_string()
    } else {
        script.to_string_lossy().into_owned()
    };
    command.push(script_arg);
    command.extend(args.iter().cloned());

    Ok(BrowserHarnessInvocation {
        executable,
        harness_args: parts,
        script,
        args: args.to_vec(),
        current_dir,
        command,
    })
}

/// Launch the browser harness command, capturing stdout/stderr and exit status.
pub fn browser_harness_run_checked(
    command: Option<&str>,
    script: impl AsRef<Path>,
    args: &[String],
    current_dir: impl AsRef<Path>,
) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
    browser_harness_invocation_checked(command, script, args, current_dir)?.launch()
}

/// Launch the browser harness with additional environment variables.
pub fn browser_harness_run_checked_with_env(
    command: Option<&str>,
    script: impl AsRef<Path>,
    args: &[String],
    current_dir: impl AsRef<Path>,
    extra_env: &[(&str, &std::ffi::OsStr)],
) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
    browser_harness_invocation_checked(command, script, args, current_dir)?
        .launch_with_env(extra_env)
}

#[cfg(test)]
#[path = "execute_tests.rs"]
mod execute_tests;

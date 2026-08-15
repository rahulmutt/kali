//! Derive a verdict class from one kali run and one node run.
//!
//! The class -- not the literal output -- is what an oracle case asserts. That
//! is the whole point: the register's §0.2 has been stale since 2026-07-24
//! because a verdict was prose a human had to re-derive. As a class, a change
//! is a red test.

/// One side's captured process result. `code` is `None` when the process was
/// killed (timeout) or died to a signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Fixed,
    Silent,
    FailClosed,
    FlInternal,
    AcceptsInvalid,
    BothReject,
    Timeout,
    Nondeterministic,
}

impl Verdict {
    /// The spelling a case file writes in `verdict = "..."`.
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Fixed => "fixed",
            Verdict::Silent => "silent",
            Verdict::FailClosed => "fail_closed",
            Verdict::FlInternal => "fl_internal",
            Verdict::AcceptsInvalid => "accepts_invalid",
            Verdict::BothReject => "both_reject",
            Verdict::Timeout => "timeout",
            Verdict::Nondeterministic => "nondeterministic",
        }
    }
}

/// Is `code` in `specs/15-errors.md`'s public range registry, excluding the
/// `E0xxx` internal family?
///
/// `E4xxx` is deliberately NOT documented-in-spec: `E4003` (fuel trap) and
/// `E4201` (WebAssembly translation error) are real and reachable, but the
/// spec's range table has no `E4xxx` row at all. They therefore classify as
/// `FL_INTERNAL` -- the right verdict, currently for the wrong reason. Task 4
/// closes the taxonomy gap; this function is not where an exception list goes,
/// because hiding a spec gap inside a test tool is how it stays open.
pub fn is_documented_code(code: &str) -> bool {
    let Some(digits) = code.strip_prefix('E') else {
        return false;
    };
    if digits.len() != 4 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    matches!(digits.as_bytes()[0], b'5' | b'6' | b'7' | b'8' | b'9')
}

/// The first `error[Ennnn]` code in a captured stderr, if any.
fn first_error_code(stderr: &str) -> Option<String> {
    let start = stderr.find("error[")? + "error[".len();
    let rest = &stderr[start..];
    let end = rest.find(']')?;
    Some(rest[..end].to_string())
}

fn refused(run: &Run) -> bool {
    run.code != Some(0)
}

/// Two runs of the same side, compared. Used to detect nondeterminism before a
/// verdict is recorded.
///
/// Two timeouts do NOT agree. A pair of hangs says the program never settled,
/// which is not evidence that its behaviour is stable.
pub fn runs_agree(a: &Run, b: &Run) -> bool {
    if a.timed_out || b.timed_out {
        return false;
    }
    a.code == b.code && a.stdout == b.stdout && a.stderr == b.stderr
}

pub fn classify(kali: &Run, node: &Run) -> Verdict {
    if kali.timed_out || node.timed_out {
        return Verdict::Timeout;
    }
    match (refused(kali), refused(node)) {
        (false, false) => {
            if kali.stdout == node.stdout {
                Verdict::Fixed
            } else {
                Verdict::Silent
            }
        }
        (true, false) => match first_error_code(&kali.stderr) {
            Some(code) if is_documented_code(&code) => Verdict::FailClosed,
            // No code at all (a panic, a bare nonzero exit) is not an honest
            // denial either -- defaulting it to FAIL_CLOSED would record a
            // crash as acceptable.
            _ => Verdict::FlInternal,
        },
        (false, true) => Verdict::AcceptsInvalid,
        (true, true) => Verdict::BothReject,
    }
}

#[cfg(test)]
#[path = "verdict_tests.rs"]
mod verdict_tests;

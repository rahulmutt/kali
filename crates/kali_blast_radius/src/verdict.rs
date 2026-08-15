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
/// The 5-family is documented only in its tens-ranges `E51xx` through `E55xx`;
/// `E50xx` and `E56xx`-`E59xx` are not. The 6-9 families are documented as
/// complete ranges: `E6xxx`, `E7xxx`, `E8xxx`, `E9xxx`.
///
/// `E4xxx` is documented in `specs/15-errors.md` as `kali_runtime`'s
/// runtime-error family, but that family is MIXED, not uniformly internal:
/// `E4003` (fuel/resource-limit trap) and `E4201` (wasm translation failure)
/// are internal like `E0xxx`, but `E4001` is a sandbox-policy denial -- an
/// honest refusal, not kali failing. `E4002` has no emitter in `crates/`
/// today; by name and band it would be the same kind of denial as `E4001`
/// if it becomes reachable, but that is inference, not traced behaviour.
/// This function still returns `false` for every `E4xxx` code, including
/// `E4001`/`E4002`: that is a known, deliberate limitation, not an
/// oversight. No register entry measured by this project exercises a
/// sandbox-effect denial, so no recorded verdict depends on it today. See
/// `docs/superpowers/followups/e4xxx-e54xx-taxonomy-collision.md`.
pub fn is_documented_code(code: &str) -> bool {
    let Some(digits) = code.strip_prefix('E') else {
        return false;
    };
    if digits.len() != 4 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let first_digit = digits.as_bytes()[0];
    if first_digit == b'5' {
        // E5xxx: only E51xx through E55xx are documented
        let second_digit = digits.as_bytes()[1];
        matches!(second_digit, b'1' | b'2' | b'3' | b'4' | b'5')
    } else {
        // E6xxx through E9xxx are fully documented families
        matches!(first_digit, b'6' | b'7' | b'8' | b'9')
    }
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

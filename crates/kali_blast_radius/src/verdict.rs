//! Derive a verdict class from one kali run and one node run.
//!
//! The class -- not the literal output -- is what an oracle case asserts. That
//! is the whole point, and the reason is historical: the register's §0.2 was
//! stale from 2026-07-24 until `809767dc67` (2026-08-15), because a verdict was
//! prose a human had to re-derive. §0.2 is now generated from the oracle cases
//! under `crates/kali_cli/tests/cases/oracle/`, so the staleness that motivated
//! this module is closed. As a class, a change is a red test -- which is what
//! keeps it closed.

/// One side's captured process result. `code` is `None` when the process was
/// killed (timeout) or died to a signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

/// Which captured stream carries the observation a case is making.
///
/// Almost every register entry renders its damage on stdout, and that is the
/// default. R-33 is the exception that forced this to exist: `console.warn`
/// writes to STDERR on both engines, so a stdout-only comparison sees two
/// empty strings, calls the pair FIXED, and retires a live defect from the
/// damage set without ever reading the channel the defect is on. A green
/// verdict for an unobserved defect is the single most dangerous output this
/// instrument can produce, so the stream is now something a case states rather
/// than something the classifier assumes.
///
/// This selects ONLY the both-engines-exited-0 equality comparison. It does
/// not touch the timeout arm, the refusal arms, or error-code detection --
/// see `classify_observing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObservedStream {
    #[default]
    Stdout,
    Stderr,
}

impl ObservedStream {
    /// The spelling a case file writes in `observe = "..."`.
    pub fn as_str(self) -> &'static str {
        match self {
            ObservedStream::Stdout => "stdout",
            ObservedStream::Stderr => "stderr",
        }
    }
}

impl Run {
    /// The stream this run is being observed on.
    fn observed(&self, stream: ObservedStream) -> &str {
        match stream {
            ObservedStream::Stdout => &self.stdout,
            ObservedStream::Stderr => &self.stderr,
        }
    }
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

/// Classify a pair of runs, observing stdout.
///
/// Kept as the whole API's front door because it is what almost every case
/// wants; it is exactly `classify_observing(kali, node, Stdout)`.
///
/// NOT DEAD CODE, despite having no non-test caller today -- `run_oracle` calls
/// `classify_observing` so it can pass a case's `observe`. This function is
/// deliberately retained as two things: the documented default entry point, and
/// the byte-identity anchor for the stream selector. Its unit tests assert that
/// it agrees with `classify_observing(.., Stdout)` across every arm, which is
/// what guarantees that adding the selector changed no verdict already recorded
/// in `cases/oracle/`. Deleting it would remove that guarantee, not just a
/// wrapper.
pub fn classify(kali: &Run, node: &Run) -> Verdict {
    classify_observing(kali, node, ObservedStream::Stdout)
}

/// Classify a pair of runs, observing `stream`.
///
/// `stream` reaches EXACTLY ONE decision: the equality test in the
/// both-engines-exited-0 arm, which is the only place a verdict depends on
/// what the programs printed. Everything else is deliberately stream-blind:
///
/// - The timeout arm never reads output at all.
/// - Refusal is decided by the EXIT CODE, not by any stream.
/// - Error-code detection always reads `stderr`, whatever is being observed.
///   A refusal is diagnosed from the diagnostic, and a `kali` that fails
///   closed writes `error[Ennnn]` to stderr regardless of which stream the
///   case is comparing. Reading the code off the observed stream would make
///   an `observe = "stderr"` case classify FAIL_CLOSED versus FL_INTERNAL by
///   accident of where the program's own output landed.
pub fn classify_observing(kali: &Run, node: &Run, stream: ObservedStream) -> Verdict {
    if kali.timed_out || node.timed_out {
        return Verdict::Timeout;
    }
    match (refused(kali), refused(node)) {
        (false, false) => {
            if kali.observed(stream) == node.observed(stream) {
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

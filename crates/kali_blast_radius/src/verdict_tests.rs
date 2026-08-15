use super::*;

fn ok(stdout: &str) -> Run {
    Run {
        code: Some(0),
        stdout: stdout.into(),
        stderr: String::new(),
        timed_out: false,
    }
}

fn failed(code: i32, stderr: &str) -> Run {
    Run {
        code: Some(code),
        stdout: String::new(),
        stderr: stderr.into(),
        timed_out: false,
    }
}

fn timed_out() -> Run {
    Run {
        code: None,
        stdout: String::new(),
        stderr: String::new(),
        timed_out: true,
    }
}

#[test]
fn equal_output_on_both_sides_is_fixed() {
    assert_eq!(classify(&ok("7\n"), &ok("7\n")), Verdict::Fixed);
}

#[test]
fn exit_zero_both_sides_with_different_output_is_silent() {
    assert_eq!(classify(&ok("0\n"), &ok("1\n")), Verdict::Silent);
}

#[test]
fn a_documented_denial_against_a_working_node_is_fail_closed() {
    let kali = failed(1, "error[E5506]: feature unavailable in current phase");
    assert_eq!(classify(&kali, &ok("1\n")), Verdict::FailClosed);
}

#[test]
fn an_internal_e0xxx_against_a_working_node_is_fl_internal() {
    let kali = failed(1, "error[E0001]: internal compiler error");
    assert_eq!(classify(&kali, &ok("1\n")), Verdict::FlInternal);
}

#[test]
fn the_undocumented_e4xxx_family_is_fl_internal() {
    // E4003 (fuel trap) and E4201 (wasm translation) are real and reachable but
    // absent from specs/15-errors.md's range table -- see spec §7.1.
    assert_eq!(
        classify(&failed(1, "error[E4003]: trap"), &ok("x\n")),
        Verdict::FlInternal
    );
    assert_eq!(
        classify(&failed(1, "error[E4201]: translation"), &ok("x\n")),
        Verdict::FlInternal
    );
}

#[test]
fn kali_accepting_what_node_refuses_is_accepts_invalid() {
    let node = failed(
        1,
        "SyntaxError: More than one default clause in switch statement",
    );
    assert_eq!(classify(&ok("v=d2\n"), &node), Verdict::AcceptsInvalid);
}

#[test]
fn both_refusing_is_both_reject() {
    let kali = failed(1, "error[E5506]: nope");
    let node = failed(1, "SyntaxError: nope");
    assert_eq!(classify(&kali, &node), Verdict::BothReject);
}

#[test]
fn a_timeout_on_either_side_outranks_every_other_verdict() {
    assert_eq!(classify(&timed_out(), &ok("1\n")), Verdict::Timeout);
    assert_eq!(classify(&ok("1\n"), &timed_out()), Verdict::Timeout);
}

#[test]
fn a_denial_with_no_recognisable_code_is_fl_internal_not_fail_closed() {
    // A panic or an unadorned failure is not an honest denial. Defaulting it to
    // FAIL_CLOSED would count a crash as acceptable behaviour.
    let kali = failed(101, "thread 'main' panicked at src/main.rs:1:1");
    assert_eq!(classify(&kali, &ok("1\n")), Verdict::FlInternal);
}

#[test]
fn runs_agree_compares_output_and_exit_but_two_timeouts_never_agree() {
    assert!(runs_agree(&ok("a"), &ok("a")));
    assert!(!runs_agree(&ok("a"), &ok("b")));
    assert!(!runs_agree(&failed(1, "x"), &failed(2, "x")));
    // A pair of timeouts is not evidence of stable behaviour.
    assert!(!runs_agree(&timed_out(), &timed_out()));
}

#[test]
fn documented_ranges_follow_the_errors_spec() {
    for code in [
        "E5101", "E5203", "E5506", "E6004", "E7001", "E8002", "E9100",
    ] {
        assert!(
            is_documented_code(code),
            "{code} is in the spec's public ranges"
        );
    }
    for code in [
        "E0001", "E4003", "E4201", "E1000", "W3002", "nonsense", "E5000", "E5099", "E5601", "E5999",
    ] {
        assert!(
            !is_documented_code(code),
            "{code} is not a documented error code"
        );
    }
}

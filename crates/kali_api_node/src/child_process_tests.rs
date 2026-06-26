use crate::*;

#[test]
fn child_process_helpers_capture_command_output() {
    let (command, args): (&str, &[&str]) = if cfg!(windows) {
        ("cmd", &["/C", "echo", "child-process"])
    } else {
        ("sh", &["-lc", "printf child-process"])
    };

    let output = NodeChildProcess::spawn_sync(command, args).expect("spawn child process");
    assert_eq!(output.status(), 0);
    assert_eq!(
        String::from_utf8(output.stdout().to_vec())
            .expect("stdout")
            .trim_end(),
        "child-process"
    );
    assert!(output.stderr().is_empty(), "stderr: {:?}", output.stderr());
}

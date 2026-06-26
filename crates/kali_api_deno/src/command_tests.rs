use crate::*;

#[test]
fn command_helper_runs_child_process_and_captures_output() {
    let mut command = DenoCommand::new("sh");
    command
        .args(["-c", "printf '%s' \"deno-command\""])
        .current_dir("./")
        .env("DENO_HELPER", "enabled");

    let output = command.spawn().expect("spawn command");
    assert_eq!(output.status(), 0);
    assert_eq!(output.stdout(), b"deno-command");
    assert_eq!(output.stderr(), b"");
    assert_eq!(output.text_stdout().expect("stdout text"), "deno-command");
}

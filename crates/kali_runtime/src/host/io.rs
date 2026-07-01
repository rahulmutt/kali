//! Stdout/stderr buffering and console-value formatting.
use crate::*;

pub(crate) fn append_stdout(state: &mut KaliHostState, text: String) {
    state.stdout.push_str(&text);
    state.stdout.push('\n');
}

pub(crate) fn append_stdout_raw(state: &mut KaliHostState, text: String) {
    state.stdout.push_str(&text);
}

pub(crate) fn append_stderr(state: &mut KaliHostState, text: String) {
    state.stderr.push_str(&text);
    state.stderr.push('\n');
}

pub(crate) fn append_stderr_raw(state: &mut KaliHostState, text: String) {
    state.stderr.push_str(&text);
}

pub(crate) fn format_console_value(caller: &mut Caller<'_, KaliHostState>, value: i64) -> String {
    if let Some(bytes) = decode_string_handle_bytes(caller, value) {
        if let Ok(text) = String::from_utf8(bytes) {
            return text;
        }
    }

    value.to_string()
}

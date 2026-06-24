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
    let raw = value as u64;
    if raw & STRING_HANDLE_TAG != 0 {
        let offset = ((raw >> 32) & 0x7fff_ffff) as i32;
        let len = (raw & 0xffff_ffff) as i32;
        if let Ok(bytes) = read_guest_bytes(caller, offset, len) {
            if let Ok(text) = String::from_utf8(bytes) {
                return text;
            }
        }
    }

    value.to_string()
}

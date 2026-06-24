//! kali_runtime-specific test builders (compiled under cfg(test)).

pub(crate) fn compile_wat(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap_or_else(|error| panic!("valid wat error: {error}\n{wat}"))
}

pub(crate) fn wat_assert_buffer_eq(start: i32, expected: &str) -> String {
    let mut checks = String::new();
    for (index, byte) in expected.as_bytes().iter().enumerate() {
        checks.push_str(&format!(
                "                i32.const {}\n                i32.load8_u\n                i32.const {}\n                i32.ne\n                if\n                    unreachable\n                end\n",
                start + index as i32,
                byte
            ));
    }
    checks
}

#[cfg(unix)]
pub(crate) fn browser_exit_status(code: i32) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code << 8)
}

#[cfg(windows)]
pub(crate) fn browser_exit_status(code: i32) -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code as u32)
}

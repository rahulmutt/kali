use super::*;

#[test]
fn write_stdout_bytes_imports_and_calls_host() {
    let src = "const out = new Array(2); out[0] = 80; out[1] = 52; Kali.writeStdoutBytes(out);";
    let program = parse_and_lower_lir(src);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let text = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");

    // The host import must be declared. Extract the wasm function index
    // wasmprinter assigns it so the call site can be checked against the real
    // index rather than a bare `call` (which matches almost any program).
    let import_line = text
        .lines()
        .find(|line| line.contains("\"stdout_write_bytes\""))
        .unwrap_or_else(|| panic!("missing stdout_write_bytes import:\n{text}"));
    let index = import_line
        .split("(;")
        .nth(1)
        .and_then(|rest| rest.split(";)").next())
        .and_then(|digits| digits.trim().parse::<u32>().ok())
        .unwrap_or_else(|| panic!("could not parse import index from: {import_line}"));

    // The intrinsic must lower to a `call <index>` targeting that exact import.
    let call_needle = format!("call {index}");
    assert!(
        text.lines().any(|line| line.trim() == call_needle),
        "missing `{call_needle}` to the stdout_write_bytes import:\n{text}"
    );
}

#[test]
fn write_stdout_bytes_zero_args_reports_diagnostic_without_panicking() {
    // The recognizer matches on callee text + object only, so a zero-arg call
    // reaches the emit branch. It must reject with an E5506 diagnostic rather
    // than indexing `children[1]` out of bounds and panicking.
    let program = parse_and_lower_lir("Kali.writeStdoutBytes();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("Kali.writeStdoutBytes requires exactly one array argument")
        }),
        "expected a zero-arg writeStdoutBytes diagnostic: {:?}",
        result.diagnostics
    );
}

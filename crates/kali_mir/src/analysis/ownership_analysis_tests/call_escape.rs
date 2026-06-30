use super::*;

#[test]
fn test_call_arguments_escape_to_unknown_callees() {
    let mir = analyze("const answer = 1; sink(answer);");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
}

#[test]
fn test_inline_pure_function_calls_do_not_force_argument_escape() {
    let mir = analyze("const answer = 1; (function identity(x) { return 0; })(answer);");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::Stack);
    assert!(!binding.escapes);
}

#[test]
fn test_inline_leaking_function_calls_still_escape_arguments() {
    let mir = analyze("const answer = 1; (function leak(x) { return x; })(answer);");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
    assert_eq!(
        binding.thread_boundary_disposition(),
        ThreadBoundaryDisposition::LocalOnly
    );
    assert!(binding.is_thread_local());
    assert!(!binding.is_thread_shareable());
}

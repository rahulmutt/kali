use super::*;

#[test]
fn test_stack_local_bindings_stay_stack_allocated() {
    let mir = analyze("const answer = 40 + 2;");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.kind, MirBindingKind::Local);
    assert_eq!(binding.ownership, OwnershipClass::Stack);
    assert!(!binding.escapes);
    assert_eq!(binding.layout, LayoutDescriptor::scalar("number"));
}

#[test]
fn test_returned_bindings_become_owned_heap() {
    let mir = analyze("function make() { const answer = 40 + 2; return answer; }");
    let function = mir.function("make").expect("make function");
    let binding = function.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
    assert_eq!(binding.layout, LayoutDescriptor::scalar("number"));
}

#[test]
fn test_captured_bindings_become_shared_heap() {
    let mir = analyze(
        "function outer() { const answer = 1; function inner() { return answer; } return inner; }",
    );
    let outer = mir.function("outer").expect("outer function");
    let binding = outer.binding("answer").expect("answer binding");
    let inner = mir.function("inner").expect("inner function");
    let inner_binding = inner.binding("inner").expect("inner binding");

    assert_eq!(binding.ownership, OwnershipClass::SharedHeap);
    assert!(binding.escapes);
    assert_eq!(binding.captured_by, vec!["inner".to_string()]);
    assert_eq!(
        inner_binding.layout,
        LayoutDescriptor::Closure {
            captures: vec!["answer".to_string()],
        }
    );
    assert_eq!(
        binding.thread_boundary_disposition(),
        ThreadBoundaryDisposition::SharedOnly
    );
    assert!(binding.is_thread_shareable());
    assert!(!binding.is_thread_local());
}

#[test]
fn test_non_escaping_closure_captures_stay_borrowed() {
    let mir = analyze(
        "function outer() { const answer = 1; function inner() { return answer; } inner(); return 0; }",
    );
    let outer = mir.function("outer").expect("outer function");
    let binding = outer.binding("answer").expect("answer binding");
    let inner = mir.function("inner").expect("inner function");
    let inner_binding = inner.binding("inner").expect("inner binding");

    assert_eq!(binding.ownership, OwnershipClass::Borrowed);
    assert!(!binding.escapes);
    assert_eq!(binding.captured_by, vec!["inner".to_string()]);
    assert_eq!(inner_binding.ownership, OwnershipClass::Stack);
    assert!(!inner_binding.escapes);
    assert_eq!(
        inner_binding.layout,
        LayoutDescriptor::Closure {
            captures: vec!["answer".to_string()],
        }
    );
}

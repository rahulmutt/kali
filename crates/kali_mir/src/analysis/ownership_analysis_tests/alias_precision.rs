use super::*;

#[test]
fn test_aliased_function_expressions_preserve_direct_call_precision() {
    let mir =
        analyze("const identity = function(x) { return 0; }; const answer = 1; identity(answer);");
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::Stack);
    assert!(!binding.escapes);
}

#[test]
fn test_function_alias_chains_preserve_direct_call_precision() {
    let mir = analyze(
        "const identity = function(x) { return 0; }; const alias = identity; const alias2 = alias; const answer = 1; alias2(answer);",
    );
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::Stack);
    assert!(!binding.escapes);
}

#[test]
fn test_aliased_function_expressions_still_track_nested_closure_escapes() {
    let mir = analyze(
        "const leak = function outer(x) { function inner() { return x; } return inner; }; const answer = 1; leak(answer);",
    );
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
}

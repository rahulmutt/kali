use crate::test_support::*;
use crate::*;

#[test]
fn test_borrowed_lifetime_reports_are_deterministic() {
    let mir = analyze(
        "function alpha(x) { return x; } function beta(y) { function inner() { return y; } inner(); return y; }",
    );

    let module = mir.module_scope().expect("module scope");
    let alpha_binding = module.binding("alpha").expect("alpha binding");
    assert!(alpha_binding.borrowed_lifetime("module").is_none());

    let alpha = mir.function("alpha").expect("alpha function");
    let alpha_param = alpha.binding("x").expect("alpha param");
    assert_eq!(
        alpha_param.borrowed_lifetime("alpha"),
        Some(BorrowedLifetime {
            scope: "alpha".to_string(),
            name: "x".to_string(),
            captured_by: Vec::new(),
        })
    );

    let beta = mir.function("beta").expect("beta function");
    let beta_param = beta.binding("y").expect("beta param");
    assert_eq!(
        beta_param.borrowed_lifetime("beta"),
        Some(BorrowedLifetime {
            scope: "beta".to_string(),
            name: "y".to_string(),
            captured_by: vec!["inner".to_string()],
        })
    );

    assert_eq!(
        mir.borrowed_lifetimes(),
        vec![
            BorrowedLifetime {
                scope: "alpha".to_string(),
                name: "x".to_string(),
                captured_by: Vec::new(),
            },
            BorrowedLifetime {
                scope: "beta".to_string(),
                name: "y".to_string(),
                captured_by: vec!["inner".to_string()],
            },
        ]
    );
}

#[test]
fn test_borrowed_lifetime_reports_collapse_exact_duplicates() {
    let binding = MirBinding {
        name: "value".to_string(),
        kind: MirBindingKind::Local,
        ownership: OwnershipClass::Borrowed,
        layout: LayoutDescriptor::scalar("number"),
        escapes: false,
        captured_by: vec!["inner".to_string()],
    };

    let function = MirFunction {
        name: Some("dup".to_string()),
        kind: MirFunctionKind::Function,
        function_flavor: None,
        bindings: vec![binding.clone(), binding.clone()],
    };

    assert_eq!(
        function.borrowed_lifetimes("dup"),
        vec![BorrowedLifetime {
            scope: "dup".to_string(),
            name: "value".to_string(),
            captured_by: vec!["inner".to_string()],
        }]
    );

    let program = MirProgram {
        root: MirNodeId::new(0),
        nodes: Vec::new(),
        functions: vec![function.clone(), function],
        arena_facts: Vec::new(),
        parent_labels: std::collections::BTreeMap::new(),
    };

    assert_eq!(
        program.borrowed_lifetimes(),
        vec![BorrowedLifetime {
            scope: "dup".to_string(),
            name: "value".to_string(),
            captured_by: vec!["inner".to_string()],
        }]
    );
}

#[test]
fn test_scope_filtered_mir_summaries_stay_deterministic() {
    let mir = analyze("function alpha(x) { return x; } function beta(y) { return y; }");

    assert_eq!(
        mir.borrowed_lifetimes_in_scope("alpha"),
        vec![BorrowedLifetime {
            scope: "alpha".to_string(),
            name: "x".to_string(),
            captured_by: Vec::new(),
        }]
    );
    assert_eq!(
        mir.borrowed_lifetimes_in_scope("beta"),
        vec![BorrowedLifetime {
            scope: "beta".to_string(),
            name: "y".to_string(),
            captured_by: Vec::new(),
        }]
    );
    assert!(mir.borrowed_lifetimes_in_scope("module").is_empty());
    assert_eq!(
        mir.module_borrowed_lifetimes(),
        mir.borrowed_lifetimes_in_scope("module")
    );

    let beta_profile = mir.thread_boundary_profile_in_scope("beta");
    assert_eq!(
        mir.module_thread_boundary_profile().bindings,
        mir.thread_boundary_profile_in_scope("module").bindings
    );
    assert!(beta_profile
        .bindings
        .iter()
        .all(|binding| binding.scope == "beta"));
    assert_eq!(beta_profile.bindings.len(), 2);
    assert_eq!(beta_profile.bindings[0].name, "beta");
    assert_eq!(beta_profile.bindings[1].name, "y");
}

#[test]
fn test_module_scope_summary_helpers_cover_borrowed_bindings() {
    let mir = analyze("const answer = 1; function inner() { return answer; } inner();");

    assert_eq!(
        mir.module_borrowed_lifetimes(),
        vec![BorrowedLifetime {
            scope: "module".to_string(),
            name: "answer".to_string(),
            captured_by: vec!["inner".to_string()],
        }]
    );

    let profile = mir.module_thread_boundary_profile();
    assert_eq!(
        profile.bindings,
        vec![
            ThreadBoundaryBinding {
                scope: "module".to_string(),
                name: "answer".to_string(),
                disposition: ThreadBoundaryDisposition::LocalOnly,
            },
            ThreadBoundaryBinding {
                scope: "module".to_string(),
                name: "inner".to_string(),
                disposition: ThreadBoundaryDisposition::LocalOnly,
            },
        ]
    );
}

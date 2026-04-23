use super::*;
use kali_common::FileId;
use kali_hir::{HirLowerer, HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};
use kali_lexer::Lexer;
use kali_parser::Parser;

fn parse_and_lower_hir(source: &str) -> HirLoweringResult {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let statements = parser.parse(None).statements;
    let mut lowerer = HirLowerer::new();
    lowerer.lower_statements(&statements)
}

fn analyze(source: &str) -> MirProgram {
    let hir = parse_and_lower_hir(source);
    MirLowerer::new().lower_hir_result(&hir)
}

#[test]
fn test_mir_lowering_preserves_program_shape() {
    let hir = parse_and_lower_hir("const answer = 40 + 2;");
    let mir = MirLowerer::new().lower_hir_result(&hir);

    assert_eq!(mir.nodes[mir.root.0 as usize].kind, MirNodeKind::Program);
    assert_eq!(mir.nodes[mir.root.0 as usize].children.len(), 1);
    assert_eq!(
        mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize].kind,
        MirNodeKind::Decl
    );
}

#[test]
fn test_call_expressions_lower_to_call_nodes() {
    let hir = parse_and_lower_hir("foo(bar, 1);");
    let mir = MirLowerer::new().lower_hir_result(&hir);
    let expr_stmt = &mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize];
    let call = expr_stmt
        .children
        .iter()
        .map(|child| &mir.nodes[child.0 as usize])
        .find(|node| node.kind == MirNodeKind::Call)
        .expect("call node");
    assert_eq!(call.children.len(), 3);
}

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

#[test]
fn test_ownership_classes_define_thread_boundary_rules() {
    assert_eq!(
        OwnershipClass::Stack.thread_boundary_disposition(),
        ThreadBoundaryDisposition::LocalOnly
    );
    assert_eq!(
        OwnershipClass::OwnedHeap.thread_boundary_disposition(),
        ThreadBoundaryDisposition::LocalOnly
    );
    assert_eq!(
        OwnershipClass::Borrowed.thread_boundary_disposition(),
        ThreadBoundaryDisposition::LocalOnly
    );
    assert_eq!(
        OwnershipClass::SharedHeap.thread_boundary_disposition(),
        ThreadBoundaryDisposition::SharedOnly
    );
    assert!(!OwnershipClass::Stack.is_thread_shareable());
    assert!(OwnershipClass::SharedHeap.is_thread_shareable());
    assert!(OwnershipClass::Stack.is_thread_local());
    assert!(!OwnershipClass::SharedHeap.is_thread_local());
}

#[test]
fn test_thread_boundary_profiles_split_shareable_and_local_bindings() {
    let mir = analyze(
        "function outer() { const shared = 1; const localOnly = 2; function inner() { return shared; } return inner; }",
    );
    let profile = mir.thread_boundary_profile();

    let shared = profile
        .bindings
        .iter()
        .find(|binding| binding.scope == "outer" && binding.name == "shared")
        .expect("shared binding");
    assert_eq!(shared.disposition, ThreadBoundaryDisposition::SharedOnly);

    let local = profile
        .bindings
        .iter()
        .find(|binding| binding.scope == "outer" && binding.name == "localOnly")
        .expect("local binding");
    assert_eq!(local.disposition, ThreadBoundaryDisposition::LocalOnly);

    let inner = profile
        .bindings
        .iter()
        .find(|binding| binding.scope == "outer" && binding.name == "inner")
        .expect("inner binding");
    assert_eq!(inner.disposition, ThreadBoundaryDisposition::LocalOnly);

    let outer = profile
        .bindings
        .iter()
        .find(|binding| binding.scope == "module" && binding.name == "outer")
        .expect("outer binding");
    assert_eq!(outer.disposition, ThreadBoundaryDisposition::LocalOnly);
}

#[test]
fn test_thread_boundary_profile_merges_duplicate_entries_with_shared_precedence() {
    let profile = ThreadBoundaryProfile {
        bindings: vec![
            ThreadBoundaryBinding {
                scope: "outer".to_string(),
                name: "value".to_string(),
                disposition: ThreadBoundaryDisposition::LocalOnly,
            },
            ThreadBoundaryBinding {
                scope: "outer".to_string(),
                name: "value".to_string(),
                disposition: ThreadBoundaryDisposition::SharedOnly,
            },
        ],
    }
    .finalize();

    assert_eq!(profile.bindings.len(), 1);
    assert_eq!(profile.bindings[0].scope, "outer");
    assert_eq!(profile.bindings[0].name, "value");
    assert_eq!(
        profile.bindings[0].disposition,
        ThreadBoundaryDisposition::SharedOnly
    );
}

#[test]
fn test_binding_thread_boundary_entry_uses_scope_and_disposition() {
    let binding = MirBinding {
        name: "value".to_string(),
        kind: MirBindingKind::Local,
        ownership: OwnershipClass::SharedHeap,
        layout: LayoutDescriptor::scalar("number"),
        escapes: true,
        captured_by: vec!["inner".to_string()],
    };

    let entry = binding.thread_boundary_binding("outer");
    assert_eq!(entry.scope, "outer");
    assert_eq!(entry.name, "value");
    assert_eq!(entry.disposition, ThreadBoundaryDisposition::SharedOnly);
}

#[test]
fn test_object_literal_values_escape_without_treating_keys_as_identifiers() {
    let hir = HirLoweringResult {
        root: HirNodeId::new(0),
        nodes: vec![
            HirNode {
                kind: HirNodeKind::Program,
                span: None,
                text: None,
                children: vec![HirNodeId::new(1), HirNodeId::new(5)],
            },
            HirNode {
                kind: HirNodeKind::VarDecl,
                span: None,
                text: Some("const".to_string()),
                children: vec![HirNodeId::new(2)],
            },
            HirNode {
                kind: HirNodeKind::VarDeclarator,
                span: None,
                text: Some("answer".to_string()),
                children: vec![HirNodeId::new(3), HirNodeId::new(4)],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("answer".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::Literal,
                span: None,
                text: Some("1".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::VarDecl,
                span: None,
                text: Some("const".to_string()),
                children: vec![HirNodeId::new(7)],
            },
            HirNode {
                kind: HirNodeKind::ObjectExpr,
                span: None,
                text: None,
                children: vec![HirNodeId::new(8)],
            },
            HirNode {
                kind: HirNodeKind::VarDeclarator,
                span: None,
                text: Some("bag".to_string()),
                children: vec![HirNodeId::new(9), HirNodeId::new(6)],
            },
            HirNode {
                kind: HirNodeKind::ObjectProperty,
                span: None,
                text: Some("init".to_string()),
                children: vec![HirNodeId::new(10), HirNodeId::new(11)],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("bag".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::Literal,
                span: None,
                text: Some("answer".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("answer".to_string()),
                children: vec![],
            },
        ],
        diagnostics: vec![],
    };

    let mir = MirLowerer::new().lower_hir_result(&hir);
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
    assert_eq!(binding.captured_by, Vec::<String>::new());
}

#[test]
fn test_array_element_values_escape_to_heap_storage() {
    let hir = HirLoweringResult {
        root: HirNodeId::new(0),
        nodes: vec![
            HirNode {
                kind: HirNodeKind::Program,
                span: None,
                text: None,
                children: vec![HirNodeId::new(1), HirNodeId::new(5)],
            },
            HirNode {
                kind: HirNodeKind::VarDecl,
                span: None,
                text: Some("const".to_string()),
                children: vec![HirNodeId::new(2)],
            },
            HirNode {
                kind: HirNodeKind::VarDeclarator,
                span: None,
                text: Some("answer".to_string()),
                children: vec![HirNodeId::new(3), HirNodeId::new(4)],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("answer".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::Literal,
                span: None,
                text: Some("1".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::VarDecl,
                span: None,
                text: Some("const".to_string()),
                children: vec![HirNodeId::new(6)],
            },
            HirNode {
                kind: HirNodeKind::VarDeclarator,
                span: None,
                text: Some("bag".to_string()),
                children: vec![HirNodeId::new(7), HirNodeId::new(8)],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("bag".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::ArrayExpr,
                span: None,
                text: None,
                children: vec![HirNodeId::new(9)],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("answer".to_string()),
                children: vec![],
            },
        ],
        diagnostics: vec![],
    };

    let mir = MirLowerer::new().lower_hir_result(&hir);
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
    assert!(binding.captured_by.is_empty());
}

#[test]
fn test_assignment_into_member_expressions_marks_rhs_escape() {
    let hir = HirLoweringResult {
        root: HirNodeId::new(0),
        nodes: vec![
            HirNode {
                kind: HirNodeKind::Program,
                span: None,
                text: None,
                children: vec![HirNodeId::new(1), HirNodeId::new(5)],
            },
            HirNode {
                kind: HirNodeKind::VarDecl,
                span: None,
                text: Some("const".to_string()),
                children: vec![HirNodeId::new(2)],
            },
            HirNode {
                kind: HirNodeKind::VarDeclarator,
                span: None,
                text: Some("answer".to_string()),
                children: vec![HirNodeId::new(3), HirNodeId::new(4)],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("answer".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::Literal,
                span: None,
                text: Some("1".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::AssignmentExpr,
                span: None,
                text: Some("=".to_string()),
                children: vec![HirNodeId::new(6), HirNodeId::new(8)],
            },
            HirNode {
                kind: HirNodeKind::MemberExpr,
                span: None,
                text: Some("value".to_string()),
                children: vec![HirNodeId::new(7)],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("box".to_string()),
                children: vec![],
            },
            HirNode {
                kind: HirNodeKind::Ident,
                span: None,
                text: Some("answer".to_string()),
                children: vec![],
            },
        ],
        diagnostics: vec![],
    };

    let mir = MirLowerer::new().lower_hir_result(&hir);
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
    assert!(binding.captured_by.is_empty());
}

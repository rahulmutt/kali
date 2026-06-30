use super::*;

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
        function_flavors: Vec::new(),
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
        function_flavors: Vec::new(),
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
        function_flavors: Vec::new(),
        diagnostics: vec![],
    };

    let mir = MirLowerer::new().lower_hir_result(&hir);
    let module = mir.module_scope().expect("module scope");
    let binding = module.binding("answer").expect("answer binding");

    assert_eq!(binding.ownership, OwnershipClass::OwnedHeap);
    assert!(binding.escapes);
    assert!(binding.captured_by.is_empty());
}

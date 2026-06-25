use crate::*;

#[test]
fn test_hir_validation_rejects_out_of_bounds_children() {
    let hir = LoweringResult {
        root: HirNodeId::new(0),
        nodes: vec![HirNode {
            kind: HirNodeKind::Program,
            span: None,
            text: None,
            children: vec![HirNodeId::new(1)],
        }],
        function_flavors: Vec::new(),
        diagnostics: Vec::new(),
    };

    let error = hir
        .validate()
        .expect_err("invalid HIR should fail validation");
    assert!(error.contains("HIR"), "error: {error}");
    assert!(error.contains("child node id 1"), "error: {error}");
}

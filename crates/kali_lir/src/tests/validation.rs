use super::*;

#[test]
fn test_lir_validation_rejects_out_of_bounds_children() {
    let lir = LirProgram {
        root: LirNodeId::new(0),
        nodes: vec![LirNode {
            kind: LirNodeKind::Program,
            text: None,
            children: vec![LirNodeId::new(1)],
            function_flavor: None,
        }],
    };

    let error = lir
        .validate()
        .expect_err("invalid LIR should fail validation");
    assert!(error.contains("LIR"), "error: {error}");
    assert!(error.contains("child node id 1"), "error: {error}");
}

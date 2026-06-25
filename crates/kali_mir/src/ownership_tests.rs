use crate::test_support::*;
use crate::*;

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
fn test_representation_fingerprints_distinguish_ownership_classes() {
    let base_layout = LayoutDescriptor::scalar("number");
    let stack_binding = MirBinding {
        name: "value".to_string(),
        kind: MirBindingKind::Local,
        ownership: OwnershipClass::Stack,
        layout: base_layout.clone(),
        escapes: false,
        captured_by: Vec::new(),
    };
    let shared_binding = MirBinding {
        name: "value".to_string(),
        kind: MirBindingKind::Local,
        ownership: OwnershipClass::SharedHeap,
        layout: base_layout,
        escapes: true,
        captured_by: vec!["inner".to_string()],
    };

    assert_eq!(
        stack_binding.layout_fingerprint(),
        shared_binding.layout_fingerprint()
    );
    assert_ne!(
        stack_binding.representation_fingerprint(),
        shared_binding.representation_fingerprint()
    );
    assert_eq!(
        stack_binding.representation_fingerprint(),
        "ownership=stack;layout=Scalar(number)"
    );
    assert_eq!(
        shared_binding.representation_fingerprint(),
        "ownership=shared-heap;layout=Scalar(number)"
    );
}

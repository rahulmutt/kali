use super::*;

#[test]
fn normalize_runtime_profiles_is_shared_between_callers() {
    assert_eq!(
        normalize_runtime_profiles(vec![
            " wasm-threads ".to_string(),
            "alpha".to_string(),
            "wasm-threads".to_string(),
            "alpha".to_string(),
        ]),
        vec!["alpha".to_string(), "wasm-threads".to_string()]
    );
}

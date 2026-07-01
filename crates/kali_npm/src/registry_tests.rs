use crate::test_support::*;
use crate::*;
use std::sync::atomic::Ordering;

#[test]
fn requested_version_ranges_select_highest_matching_release() {
    let mut versions = serde_json::Map::new();
    versions.insert("1.0.0".to_string(), serde_json::Value::Null);
    versions.insert("1.2.0".to_string(), serde_json::Value::Null);
    versions.insert("2.0.0".to_string(), serde_json::Value::Null);

    assert_eq!(
        select_registry_version("lodash", &versions, Some("^1.0.0")).unwrap(),
        "1.2.0"
    );
    assert_eq!(
        select_registry_version("lodash", &versions, None).unwrap(),
        "2.0.0"
    );
}

#[test]
fn registry_metadata_is_cached_within_a_process() {
    let metadata = r#"{
  "versions": {
    "1.0.0": {
      "dist": {
        "tarball": "https://example.com/lodash-1.0.0.tgz",
        "integrity": "sha512-demo"
      }
    },
    "1.2.0": {
      "dist": {
        "tarball": "https://example.com/lodash-1.2.0.tgz",
        "integrity": "sha512-demo"
      }
    }
  }
}"#;
    let (metadata_url, hits, stop, handle) = start_metadata_server(metadata);

    let resolved_first =
        resolve_npm_like_package("npm", "lodash", "lodash", &metadata_url, Some("^1.0.0")).unwrap();
    let resolved_second =
        resolve_npm_like_package("npm", "lodash", "lodash", &metadata_url, Some("^1.0.0")).unwrap();

    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap();

    assert_eq!(resolved_first.version, "1.2.0");
    assert_eq!(resolved_second.version, "1.2.0");
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn audit_package_version_metadata_rejects_native_exports_entrypoints() {
    let version_meta = serde_json::json!({
        "exports": {
            "import": "index.js",
            "node": "native.node"
        }
    });

    let findings = audit_package_version_metadata(
        "npm",
        "native-exports",
        "1.0.0",
        version_meta.as_object().unwrap(),
    );

    assert!(findings.iter().any(|diagnostic| {
        diagnostic.code == Some(e6::INCOMPATIBLE_PACKAGE as u32)
            && diagnostic.message.contains(
                "native addon exports target and falls outside the pure JS/TS package contract",
            )
    }));
}

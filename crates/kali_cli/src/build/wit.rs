//! WIT emission + browser-bundle sourcemaps.

use std::path::Path;

use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LibraryExport {
    pub name: String,
    pub signature: String,
}

pub fn browser_bundle_source_map(
    source_path: &Path,
    js_path: &Path,
    source_contents: &str,
    exports: &[LibraryExport],
) -> String {
    let source_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("input.ts")
        .to_string();
    let js_name = js_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("bundle.js")
        .to_string();
    let names: Vec<String> = exports.iter().map(|export| export.name.clone()).collect();
    json!({
        "version": 3,
        "file": js_name,
        "sourceRoot": "",
        "sources": [source_name],
        "sourcesContent": [source_contents],
        "names": names,
        "mappings": "",
    })
    .to_string()
}

pub fn library_wit_for(module_name: &str, exports: &[LibraryExport]) -> String {
    let mut wit = String::from("package kali:embed;\n\nworld library {\n");
    wit.push_str(&format!("  // module: {}\n", module_name));
    for export in exports {
        wit.push_str(&format!(
            "  // signature: {}\n  export {}: func();\n",
            export.signature,
            sanitize_wit_identifier(&export.name)
        ));
    }
    wit.push_str("}\n");
    wit
}

fn sanitize_wit_identifier(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        let keep = ch.is_ascii_alphanumeric() || ch == '_';
        if index == 0 && ch.is_ascii_digit() {
            out.push('_');
            out.push(ch);
        } else if keep {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

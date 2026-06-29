use super::*;

#[test]
fn test_rejects_late_network_members_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx network', () => {{ {} }});\n",
            late_network_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_network_rejection(&stderr);
}

#[test]
fn test_rejects_late_network_members_in_tsx_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx network', () => {{ {} }});\n",
            late_network_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_network_rejection_json(errors);
}

#[test]
fn test_rejects_late_browser_compatibility_forms_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx compatibility', () => {{ {} }});\n",
            late_browser_tsx_compatibility_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_tsx_compatibility_rejection(&stderr);
}

#[test]
fn test_rejects_late_browser_compatibility_forms_in_tsx_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx compatibility', () => {{ {} }});\n",
            late_browser_tsx_compatibility_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_tsx_compatibility_rejection_json(errors);
}

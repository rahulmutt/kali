use super::*;

#[test]
fn build_rejects_late_process_control_members_in_browser_bundle_tsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.tsx");
        fs::write(&source_path, late_process_control_source()).expect("write source");

        let mut cli = Command::new(kali_bin());
        cli.current_dir(dir.path());
        if output_json {
            cli.arg("--output").arg("json");
        }
        let output = cli
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "build");
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert_browser_late_process_control_rejection_json(errors);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_late_process_control_rejection(&stderr);
        }
    }
}

#[test]
fn build_supports_nullish_coalescing_in_browser_bundle_tsx_input() {
    for output_json in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.tsx");
        fs::write(&source_path, nullish_coalescing_source()).expect("write source");

        let mut cli = Command::new(kali_bin());
        cli.current_dir(dir.path());
        if output_json {
            cli.arg("--output").arg("json");
        }
        let output = cli
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success());
        assert_eq!(output.status.code(), Some(0));
        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], "build");
            assert_eq!(json["success"], true);
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        }
    }
}

use super::*;
use clap::{CommandFactory, Parser};
use tempfile::tempdir;

#[test]
fn discovers_source_files_and_declaration_files() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).unwrap();
    fs::write(dir.path().join("main.ts"), "const main = 1;").unwrap();
    fs::write(dir.path().join("types.d.ts"), "declare const x: number;").unwrap();
    fs::create_dir_all(dir.path().join(".hidden")).unwrap();
    fs::write(
        dir.path().join(".hidden").join("skip.ts"),
        "const skip = 1;",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("child")).unwrap();
    fs::write(
        dir.path().join("child").join("kali.json"),
        r#"{"schemaVersion":1}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("child").join("nested.ts"),
        "const nested = 1;",
    )
    .unwrap();

    let files = discover_source_files(dir.path());
    assert!(files.contains(&dir.path().join("main.ts")));
    assert!(files.contains(&dir.path().join("types.d.ts")));
    assert!(!files.contains(&dir.path().join(".hidden").join("skip.ts")));
    assert!(!files.contains(&dir.path().join("child").join("nested.ts")));
}

#[test]
fn discover_source_files_respects_kali_json_exclude() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("kali.json"),
        r#"{"schemaVersion":1,"exclude":["dist/**"]}"#,
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("dist")).unwrap();
    fs::write(dir.path().join("dist").join("bundle.ts"), "const x = 1;").unwrap();

    let files = discover_source_files(dir.path());
    assert!(!files.contains(&dir.path().join("dist").join("bundle.ts")));
}

#[test]
fn build_command_parses_max_specializations_override() {
    let args = Args::parse_from(["kali", "build", "--max-specializations", "32", "main.ts"]);
    match args.command {
        Some(Commands::Build {
            max_specializations,
            ..
        }) => {
            assert_eq!(max_specializations, Some(32));
        }
        other => panic!("expected build command, got {other:?}"),
    }
}

#[test]
fn doctor_command_parses_without_arguments() {
    let args = Args::parse_from(["kali", "doctor"]);
    match args.command {
        Some(Commands::Doctor) => {}
        other => panic!("expected doctor command, got {other:?}"),
    }
}

#[test]
fn package_audit_command_parses_preview_flag() {
    let args = Args::parse_from(["kali", "package-audit", "--preview", "lodash"]);
    match args.command {
        Some(Commands::PackageAudit {
            target,
            preview,
            api,
            compat,
            wasm_threads,
            sandbox,
        }) => {
            assert_eq!(target, vec![String::from("lodash")]);
            assert!(preview);
            assert!(api.is_none());
            assert!(compat.is_empty());
            assert!(!wasm_threads);
            assert!(sandbox.is_none());
        }
        other => panic!("expected package-audit command, got {other:?}"),
    }
}

#[test]
fn package_audit_preview_flag_stays_hidden_from_help() {
    let mut command = Args::command();
    let help = command
        .find_subcommand_mut("package-audit")
        .expect("package-audit subcommand")
        .render_long_help()
        .to_string();

    assert!(!help.contains("--preview"), "help: {help}");
    assert!(
        help.contains("Registry package target to audit"),
        "help: {help}"
    );
}

#[test]
fn readme_cli_examples_remain_parseable() {
    let cargo_manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let readme = std::fs::read_to_string(repo_root.join("README.md")).expect("read README");

    let mut in_use_cli_section = false;
    let mut in_code_block = false;
    let mut examples = Vec::new();

    for line in readme.lines() {
        let trimmed = line.trim_start();
        if trimmed == "## Use the CLI" {
            in_use_cli_section = true;
            continue;
        }
        if in_use_cli_section && trimmed.starts_with("## ") {
            break;
        }
        if !in_use_cli_section {
            continue;
        }
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block && trimmed.starts_with("kali ") {
            let command = trimmed
                .split_once(" #")
                .map_or(trimmed, |(command, _)| command)
                .trim_end();
            examples.push(command.to_string());
        }
    }

    assert!(!examples.is_empty(), "expected README CLI examples");

    for example in examples {
        let normalized = example
            .replace("[-- args...]", "-- guest-arg")
            .replace("[files...]", "main.ts")
            .replace("<file>", "main.ts")
            .replace("<package>", "lodash");
        let argv: Vec<_> = normalized.split_whitespace().collect();
        Args::parse_from(argv);
    }
}

#[test]
fn run_command_splits_guest_args_after_double_dash() {
    let args = Args::parse_from(["kali", "run", "--api", "node", "main.ts", "--", "1.2.3"]);
    match args.command {
        Some(Commands::Run {
            file, guest_args, ..
        }) => {
            assert_eq!(file, "main.ts");
            assert_eq!(guest_args, vec![String::from("1.2.3")]);
        }
        other => panic!("expected run command, got {other:?}"),
    }
}

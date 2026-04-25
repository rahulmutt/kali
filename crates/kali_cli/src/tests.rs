use super::*;
use clap::Parser;
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

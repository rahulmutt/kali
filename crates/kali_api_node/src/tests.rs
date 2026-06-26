use super::*;
use tempfile::tempdir;

#[test]
fn fs_helpers_round_trip_files_and_directories() {
    let dir = tempdir().expect("tempdir");
    let fs = NodeFs::new(dir.path());

    fs.mkdir("nested", false).expect("mkdir");
    fs.write_text_file("nested/alpha.txt", "alpha")
        .expect("write text");
    fs.write_file("nested/beta.bin", [0, 1, 2])
        .expect("write file");
    fs.rename("nested/alpha.txt", "nested/renamed.txt")
        .expect("rename file");

    assert_eq!(
        fs.read_file("nested/beta.bin").expect("read file"),
        vec![0, 1, 2]
    );
    assert_eq!(
        fs.read_text_file("nested/renamed.txt").expect("read text"),
        "alpha"
    );
    assert_eq!(
        fs.readdir("nested").expect("readdir"),
        vec!["beta.bin".to_string(), "renamed.txt".to_string()]
    );

    let stat = fs.stat("nested/renamed.txt").expect("stat");
    assert!(stat.is_file());
    assert!(!stat.is_dir());
    assert!(!stat.is_symlink());
    assert_eq!(stat.len(), 5);

    let lstat = fs.lstat("nested/renamed.txt").expect("lstat");
    assert!(lstat.is_file());
    assert!(!lstat.is_dir());
    assert!(!lstat.is_symlink());

    fs.remove("nested/beta.bin", false).expect("remove file");
    fs.remove("nested", true).expect("remove dir");
    assert!(!fs.exists("nested"));
}

#[test]
fn fs_promises_helpers_match_sync_helpers() {
    let dir = tempdir().expect("tempdir");
    let fs = NodeFsPromises::new(dir.path());

    fs.mkdir("nested", false).expect("mkdir");
    fs.write_text_file("nested/alpha.txt", "alpha")
        .expect("write text");
    fs.write_file("nested/beta.bin", [0, 1, 2])
        .expect("write file");
    fs.rename("nested/alpha.txt", "nested/renamed.txt")
        .expect("rename file");

    assert_eq!(
        fs.read_file("nested/beta.bin").expect("read file"),
        vec![0, 1, 2]
    );
    assert_eq!(
        fs.read_text_file("nested/renamed.txt").expect("read text"),
        "alpha"
    );
    assert_eq!(
        fs.readdir("nested").expect("readdir"),
        vec!["beta.bin".to_string(), "renamed.txt".to_string()]
    );

    let stat = fs.stat("nested/renamed.txt").expect("stat");
    assert!(stat.is_file());
    assert!(!stat.is_dir());
    assert!(!stat.is_symlink());
    assert_eq!(stat.len(), 5);

    let lstat = fs.lstat("nested/renamed.txt").expect("lstat");
    assert!(lstat.is_file());
    assert!(!lstat.is_dir());
    assert!(!lstat.is_symlink());

    fs.remove("nested/beta.bin", false).expect("remove file");
    fs.remove("nested", true).expect("remove dir");
    assert!(!fs.exists("nested"));
}

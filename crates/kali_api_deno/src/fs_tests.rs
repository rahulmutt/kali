use crate::*;
use tempfile::tempdir;

#[test]
fn filesystem_round_trips_files_and_metadata() {
    let dir = tempdir().expect("tempdir");
    let fs = DenoFs::new(dir.path());

    fs.mkdir("nested", false).expect("mkdir");
    fs.write_text_file("nested/alpha.txt", "alpha")
        .expect("write text");
    fs.write_file("nested/beta.bin", [0, 1, 2])
        .expect("write file");

    let mut created = fs.create("nested/gamma.txt").expect("create file");
    created.write_all("gamma").expect("write created file");
    created.flush().expect("flush created file");

    let mut opened = fs.open("nested/gamma.txt").expect("open file");
    assert_eq!(opened.read_to_string().expect("read opened file"), "gamma");
    assert_eq!(opened.metadata().expect("opened metadata").len(), 5);

    fs.rename("nested/gamma.txt", "nested/delta.txt")
        .expect("rename file");

    assert_eq!(
        fs.read_text_file("nested/alpha.txt").expect("read text"),
        "alpha"
    );
    assert_eq!(
        fs.read_file("nested/beta.bin").expect("read file"),
        vec![0, 1, 2]
    );
    assert_eq!(
        fs.readdir("nested").expect("readdir"),
        vec![
            String::from("alpha.txt"),
            String::from("beta.bin"),
            String::from("delta.txt"),
        ]
    );

    let stat = fs.stat("nested/alpha.txt").expect("stat");
    assert!(stat.is_file());
    assert!(!stat.is_dir());
    assert!(!stat.is_symlink());
    assert_eq!(stat.len(), 5);

    let lstat = fs.lstat("nested/delta.txt").expect("lstat");
    assert!(lstat.is_file());
    assert!(!lstat.is_dir());
    assert!(!lstat.is_symlink());

    fs.remove("nested/beta.bin", false).expect("remove file");
    fs.remove("nested", true).expect("remove dir");
    assert!(!fs.exists("nested"));
}

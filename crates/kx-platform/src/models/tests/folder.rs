//! The models folder on a real disk: made with only its topmost new folder guarded, scanned A to Z without hidden
//! entries, junctions or names that are not UTF-8, and files removed, a missing one with no error.

use super::*;
use std::fs;

/// A fresh folder for test `name` under the system's temporary folder, gone when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kx-models-{name}-{}", std::process::id()));
        drop(fs::remove_dir_all(&dir));
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

#[test]
fn make_builds_the_folders_and_guards_only_the_topmost_it_made() {
    let base = Scratch::new("make");
    let disk = Arc::new(roomy(need()));
    let models = WorkerModels::new(
        Script::<FetchFrames>::default(),
        Arc::new(Hands::default()),
        disk.clone(),
    );
    let deep = base.0.join("a").join("b");
    models.make(&deep).unwrap();
    assert!(deep.is_dir());
    let side = base.0.join("a").join("c");
    models.make(&side).unwrap();
    models.make(&side).unwrap();
    let guarded = disk.guarded.lock().unwrap().clone();
    assert_eq!(
        guarded,
        [base.0.clone(), side],
        "an existing folder is left as it is"
    );
}

#[test]
fn a_guarded_folder_the_system_refuses_makes_nothing() {
    let base = Scratch::new("refused");
    let disk = FakeDisk {
        refuse: true,
        ..roomy(need())
    };
    let models = host(&Script::default(), disk);
    let made = models.make(&base.0.join("a"));
    assert_eq!(made, Err(ModelError::ReadOnly));
    assert!(!base.0.exists());
}

#[test]
fn a_folder_that_fails_below_the_guarded_one_takes_the_folders_made_away() {
    let base = Scratch::new("below");
    let disk = Arc::new(roomy(need()));
    let models = WorkerModels::new(
        Script::<FetchFrames>::default(),
        Arc::new(Hands::default()),
        disk.clone(),
    );
    let made = models.make(&base.0.join("a").join("bad?name"));
    assert_eq!(made, Err(ModelError::Io), "Windows refuses the name");
    assert!(!base.0.exists(), "only the empty folders it made went");
    assert_eq!(*disk.guarded.lock().unwrap(), std::slice::from_ref(&base.0));
}

#[test]
fn a_file_where_the_folder_should_be_is_refused_and_kept() {
    let base = Scratch::new("file");
    fs::create_dir_all(&base.0).unwrap();
    let file = base.0.join("models");
    fs::write(&file, [1]).unwrap();
    let models = host(&Script::default(), roomy(need()));
    assert_eq!(models.make(&file), Err(ModelError::Io));
    assert!(file.is_file());
}

#[test]
fn a_scan_lists_folders_and_model_files_a_to_z_without_hidden_ones() {
    let base = Scratch::new("scan");
    for dir in ["b", "A", "secret.gguf"] {
        fs::create_dir_all(base.0.join(dir)).unwrap();
    }
    for (file, size) in [
        ("x.GGUF", 3),
        ("w.bin", 2),
        ("notes.txt", 1),
        ("hid.gguf", 1),
        ("gguf", 1),
    ] {
        fs::write(base.0.join(file), vec![0; size]).unwrap();
    }
    let disk = FakeDisk {
        hidden: vec!["hid.gguf"],
        ..roomy(need())
    };
    let models = host(&Script::default(), disk);
    let exts = ["gguf".to_owned(), "bin".to_owned()];
    let found = models.scan(&base.0, &exts).unwrap();
    let folders: Vec<&str> = found.folders.iter().map(|f| f.expose().as_str()).collect();
    assert_eq!(folders, ["A", "b", "secret.gguf"]);
    let files: Vec<(&str, u64)> = found
        .files
        .iter()
        .map(|f| (f.name.expose().as_str(), f.bytes))
        .collect();
    assert_eq!(files, [("w.bin", 2), ("x.GGUF", 3)]);
    let gone = models.scan(&base.0.join("gone"), &exts);
    assert_eq!(gone, Err(ModelError::Missing));
}

#[test]
fn a_scan_skips_a_junction_and_a_name_that_is_not_utf8() {
    use std::os::windows::ffi::OsStringExt;
    let base = Scratch::new("odd");
    let (inside, target) = (base.0.join("in"), base.0.join("target"));
    fs::create_dir_all(&inside).unwrap();
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("t.gguf"), [1]).unwrap();
    let mut odd = std::ffi::OsString::from_wide(&[UNPAIRED]);
    odd.push(".gguf");
    fs::write(inside.join(odd), [1]).unwrap();
    fs::write(inside.join("ok.gguf"), [1]).unwrap();
    let shell = PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join(SHELL);
    let link = inside.join("link");
    let made = std::process::Command::new(shell)
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        made.status.success() && link.is_dir(),
        "a junction to target"
    );
    let models = host(&Script::default(), roomy(need()));
    let found = models.scan(&inside, &["gguf".to_owned()]).unwrap();
    assert!(found.folders.is_empty(), "the junction is left out");
    let files: Vec<&str> = found
        .files
        .iter()
        .map(|f| f.name.expose().as_str())
        .collect();
    assert_eq!(files, ["ok.gguf"]);
}

/// A lone UTF-16 surrogate, which Windows takes in a file name and UTF-8 cannot spell.
const UNPAIRED: u16 = 0xD800;
/// The system's command shell, under the Windows folder.
const SHELL: &str = r"System32\cmd.exe";

#[test]
fn remove_deletes_a_file_and_one_already_gone_is_no_error() {
    let base = Scratch::new("remove");
    fs::create_dir_all(&base.0).unwrap();
    let file = base.0.join("a.gguf");
    fs::write(&file, [1]).unwrap();
    let models = host(&Script::default(), roomy(need()));
    models.remove(&file).unwrap();
    assert!(!file.exists());
    assert_eq!(models.remove(&file), Ok(()));
}

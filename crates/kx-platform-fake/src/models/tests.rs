//! The fake model files, held to the port: drives as set, folders made only on a known drive, scans A to Z by
//! extension in any case, fingerprints, removal, failures once per call and paths the port's rule refuses. Downloads
//! are in `fetch`.

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::FetchNote;
use std::path::PathBuf;
use std::thread::{self, ThreadId};

mod fetch;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);
/// The drive the tests use.
const ROOT: &str = r"D:\";
/// The models folder.
const DIR: &str = r"D:\KeyXtend\models";
/// A model file in it; no output may show its name.
const FILE: &str = r"D:\KeyXtend\models\parakeet-5521.gguf";
/// Its SHA-256 in hex.
const HEX: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
/// Its size in bytes.
const BYTES: u64 = 776_000;

fn print() -> Fingerprint {
    Fingerprint::from_hex(HEX).unwrap()
}

fn drive() -> Drive {
    Drive {
        root: Redacted::new(PathBuf::from(ROOT)),
        name: Redacted::new("D: Data".into()),
        free: 90_000_000_000,
    }
}

/// The extensions a scan asks for.
fn exts() -> Vec<String> {
    vec!["gguf".into(), "bin".into()]
}

/// A fake with the test drive and its models folder.
fn with_dir() -> FakeModelFiles {
    let fake = FakeModelFiles::new();
    fake.set_drives(vec![drive()]);
    fake.add_folder(Path::new(DIR));
    fake
}

/// The names a scan lists, folders then files.
fn names(scan: &FolderScan) -> (Vec<String>, Vec<(String, u64)>) {
    let folders = scan.folders.iter().map(|n| n.expose().clone()).collect();
    let files = scan
        .files
        .iter()
        .map(|f| (f.name.expose().clone(), f.bytes));
    (folders, files.collect())
}

#[test]
fn drives_answer_as_set_and_a_folder_is_made_only_on_a_known_drive() {
    let fake = FakeModelFiles::new();
    assert!(fake.drives().is_empty());
    assert_eq!(
        fake.make(Path::new(DIR)),
        Err(ModelError::Missing),
        "no drive yet"
    );
    fake.set_drives(vec![drive()]);
    assert_eq!(fake.drives(), [drive()]);
    assert_eq!(fake.make(Path::new(DIR)), Ok(()));
    assert!(fake.has_folder(Path::new(DIR)) && fake.has_folder(Path::new(r"D:\KeyXtend")));
    fake.add_folder(Path::new(r"E:\"));
    assert_eq!(
        fake.make(Path::new(r"e:/USB/m")),
        Ok(()),
        "a root added by hand"
    );
    assert_eq!(fake.calls(FileCall::Make), 3);
}

#[test]
fn every_path_the_ports_rule_refuses_is_a_bad_path() {
    let fake = with_dir();
    for bad in [
        "models",
        r"\models",
        r"D:\a\..\b",
        r"D:\NUL.gguf",
        r"D:\m:x",
    ] {
        let path = Path::new(bad);
        assert_eq!(fake.make(path), Err(ModelError::BadPath), "{bad}");
        assert_eq!(fake.scan(path, &exts()).err(), Some(ModelError::BadPath));
        assert_eq!(fake.fingerprint(path), Err(ModelError::BadPath));
        assert_eq!(fake.remove(path), Err(ModelError::BadPath));
    }
}

#[test]
fn a_scan_lists_subfolders_and_model_files_a_to_z_in_any_case() {
    let fake = with_dir();
    let put = |name: &str, bytes| fake.add_file(&Path::new(DIR).join(name), bytes, print());
    put("b.GGUF", 2);
    put("a.bin", 1);
    put("notes.txt", 3);
    fake.add_file(Path::new(r"D:\KeyXtend\models\sub\c.gguf"), 4, print());
    fake.add_folder(Path::new(r"D:\KeyXtend\models\Old"));
    let scan = fake
        .scan(Path::new(r"d:/keyxtend/MODELS/"), &exts())
        .unwrap();
    let (folders, files) = names(&scan);
    assert_eq!(folders, ["Old", "sub"]);
    assert_eq!(files, [("a.bin".into(), 1), ("b.GGUF".into(), 2)]);
    let gone = fake.scan(Path::new(r"D:\nowhere"), &exts());
    assert_eq!(gone.err(), Some(ModelError::Missing));
}

#[test]
fn doubled_separators_read_as_one_as_windows_reads_them() {
    let fake = with_dir();
    assert!(fake.has_folder(Path::new(r"D:\\KeyXtend//models\")));
    assert_eq!(fake.make(Path::new(r"D:\\New")), Ok(()));
    let (folders, _) = names(&fake.scan(Path::new(ROOT), &exts()).unwrap());
    assert_eq!(folders, ["KeyXtend", "New"]);
}

#[test]
fn a_fingerprint_reads_the_files_own_and_a_vanished_file_is_missing() {
    let fake = with_dir();
    fake.add_file(Path::new(FILE), BYTES, print());
    assert_eq!(fake.fingerprint(Path::new(FILE)), Ok(print()));
    fake.vanish(Path::new(FILE));
    assert_eq!(fake.fingerprint(Path::new(FILE)), Err(ModelError::Missing));
    assert!(fake.scan(Path::new(DIR), &exts()).unwrap().files.is_empty());
    assert_eq!(
        fake.fingerprint(Path::new(DIR)),
        Err(ModelError::Missing),
        "a folder"
    );
}

#[test]
fn a_vanished_folder_takes_everything_under_it() {
    let fake = with_dir();
    fake.add_file(Path::new(FILE), BYTES, print());
    fake.vanish(Path::new(r"D:\KeyXtend"));
    assert!(!fake.has_folder(Path::new(DIR)) && !fake.has_file(Path::new(FILE)));
    let gone = fake.scan(Path::new(DIR), &exts());
    assert_eq!(gone.err(), Some(ModelError::Missing));
    assert!(fake.has_folder(Path::new(ROOT)), "the drive stays");
}

#[test]
fn remove_deletes_a_file_and_a_missing_one_is_no_error() {
    let fake = with_dir();
    fake.add_file(Path::new(FILE), BYTES, print());
    assert_eq!(fake.remove(Path::new(FILE)), Ok(()));
    assert!(!fake.has_file(Path::new(FILE)));
    assert_eq!(fake.remove(Path::new(FILE)), Ok(()));
    assert_eq!(fake.calls(FileCall::Remove), 2);
}

#[test]
fn a_failure_set_for_a_call_fails_it_once_and_is_counted() {
    let fake = with_dir();
    fake.add_file(Path::new(FILE), BYTES, print());
    let cases = [
        (FileCall::Make, ModelError::ReadOnly),
        (FileCall::Scan, ModelError::Io),
        (FileCall::Fingerprint, ModelError::Io),
        (FileCall::Remove, ModelError::ReadOnly),
    ];
    for (call, error) in cases {
        fake.fail(call, error);
    }
    let dir = Path::new(DIR);
    let first = [
        fake.make(dir).err(),
        fake.scan(dir, &exts()).err(),
        fake.fingerprint(Path::new(FILE)).err(),
        fake.remove(Path::new(FILE)).err(),
    ];
    assert_eq!(first, cases.map(|(_, e)| Some(e)));
    assert!(
        fake.has_file(Path::new(FILE)),
        "a failed remove keeps the file"
    );
    assert_eq!(fake.make(dir), Ok(()));
    assert_eq!(fake.calls(FileCall::Make), 2, "refused calls count too");
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "never returns")]
fn a_failure_the_port_never_returns_is_a_test_mistake() {
    FakeModelFiles::new().fail(FileCall::Scan, ModelError::Full);
}

#[test]
fn debug_never_prints_a_path() {
    let fake = with_dir();
    fake.add_file(Path::new(FILE), BYTES, print());
    let shown = format!("{fake:?}");
    assert!(
        !shown.contains("parakeet") && !shown.contains("KeyXtend"),
        "{shown}"
    );
    assert!(shown.contains("entries"), "{shown}");
}

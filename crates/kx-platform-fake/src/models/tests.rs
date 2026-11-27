//! The fake model files, held to the port: drives as set, folders made only on a known drive, scans A to Z by
//! extension in any case, fingerprints, removal, failures once per call and paths the port's rule refuses. Downloads
//! are in `fetch`.

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::FetchNote;
use kx_test_support::paths::{native, native_path};
use std::path::PathBuf;
use std::thread::{self, ThreadId};

mod fetch;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);
/// Its SHA-256 in hex.
const HEX: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
/// Its size in bytes.
const BYTES: u64 = 776_000;

/// The drive the tests use, in this system's form.
fn root() -> PathBuf {
    native_path(r"D:\")
}

/// The models folder.
fn dir() -> PathBuf {
    native_path(r"D:\KeyXtend\models")
}

/// A model file in it; no output may show its name.
fn file() -> PathBuf {
    native_path(r"D:\KeyXtend\models\parakeet-5521.gguf")
}

fn print() -> Fingerprint {
    Fingerprint::from_hex(HEX).unwrap()
}

fn drive() -> Drive {
    Drive {
        root: Redacted::new(root()),
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
    fake.add_folder(&dir());
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
    assert_eq!(fake.make(&dir()), Err(ModelError::Missing), "no drive yet");
    fake.set_drives(vec![drive()]);
    assert_eq!(fake.drives(), [drive()]);
    assert_eq!(fake.make(&dir()), Ok(()));
    assert!(fake.has_folder(&dir()) && fake.has_folder(&native_path(r"D:\KeyXtend")));
    fake.add_folder(&native_path(r"E:\"));
    assert_eq!(
        fake.make(&native_path("e:/USB/m")),
        Ok(()),
        "a root added by hand"
    );
    assert_eq!(fake.calls(FileCall::Make), 3);
}

#[test]
fn every_path_the_ports_rule_refuses_is_a_bad_path() {
    let fake = with_dir();
    let rooted = [r"D:\a\..\b", r"D:\NUL.gguf", r"D:\m:x"].map(native);
    // `\models` names the current drive on Windows and holds a backslash elsewhere, so both styles refuse it.
    let plain = ["models".to_owned(), r"\models".to_owned()];
    for bad in plain.iter().chain(&rooted) {
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
    let put = |name: &str, bytes| fake.add_file(&dir().join(name), bytes, print());
    put("b.GGUF", 2);
    put("a.bin", 1);
    put("notes.txt", 3);
    fake.add_file(&native_path(r"D:\KeyXtend\models\sub\c.gguf"), 4, print());
    fake.add_folder(&native_path(r"D:\KeyXtend\models\Old"));
    let scan = fake
        .scan(&native_path("d:/keyxtend/MODELS/"), &exts())
        .unwrap();
    let (folders, files) = names(&scan);
    assert_eq!(folders, ["Old", "sub"]);
    assert_eq!(files, [("a.bin".into(), 1), ("b.GGUF".into(), 2)]);
    let gone = fake.scan(&native_path(r"D:\nowhere"), &exts());
    assert_eq!(gone.err(), Some(ModelError::Missing));
}

#[test]
fn doubled_separators_read_as_one_as_windows_reads_them() {
    let fake = with_dir();
    assert!(fake.has_folder(&native_path(r"D:\\KeyXtend//models\")));
    assert_eq!(fake.make(&native_path(r"D:\\New")), Ok(()));
    let (folders, _) = names(&fake.scan(&root(), &exts()).unwrap());
    assert_eq!(folders, ["KeyXtend", "New"]);
}

#[test]
fn a_fingerprint_reads_the_files_own_and_a_vanished_file_is_missing() {
    let fake = with_dir();
    fake.add_file(&file(), BYTES, print());
    assert_eq!(fake.fingerprint(&file()), Ok(print()));
    fake.vanish(&file());
    assert_eq!(fake.fingerprint(&file()), Err(ModelError::Missing));
    assert!(fake.scan(&dir(), &exts()).unwrap().files.is_empty());
    assert_eq!(
        fake.fingerprint(&dir()),
        Err(ModelError::Missing),
        "a folder"
    );
}

#[test]
fn a_vanished_folder_takes_everything_under_it() {
    let fake = with_dir();
    fake.add_file(&file(), BYTES, print());
    fake.vanish(&native_path(r"D:\KeyXtend"));
    assert!(!fake.has_folder(&dir()) && !fake.has_file(&file()));
    let gone = fake.scan(&dir(), &exts());
    assert_eq!(gone.err(), Some(ModelError::Missing));
    assert!(fake.has_folder(&root()), "the drive stays");
}

#[test]
fn remove_deletes_a_file_and_a_missing_one_is_no_error() {
    let fake = with_dir();
    fake.add_file(&file(), BYTES, print());
    assert_eq!(fake.remove(&file()), Ok(()));
    assert!(!fake.has_file(&file()));
    assert_eq!(fake.remove(&file()), Ok(()));
    assert_eq!(fake.calls(FileCall::Remove), 2);
}

#[test]
fn a_failure_set_for_a_call_fails_it_once_and_is_counted() {
    let fake = with_dir();
    fake.add_file(&file(), BYTES, print());
    let cases = [
        (FileCall::Make, ModelError::ReadOnly),
        (FileCall::Scan, ModelError::Io),
        (FileCall::Fingerprint, ModelError::Io),
        (FileCall::Remove, ModelError::ReadOnly),
    ];
    for (call, error) in cases {
        fake.fail(call, error);
    }
    let folder = dir();
    let first = [
        fake.make(&folder).err(),
        fake.scan(&folder, &exts()).err(),
        fake.fingerprint(&file()).err(),
        fake.remove(&file()).err(),
    ];
    assert_eq!(first, cases.map(|(_, e)| Some(e)));
    assert!(fake.has_file(&file()), "a failed remove keeps the file");
    assert_eq!(fake.make(&folder), Ok(()));
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
    fake.add_file(&file(), BYTES, print());
    let shown = format!("{fake:?}");
    assert!(
        !shown.contains("parakeet") && !shown.contains("KeyXtend"),
        "{shown}"
    );
    assert!(shown.contains("entries"), "{shown}");
}

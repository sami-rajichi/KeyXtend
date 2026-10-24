//! `words fetch` on files in a temp folder; the download step is a closure, so nothing is downloaded.

use std::path::{Path, PathBuf};

use super::super::WordsError;
use super::super::fetch::{self, Get};
use super::super::fixture::{self, abc_source};
use super::super::manifest::Manifest;
use crate::devtools::paths::join_rel;
use crate::test_support::TempDir;

/// Writes `bytes` to the source `rel` under `dir`.
fn write(dir: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
    let path = join_rel(dir.to_path_buf(), rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Writes `abc` to every downloaded source of `manifest` under `dir`.
fn all_abc(dir: &Path, manifest: &Manifest) {
    for rel in manifest.sources.keys() {
        write(dir, rel, b"abc");
    }
}

/// Runs `fetch` with `get`, returning its result and the lines it said.
fn run(
    manifest: &Manifest,
    dir: &Path,
    get: &mut Get<'_>,
) -> (Result<(), WordsError>, Vec<String>) {
    let mut lines = Vec::new();
    let result = fetch::run(manifest, dir, get, &mut |line| lines.push(line.to_string()));
    (result, lines)
}

/// Runs `fetch` and fails if it tries to download anything.
fn run_offline(manifest: &Manifest, dir: &Path) -> (Result<(), WordsError>, Vec<String>) {
    let mut calls = Vec::new();
    let out = run(manifest, dir, &mut |url, _, _| {
        calls.push(url.to_string());
        Err(WordsError::Download(url.to_string()))
    });
    assert!(calls.is_empty(), "nothing is downloaded: {calls:?}");
    out
}

#[test]
fn a_matching_file_passes() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-ok");
    all_abc(dir.path(), &manifest);
    let (result, lines) = run_offline(&manifest, dir.path());
    assert_eq!(result, Ok(()));
    assert_eq!(lines.len(), manifest.sources.len());
    assert!(lines.iter().all(|l| l.starts_with("ok")), "{lines:?}");
}

#[test]
fn a_changed_byte_fails_and_names_the_file() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-byte");
    all_abc(dir.path(), &manifest);
    write(dir.path(), "d/abc.txt", b"abd");
    let (result, lines) = run_offline(&manifest, dir.path());
    let Err(WordsError::Bad(bad)) = result else {
        panic!("a changed byte passed: {lines:?}");
    };
    assert_eq!(bad.len(), 1);
    assert!(bad[0].starts_with("d/abc.txt: SHA-256 "), "{bad:?}");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("BAD") && l.contains("d/abc.txt")),
        "{lines:?}"
    );
}

#[test]
fn a_wrong_size_fails_before_hashing() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-size");
    all_abc(dir.path(), &manifest);
    write(dir.path(), "d/abc.txt", b"abcd");
    let (result, _) = run_offline(&manifest, dir.path());
    let expected = vec!["d/abc.txt: 4 bytes, expected 3".to_string()];
    assert_eq!(result, Err(WordsError::Bad(expected)));
}

#[test]
fn a_missing_file_is_downloaded_to_a_part_file_then_renamed() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-get");
    all_abc(dir.path(), &manifest);
    let target = join_rel(dir.path().to_path_buf(), "d/abc.txt");
    std::fs::remove_file(&target).unwrap();
    let mut urls = Vec::new();
    let (result, lines) = run(&manifest, dir.path(), &mut |url, size, out| {
        urls.push((url.to_string(), size, out.to_path_buf()));
        std::fs::write(out, b"abc").map_err(|e| WordsError::Io(e.to_string()))
    });
    assert_eq!(result, Ok(()));
    let part = PathBuf::from(format!("{}.part", target.display()));
    assert_eq!(
        urls,
        [(
            "https://example.invalid/data/x/abc.txt".to_string(),
            3,
            part.clone()
        )]
    );
    assert_eq!(std::fs::read(&target).unwrap(), b"abc");
    assert!(!part.exists());
    assert!(
        lines.contains(&"fetched d/abc.txt".to_string()),
        "{lines:?}"
    );
}

#[test]
fn a_wrong_download_is_refused_and_removed() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-bad-get");
    all_abc(dir.path(), &manifest);
    let target = join_rel(dir.path().to_path_buf(), "d/abc.txt");
    std::fs::remove_file(&target).unwrap();
    let (result, _) = run(&manifest, dir.path(), &mut |_, _, out| {
        std::fs::write(out, b"xyz").map_err(|e| WordsError::Io(e.to_string()))
    });
    let Err(WordsError::Bad(bad)) = result else {
        panic!("a wrong download passed");
    };
    assert!(
        bad[0].starts_with("d/abc.txt: the download is wrong: SHA-256"),
        "{bad:?}"
    );
    assert!(!target.exists());
    assert!(!PathBuf::from(format!("{}.part", target.display())).exists());
}

#[test]
fn a_missing_unzipped_file_names_its_zip() {
    let unzipped = format!(
        r#""d/in.txt" = {{ zip = "d/abc.txt", sha256 = "{}", size = 3 }}"#,
        fixture::ABC_SHA256
    );
    let manifest = fixture::manifest_with(&format!("{}\n{unzipped}", abc_source())).unwrap();
    let dir = TempDir::new("words-zip");
    all_abc(dir.path(), &manifest);
    std::fs::remove_file(join_rel(dir.path().to_path_buf(), "d/in.txt")).unwrap();
    let (result, _) = run_offline(&manifest, dir.path());
    let expected = vec!["d/in.txt: missing; unzip it from d/abc.txt".to_string()];
    assert_eq!(result, Err(WordsError::Bad(expected)));
}

#[test]
fn verify_names_each_missing_or_wrong_file() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-verify");
    write(dir.path(), "d/fr.aff", b"abc");
    write(dir.path(), "d/fr.dic", b"abX");
    assert_eq!(fetch::verify(&manifest, dir.path(), &["d/fr.aff"]), Ok(()));
    let files = ["d/fr.aff", "d/fr.dic", "w/large_fr.gz", "nope"];
    let Err(WordsError::Bad(bad)) = fetch::verify(&manifest, dir.path(), &files) else {
        panic!("verify passed");
    };
    let starts = [
        "d/fr.dic: SHA-256",
        "w/large_fr.gz: missing",
        "nope: not in words.toml",
    ];
    assert_eq!(bad.len(), starts.len());
    for (line, start) in bad.iter().zip(starts) {
        assert!(line.starts_with(start), "{line}");
    }
}

#[test]
fn a_failed_download_names_the_file_and_leaves_no_part_file() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-fail-get");
    all_abc(dir.path(), &manifest);
    let target = join_rel(dir.path().to_path_buf(), "d/abc.txt");
    std::fs::remove_file(&target).unwrap();
    let (result, _) = run(&manifest, dir.path(), &mut |url, _, out| {
        std::fs::write(out, b"ab").unwrap();
        Err(WordsError::Download(format!("{url}: curl ended with 22")))
    });
    let why =
        "d/abc.txt: download failed: https://example.invalid/data/x/abc.txt: curl ended with 22";
    assert_eq!(result, Err(WordsError::Bad(vec![why.to_string()])));
    assert!(!PathBuf::from(format!("{}.part", target.display())).exists());
}

#[test]
fn a_failed_rename_names_the_file_and_leaves_no_part_file() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-rename");
    all_abc(dir.path(), &manifest);
    let target = join_rel(dir.path().to_path_buf(), "d/abc.txt");
    std::fs::remove_file(&target).unwrap();
    let (result, lines) = run(&manifest, dir.path(), &mut |_, _, out| {
        std::fs::create_dir(&target).unwrap();
        std::fs::write(out, b"abc").map_err(|e| WordsError::Io(e.to_string()))
    });
    let Err(WordsError::Bad(bad)) = result else {
        panic!("a failed rename passed: {lines:?}");
    };
    assert_eq!(bad.len(), 1);
    assert!(bad[0].starts_with("d/abc.txt: file error: "), "{bad:?}");
    assert!(!PathBuf::from(format!("{}.part", target.display())).exists());
}

#[test]
fn a_folder_where_a_file_belongs_is_bad() {
    let manifest = fixture::manifest_with(&abc_source()).unwrap();
    let dir = TempDir::new("words-folder");
    all_abc(dir.path(), &manifest);
    let target = join_rel(dir.path().to_path_buf(), "d/abc.txt");
    std::fs::remove_file(&target).unwrap();
    std::fs::create_dir(&target).unwrap();
    let (result, _) = run_offline(&manifest, dir.path());
    let expected = vec!["d/abc.txt: a folder, not a file".to_string()];
    assert_eq!(result, Err(WordsError::Bad(expected)));
}

#[test]
fn a_file_that_cannot_be_read_is_bad_and_the_others_are_still_checked() {
    // A name over every OS's 255-character limit fails with an error other than "not found".
    let long = format!("d/{}.txt", "a".repeat(300));
    let blocked = format!(
        r#""{long}" = {{ base = "test", sha256 = "{}", size = 3 }}"#,
        fixture::ABC_SHA256
    );
    let manifest = fixture::manifest_with(&format!("{}\n{blocked}", abc_source())).unwrap();
    let dir = TempDir::new("words-unreadable");
    for rel in manifest.sources.keys().filter(|r| **r != long) {
        write(dir.path(), rel, b"abc");
    }
    let (result, lines) = run_offline(&manifest, dir.path());
    let Err(WordsError::Bad(bad)) = result else {
        panic!("an unreadable file passed: {lines:?}");
    };
    assert_eq!(bad.len(), 1);
    assert!(
        bad[0].starts_with(&format!("{long}: file error: ")),
        "{bad:?}"
    );
    let ok = lines.iter().filter(|l| l.starts_with("ok")).count();
    assert_eq!(ok, manifest.sources.len() - 1, "{lines:?}");
}

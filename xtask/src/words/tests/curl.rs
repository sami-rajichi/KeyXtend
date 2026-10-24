//! The curl command: which program runs on any OS, and the arguments that keep a download safe.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::super::fetch::{curl_args, curl_program};
use super::super::fixture;
use crate::test_support::{TempDir, fake_env};

#[test]
fn without_the_windows_folder_curl_comes_from_the_path() {
    let manifest = fixture::manifest_with("").unwrap();
    let program = curl_program(&manifest.fetch, fake_env(vec![]));
    assert_eq!(program, PathBuf::from("kx-test-no-such-curl"));
}

#[test]
fn the_windows_curl_runs_only_when_it_is_there() {
    let manifest = fixture::manifest_with("").unwrap();
    let dir = TempDir::new("words-curl");
    let env = fake_env(vec![("KX_TEST_NO_SUCH_VAR", dir.path().to_path_buf())]);
    let fallback = PathBuf::from("kx-test-no-such-curl");
    assert_eq!(curl_program(&manifest.fetch, &env), fallback);
    std::fs::write(dir.path().join("curl.exe"), b"").unwrap();
    assert_eq!(
        curl_program(&manifest.fetch, &env),
        dir.path().join("curl.exe")
    );
}

#[test]
fn curl_gets_the_size_limit_the_output_and_the_url_last() {
    let manifest = fixture::manifest_with("").unwrap();
    let args = curl_args(
        &manifest.fetch,
        "https://x.invalid/a",
        3,
        Path::new("a.part"),
    );
    let expected = [
        "-q",
        "--fail",
        "--max-filesize",
        "3",
        "--output",
        "a.part",
        "--url",
        "https://x.invalid/a",
    ];
    assert_eq!(args, expected.map(OsString::from));
}

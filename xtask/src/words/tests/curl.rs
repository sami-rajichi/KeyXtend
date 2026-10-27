//! The curl command: which program runs on any OS, and the arguments that keep a download safe.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::super::fetch::{curl_args, curl_program};
use super::super::fixture;
use crate::devtools::paths::{join_rel, text};
use crate::test_support::{TempDir, fake_env};

#[test]
fn without_the_windows_folder_curl_comes_from_the_path() {
    let fetch = fixture::manifest_with("").unwrap().fetch;
    let program = curl_program(&fetch, fake_env(vec![]));
    assert_eq!(program, PathBuf::from(&fetch.fallback));
}

#[test]
fn the_windows_curl_runs_only_when_it_is_there() {
    let fetch = fixture::manifest_with("").unwrap().fetch;
    let dir = TempDir::new("words-curl");
    let root = text(dir.path());
    let env = |key: &str| (key == fetch.root_env).then(|| root.clone());
    assert_eq!(curl_program(&fetch, env), PathBuf::from(&fetch.fallback));
    let system = join_rel(dir.path().to_path_buf(), &fetch.curl);
    std::fs::write(&system, b"").unwrap();
    assert_eq!(curl_program(&fetch, env), system);
}

#[test]
fn curl_gets_the_size_limit_the_output_and_the_url_last() {
    let fetch = fixture::manifest_with("").unwrap().fetch;
    let (url, out) = ("https://x.invalid/a", "a.part");
    let args = curl_args(&fetch, url, 3, Path::new(out));
    let f = &fetch;
    let last = [
        f.size_flag.as_str(),
        "3",
        f.out_flag.as_str(),
        out,
        f.url_flag.as_str(),
        url,
    ];
    let expected = f.args.iter().map(String::as_str).chain(last);
    assert_eq!(args, expected.map(OsString::from).collect::<Vec<_>>());
}

//! F5 on a made-up worker: its `voice` module keeps from the network crate and its `fetch` module from the audio crate,
//! whatever file of theirs names it, while comments, literals, longer names and other modules pass.

use std::fs;

use super::*;
use crate::test_support::TempDir;
use crate::tidy::testkit::WorkspaceBuilder;

/// The package the tests keep apart.
const WORKER: &str = "keyxtend-worker";
/// Where the workspace holds it.
const APPS: &str = "apps";

/// One `apart` entry for the worker.
fn apart(module: &str, banned: &str) -> Apart {
    Apart {
        package: WORKER.to_string(),
        module: module.to_string(),
        banned: vec![banned.to_string()],
    }
}

/// A clean file each module holds, so neither fails closed unless a test wants it to.
const CLEAN: [(&str, &str); 2] = [
    ("src/voice/none.rs", "fn a() {}\n"),
    ("src/fetch/none.rs", "fn b() {}\n"),
];

/// F5's findings on a worker holding `files` (a path inside the package, then its text), as (place, rule).
fn bare(label: &str, files: &[(&str, &str)]) -> Vec<(String, &'static str)> {
    let dir = TempDir::new(label);
    for (rel, text) in files {
        let path = dir.path().join(APPS).join(WORKER).join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let ws = WorkspaceBuilder::new()
        .root(dir.path().to_path_buf())
        .member(WORKER, APPS, vec![])
        .apart(vec![
            apart("src/voice", "kx-platform-net"),
            apart("src/fetch", "cpal"),
        ])
        .build();
    let found = f5_apart(&ws);
    assert!(found.iter().all(|v| v.severity == Severity::Error));
    found.into_iter().map(|v| (v.place, v.rule)).collect()
}

/// [`bare`]'s findings with the [`CLEAN`] files added.
fn found(label: &str, files: &[(&str, &str)]) -> Vec<(String, &'static str)> {
    bare(label, &[&CLEAN[..], files].concat())
}

/// The place F5 gives line `line` of `rel` in the worker.
fn at(rel: &str, line: usize) -> (String, &'static str) {
    (format!("{APPS}/{WORKER}/{rel}:{line}"), F5)
}

#[test]
fn a_voice_file_naming_the_network_crate_fails() {
    let files = [("src/voice/mic.rs", "//! Mic.\nuse kx_platform_net::Http;\n")];
    assert_eq!(found("f5-voice", &files), [at("src/voice/mic.rs", 2)]);
}

#[test]
fn a_fetch_file_naming_the_audio_crate_fails() {
    let files = [("src/fetch.rs", "fn f() { let _ = cpal::default_host(); }\n")];
    assert_eq!(found("f5-fetch", &files), [at("src/fetch.rs", 1)]);
}

#[test]
fn the_modules_own_file_and_its_deeper_folders_count() {
    let files = [
        ("src/voice.rs", "use kx_platform_net as net;\n"),
        (
            "src/voice/a/b.rs",
            "\n\nfn f() { kx_platform_net::get(); }\n",
        ),
    ];
    let want = [at("src/voice.rs", 1), at("src/voice/a/b.rs", 3)];
    assert_eq!(found("f5-deep", &files), want);
}

#[test]
fn comments_literals_longer_names_and_other_modules_pass() {
    let voice = "// kx_platform_net never here\nconst S: &str = \"kx_platform_net\";\nlet my_kx_platform_net = 1;\n\
                 use cpal::Host;\n";
    let files = [
        ("src/voice.rs", voice),
        ("src/fetch.rs", "use kx_platform_net::Http;\n/* cpal */\n"),
        (
            "src/main.rs",
            "use cpal::Host;\nuse kx_platform_net::Http;\n",
        ),
        ("src/voicemail.rs", "use kx_platform_net::Http;\n"),
    ];
    assert_eq!(found("f5-pass", &files), []);
}

#[test]
fn a_missing_package_or_a_module_without_a_file_fails_closed() {
    let ws = WorkspaceBuilder::new()
        .apart(vec![apart("src/voice", "kx-platform-net")])
        .build();
    assert_eq!(f5_apart(&ws).len(), 1, "no such package");
    let files = [("src/main.rs", "fn main() {}\n")];
    let got = bare("f5-empty", &files);
    assert_eq!(got.len(), 2, "neither module has a file: {got:?}");
}

//! Tests for F3: `unsafe` in the bridge crate lives only in its bridge folder, each use under a safety comment.

use super::*;
use crate::test_support::{TempDir, tidy_config};
use crate::tidy::testkit::WorkspaceBuilder;
use proptest::prelude::*;

/// The bridge crate's folder, as the test config names it.
const CRATE: &str = "crates/kx-ui";

/// An unsafe block with its safety comment.
const DOCUMENTED: &str = "// SAFETY: the pointer is valid.\nfn f() { unsafe { g() } }\n";

/// Runs F3 on a `kx-ui` member whose files are `files` (path under the crate, text).
fn f3_with(files: &[(&str, &str)]) -> Vec<Violation> {
    let root = TempDir::new("f3");
    for (rel, text) in files {
        let path = root.path().join(CRATE).join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let ws = WorkspaceBuilder::new()
        .root(root.path().to_path_buf())
        .member("kx-ui", "crates", vec![])
        .build();
    f3_bridge_unsafe(&ws)
}

/// The places of `violations`, in order.
fn places(violations: &[Violation]) -> Vec<&str> {
    violations.iter().map(|v| v.place.as_str()).collect()
}

#[test]
fn documented_unsafe_in_the_bridges_passes() {
    let allow = "#![allow(unsafe_code, reason = \"cxx-qt\")]\n";
    let found = f3_with(&[
        ("src/bridges/mod.rs", allow),
        ("src/bridges/ui.rs", DOCUMENTED),
    ]);
    assert_eq!(found, vec![]);
}

#[test]
fn unsafe_outside_the_bridges_is_an_f3_error() {
    let found = f3_with(&[(
        "src/lib.rs",
        "#![deny(unsafe_code)]\n\nfn f() { unsafe { g() } }\n",
    )]);
    assert_eq!(places(&found), ["crates/kx-ui/src/lib.rs:3"]);
    assert_eq!((found[0].rule, found[0].severity), (F3, Severity::Error));
    assert!(
        found[0].message.contains("src/bridges"),
        "{}",
        found[0].message
    );
}

#[test]
fn an_allow_outside_the_bridges_is_an_f3_error() {
    let found = f3_with(&[
        ("src/hub.rs", "#[allow(unsafe_code)]\nmod x {}\n"),
        ("src/look.rs", "#![expect(unsafe_code)]\n"),
    ]);
    assert_eq!(found.len(), 2, "{found:?}");
}

#[test]
fn the_build_script_and_tests_are_outside_the_bridges() {
    let found = f3_with(&[
        ("build.rs", DOCUMENTED),
        ("tests/texts/main.rs", DOCUMENTED),
    ]);
    let mut got = places(&found);
    got.sort_unstable();
    assert_eq!(
        got,
        [
            "crates/kx-ui/build.rs:2",
            "crates/kx-ui/tests/texts/main.rs:2"
        ]
    );
}

#[test]
fn unsafe_in_the_bridges_without_a_safety_comment_is_an_f3_error() {
    let found = f3_with(&[(
        "src/bridges/ui.rs",
        "// A bridge.\nunsafe extern \"C++\" {}\n",
    )]);
    assert_eq!(places(&found), ["crates/kx-ui/src/bridges/ui.rs:2"]);
    assert!(found[0].message.contains("SAFETY"), "{}", found[0].message);
}

#[test]
fn a_safety_comment_may_run_over_several_lines_above_attributes() {
    let text = concat!(
        "// SAFETY: the header and type are cxx-qt-lib's own,\n",
        "// so both sides match.\n",
        "#[doc = \"x\"]\n",
        "unsafe extern \"C++\" {}\n",
    );
    assert_eq!(f3_with(&[("src/bridges/ui.rs", text)]), vec![]);
}

#[test]
fn a_safety_comment_stops_at_code() {
    let text = "// SAFETY: for the first one.\nlet a = 1;\nunsafe { g() }\n";
    let found = f3_with(&[("src/bridges/ui.rs", text)]);
    assert_eq!(places(&found), ["crates/kx-ui/src/bridges/ui.rs:3"]);
}

#[test]
fn words_in_comments_strings_and_names_are_not_code() {
    let text = concat!(
        "//! Holds no unsafe code.\n",
        "/* unsafe\n unsafe */\n",
        "const A: &str = \"unsafe { }\";\n",
        "const Q: char = '\"';\n",
        "fn not_unsafe() {}\n",
        "#![deny(unsafe_code)]\n",
    );
    assert_eq!(f3_with(&[("src/lib.rs", text)]), vec![]);
}

#[test]
fn a_string_that_spans_lines_hides_its_words() {
    let text = "const A: &str = \"one\nunsafe\";\nfn f() {}\n";
    assert_eq!(f3_with(&[("src/lib.rs", text)]), vec![]);
}

#[test]
fn only_rust_files_are_read_and_target_is_skipped() {
    let found = f3_with(&[
        ("qml/main.qml", "unsafe"),
        ("target/debug/build/out.rs", "unsafe { }"),
    ]);
    assert_eq!(found, vec![]);
}

#[test]
fn a_workspace_without_the_bridge_crate_passes() {
    let ws = WorkspaceBuilder::new()
        .member("kx-lib", "crates", vec![])
        .build();
    assert_eq!(f3_bridge_unsafe(&ws), vec![]);
}

#[test]
fn unsafe_in_another_crate_is_left_to_the_compiler() {
    let root = TempDir::new("f3-other");
    let path = root.path().join("crates/kx-lib/src/lib.rs");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "unsafe { }\n").unwrap();
    let ws = WorkspaceBuilder::new()
        .root(root.path().to_path_buf())
        .member("kx-lib", "crates", vec![])
        .build();
    assert_eq!(f3_bridge_unsafe(&ws), vec![]);
}

#[test]
fn unsafe_uses_are_found_by_whole_word_outside_literals() {
    let code = |line: &str| keywords(&Scan::default().code(line));
    assert_eq!(code("unsafe { f() }"), 1);
    assert_eq!(code("let x = r#unsafe_ish; // unsafe"), 0);
    assert_eq!(code("#[allow(unsafe_code)] unsafe fn f() {}"), 1);
    assert_eq!(code("'a: loop { let c = '\\''; unsafe {} }"), 1);
    assert_eq!(
        code("let q = '\"'; unsafe {}"),
        1,
        "a quote in a char opens no string"
    );
}

#[test]
fn an_allow_over_several_lines_or_in_a_list_is_found() {
    let multi = "#![allow(\n    clippy::x,\n    unsafe_code,\n    reason = \"cxx-qt\"\n)]\n";
    let found = f3_with(&[("src/lib.rs", multi)]);
    assert_eq!(places(&found), ["crates/kx-ui/src/lib.rs:3"]);
    let listed = concat!(
        "#[allow(dead_code, unsafe_code)]\n",
        "#[cfg_attr(test, expect(clippy::x, unsafe_code))]\n",
        "#![warn( unsafe_code )]\n",
    );
    let found = f3_with(&[("src/geom.rs", listed)]);
    let at = |n: u8| format!("crates/kx-ui/src/geom.rs:{n}");
    assert_eq!(places(&found), [at(1), at(2), at(3)]);
}

#[test]
fn deny_forbid_and_longer_names_are_no_allow() {
    let text = concat!(
        "#![deny(unsafe_code)]\n",
        "#![forbid(unsafe_code)]\n",
        "#[allow(unsafe_code_x, x_unsafe_code)]\n",
        "#[allow(dead_code)]\n",
        "fn f(unsafe_code: u8) {}\n",
        "const S: &str = \"allow(unsafe_code)\";\n",
    );
    assert_eq!(f3_with(&[("src/lib.rs", text)]), vec![]);
}

#[test]
fn a_raw_string_ends_at_its_own_quote() {
    let text = concat!(
        "const P: &str = r\"C:\\\";\n",
        "const Q: &str = r#\"say \"unsafe\" here\"#;\n",
        "const B: &[u8] = br\"\\\";\n",
        "fn f() { unsafe { g() } }\n",
    );
    let found = f3_with(&[("src/lib.rs", text)]);
    assert_eq!(places(&found), ["crates/kx-ui/src/lib.rs:4"]);
}

#[test]
fn nested_block_comments_hide_their_words() {
    let text = "/* a /* b */ unsafe */\nfn f() {}\n/* one\n unsafe */ unsafe fn g() {}\n";
    let found = f3_with(&[("src/lib.rs", text)]);
    assert_eq!(places(&found), ["crates/kx-ui/src/lib.rs:4"]);
}

/// Text rich in the characters the scan reacts to.
const TRICKY: &str = "[a-z_ '#*/{}()\\\\\"\n]{0,80}";

/// A literal or comment's inside: words and the characters the scan reacts to, without line breaks.
const INSIDE: &str = "[a-z_ '#*/{}()\\\\\"]{0,30}";

proptest! {
    #[test]
    fn the_scan_never_panics(text in any::<String>(), tricky in TRICKY) {
        let tidy = tidy_config();
        for t in [&text, &tricky] {
            let _ = findings(t, true, &tidy);
            let _ = findings(t, false, &tidy);
        }
    }

    #[test]
    fn words_in_literals_and_comments_are_never_code(s in INSIDE) {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        let mut lines = vec![
            (format!("// {s} unsafe"), 0),
            (format!("let a = \"{escaped} unsafe\"; unsafe {{}}"), 1),
        ];
        if !s.contains("\"#") {
            lines.push((format!("let b = r#\"{s} unsafe\"#; unsafe {{}}"), 1));
        }
        if !s.contains("*/") && !s.contains("/*") {
            lines.push((format!("/* {s} unsafe */ unsafe {{}}"), 1));
        }
        for (line, want) in lines {
            prop_assert_eq!(keywords(&Scan::default().code(&line)), want, "{}", line);
        }
    }
}

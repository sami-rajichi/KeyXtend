//! Unit, walker and property tests for the F1/F2 file rules.

use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;

use super::*;
use crate::test_support::{make_dir_link, tidy_config, unique_temp_dir};
use crate::tidy::testkit::WorkspaceBuilder;

// F1: file length thresholds (edge case 8). "Passes" means tidy exits 0, so 400 lines warns.

#[test]
fn file_of_300_lines_passes() {
    assert_eq!(length_severity(300, 300, 400), None);
}

#[test]
fn file_of_301_lines_is_a_warning() {
    assert_eq!(length_severity(301, 300, 400), Some(Severity::Warning));
}

#[test]
fn file_of_400_lines_is_a_warning() {
    assert_eq!(length_severity(400, 300, 400), Some(Severity::Warning));
}

#[test]
fn file_of_401_lines_is_an_error() {
    assert_eq!(length_severity(401, 300, 400), Some(Severity::Error));
}

// F1: property test (edge case 9).

proptest! {
    /// The CRLF and LF encodings of the same lines always count the same, whether or
    /// not the text ends with a trailing newline.
    #[test]
    fn crlf_and_lf_count_the_same(
        lines in prop::collection::vec("[a-zA-Z0-9 ]{0,8}", 0..12),
        trailing_newline in any::<bool>(),
    ) {
        let mut lf = lines.join("\n");
        let mut crlf = lines.join("\r\n");
        if trailing_newline {
            lf.push('\n');
            crlf.push_str("\r\n");
        }
        prop_assert_eq!(count_lines(&lf), count_lines(&crlf));
    }
}

// F1: reading real files (edge case 12).

#[test]
fn non_utf8_file_is_an_f1_error_without_panicking() {
    let root = unique_temp_dir("f1-utf8");
    fs::create_dir_all(root.join("xtask")).unwrap();
    fs::write(root.join("xtask/bad.rs"), [0x66, 0x6e, 0xff, 0xfe]).unwrap();

    let ws = WorkspaceBuilder::new().root(root.clone()).build();
    let violations = f1_file_length(&ws);

    fs::remove_dir_all(&root).ok();

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F1);
    assert_eq!(violations[0].severity, Severity::Error);
    assert!(violations[0].message.contains("UTF-8"));
}

// F1: walker errors (edge case 7: a missing scan dir is silent; edge case 12: no panics).

#[test]
fn missing_scan_dir_is_silent() {
    let root = unique_temp_dir("f1-missing-scan-dir");
    fs::create_dir_all(&root).unwrap();

    let (found, violations) = walk_source_files(&root, &tidy_config());

    fs::remove_dir_all(&root).ok();

    assert_eq!(found, Vec::<PathBuf>::new());
    assert_eq!(violations, vec![]);
}

#[test]
fn scan_dir_that_is_a_file_is_an_f1_error() {
    let root = unique_temp_dir("f1-not-a-dir");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("xtask"), "not a directory").unwrap();

    let ws = WorkspaceBuilder::new().root(root.clone()).build();
    let violations = f1_file_length(&ws);

    fs::remove_dir_all(&root).ok();

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F1);
    assert_eq!(violations[0].severity, Severity::Error);
    assert_eq!(violations[0].place, "xtask");
}

// F1: walker (nested target/, wrong extension, a long file).

#[test]
fn walker_finds_scanned_files_and_skips_target_and_wrong_extension() {
    let root = unique_temp_dir("walk");
    fs::create_dir_all(root.join("xtask/target/deep")).unwrap();
    fs::write(root.join("xtask/target/deep/ignored.rs"), "x").unwrap();
    fs::write(root.join("xtask/notes.txt"), "not rust").unwrap();
    let long_file = root.join("xtask/keep.rs");
    fs::write(&long_file, "// a line\n".repeat(20)).unwrap();

    let (found, violations) = walk_source_files(&root, &tidy_config());

    fs::remove_dir_all(&root).ok();

    assert_eq!(found, vec![long_file]);
    assert_eq!(violations, vec![]);
}

// F1: the walker never follows a folder link, which could loop forever.

#[test]
fn walker_skips_a_folder_link_that_loops_back() {
    let root = unique_temp_dir("walk-loop");
    let xtask = root.join("xtask");
    fs::create_dir_all(&xtask).unwrap();
    let file = xtask.join("a.rs");
    fs::write(&file, "x").unwrap();
    make_dir_link(&xtask.join("loop"), &xtask);

    let (found, violations) = walk_source_files(&root, &tidy_config());

    fs::remove_dir_all(&root).ok();

    assert_eq!(found, vec![file]);
    assert_eq!(violations, vec![]);
}

// F2: comment handling (edge case 10).

#[test]
fn commented_out_attr_is_an_f2_error() {
    let root = unique_temp_dir("f2-line-comment");
    fs::create_dir_all(root.join("crates/kx-foo/src")).unwrap();
    fs::write(
        root.join("crates/kx-foo/src/lib.rs"),
        "// #![forbid(unsafe_code)]\nfn x() {}\n",
    )
    .unwrap();

    let ws = WorkspaceBuilder::new()
        .root(root.clone())
        .member("kx-foo", "crates", vec![])
        .target(&["lib"], "crates/kx-foo/src/lib.rs")
        .build();
    let violations = f2_unsafe_attr(&ws);

    fs::remove_dir_all(&root).ok();

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F2);
    assert_eq!(violations[0].severity, Severity::Error);
}

#[test]
fn attr_inside_block_comment_is_an_f2_error() {
    let root = unique_temp_dir("f2-block-comment");
    fs::create_dir_all(root.join("crates/kx-foo/src")).unwrap();
    fs::write(
        root.join("crates/kx-foo/src/lib.rs"),
        "/*\n#![forbid(unsafe_code)]\n*/\nfn x() {}\n",
    )
    .unwrap();

    let ws = WorkspaceBuilder::new()
        .root(root.clone())
        .member("kx-foo", "crates", vec![])
        .target(&["lib"], "crates/kx-foo/src/lib.rs")
        .build();
    let violations = f2_unsafe_attr(&ws);

    fs::remove_dir_all(&root).ok();

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F2);
    assert_eq!(violations[0].severity, Severity::Error);
}

#[test]
fn active_attr_line_is_not_an_f2_violation() {
    let root = unique_temp_dir("f2-active");
    fs::create_dir_all(root.join("crates/kx-foo/src")).unwrap();
    fs::write(
        root.join("crates/kx-foo/src/lib.rs"),
        "#![forbid(unsafe_code)]\nfn x() {}\n",
    )
    .unwrap();

    let ws = WorkspaceBuilder::new()
        .root(root.clone())
        .member("kx-foo", "crates", vec![])
        .target(&["lib"], "crates/kx-foo/src/lib.rs")
        .build();
    let violations = f2_unsafe_attr(&ws);

    fs::remove_dir_all(&root).ok();

    assert_eq!(violations, vec![]);
}

// F2: platform crates are exempt (edge case 11).

#[test]
fn platform_crate_without_attr_is_not_an_f2_violation() {
    // The target file is never created: a platform crate must be skipped before any read.
    let ws = WorkspaceBuilder::new()
        .member("kx-platform-foo", "crates", vec![])
        .target(&["lib"], "crates/kx-platform-foo/src/lib.rs")
        .build();

    assert_eq!(f2_unsafe_attr(&ws), vec![]);
}

#[test]
fn test_only_platform_crate_without_attr_is_an_f2_error() {
    // `kx-platform-fake` is in the trimmed config's `test_only` list, so it gets no exemption.
    let root = unique_temp_dir("f2-test-only");
    let rel = "crates/kx-platform-fake/src/lib.rs";
    fs::create_dir_all(root.join("crates/kx-platform-fake/src")).unwrap();
    fs::write(root.join(rel), "fn x() {}\n").unwrap();

    let ws = WorkspaceBuilder::new()
        .root(root.clone())
        .member("kx-platform-fake", "crates", vec![])
        .target(&["lib"], rel)
        .build();
    let violations = f2_unsafe_attr(&ws);

    fs::remove_dir_all(&root).ok();

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F2);
    assert_eq!(violations[0].place, rel);
}

#[test]
fn non_root_kind_target_is_never_checked() {
    // A "test" target isn't in root_kinds, and its file doesn't exist: it must be skipped.
    let ws = WorkspaceBuilder::new()
        .member("kx-foo", "crates", vec![])
        .target(&["test"], "crates/kx-foo/tests/it.rs")
        .build();

    assert_eq!(f2_unsafe_attr(&ws), vec![]);
}

//! Tests for F4: product code starts no process itself, while tests, test modules and build scripts may.

use super::*;
use crate::test_support::{TempDir, tidy_config};
use crate::tidy::testkit::WorkspaceBuilder;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

/// A spawn through the standard library, on lines 1 and 2.
const SPAWN: &str = "use std::process::Command;\nfn f() { Command::new(\"x\"); }\n";

/// Runs F4 on a `kx-lib` member under `sub` whose files are `files` (path under the crate, text).
fn f4_in(sub: &str, files: &[(&str, &str)]) -> Vec<Violation> {
    let root = TempDir::new("f4");
    for (rel, text) in files {
        let path = root.path().join(sub).join("kx-lib").join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let ws = WorkspaceBuilder::new()
        .root(root.path().to_path_buf())
        .member("kx-lib", sub, vec![])
        .build();
    f4_no_spawn(&ws)
}

/// Runs F4 on a `kx-lib` member under `crates`.
fn f4_with(files: &[(&str, &str)]) -> Vec<Violation> {
    f4_in("crates", files)
}

/// The places of `violations`, sorted.
fn places(violations: &[Violation]) -> Vec<&str> {
    let mut got: Vec<&str> = violations.iter().map(|v| v.place.as_str()).collect();
    got.sort_unstable();
    got
}

/// `file`'s place in `kx-lib` at each of `lines`.
fn at(file: &str, lines: &[usize]) -> Vec<String> {
    lines
        .iter()
        .map(|n| format!("crates/kx-lib/{file}:{n}"))
        .collect()
}

#[test]
fn a_command_in_product_code_is_an_f4_error() {
    let found = f4_with(&[("src/lib.rs", SPAWN)]);
    assert_eq!(places(&found), at("src/lib.rs", &[1, 2]));
    assert_eq!((found[0].rule, found[0].severity), (F4, Severity::Error));
    let message = &found[0].message;
    assert!(message.contains("std::process::Command"), "{message}");
}

#[test]
fn every_way_of_naming_a_command_is_found() {
    let text = concat!(
        "fn a() { std::process::Command::new(\"x\"); }\n",
        "use std::process::{\n",
        "    Command as Run,\n",
        "    Stdio,\n",
        "};\n",
    );
    let found = f4_with(&[("src/run.rs", text)]);
    assert_eq!(places(&found), at("src/run.rs", &[1, 3]));
}

#[test]
fn a_qprocess_in_a_header_is_an_f4_error_and_qml_is_not_read() {
    let header = "#include <QProcess>\n// QProcess\nQProcess p;\n";
    let found = f4_with(&[("cpp/run.h", header), ("qml/Run.qml", "QProcess {}")]);
    assert_eq!(places(&found), at("cpp/run.h", &[1, 3]));
}

#[test]
fn a_command_of_our_own_or_in_words_is_no_spawn() {
    let text = concat!(
        "pub enum Command { Show }\n",
        "// std::process::Command\n",
        "const S: &str = \"std::process::Command\";\n",
        "fn f(c: &dyn CommandExt) {}\n",
    );
    assert_eq!(f4_with(&[("src/ui.rs", text)]), vec![]);
}

#[test]
fn integration_tests_and_test_modules_may_spawn() {
    let found = f4_with(&[
        ("tests/cli/main.rs", SPAWN),
        (
            "src/lib.rs",
            "mod worker;\nmod sub;\n#[cfg(test)]\nmod fake;\n",
        ),
        ("src/fake.rs", SPAWN),
        (
            "src/worker.rs",
            "#[cfg(all(test, windows))]\npub(crate) mod tests;\n",
        ),
        ("src/worker/tests.rs", "mod rights;\n"),
        ("src/worker/tests/rights.rs", SPAWN),
        (
            "src/sub/mod.rs",
            "#[cfg(test)]\n#[allow(dead_code)]\nmod tests;\n",
        ),
        ("src/sub/tests.rs", SPAWN),
    ]);
    assert_eq!(found, vec![]);
}

#[test]
fn an_inline_test_module_may_spawn_but_code_after_it_may_not() {
    let text = concat!(
        "#[cfg(test)]\n",
        "mod tests {\n",
        "    fn f() { let s = \"}\"; std::process::Command::new(s); }\n",
        "}\n",
        "fn g() { std::process::Command::new(\"x\"); }\n",
    );
    let found = f4_with(&[("src/win.rs", text)]);
    assert_eq!(places(&found), at("src/win.rs", &[5]));
}

#[test]
fn a_cfg_that_also_builds_outside_tests_shields_nothing() {
    let lib = concat!(
        "#[cfg(any(windows, test))]\nmod a;\n",
        "#[cfg(not(test))]\nmod b;\n",
        "#[cfg(all(not(test), windows))]\nmod c;\n",
        "#[cfg(test)]\nfn d() {}\nmod e;\n",
    );
    let found = f4_with(&[
        ("src/lib.rs", lib),
        ("src/a.rs", SPAWN),
        ("src/b.rs", SPAWN),
        ("src/c.rs", SPAWN),
        ("src/e.rs", SPAWN),
    ]);
    assert_eq!(found.len(), 8, "{found:?}");
}

#[test]
fn build_scripts_at_the_package_root_may_spawn() {
    let found = f4_with(&[
        ("build.rs", SPAWN),
        ("qtbuild.rs", SPAWN),
        ("src/build.rs", SPAWN),
    ]);
    assert_eq!(places(&found), at("src/build.rs", &[1, 2]));
}

#[test]
fn packages_outside_the_spawn_folders_are_not_read() {
    assert_eq!(f4_in("tools", &[("src/main.rs", SPAWN)]), vec![]);
}

#[test]
fn test_modules_are_read_from_tokens() {
    let code = |text: &str| text.lines().map(String::from).collect::<Vec<_>>();
    let mods = test_mods(&code(
        "#[cfg(test)] pub(super) mod a;\n#[cfg(test)]\nmod b {\n}\n",
    ));
    assert_eq!(mods.names, ["a", "b"]);
    assert_eq!(mods.spans, [RangeInclusive::new(2, 3)]);
    let open = test_mods(&code("#[cfg(test)]\nmod b {\n"));
    assert_eq!(open, Mods::default(), "an inline module that never closes");
}

/// Pieces the test-module reader reacts to.
const PIECES: [&str; 17] = [
    "#", "[", "]", "(", ")", "{", "}", ";", ",", "cfg", "test", "all", "not", "mod", "pub", "x",
    "\n",
];

proptest! {
    #[test]
    fn the_scan_never_panics(text in any::<String>(), parts in vec(select(&PIECES[..]), 0..60)) {
        let banned = tidy_config().banned_spawns;
        for t in [text, parts.join(" ")] {
            let mut scan = Scan::default();
            let code: Vec<String> = t.lines().map(|l| scan.code(l)).collect();
            let mods = test_mods(&code);
            let _ = findings(&code, &mods.spans, &banned);
        }
    }
}

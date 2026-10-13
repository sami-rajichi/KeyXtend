//! The stage settings in the root `Cargo.toml` name this app and the QML folder kx-ui pins, and list each QML module
//! the app imports with every plugin Qt loads for it, so `cargo xtask stage` ships what the uiAccess build loads.

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

/// The root manifest, from this crate.
const ROOT_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");

/// The `[workspace.metadata.stage]` section.
fn stage() -> toml::Table {
    let text = std::fs::read_to_string(ROOT_MANIFEST).unwrap();
    let root: toml::Table = text.parse().unwrap();
    let section = &root["workspace"]["metadata"]["stage"];
    section.as_table().unwrap().clone()
}

#[test]
fn the_stage_puts_qml_modules_where_the_app_pins_them() {
    assert_eq!(stage()["qml_dir"].as_str(), Some(kx_ui::QML_DIR));
}

#[test]
fn the_stage_builds_this_app() {
    assert_eq!(stage()["app"].as_str(), Some(env!("CARGO_PKG_NAME")));
}

/// QML modules that need no staged file because a library Qt Quick loads registers them (measured in P3 Task 20).
const FROM_LIBRARIES: &[&str] = &["QtQml.Models"];
/// kx-ui's shipped QML, from this crate.
const KX_UI_QML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/kx-ui/qml");
/// The keyword that starts an import line.
const IMPORT: &str = "import ";
/// The quote that starts a file or folder import, which names no module.
const QUOTE: char = '"';

/// Every module the shipped QML imports, by URI.
fn imports() -> std::collections::BTreeSet<String> {
    let mut found = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(KX_UI_QML).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        let uris = text
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix(IMPORT));
        let modules = uris.filter_map(|rest| rest.split_whitespace().next());
        found.extend(
            modules
                .filter(|uri| !uri.starts_with(QUOTE))
                .map(str::to_string),
        );
    }
    found
}

/// True when `uri` is the app's own module, which kx-ui compiles into the program.
fn compiled_in(uri: &str) -> bool {
    uri == kx_ui::QML_URI
}

#[test]
fn only_the_apps_own_module_is_compiled_in() {
    let own = imports().into_iter().filter(|uri| compiled_in(uri)).count();
    assert_eq!(own, 1, "exactly one import is the app's own module");
    let (path, _) = kx_ui::MAIN_QML.rsplit_once('/').unwrap();
    for folder in path.split('/').filter(|f| !f.is_empty()) {
        assert!(!compiled_in(folder), "{folder} is not the app's module");
    }
}

/// The file that describes a QML module.
const QMLDIR: &str = "qmldir";
/// The qmldir keyword of a plugin line.
const PLUGIN: &str = "plugin";
/// The word before `plugin` that marks a plugin Qt skips only when a library the program links registered the module.
const OPTIONAL: &str = "optional";
/// QML modules a library the program links registers, by URI and library (measured in P3 Task 20).
const BY_PROGRAM: &[(&str, &str)] = &[("QtQml", "Qt6Qml")];
/// The app's program, which cargo builds for this test.
const PROGRAM: &str = env!("CARGO_BIN_EXE_keyxtend");
/// qmake's flag and key that print Qt's QML folder.
const QML_QUERY: [&str; 2] = ["-query", "QT_INSTALL_QML"];

/// The files of the stage's `qml` list.
fn staged_qml(stage: &toml::Table) -> Vec<String> {
    let list = stage["qml"].as_array().unwrap().iter();
    list.filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

/// The file name of the library `name` on this system, as Qt names a plugin.
fn lib_file(name: &str) -> String {
    format!("{DLL_PREFIX}{name}{DLL_SUFFIX}")
}

/// The library files Qt loads for a qmldir's `text`, relative to its folder; optional ones only when the program does
/// not register the module (`linked`).
fn needed_plugins(text: &str, linked: bool) -> Vec<String> {
    let lines = text
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>());
    lines
        .filter_map(|words| {
            let words = match words.strip_prefix(&[OPTIONAL]) {
                Some(_) if linked => return None,
                Some(rest) => rest,
                None => words.as_slice(),
            };
            match words {
                [PLUGIN, name] => Some(lib_file(name)),
                [PLUGIN, name, path] => Some(format!("{path}/{}", lib_file(name))),
                _ => None,
            }
        })
        .collect()
}

#[test]
fn a_qmldir_needs_its_optional_plugins_unless_the_program_registers_the_module() {
    let text = "module M\noptional plugin a\nplugin b\nplugin c sub\nclassname P\n";
    let (a, b, c) = (
        lib_file("a"),
        lib_file("b"),
        format!("sub/{}", lib_file("c")),
    );
    assert_eq!(needed_plugins(text, false), [a, b.clone(), c.clone()]);
    assert_eq!(needed_plugins(text, true), [b, c]);
}

/// Qt's QML folder, asked of the qmake the stage settings name, as `cargo xtask stage` does.
fn qt_qml(stage: &toml::Table) -> std::path::PathBuf {
    let var = stage["qmake_env"].as_str().unwrap();
    let qmake = std::env::var(var).unwrap_or_else(|_| panic!("{var} must name Qt's qmake"));
    let out = std::process::Command::new(qmake)
        .args(QML_QUERY)
        .output()
        .unwrap();
    assert!(out.status.success(), "qmake failed: {out:?}");
    String::from_utf8(out.stdout).unwrap().trim().into()
}

#[test]
fn every_staged_qmldir_has_the_plugins_qt_loads() {
    let stage = stage();
    let (staged, qml) = (staged_qml(&stage), qt_qml(&stage));
    let folders = staged.iter().filter_map(|f| {
        let folder = f.strip_suffix(QMLDIR)?;
        (folder.is_empty() || folder.ends_with('/')).then_some((f, folder))
    });
    let mut spared = Vec::new();
    for (qmldir, folder) in folders {
        let text = std::fs::read_to_string(qml.join(qmldir)).unwrap();
        let uri = folder.trim_end_matches('/').replace('/', ".");
        let linked = BY_PROGRAM.iter().any(|(m, _)| *m == uri);
        if linked && needed_plugins(&text, true) != needed_plugins(&text, false) {
            spared.push(uri);
        }
        for plugin in needed_plugins(&text, linked) {
            let file = format!("{folder}{plugin}");
            assert!(
                staged.contains(&file),
                "{qmldir} needs {file} in the stage's qml list"
            );
        }
    }
    for (uri, _) in BY_PROGRAM {
        let used = spared.iter().any(|s| s == uri);
        assert!(
            used,
            "{uri} spares no staged plugin; drop it from BY_PROGRAM"
        );
    }
}

#[test]
fn the_program_links_each_library_that_spares_a_plugin() {
    let program = std::fs::read(PROGRAM).unwrap();
    for (uri, lib) in BY_PROGRAM {
        // An imported library's name ends in a zero byte in the program's import table.
        let name = format!("{}\0", lib_file(lib));
        let named = program.windows(name.len()).any(|w| w == name.as_bytes());
        assert!(
            named,
            "the program no longer links {lib}; drop {uri} from BY_PROGRAM"
        );
    }
}

#[test]
fn every_qml_import_has_its_qmldir_staged() {
    let stage = stage();
    let staged = staged_qml(&stage);
    let imports = imports();
    assert!(
        imports.iter().any(|uri| compiled_in(uri)),
        "the app's own module is imported"
    );
    for uri in imports
        .iter()
        .filter(|uri| !compiled_in(uri) && !FROM_LIBRARIES.contains(&uri.as_str()))
    {
        let qmldir = format!("{}/{QMLDIR}", uri.replace('.', "/"));
        assert!(
            staged.contains(&qmldir),
            "{uri} needs {qmldir} in the stage's qml list"
        );
    }
    for uri in FROM_LIBRARIES {
        assert!(
            imports.contains(*uri),
            "{uri} is no longer imported; drop it from FROM_LIBRARIES"
        );
    }
}

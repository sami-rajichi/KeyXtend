use super::*;
use proptest::prelude::*;

/// `name` as an OS string.
fn os(name: &str) -> OsString {
    OsString::from(name)
}

#[test]
fn qt_plugin_import_and_scene_graph_variables_are_qt_ones() {
    for name in [
        "QT_PLUGIN_PATH",
        "QT_QPA_PLATFORM_PLUGIN_PATH",
        "QML2_IMPORT_PATH",
        "QML_IMPORT_PATH",
        "QSG_RHI_BACKEND",
        "QV4_FORCE_INTERPRETER",
        "QT_SCALE_FACTOR",
    ] {
        assert!(is_qt(&os(name)), "{name}");
    }
}

#[test]
fn case_does_not_matter_as_windows_ignores_it() {
    for name in ["qt_plugin_path", "Qml2_Import_Path", "qsg_info"] {
        assert!(is_qt(&os(name)), "{name}");
    }
}

#[test]
fn other_variables_stay() {
    for name in [
        "PATH",
        "QMAKE",
        "QTDIR",
        "Q",
        "",
        "XQT_PLUGIN_PATH",
        "KX_QT_X",
        "SystemRoot",
    ] {
        assert!(!is_qt(&os(name)), "{name}");
    }
}

#[cfg(windows)]
#[test]
fn a_name_that_is_not_unicode_is_not_one_qt_reads() {
    use std::os::windows::ffi::OsStringExt;
    let mut units: Vec<u16> = "QT_".encode_utf16().collect();
    units.push(0xD800);
    assert!(!is_qt(&OsString::from_wide(&units)));
}

#[test]
fn qt_names_keeps_only_qt_ones_in_order() {
    let names = ["PATH", "QT_PLUGIN_PATH", "HOME", "qml2_import_path"].map(os);
    assert_eq!(qt_names(names), ["QT_PLUGIN_PATH", "qml2_import_path"]);
}

proptest! {
    #[test]
    fn a_name_is_qt_exactly_when_a_prefix_starts_it(name in "[A-Za-z0-9_]{0,12}") {
        let upper = name.to_ascii_uppercase();
        let want = QT_PREFIXES.iter().any(|p| upper.starts_with(p));
        prop_assert_eq!(is_qt(&os(&name)), want);
    }
}

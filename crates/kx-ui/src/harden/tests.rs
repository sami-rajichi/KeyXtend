use super::*;
use proptest::prelude::*;

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
        assert!(is_qt(name.as_bytes()), "{name}");
    }
}

#[test]
fn every_drawing_variable_is_a_qt_one_so_the_clear_drops_an_outside_value_first() {
    for (name, value) in DRAW_VARS {
        assert!(is_qt(name.as_bytes()) && !value.is_empty(), "{name}");
    }
}

/// What Qt 6.11.2 does alone: two frames may wait, the vblank thread runs, and the update timer waits 5 ms.
const QT_OWN: [(&str, &str); 3] = [(LATENCY_VAR, "2"), (NO_VBLANK_VAR, "0"), (IDLE_VAR, "5")];

/// The `[ui]` defaults.
fn defaults() -> UiSettings {
    UiSettings::from_defaults().unwrap()
}

#[test]
fn the_default_timing_variables_say_what_qt_does_alone_and_frames_wait_for_the_refresh() {
    let set = timing_vars(&defaults());
    let set: Vec<(&str, &str)> = set.iter().map(|(n, v)| (*n, v.as_str())).collect();
    assert_eq!(set, QT_OWN);
    assert_eq!(
        swap_interval(&defaults()),
        1,
        "each frame waits for the refresh (ADR-0026)"
    );
}

#[test]
fn every_timing_variable_is_a_qt_one_so_the_clear_drops_an_outside_value_first() {
    for (name, _) in timing_vars(&defaults()) {
        assert!(is_qt(name.as_bytes()), "{name}");
    }
}

#[test]
fn each_knob_sets_its_own_variable() {
    let s = UiSettings {
        frame_latency: 0,
        vblank_updates: false,
        update_idle_ms: 0,
        present_vsync: false,
        ..defaults()
    };
    let set = timing_vars(&s);
    let set: Vec<(&str, &str)> = set.iter().map(|(n, v)| (*n, v.as_str())).collect();
    assert_eq!(
        set,
        [(LATENCY_VAR, "0"), (NO_VBLANK_VAR, "1"), (IDLE_VAR, "0")]
    );
    assert_eq!(swap_interval(&s), 0, "no frame waits for the refresh");
}

#[test]
fn case_does_not_matter_as_windows_ignores_it() {
    for name in ["qt_plugin_path", "Qml2_Import_Path", "qsg_info"] {
        assert!(is_qt(name.as_bytes()), "{name}");
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
        assert!(!is_qt(name.as_bytes()), "{name}");
    }
}

#[test]
fn a_name_with_any_tail_after_its_prefix_is_a_qt_one() {
    // A last letter the code page could not narrow, and a UTF-8 tail.
    for name in [
        &b"QT_PLUGIN_PAT\xC4"[..],
        "QML2_IMPORT_PATĤ".as_bytes(),
        b"QSG_\xFF",
    ] {
        assert!(is_qt(name), "{name:?}");
    }
}

#[test]
fn a_prefix_that_is_not_ascii_is_not_one() {
    for name in ["ＱＴ_PLUGIN_PATH", "QṪ_PLUGIN_PATH"] {
        assert!(!is_qt(name.as_bytes()), "{name}");
    }
}

#[test]
fn qt_names_keeps_only_qt_ones_in_order() {
    let list = b"PATH\0QT_PLUGIN_PATH\0HOME\0qml2_import_path\0\0=C:\0";
    let want: [&[u8]; 2] = [b"QT_PLUGIN_PATH", b"qml2_import_path"];
    assert_eq!(qt_names(list), want);
}

proptest! {
    #[test]
    fn a_prefix_in_either_case_with_any_tail_is_qt(
        prefix in proptest::sample::select(QT_PREFIXES.to_vec()),
        lower in any::<bool>(),
        tail in proptest::collection::vec(any::<u8>(), 0..8),
    ) {
        let head = if lower { prefix.to_ascii_lowercase() } else { prefix.to_owned() };
        prop_assert!(is_qt(&[head.as_bytes(), &tail].concat()));
    }

    #[test]
    fn a_name_is_qt_exactly_when_a_prefix_starts_it(name in proptest::collection::vec(any::<u8>(), 0..12)) {
        let starts = |p: &&str| name.len() >= p.len() && name[..p.len()].eq_ignore_ascii_case(p.as_bytes());
        prop_assert_eq!(is_qt(&name), QT_PREFIXES.iter().any(starts));
    }

    #[test]
    fn qt_names_never_holds_an_end_or_a_name_that_is_not_qt(
        list in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        for name in qt_names(&list) {
            prop_assert!(is_qt(name) && !name.contains(&NAME_END));
        }
    }
}

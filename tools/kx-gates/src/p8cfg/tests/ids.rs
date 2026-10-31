//! The panel's ids and the history folder against kx-ui's QML and the app, so a renamed part breaks a test here.

use crate::config::load;

// Workspace paths from `tools/kx-gates/src/p8cfg/tests`; a moved file breaks the build here, never silently.
/// The panel's names, its gate-id group and part ids among them.
const NAMES_QML: &str = include_str!("../../../../../crates/kx-ui/qml/ClipNames.qml");
/// The row, which gives itself its id by its place.
const ROW_QML: &str = include_str!("../../../../../crates/kx-ui/qml/ClipRow.qml");
/// The list, which gives the list and the page buttons their ids.
const LIST_QML: &str = include_str!("../../../../../crates/kx-ui/qml/ClipList.qml");
/// The panel's body, which gives the search line its id.
const BOX_QML: &str = include_str!("../../../../../crates/kx-ui/qml/ClipBox.qml");
/// The footer, which gives Insert its id.
const FOOT_QML: &str = include_str!("../../../../../crates/kx-ui/qml/ClipFoot.qml");
/// The app's history folder, under its data folder, which takes kx-platform's name.
const APP_CLIP: &str = include_str!("../../../../../apps/keyxtend/src/clip.rs");

#[test]
fn the_panel_ids_are_the_ones_kx_ui_gives_its_parts() {
    let g = load().expect("loads").g20_product;
    assert!(
        NAMES_QML.contains(&format!("group: \"{}\"", g.group)),
        "group"
    );
    let named = |key: &str, id: &str| NAMES_QML.contains(&format!("{key}: \"{id}\""));
    let ids = [
        ("list", &g.list),
        ("row", &g.row),
        ("search", &g.search_line),
        ("insert", &g.insert),
        ("pageOn", &g.page_on),
    ];
    for (key, id) in ids {
        assert!(named(key, id), "{key}: {id}");
    }
    assert!(
        ROW_QML.contains("res.uid(names.group, names.ids.row + place)"),
        "rows"
    );
    assert!(LIST_QML.contains("list.res.uid(list.names.group, list.names.ids.list)"));
    assert!(LIST_QML.contains("list.res.uid(list.names.group, list.names.ids.pageOn)"));
    assert!(
        BOX_QML.contains("box.res.uid(names.group, names.ids.search)"),
        "search"
    );
    assert!(
        FOOT_QML.contains("foot.uid(foot.names.ids.insert)"),
        "insert"
    );
    assert_eq!(g.id(&g.insert), "clip.insert");
    assert_eq!(g.row_id(4), "clip.row-4");
}

#[test]
fn the_history_folder_is_the_apps_own() {
    let g = load().expect("loads").g20_product;
    assert_eq!(g.history, kx_platform::CLIP_DIR);
    assert!(APP_CLIP.contains("pub use kx_platform::CLIP_DIR;"));
}

#[test]
fn a_marker_that_names_a_file_the_product_writes_is_refused() {
    let good = load().expect("loads");
    let taken = [
        "settings.toml",
        "Settings.TOML",
        "settings.previous.toml",
        "settings.broken.toml",
        "settings.toml.tmp",
        "logs",
        "CLIPBOARD",
    ];
    for name in taken {
        let mut c = good.clone();
        c.g20_product.marker = name.to_string();
        let e = super::check(&c).expect_err(name);
        assert!(e.contains("g20_product.marker"), "{name}: {e}");
    }
}

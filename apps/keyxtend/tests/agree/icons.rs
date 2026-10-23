//! The board's icons: each one it draws, the shortcuts layer's cells on every page among them, resolves in `icons.toml`
//! or its key shows a text label instead; every named-key icon names a real board key, and every shortcut icon a
//! shortcut on the owner's pages (moved from `kx-theme`, which cannot see the board).

use crate::{designs, pages, views};
use kx_mod_keyboard::panel::CURRENT_ICON;
use kx_mod_keyboard::view::{NEXT_ICON, SHORTCUT_ICON_PREFIX};
use kx_mod_keyboard::{BoardSpec, board as boxes};
use kx_theme::Themes;
use std::collections::BTreeSet;

/// `icons.toml` as a table of groups, such as `keys` or `lang`.
fn icons() -> toml::Table {
    toml::Table::try_from(&Themes::shipped().unwrap().icons).unwrap()
}

/// True when icon id `id`, a group and a name such as `lang.cycle`, names a file in `icons`.
fn resolves(icons: &toml::Table, id: &str) -> bool {
    let Some((group, name)) = id.split_once('.') else {
        return false;
    };
    let file = icons.get(group).and_then(|g| g.get(name));
    file.is_some_and(toml::Value::is_str)
}

#[test]
fn every_icon_the_board_and_the_layer_draw_resolves_or_its_key_has_a_label() {
    let icons = icons();
    let (mut drawn, mut texts) = (BTreeSet::new(), BTreeSet::new());
    for view in designs().into_iter().flat_map(views) {
        for k in view.keys {
            let Some(icon) = &k.icon else { continue };
            let shown = resolves(&icons, icon);
            assert!(shown || k.label.is_some(), "{icon} shows nothing");
            if shown { &mut drawn } else { &mut texts }.insert(icon.clone());
        }
    }
    let sc = |id: &str| format!("{SHORTCUT_ICON_PREFIX}{id}");
    assert!(drawn.contains(&sc("copy-all")), "a chain shows its icon");
    assert!(drawn.contains(NEXT_ICON), "the Next key's chevron");
    assert!(texts.contains(&sc("f5")), "F5 shows its text");
    assert!(resolves(&icons, CURRENT_ICON), "the list's tick");
}

#[test]
fn every_named_key_icon_names_a_board_key() {
    let spec = BoardSpec::shipped().unwrap();
    let known: BTreeSet<String> = boxes(&spec, 0.0, 1.0)
        .keys
        .into_iter()
        .map(|k| k.id)
        .collect();
    let icons = Themes::shipped().unwrap().icons;
    for id in icons.keys.keys() {
        assert!(known.contains(id), "icons.toml [keys] {id} is no board key");
    }
}

#[test]
fn every_shortcut_icon_names_a_shortcut_on_the_owners_pages() {
    let known: BTreeSet<String> = pages()
        .concat()
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    let icons = Themes::shipped().unwrap().icons;
    assert!(!icons.shortcuts.is_empty(), "the layer has icons");
    for id in icons.shortcuts.keys() {
        assert!(
            known.contains(id),
            "icons.toml [shortcuts] {id} is on no page"
        );
    }
}

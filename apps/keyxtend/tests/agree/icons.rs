//! The board's icons: each one it draws resolves in `icons.toml` or its key shows a text label instead, and every
//! named-key icon in `icons.toml` names a real board key (moved from `kx-theme`, which cannot see the board).

use crate::{board, designs};
use kx_mod_keyboard::panel::CURRENT_ICON;
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
fn every_icon_the_board_draws_resolves_or_its_key_has_a_label() {
    let icons = icons();
    let mut drawn = 0;
    for design in designs() {
        for k in board(design).keys {
            let Some(icon) = &k.icon else { continue };
            let shown = resolves(&icons, icon);
            assert!(shown || k.label.is_some(), "{icon} shows nothing");
            drawn += usize::from(shown);
        }
    }
    assert!(drawn > 0, "some keys draw an icon");
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

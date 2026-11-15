//! The bar's icons agree with the hub's ids: each bar button has its icon, and each note's icon names a hint.

use super::hints;
use kx_module_api::ports::BarButton;
use kx_theme::Themes;
use kx_ui::i18n::keys::{bar_button_id, bar_hint_id};

#[test]
fn every_bar_button_has_its_icon_and_every_note_icon_names_a_hint() {
    let themes = Themes::shipped().expect("the tokens load");
    let icons = toml::Table::try_from(&themes.icons.voice).expect("the bar's icons as a table");
    let ids = BarButton::ALL.map(bar_button_id);
    let missing: Vec<&str> = ids
        .into_iter()
        .filter(|id| !icons.contains_key(*id))
        .collect();
    assert!(missing.is_empty(), "icons.toml [voice] lacks {missing:?}");
    let hints: Vec<&str> = hints().iter().map(bar_hint_id).collect();
    let notes = icons["notes"].as_table().expect("[voice.notes]");
    let stray: Vec<&String> = notes
        .keys()
        .filter(|k| !hints.contains(&k.as_str()))
        .collect();
    assert!(stray.is_empty(), "[voice.notes] names no hint: {stray:?}");
}

//! Press shade tests against the shipped `themes.toml`: a pressed key reads unlike a hovered or resting one.

use super::*;
use crate::Themes;
use crate::contrast::luminance;
use crate::rgba::Rgba;

fn text() -> String {
    crate::THEMES.to_string()
}

fn load() -> Themes {
    Themes::shipped().expect("tokens load")
}

/// Every key colour of `p` that a hover or press lies on.
fn fills(p: &Palette) -> [Rgba; 8] {
    [
        p.key, p.key_mod, p.key_act, p.enter, p.danger, p.on, p.lock, p.rec,
    ]
}

#[test]
fn a_press_reads_darker_than_the_hover_on_every_light_key() {
    for t in &load().list {
        let p = &t.light;
        for fill in fills(p) {
            let (hover, press) = (p.hover.on(fill), p.press.on(fill));
            assert!(
                luminance(press) < luminance(hover),
                "{} light on {fill:?}",
                t.look.id
            );
        }
    }
}

#[test]
fn dark_keys_keep_the_shared_press_which_darkens_where_the_hover_lightens() {
    let all = load();
    for t in &all.list {
        let p = &t.dark;
        assert_eq!(p.press, all.common.press, "{}", t.look.id);
        for fill in fills(p) {
            let (hover, press) = (p.hover.on(fill), p.press.on(fill));
            let ok = luminance(press) < luminance(fill) && luminance(fill) < luminance(hover);
            assert!(ok, "{} dark on {fill:?}", t.look.id);
        }
    }
}

#[test]
fn a_theme_keeps_its_legend_colours_when_pressed() {
    for t in &load().list {
        assert!(t.light.press_ink.is_none() && t.dark.press_ink.is_none());
    }
}

#[test]
fn a_mode_table_without_its_own_press_takes_the_shared_one() {
    let shipped = load();
    let native = &shipped.list[0].light;
    assert_ne!(
        native.press, shipped.common.press,
        "Native light writes its own"
    );
    let mut file: toml::Table = toml::from_str(&text()).expect("themes.toml parses");
    let themes = file.get_mut("theme").and_then(toml::Value::as_array_mut);
    let light = themes.and_then(|t| t[0].get_mut("light")?.as_table_mut());
    let dropped = light.and_then(|l| l.remove("press"));
    assert!(dropped.is_some(), "the token was there");
    let gone = toml::to_string(&file).expect("the file writes back");
    let (themes, common) = parse(&gone).expect("the press token is optional");
    assert_eq!(themes[0].light.press, common.press);
}

#[test]
fn the_press_text_cannot_be_written() {
    let at = "hover = \"#0000000F\"";
    let typed = text().replacen(at, &format!("{at}\npress_ink = \"#FFFFFF\""), 1);
    let e = parse(&typed).expect_err("only high contrast sets it");
    assert!(e.contains("native.light") && e.contains("press_ink"), "{e}");
}

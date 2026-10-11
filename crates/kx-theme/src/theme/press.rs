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

/// Native light's hover line, after which a test writes a token into Native light.
const AT: &str = r##"hover = "#0000000F""##;

/// The shipped file with `line` written into Native light.
fn with(line: &str) -> String {
    text().replacen(AT, &format!("{AT}\n{line}"), 1)
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
fn a_light_press_is_the_shared_press_laid_over_its_hover() {
    let all = load();
    for t in &all.list {
        let p = &t.light;
        assert_eq!(p.press, all.common.press.stack(p.hover), "{}", t.look.id);
    }
    let native = all.list[0].light.press;
    assert_eq!(native, Rgba::parse("#0000001D").expect("as drawn"));
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
fn a_mode_table_may_write_its_own_press() {
    let (themes, _) = parse(&with(r##"press = "#00000030""##)).expect("press is a token");
    let own = Rgba::parse("#00000030").expect("its own");
    assert_eq!(themes[0].light.press, own);
}

#[test]
fn a_theme_keeps_its_legend_colours_when_pressed() {
    for t in &load().list {
        assert!(t.light.press_ink.is_none() && t.dark.press_ink.is_none());
    }
}

#[test]
fn the_press_text_cannot_be_written() {
    let e = parse(&with(r##"press_ink = "#FFFFFF""##)).expect_err("only high contrast sets it");
    assert!(e.contains("native.light") && e.contains("press_ink"), "{e}");
}

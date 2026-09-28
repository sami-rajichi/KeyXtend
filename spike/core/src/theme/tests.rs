//! Theme tests against the real themes.toml.

use super::*;

/// The spec's minimum legend contrast, used only to exercise the check.
const MIN: f32 = 4.5;
/// Pairs under the minimum exactly as the mock-up has them; the owner kept them at the look check (2026-09-28).
const KNOWN_LOW: [&str; 11] = [
    "et66.light: legend_2 on plate",
    "dolch.light: legend_2 on key",
    "dolch.light: legend_2 on plate",
    "dolch.light: enter_ink on enter",
    "dolch.light: on_ink on on",
    "dolch.light: lock_ink on lock",
    "dolch.dark: legend_2 on key",
    "dolch.dark: enter_ink on enter",
    "dolch.dark: on_ink on on",
    "dolch.dark: lock_ink on lock",
    "dolch.dark: badge_ink on badge_bg",
];

fn text() -> String {
    std::fs::read_to_string(crate::config::settings_path(FILE)).expect("themes.toml reads")
}

#[test]
fn the_three_themes_load_in_light_and_dark() {
    let all = load().expect("themes.toml loads").list;
    let ids: Vec<&str> = all.iter().map(|t| t.look.id.as_str()).collect();
    assert_eq!(ids, ["native", "et66", "dolch"], "theme bar order");
    for t in &all {
        let skirt = t.look.id == "dolch";
        for mode in [Mode::Light, Mode::Dark] {
            assert_eq!(t.palette(mode).skirt.is_some(), skirt, "{}", t.look.id);
        }
    }
    assert!(matches!(all[1].look.cap, Cap::Convex { .. }));
}

#[test]
fn a_missing_token_names_the_theme_and_the_mode() {
    let all = text();
    let first = all
        .lines()
        .find(|l| l.starts_with("plate = "))
        .expect("a plate");
    let broken = all.replacen(&format!("{first}\n"), "", 1);
    let e = parse(&broken).expect_err("a token is missing");
    assert!(e.contains("native.light") && e.contains("plate"), "{e}");
}

#[test]
fn a_misspelt_token_is_refused() {
    let typo = text().replacen("chip_bg = \"#0000000E\"", "chip_gb = \"#0000000E\"", 1);
    let e = parse(&typo).expect_err("an unknown token");
    assert!(e.contains("native.light") && e.contains("chip_gb"), "{e}");
}

#[test]
fn an_empty_or_twice_used_theme_list_is_refused() {
    let empty = parse("theme = []");
    assert!(empty.expect_err("no themes").contains("no [[theme]]"));
    let first = text()
        .split("\n[[theme]]")
        .nth(1)
        .expect("a first theme")
        .to_string();
    let twice = format!("[[theme]]{first}\n[[theme]]{first}");
    let e = parse(&twice).expect_err("the same id twice");
    assert!(e.contains("\"native\""), "{e}");
}

#[test]
fn a_stray_top_level_table_is_refused() {
    let stray = format!(
        "{}
[shape]
gap_px = 4.0
",
        text()
    );
    assert!(
        parse(&stray)
            .expect_err("a leftover table")
            .contains("shape")
    );
}

#[test]
fn colours_take_six_or_eight_hex_digits() {
    let c = Rgba::parse("#F3F3F3").expect("six digits");
    assert_eq!((c.r, c.g, c.b, c.a), (0xF3, 0xF3, 0xF3, 0xFF));
    assert_eq!(Rgba::parse("#0000001F").map(|c| c.a), Ok(0x1F));
    for bad in ["F3F3F3", "#FFF", "#GGGGGG", "#F3F3F3F", ""] {
        assert!(Rgba::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn colours_reach_qt_alpha_first() {
    let c = Rgba::parse("#0000001F").expect("colour");
    assert_eq!(
        serde_json::to_string(&c).ok().as_deref(),
        Some("\"#1F000000\"")
    );
}

#[test]
fn hover_darkens_light_keys_and_lightens_dark_ones() {
    for t in &load().expect("themes.toml loads").list {
        let (l, d) = (t.light.hover, t.dark.hover);
        assert_eq!((l.r, l.g, l.b), (0, 0, 0), "{} light", t.look.id);
        assert_eq!((d.r, d.g, d.b), (255, 255, 255), "{} dark", t.look.id);
        assert!(l.a > 0 && d.a > 0, "{} hover shows", t.look.id);
    }
}

#[test]
fn a_pressed_key_darkens_in_both_modes() {
    for t in &load().expect("themes.toml loads").list {
        for p in [&t.light, &t.dark] {
            let c = p.get("press").expect("a press token");
            assert_eq!((c.r, c.g, c.b), (0, 0, 0), "{}", t.look.id);
            assert!(c.a > 0, "{} press shows", t.look.id);
        }
    }
}

#[test]
fn contrast_follows_wcag() {
    let (black, white) = (Rgba::parse("#000000"), Rgba::parse("#FFFFFF"));
    let (black, white) = (black.expect("black"), white.expect("white"));
    assert!((contrast(black, white) - 21.0).abs() < 0.01);
    assert!(
        (contrast(white, black) - 21.0).abs() < 0.01,
        "order does not matter"
    );
    assert!((contrast(white, white) - 1.0).abs() < 0.01);
}

#[test]
fn the_accent_fills_native_keys_and_keeps_their_legends_readable() {
    let native = &load().expect("themes.toml loads").list[0];
    let blue = Rgba::parse("#005FB8").expect("blue");
    let p = with_accent(&native.light, blue, MIN);
    let fills = [p.enter, p.on, p.lock, p.led_on, p.ring, p.badge_bg];
    assert!(fills.iter().all(|&c| c == blue));
    assert_eq!(
        p.enter_ink, native.light.enter_ink,
        "white on blue is readable"
    );
    let yellow = Rgba::parse("#FFD800").expect("yellow");
    let y = with_accent(&native.light, yellow, MIN);
    assert!(
        contrast(y.enter_ink, y.enter) >= MIN,
        "the ink switches to stay readable"
    );
}

#[test]
fn only_the_known_pairs_miss_the_minimum() {
    let low = low_contrast(&load().expect("themes.toml loads").list, MIN);
    let names: Vec<&str> = low
        .iter()
        .map(|l| l.split(" is ").next().unwrap_or(l))
        .collect();
    assert_eq!(names, KNOWN_LOW, "{low:?}");
}

#[test]
fn a_stray_cap_field_is_refused() {
    let stray = text().replacen("kind = \"flat\"", "kind = \"flat\"\ntilt = 2.0", 1);
    let e = parse(&stray).expect_err("an unknown cap field");
    assert!(e.contains("tilt"), "{e}");
}

#[test]
fn an_odd_weight_or_share_is_refused() {
    let heavy = text().replacen("legend_weight = 400", "legend_weight = 950", 1);
    assert!(parse(&heavy).expect_err("weight").contains("950"));
    let glow = text().replacen("lock_glow_mix = 0.0", "lock_glow_mix = 1.5", 1);
    assert!(parse(&glow).is_err(), "a share above 1");
}

#[test]
fn a_dpad_token_that_does_not_exist_is_refused() {
    let typo = text().replacen("face = \"key_act\"", "face = \"key_akt\"", 1);
    let e = parse(&typo).expect_err("an unknown token");
    assert!(e.contains("native") && e.contains("key_akt"), "{e}");
}

#[test]
fn each_theme_resolves_its_dpad_rules() {
    let all = load().expect("themes.toml loads").list;
    let d = |i: usize| all[i].look.dpad.resolve(&all[i].light).expect("dpad");
    assert_eq!(d(0).stop, all[0].light.rec);
    assert_eq!(
        d(1).on,
        all[1].light.enter,
        "ET66 lights the arrow in yellow"
    );
    assert_eq!(d(2).face, all[2].light.key_mod, "Dolch arrows are grey");
}

#[test]
fn a_legend_too_close_to_its_key_is_named() {
    let mut all = load().expect("themes.toml loads").list;
    all[0].light.legend = all[0].light.key;
    let low = low_contrast(&all, MIN);
    assert!(
        low.iter()
            .any(|l| l.starts_with("native.light: legend on key")),
        "{low:?}"
    );
    assert!(low_contrast(&all, 1.0).is_empty(), "nothing is below 1:1");
}

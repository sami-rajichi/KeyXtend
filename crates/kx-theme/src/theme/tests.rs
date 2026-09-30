//! Theme tests against the shipped `themes.toml`.

use super::*;
use crate::Themes;
use crate::accent::with_accent;
use crate::contrast::{WCAG_AA, contrast, luminance};
use crate::look::Cap;
use crate::rgba::Rgba;

fn text() -> String {
    crate::THEMES.to_string()
}

fn load() -> Themes {
    Themes::shipped().expect("tokens load")
}

#[test]
fn the_three_themes_load_in_light_and_dark() {
    let all = load().list;
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
    let all = text();
    let mut parts = all.split("\n[[theme]]");
    let (head, first) = (
        parts.next().expect("the shared table"),
        parts.next().expect("a theme"),
    );
    let empty = parse(&format!("theme = []\n{head}"));
    assert!(empty.expect_err("no themes").contains("no [[theme]]"));
    let twice = format!("{head}\n[[theme]]{first}\n[[theme]]{first}");
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
fn hover_darkens_light_keys_and_lightens_dark_ones() {
    for t in &load().list {
        let (l, d) = (t.light.hover, t.dark.hover);
        assert_eq!((l.r, l.g, l.b), (0, 0, 0), "{} light", t.look.id);
        assert_eq!((d.r, d.g, d.b), (255, 255, 255), "{} dark", t.look.id);
        assert!(l.a > 0 && d.a > 0, "{} hover shows", t.look.id);
    }
}

#[test]
fn every_theme_shares_one_set_of_mock_up_colours() {
    let c = load().common;
    let rgb = |c: Rgba| (c.r, c.g, c.b);
    assert_eq!(rgb(c.press), (0, 0, 0), "brightness(.94)");
    assert!(c.press.a > 0 && c.snip_dim.a > 0 && c.snip_tag.a > 0);
    assert_eq!(rgb(c.snip_edge), (255, 255, 255), ".snip-rect edge");
    assert_eq!(c.rec_dot, Rgba::parse("#E5484D").expect("red"), ".rec-dot");
}

#[test]
fn a_missing_shared_colour_is_refused() {
    let gone = text().replacen("snip_tag = ", "snip_tagg = ", 1);
    assert!(parse(&gone).expect_err("a typo").contains("snip_tagg"));
}

#[test]
fn the_accent_fills_native_keys_and_keeps_their_legends_readable() {
    let native = &load().list[0];
    let blue = Rgba::parse("#005FB8").expect("blue");
    let p = with_accent(&native.light, blue, WCAG_AA);
    let fills = [p.enter, p.on, p.lock, p.led_on, p.ring, p.badge_bg];
    assert!(fills.iter().all(|&c| c == blue));
    assert_eq!(
        p.enter_ink, native.light.enter_ink,
        "white on blue is readable"
    );
    let yellow = Rgba::parse("#FFD800").expect("yellow");
    let y = with_accent(&native.light, yellow, WCAG_AA);
    assert!(
        contrast(y.enter_ink, y.enter) >= WCAG_AA,
        "the ink switches to stay readable"
    );
}

#[test]
fn sculpted_skirts_stay_darker_than_their_tops() {
    for t in load().list {
        for mode in Mode::ALL {
            let p = t.palette(mode);
            let Some(s) = &p.skirt else { continue };
            let pairs = [
                ("key", s.key, p.key),
                ("key_mod", s.key_mod, p.key_mod),
                ("enter", s.enter, p.enter),
                ("danger", s.danger, p.danger),
                ("on", s.on, p.on),
                ("rec", s.rec, p.rec),
            ];
            for (n, skirt, top) in pairs {
                let at = format!("{}.{}: {n}", t.look.id, mode.key());
                assert!(luminance(skirt) < luminance(top), "{at}");
            }
        }
    }
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
    let all = load().list;
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
fn dimmed_keys_are_side_keys_faded_over_the_plate() {
    let t = load();
    let share = t.shape.dim.share;
    for th in &t.list {
        for mode in Mode::ALL {
            let (p, at) = (th.palette(mode), format!("{}.{}", th.look.id, mode.key()));
            assert_eq!(p.dim, p.key_act.over(p.plate, share), "{at}");
            assert_eq!(p.dim_ink, p.legend_act.over(p.plate, share), "{at}");
            if let Some(s) = &p.skirt {
                assert_eq!(s.dim, s.key_mod.over(p.plate, share), "{at}");
            }
            let (dim, lit) = (
                contrast(p.dim_ink, p.dim),
                contrast(p.legend_act, p.key_act),
            );
            assert!(dim < lit, "{at}: a dimmed key reads fainter");
        }
    }
}

#[test]
fn dim_colours_come_from_the_share_and_cannot_be_written() {
    let at = "hover = \"#0000000F\"";
    let typed = text().replacen(at, &format!("{at}\ndim = \"#FFFFFF\""), 1);
    let e = parse(&typed).expect_err("dim is derived");
    assert!(e.contains("native.light") && e.contains("dim"), "{e}");
}

#[test]
fn sculpted_caps_cannot_take_the_accent() {
    let at = "so Dolch never takes the accent.\naccent = ";
    let e = parse(&text().replacen(&format!("{at}false"), &format!("{at}true"), 1));
    assert!(e.expect_err("skirts keep their colours").contains("dolch"));
}

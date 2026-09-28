//! The keyboard themes from `themes.toml`: three looks, each light and soft dark.

mod look;
mod palette;

use serde::Deserialize;

use crate::config::settings_path;
pub use look::{Cap, Dpad, DpadRefs, Gradient, Look};
pub use palette::{Palette, Rgba, Shadow, Skirt, contrast};

/// The themes file, beside `spike.toml`.
const FILE: &str = "themes.toml";

/// Light or soft dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The light palette.
    Light,
    /// The soft dark palette.
    Dark,
}

impl Mode {
    /// Both modes, light first.
    pub const ALL: [Mode; 2] = [Mode::Light, Mode::Dark];

    /// The table name of this mode in `themes.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }
}

/// One theme: its look and its two palettes.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Shape and type.
    pub look: Look,
    /// Light colours.
    pub light: Palette,
    /// Soft dark colours.
    pub dark: Palette,
}

impl Theme {
    /// The palette for `mode`.
    pub fn palette(&self, mode: Mode) -> &Palette {
        match mode {
            Mode::Light => &self.light,
            Mode::Dark => &self.dark,
        }
    }
}

/// The file as written: a list of theme tables.
#[derive(Deserialize)]
struct ThemeFile {
    theme: Vec<toml::Table>,
}

/// Loads `themes.toml` from beside the exe or the spike folder.
pub fn load() -> Result<Vec<Theme>, String> {
    let file = settings_path(FILE);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", file.display()))
}

/// The themes in `text`, in file order; an error names the theme and mode at fault.
pub fn parse(text: &str) -> Result<Vec<Theme>, String> {
    let file: ThemeFile = toml::from_str(text).map_err(|e| e.to_string())?;
    let themes = file
        .theme
        .into_iter()
        .map(theme)
        .collect::<Result<Vec<_>, _>>()?;
    if themes.is_empty() {
        return Err("no [[theme]] tables".into());
    }
    for (i, t) in themes.iter().enumerate() {
        if themes[..i].iter().any(|u| u.look.id == t.look.id) {
            return Err(format!("theme id {:?} is used twice", t.look.id));
        }
    }
    Ok(themes)
}

/// One theme table: its two mode tables, then its look.
fn theme(mut t: toml::Table) -> Result<Theme, String> {
    let id = t
        .get("id")
        .and_then(toml::Value::as_str)
        .unwrap_or("?")
        .to_string();
    let light = palette(&id, Mode::Light, t.remove(Mode::Light.key()))?;
    let dark = palette(&id, Mode::Dark, t.remove(Mode::Dark.key()))?;
    let look: Look = toml::Value::Table(t)
        .try_into()
        .map_err(|e| format!("{id}: {e}"))?;
    for p in [&light, &dark] {
        look.check(p).map_err(|e| format!("{id}: {e}"))?;
    }
    Ok(Theme { look, light, dark })
}

/// One mode table, with errors prefixed `<id>.<mode>`.
fn palette(id: &str, mode: Mode, table: Option<toml::Value>) -> Result<Palette, String> {
    let at = format!("{id}.{}", mode.key());
    let table = table.ok_or_else(|| format!("{at}: missing"))?;
    table.try_into().map_err(|e| format!("{at}: {e}"))
}

/// Whichever of `a` and `b` reads better on `bg`.
fn readable(a: Rgba, b: Rgba, bg: Rgba) -> Rgba {
    if contrast(a, bg) >= contrast(b, bg) {
        a
    } else {
        b
    }
}

/// `p` with the accent `fill` on Enter, on, locked, the LED, the ring and the badge.
/// An ink that would drop below `min` on the fill switches to the main legend if that reads better.
pub fn with_accent(p: &Palette, fill: Rgba, min: f32) -> Palette {
    let mut a = p.clone();
    for c in [
        &mut a.enter,
        &mut a.on,
        &mut a.lock,
        &mut a.led_on,
        &mut a.ring,
        &mut a.badge_bg,
    ] {
        *c = fill;
    }
    for ink in [
        &mut a.enter_ink,
        &mut a.on_ink,
        &mut a.lock_ink,
        &mut a.badge_ink,
    ] {
        if contrast(*ink, fill) < min {
            *ink = readable(*ink, p.legend, fill);
        }
    }
    a
}

/// Every legend below `min` contrast with its background, as `<id>.<mode>: <pair> is <n>:1`.
pub fn low_contrast(themes: &[Theme], min: f32) -> Vec<String> {
    let mut low = Vec::new();
    for t in themes {
        for mode in Mode::ALL {
            for (pair, ink, bg) in t.palette(mode).legend_pairs() {
                let c = contrast(ink, bg);
                if c < min {
                    low.push(format!("{}.{}: {pair} is {c:.1}:1", t.look.id, mode.key()));
                }
            }
        }
    }
    low
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spec's minimum legend contrast, used only to exercise the check.
    const MIN: f32 = 4.5;
    /// Pairs under the minimum exactly as the mock-up has them; the owner decides on them at the look check.
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
        let all = load().expect("themes.toml loads");
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
        let broken = text().replacen("plate = \"#F3F3F3\"\n", "", 1);
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
        assert!(parse("theme = []").is_err());
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
    fn colours_take_six_or_eight_hex_digits() {
        let c = Rgba::parse("#F3F3F3").expect("six digits");
        assert_eq!((c.r, c.g, c.b, c.a), (0xF3, 0xF3, 0xF3, 0xFF));
        assert_eq!(Rgba::parse("#0000001F").map(|c| c.a), Ok(0x1F));
        for bad in ["F3F3F3", "#FFF", "#GGGGGG", "#F3F3F3F", ""] {
            assert!(Rgba::parse(bad).is_err(), "{bad}");
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
        let native = &load().expect("themes.toml loads")[0];
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
        let low = low_contrast(&load().expect("themes.toml loads"), MIN);
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
        let all = load().expect("themes.toml loads");
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
        let mut all = load().expect("themes.toml loads");
        all[0].light.legend = all[0].light.key;
        let low = low_contrast(&all, MIN);
        assert!(
            low.iter()
                .any(|l| l.starts_with("native.light: legend on key")),
            "{low:?}"
        );
        assert!(low_contrast(&all, 1.0).is_empty(), "nothing is below 1:1");
    }
}

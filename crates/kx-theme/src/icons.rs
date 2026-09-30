//! `icons.toml`: the icon each part shows, as a file name without its extension.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The top bar's buttons, the resize corner and the bubble.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BarIcons {
    /// Moves the keyboard.
    pub grip: String,
    /// The light or dark switch while light.
    pub mode: String,
    /// The same switch while dark.
    pub mode_on: String,
    /// Cycles the themes.
    pub theme: String,
    /// One size smaller.
    pub smaller: String,
    /// One size larger.
    pub bigger: String,
    /// Shrinks the keyboard to the bubble.
    pub minimise: String,
    /// Hides the keyboard to the tray.
    pub close: String,
    /// The resize corner.
    pub corner: String,
    /// The bubble, which brings the keyboard back.
    pub bubble: String,
}

/// The D-pad's arrows and its Stop button.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DpadIcons {
    /// Scroll up.
    pub up: String,
    /// Scroll right.
    pub right: String,
    /// Scroll left.
    pub left: String,
    /// Scroll down.
    pub down: String,
    /// Stop scrolling.
    pub stop: String,
}

/// The language key's designs that show an icon, and the language list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LangIcons {
    /// The Cycle design's globe.
    pub cycle: String,
    /// The List design's chevron.
    pub list: String,
    /// The tick on the language list's current row.
    pub current: String,
}

/// Every icon name, by the part that shows it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Icons {
    /// Named keys by board id; a key not listed shows a text label.
    pub keys: BTreeMap<String, String>,
    /// The top bar, the resize corner and the bubble.
    pub bar: BarIcons,
    /// The D-pad.
    pub dpad: DpadIcons,
    /// The language key and its list.
    pub lang: LangIcons,
}

impl Icons {
    /// Every name with where it is used, such as `bar.theme`.
    fn named(&self) -> Vec<(String, &str)> {
        let (b, d, l) = (&self.bar, &self.dpad, &self.lang);
        let fixed = [
            ("bar.grip", &b.grip),
            ("bar.mode", &b.mode),
            ("bar.mode_on", &b.mode_on),
            ("bar.theme", &b.theme),
            ("bar.smaller", &b.smaller),
            ("bar.bigger", &b.bigger),
            ("bar.minimise", &b.minimise),
            ("bar.close", &b.close),
            ("bar.corner", &b.corner),
            ("bar.bubble", &b.bubble),
            ("dpad.up", &d.up),
            ("dpad.right", &d.right),
            ("dpad.left", &d.left),
            ("dpad.down", &d.down),
            ("dpad.stop", &d.stop),
            ("lang.cycle", &l.cycle),
            ("lang.list", &l.list),
            ("lang.current", &l.current),
        ];
        let keys = self
            .keys
            .iter()
            .map(|(k, v)| (format!("keys.{k}"), v.as_str()));
        let fixed = fixed
            .into_iter()
            .map(|(at, v)| (at.to_string(), v.as_str()));
        keys.chain(fixed).collect()
    }

    /// Every icon name, once for each place that shows it.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.named().into_iter().map(|(_, n)| n).collect()
    }
}

/// True for a plain file name: lower-case ASCII letters, digits and `-`, never empty.
fn plain(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// The checked icons in `text`; a name that is not a plain file name is refused with where it is used.
pub(crate) fn parse(text: &str) -> Result<Icons, String> {
    let icons: Icons = toml::from_str(text).map_err(|e| e.to_string())?;
    match icons.named().into_iter().find(|(_, n)| !plain(n)) {
        Some((at, n)) => Err(format!(
            "{at}: {n:?} is not lower-case letters, digits and -"
        )),
        None => Ok(icons),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text() -> String {
        crate::ICONS.to_string()
    }

    #[test]
    fn the_icons_load_with_the_theme_button_on_the_palette() {
        let i = parse(crate::ICONS).expect("icons.toml loads");
        assert_eq!(i.bar.theme, "palette", "the theme button");
        assert_eq!(
            (i.bar.mode.as_str(), i.bar.mode_on.as_str()),
            ("moon", "sun")
        );
        assert_eq!(i.keys["win"], "win-logo", "our own drawing");
        assert_eq!(i.keys["enter"], "corner-down-left", "mock-up keyInner");
        assert_eq!(
            (i.lang.cycle.as_str(), i.lang.list.as_str()),
            ("globe", "chevron-up")
        );
        assert_eq!(i.lang.current, "check", "the list's current row");
        assert_eq!(i.dpad.stop, "square");
        assert!(i.names().contains(&"palette"));
    }

    #[test]
    fn a_name_that_is_not_a_plain_file_name_is_refused() {
        for bad in ["../x", "Palette", "", "a b", "x.svg"] {
            let t = text().replacen("theme = \"palette\"", &format!("theme = {bad:?}"), 1);
            let e = parse(&t).expect_err(bad);
            assert!(e.contains("bar.theme"), "{bad}: {e}");
        }
    }

    #[test]
    fn a_misspelt_or_missing_button_is_refused() {
        let typo = text().replacen("theme = \"palette\"", "theem = \"palette\"", 1);
        assert!(parse(&typo).expect_err("a typo").contains("theem"));
    }
}

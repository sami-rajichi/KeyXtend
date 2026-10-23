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

/// A paged list's page buttons.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PageIcons {
    /// Back a page.
    pub back: String,
    /// On a page.
    pub on: String,
}

/// The notice banner's icon.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NoticeIcons {
    /// Before each notice's text.
    pub icon: String,
}

/// The shortcuts layer's own keys.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LayerIcons {
    /// The Next key, with its page dots under it.
    pub next: String,
}

/// The tray icon's drawing and its menu's items.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrayIcons {
    /// Drawn on the tray icon.
    pub glyph: String,
    /// Show keyboard.
    pub show: String,
    /// Quit.
    pub quit: String,
}

/// Every icon name, by the part that shows it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Icons {
    /// Named keys by board id; a key not listed shows a text label.
    pub keys: BTreeMap<String, String>,
    /// The shortcuts layer's keys by shortcut id; a shortcut not listed shows its name.
    pub shortcuts: BTreeMap<String, String>,
    /// The shortcuts layer's Next key.
    pub layer: LayerIcons,
    /// The top bar, the resize corner and the bubble.
    pub bar: BarIcons,
    /// The D-pad.
    pub dpad: DpadIcons,
    /// The language key and its list.
    pub lang: LangIcons,
    /// Paged lists.
    pub page: PageIcons,
    /// The notice banner.
    pub notice: NoticeIcons,
    /// The tray icon and its menu.
    pub tray: TrayIcons,
}

impl Icons {
    /// Every name with where it is used, such as `bar.theme` or `shortcuts.copy`.
    fn named(&self) -> Vec<(String, &str)> {
        let (b, d, l, t) = (&self.bar, &self.dpad, &self.lang, &self.tray);
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
            ("layer.next", &self.layer.next),
            ("page.back", &self.page.back),
            ("page.on", &self.page.on),
            ("notice.icon", &self.notice.icon),
            ("tray.glyph", &t.glyph),
            ("tray.show", &t.show),
            ("tray.quit", &t.quit),
        ];
        let fixed = fixed
            .into_iter()
            .map(|(at, v)| (at.to_string(), v.as_str()));
        let keys = by_id("keys", &self.keys);
        keys.chain(by_id("shortcuts", &self.shortcuts))
            .chain(fixed)
            .collect()
    }

    /// Every icon name, once for each place that shows it.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.named().into_iter().map(|(_, n)| n).collect()
    }
}

/// Each name in `m` by its id, with where it is used: `group.id`.
fn by_id<'a>(
    group: &'static str,
    m: &'a BTreeMap<String, String>,
) -> impl Iterator<Item = (String, &'a str)> {
    m.iter()
        .map(move |(k, v)| (format!("{group}.{k}"), v.as_str()))
}

/// True for a plain file name: lower-case ASCII letters, digits and `-`, never empty.
pub(crate) fn plain(name: &str) -> bool {
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
mod tests;

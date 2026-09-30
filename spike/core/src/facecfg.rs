//! `spike.toml [keys]`, `[bar]` and `[assets]`: the spike's text for named keys and top-bar buttons until the
//! translations come (P3 Task 15), what side keys do, and where fonts and icons are. Icons come from `icons.toml`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use kx_mod_keyboard::spec::RowItem;
use kx_mod_keyboard::{BoardSpec, KeyKind};
use kx_theme::Icons;

/// What a side key does in the spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Starts or stops voice typing.
    Mic,
    /// Starts a snip.
    Snip,
    /// Types the test password after Hello.
    Fill,
    /// Lights up or goes dark; nothing else yet.
    Toggle,
    /// Shows its `note` on the status line.
    Note,
    /// Opens the Arabic test panel (gate G7).
    Panel,
}

/// A named key's text, what screen readers call it, and what a side key does in the spike.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyLook {
    /// Text drawn when `icons.toml` gives the key no icon.
    #[serde(default)]
    pub label: Option<String>,
    /// Name for screen readers.
    pub name: String,
    /// What a side key does.
    #[serde(default)]
    pub action: Option<Action>,
    /// The note an `action = "note"` key shows.
    #[serde(default)]
    pub note: Option<String>,
}

/// Every named key's look, by key id.
pub type KeysConfig = BTreeMap<String, KeyLook>;

/// One top-bar button's name for screen readers; its icon comes from `icons.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BarButton {
    /// Name for screen readers.
    pub name: String,
}

/// One suggestion chip; the spike shows samples that type their text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Chip {
    /// What it shows and types.
    pub text: String,
    /// Lucide icon before the text.
    #[serde(default)]
    pub icon: Option<String>,
}

/// `spike.toml [bar]`: the top bar and the test strip under the keyboard.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BarConfig {
    /// Moves the keyboard: click, move, click.
    pub grip: BarButton,
    /// Light or soft dark (`icons.toml` `mode` and `mode_on`).
    pub theme: BarButton,
    /// One size step smaller.
    pub smaller: BarButton,
    /// One size step larger.
    pub bigger: BarButton,
    /// Shrinks the keyboard to a bubble.
    pub minimise: BarButton,
    /// The bubble, which brings the keyboard back.
    pub bubble: BarButton,
    /// The bubble window's title, which the focus guard looks for.
    pub bubble_title: String,
    /// What a screen reader hears over the screen-wide catcher while moving or resizing.
    pub catch_name: String,
    /// Closes the keyboard.
    pub close: BarButton,
    /// Resizes: click, move, click.
    pub corner: BarButton,
    /// Labels of the size presets, one per preset.
    pub sizes: Vec<String>,
    /// Their names for screen readers.
    pub size_names: Vec<String>,
    /// Labels of the light, dark and auto buttons in the test strip.
    pub modes: [String; 3],
    /// The D-pad's up, right, left and down arrows, then Stop.
    pub dpad: [BarButton; 5],
    /// Sample chips.
    pub chips: Vec<Chip>,
}

/// `spike.toml [assets]`: folders under the settings folder, and the icon file type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetsConfig {
    /// Fonts, searched in subfolders too.
    pub fonts: String,
    /// Icons, one file per name.
    pub icons: String,
    /// Icon file extension.
    pub icon_ext: String,
    /// Font file extensions.
    pub font_exts: Vec<String>,
}

impl AssetsConfig {
    /// The file of icon `name` under settings folder `dir`.
    pub fn icon(&self, dir: &Path, name: &str) -> PathBuf {
        dir.join(&self.icons)
            .join(name)
            .with_extension(&self.icon_ext)
    }

    /// Every font file under `dir`, searched in subfolders too; links are skipped, so a loop cannot recurse.
    pub fn font_files(&self, dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let font = |e: &std::ffi::OsStr| self.font_exts.iter().any(|x| e.eq_ignore_ascii_case(x));
        let mut all = Vec::new();
        for e in entries.flatten() {
            let (Ok(kind), p) = (e.file_type(), e.path()) else {
                continue;
            };
            if kind.is_dir() {
                all.extend(self.font_files(&p));
            } else if kind.is_file() && p.extension().is_some_and(font) {
                all.push(p);
            }
        }
        all
    }
}

/// Refuses a named key of the product board without a look, a key drawing nothing, and a side key without a working action.
pub fn check_keys(keys: &KeysConfig, l: &BoardSpec, icons: &Icons) -> Result<(), String> {
    let named = l.rows.iter().flatten().filter_map(|i| match i {
        RowItem::Key(k) => Some((k.id.as_str(), k.kind)),
        RowItem::Chars(_) => None,
    });
    let side = l.side.iter().map(|s| (s.id.as_str(), KeyKind::Act));
    for (id, kind) in named.chain(side) {
        let k = keys
            .get(id)
            .ok_or_else(|| format!("keys: {id} has no [keys.{id}]"))?;
        let icon = icons.keys.contains_key(id);
        let draws = icon || k.label.is_some() || matches!(kind, KeyKind::Lang | KeyKind::Space);
        let acts = match (kind, k.action) {
            (KeyKind::Act, Some(Action::Note)) => k.note.is_some(),
            (KeyKind::Act, a) => a.is_some(),
            _ => true,
        };
        if !draws || !acts {
            return Err(format!(
                "keys: {id} needs an icon or label, and side keys an action (a note for \"note\")"
            ));
        }
    }
    Ok(())
}

/// The bar as the spike face reads it: each button's name with its icon from `icons`.
pub fn bar_json(bar: &BarConfig, icons: &Icons) -> Value {
    let (b, d) = (&icons.bar, &icons.dpad);
    let mut v = serde_json::to_value(bar).unwrap_or_default();
    let named = [
        ("grip", &b.grip),
        ("theme", &b.mode),
        ("smaller", &b.smaller),
        ("bigger", &b.bigger),
        ("minimise", &b.minimise),
        ("bubble", &b.bubble),
        ("close", &b.close),
        ("corner", &b.corner),
    ];
    for (button, icon) in named {
        v[button]["icon"] = json!(icon);
    }
    v["theme"]["icon_on"] = json!(b.mode_on);
    let dpad = [&d.up, &d.right, &d.left, &d.down, &d.stop];
    for (i, icon) in dpad.into_iter().enumerate() {
        v["dpad"][i]["icon"] = json!(icon);
    }
    v
}

/// Refuses a size label list that does not match the presets.
pub fn check_bar(bar: &BarConfig, presets: usize) -> Result<(), String> {
    if bar.sizes.len() != presets || bar.size_names.len() != presets {
        return Err(format!(
            "bar: sizes and size_names need {presets} entries, one per preset"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;

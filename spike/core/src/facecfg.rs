//! `spike.toml [keys]`, `[bar]` and `[assets]`: how each named key and top-bar button looks, what side keys do, and where fonts and icons are.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::kbgeom::{KeyKind, LayoutConfig, RowItem};
use crate::latch::Latch;

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
}

/// Which bottom corner of the screen the minimise bubble goes to; never the top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Corner {
    /// Bottom left.
    Left,
    /// Bottom right.
    Right,
}

/// How one named key looks, what screen readers call it and what it does.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyLook {
    /// Lucide icon name.
    #[serde(default)]
    pub icon: Option<String>,
    /// Text drawn instead of an icon.
    #[serde(default)]
    pub label: Option<String>,
    /// Name for screen readers.
    pub name: String,
    /// The modifier it latches.
    #[serde(default)]
    pub latch: Option<Latch>,
    /// It is Caps Lock, so it lights while Caps Lock is on.
    #[serde(default)]
    pub caps: bool,
    /// What a side key does.
    #[serde(default)]
    pub action: Option<Action>,
    /// The note an `action = "note"` key shows.
    #[serde(default)]
    pub note: Option<String>,
}

/// Every named key's look, by key id.
pub type KeysConfig = BTreeMap<String, KeyLook>;

/// One top-bar button: its icon, the icon while it is on, and its name.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BarButton {
    /// Lucide icon name.
    pub icon: String,
    /// Icon while it is on.
    #[serde(default)]
    pub icon_on: Option<String>,
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
    /// Light or soft dark.
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
    /// The bottom corner of the keyboard's screen where the bubble goes.
    pub bubble_corner: Corner,
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

/// Refuses a named key without a look, a look drawing nothing, and a side key without a working action.
pub fn check_keys(keys: &KeysConfig, l: &LayoutConfig) -> Result<(), String> {
    let named = l.rows.iter().flatten().filter_map(|i| match i {
        RowItem::Key(k) => Some((k.id.as_str(), k.kind)),
        RowItem::Chars(_) => None,
    });
    let side = l.side.iter().map(|s| (s.id.as_str(), KeyKind::Act));
    for (id, kind) in named.chain(side) {
        let k = keys
            .get(id)
            .ok_or_else(|| format!("keys: {id} has no [keys.{id}]"))?;
        let draws =
            k.icon.is_some() || k.label.is_some() || matches!(kind, KeyKind::Lang | KeyKind::Space);
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
mod tests {
    use super::*;

    fn cfg() -> crate::config::SpikeConfig {
        crate::config::load().expect("spike.toml loads")
    }

    #[test]
    fn every_named_and_side_key_has_a_look() {
        let c = cfg();
        assert_eq!(check_keys(&c.keys, &c.layout), Ok(()));
        let mut keys = c.keys.clone();
        keys.remove("esc");
        assert!(check_keys(&keys, &c.layout).is_err(), "Esc lost its look");
    }

    #[test]
    fn a_key_drawing_nothing_is_refused() {
        let c = cfg();
        let mut keys = c.keys.clone();
        let esc = keys.get_mut("esc").expect("esc");
        esc.icon = None;
        esc.label = None;
        assert!(check_keys(&keys, &c.layout).is_err());
    }

    #[test]
    fn a_side_key_needs_an_action_and_a_note_needs_text() {
        let c = cfg();
        let mut keys = c.keys.clone();
        keys.get_mut("mic").expect("mic").action = None;
        assert!(check_keys(&keys, &c.layout).is_err());
        let mut keys = c.keys.clone();
        let power = keys.get_mut("power").expect("power");
        power.action = Some(Action::Note);
        power.note = None;
        assert!(check_keys(&keys, &c.layout).is_err());
    }

    #[test]
    fn size_labels_match_the_presets() {
        let c = cfg();
        assert_eq!(check_bar(&c.bar, c.size.presets.len()), Ok(()));
        assert!(check_bar(&c.bar, c.size.presets.len() + 1).is_err());
    }

    #[test]
    fn minimise_never_looks_like_a_size_button() {
        let b = cfg().bar;
        assert_ne!(b.minimise.icon, b.smaller.icon);
        assert_ne!(b.minimise.icon, b.bigger.icon);
        assert!(!b.bubble.icon.is_empty() && !b.bubble_title.is_empty());
    }

    #[test]
    fn font_files_are_found_in_subfolders_by_extension() {
        let dir = std::env::temp_dir().join(format!("kx-fonts-{}", std::process::id()));
        let sub = dir.join("plex");
        std::fs::create_dir_all(&sub).expect("temp folder");
        for f in [sub.join("a.ttf"), dir.join("b.OTF"), dir.join("c.txt")] {
            std::fs::write(f, b"").expect("temp file");
        }
        let mut found = cfg().assets.font_files(&dir);
        found.sort();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(found, [dir.join("b.OTF"), sub.join("a.ttf")]);
    }

    #[test]
    fn a_folder_link_is_not_followed_so_a_loop_cannot_recurse() {
        let dir = std::env::temp_dir().join(format!("kx-loop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp folder");
        std::fs::write(dir.join("a.ttf"), b"").expect("temp file");
        // A junction needs no special rights, unlike a symbolic link.
        let made = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(dir.join("back"))
            .arg(&dir)
            .output()
            .is_ok_and(|o| o.status.success());
        let found = cfg().assets.font_files(&dir);
        let _ = std::fs::remove_dir(dir.join("back"));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(made, "the junction was made");
        assert_eq!(found, [dir.join("a.ttf")]);
    }

    #[test]
    fn the_bubble_goes_to_the_bottom_left_or_right_corner() {
        assert_eq!(cfg().bar.bubble_corner, Corner::Right, "by default");
        assert_eq!(
            serde_json::from_str::<Corner>("\"left\"").ok(),
            Some(Corner::Left)
        );
        assert!(
            serde_json::from_str::<Corner>("\"top\"").is_err(),
            "never on top"
        );
    }

    #[test]
    fn an_icon_is_a_file_named_after_it() {
        let c = cfg();
        let p = c.assets.icon(Path::new("base"), "mic");
        assert!(
            p.ends_with(
                Path::new(&c.assets.icons)
                    .join("mic")
                    .with_extension(&c.assets.icon_ext)
            )
        );
    }
}

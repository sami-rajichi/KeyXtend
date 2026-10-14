//! Where the bundled fonts, icons and sounds sit in the repo's `assets/` folder and under which names the app bundles
//! them, and which files each bundled font family has. The app never reads these folders: kx-ui compiles the files in,
//! and only its build and tests and xtask turn on the `files` feature that reads them (ADR-0004).

#[cfg(feature = "files")]
mod read;

#[cfg(feature = "files")]
pub use read::{Bundle, Bundled, SoundFile, bundle, font_files, icon_file, is_font, sound_files};

use crate::ThemeError;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// Folders under `assets/` that hold icons: Lucide's, then our own drawings.
pub const ICON_DIRS: [&str; 2] = ["icons/lucide", "icons"];
/// The icon file type.
pub const ICON_EXT: &str = "svg";
/// The folder under `assets/` with one folder per font family.
pub const FONT_DIR: &str = "fonts";
/// Font file types the app bundles.
pub const FONT_EXTS: [&str; 2] = ["ttf", "otf"];
/// The folder under `assets/` that holds the sounds `sounds.toml` names, beside their recipes and notes.
pub const SOUND_DIR: &str = "sounds";
/// The bundle's one icon folder, which holds Lucide's icons and our own drawings side by side.
pub const ICONS_AT: &str = "icons";
/// The URL scheme Qt gives its image providers.
pub const IMAGE_SCHEME: &str = "image";
/// The image provider that draws the bundle's icons in their tint, so QML needs no effect to tint them.
pub const ICON_PROVIDER: &str = "kxicon";
/// Font families Windows 11 ships, so the app never bundles them; any other family a theme names is bundled.
pub const SYSTEM_FONTS: [&str; 2] = ["Segoe UI Variable Text", "Segoe UI"];
/// The circle a mark with no letter before it sits on (U+25CC), which every mark font draws.
pub const MARK_CIRCLE: char = '\u{25CC}';
/// The marks every mark font draws on that circle: the harakat the keyboard names (fathatan to sukun) and the
/// superscript alef; the app pins them to the keyboard's names.
pub const MARKS: [RangeInclusive<char>; 2] = ['\u{064B}'..='\u{0652}', '\u{0670}'..='\u{0670}'];

/// Name of the font table, for errors.
const FONT_FILES_FILE: &str = "font_files.toml";
/// The font table, compiled in so the app knows each family's files without the folder (ADR-0021).
const FONT_FILES: &str = include_str!("../tokens/font_files.toml");

/// Each bundled font family and its files, from `font_files.toml`; Windows' own families have none, so it leaves them
/// out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFiles(BTreeMap<String, Vec<String>>);

impl FontFiles {
    /// The shipped table.
    ///
    /// # Errors
    ///
    /// `font_files.toml` does not parse.
    pub fn shipped() -> Result<Self, ThemeError> {
        let why = |e: toml::de::Error| ThemeError {
            file: FONT_FILES_FILE,
            why: e.to_string(),
        };
        toml::from_str(FONT_FILES).map(Self).map_err(why)
    }

    /// Every bundled family, in name order.
    pub fn families(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// The files of `family`, as paths under `assets/fonts/`; `None` when it is not bundled.
    #[must_use]
    pub fn files(&self, family: &str) -> Option<&[String]> {
        self.0.get(family).map(Vec::as_slice)
    }

    /// The files of `family` under the names the bundle gives them; `None` when it is not bundled.
    #[must_use]
    pub fn names(&self, family: &str) -> Option<Vec<String>> {
        let files = self.files(family)?;
        Some(files.iter().map(|rel| font_name(rel)).collect())
    }
}

/// The URL QML asks the bundle's icons by, as `<url>/<rrggbb tint>/<name>.<ICON_EXT>`.
#[must_use]
pub fn icon_url() -> String {
    format!("{IMAGE_SCHEME}://{ICON_PROVIDER}")
}

/// The bundle name of font file `rel`, a path under `assets/fonts/` such as `rubik/Rubik[wght].ttf`: each part made
/// URL-safe, under `FONT_DIR`.
#[must_use]
pub fn font_name(rel: &str) -> String {
    let parts: Vec<String> = rel.split('/').map(safe).collect();
    format!("{FONT_DIR}/{}", parts.join("/"))
}

/// `name` with every character a URL path would escape, such as `[` or `,`, made a `-`.
fn safe(name: &str) -> String {
    let keep = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_');
    name.chars()
        .map(|c| if keep(c) { c } else { '-' })
        .collect()
}

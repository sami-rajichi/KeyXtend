//! Where the bundled fonts and icons sit in the repo's `assets/` folder and under which names the app bundles them,
//! for kx-ui's build script and the checks. The app never reads these folders: kx-ui compiles the files in.

use crate::Themes;
use std::collections::BTreeSet;
use std::io;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

/// Folders under `assets/` that hold icons: Lucide's, then our own drawings.
pub const ICON_DIRS: [&str; 2] = ["icons/lucide", "icons"];
/// The icon file type.
pub const ICON_EXT: &str = "svg";
/// The folder under `assets/` with one folder per font family.
pub const FONT_DIR: &str = "fonts";
/// Font file types the app bundles.
pub const FONT_EXTS: [&str; 2] = ["ttf", "otf"];
/// The bundle's one icon folder, which holds Lucide's icons and our own drawings side by side.
pub const ICONS_AT: &str = "icons";
/// Font families Windows 11 ships, so the app never bundles them; any other family a theme names is bundled.
pub const SYSTEM_FONTS: [&str; 2] = ["Segoe UI Variable Text", "Segoe UI"];
/// The circle a mark with no letter before it sits on (U+25CC), which every mark font draws.
pub const MARK_CIRCLE: char = '\u{25CC}';
/// The marks every mark font draws on that circle: the harakat the keyboard names (fathatan to sukun) and the
/// superscript alef; the app pins them to the keyboard's names.
pub const MARKS: [RangeInclusive<char>; 2] = ['\u{064B}'..='\u{0652}', '\u{0670}'..='\u{0670}'];

/// A file the app bundles: its name in the bundle and its file in the repo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundled {
    /// Its path in the bundle, such as `icons/x.svg` or `fonts/rubik/Rubik-wght-.ttf`.
    pub name: String,
    /// The file it is copied from.
    pub file: PathBuf,
}

/// Everything the app bundles from the `assets` folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    /// Each icon the tokens name, once, in name order.
    pub icons: Vec<Bundled>,
    /// Every font file, each under its family folder with a name that needs no escaping in a URL.
    pub fonts: Vec<Bundled>,
}

/// What the app bundles from the `assets` folder.
///
/// # Errors
///
/// The tokens do not load, an icon they name has no file, or a font folder cannot be read.
pub fn bundle(assets: &Path) -> Result<Bundle, String> {
    let themes = Themes::shipped().map_err(|e| e.to_string())?;
    let names: BTreeSet<&str> = themes.icons.names().into_iter().collect();
    let icon = |name: &str| {
        let file = icon_file(assets, name).ok_or(format!("no icon file for {name}"))?;
        let name = format!("{ICONS_AT}/{name}.{ICON_EXT}");
        Ok(Bundled { name, file })
    };
    let icons = names.into_iter().map(icon).collect::<Result<_, String>>()?;
    let files = font_files(assets).map_err(|e| format!("{FONT_DIR}: {e}"))?;
    let fonts = files.into_iter().map(font).collect();
    Ok(Bundle { icons, fonts })
}

/// Font `file` under its family folder, both names made URL-safe.
fn font(file: PathBuf) -> Bundled {
    let part = |p: Option<&std::ffi::OsStr>| safe(&p.unwrap_or_default().to_string_lossy());
    let family = part(file.parent().and_then(Path::file_name));
    let name = format!("{FONT_DIR}/{family}/{}", part(file.file_name()));
    Bundled { name, file }
}

/// `name` with every character a URL path would escape, such as `[` or `,`, made a `-`.
fn safe(name: &str) -> String {
    let keep = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_');
    name.chars()
        .map(|c| if keep(c) { c } else { '-' })
        .collect()
}

/// The file for icon `name` in the `assets` folder, Lucide's first; `None` when no icon folder has it.
#[must_use]
pub fn icon_file(assets: &Path, name: &str) -> Option<PathBuf> {
    ICON_DIRS
        .iter()
        .map(|dir| assets.join(dir).join(format!("{name}.{ICON_EXT}")))
        .find(|path| path.is_file())
}

/// True when `path` is a font file the app bundles.
#[must_use]
pub fn is_font(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| FONT_EXTS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// Every font file in the `assets` folder's family folders, in path order.
///
/// # Errors
///
/// A folder that cannot be read.
pub fn font_files(assets: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for family in std::fs::read_dir(assets.join(FONT_DIR))? {
        let family = family?.path();
        if !family.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(family)? {
            let file = file?.path();
            if is_font(&file) {
                files.push(file);
            }
        }
    }
    files.sort();
    Ok(files)
}

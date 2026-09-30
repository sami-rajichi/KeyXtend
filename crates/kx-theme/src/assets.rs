//! Where the bundled fonts and icons sit in the repo's `assets/` folder, for kx-ui's build script and the checks.
//! The app never reads these folders: kx-ui compiles the files in as Qt resources.

use std::io;
use std::path::{Path, PathBuf};

/// Folders under `assets/` that hold icons: Lucide's, then our own drawings.
pub const ICON_DIRS: [&str; 2] = ["icons/lucide", "icons"];
/// The icon file type.
pub const ICON_EXT: &str = "svg";
/// The folder under `assets/` with one folder per font family.
pub const FONT_DIR: &str = "fonts";
/// Font file types the app bundles.
pub const FONT_EXTS: [&str; 2] = ["ttf", "otf"];

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

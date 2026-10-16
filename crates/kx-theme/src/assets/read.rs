//! The readers of the `assets` folder, behind the `files` feature: the files the app bundles, each font file, each icon
//! file and each sound, read and checked for kx-ui's build and xtask only.

use super::{FONT_DIR, FONT_EXTS, ICON_DIRS, ICON_EXT, ICONS_AT, SOUND_DIR, font_name};
use crate::Themes;
use crate::sounds::{Sounds, wav};
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

/// A file the app bundles: its name in the bundle and its file in the repo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundled {
    /// Its path in the bundle, such as `icons/x.svg` or `fonts/rubik/Rubik-wght-.ttf`.
    pub name: String,
    /// The file it is copied from.
    pub file: PathBuf,
}

/// A sound the app embeds: its file name in `SOUND_DIR` and its checked sample bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundFile {
    /// Its name, as `sounds.toml` writes it.
    pub name: String,
    /// Its samples, in the shipping format.
    pub samples: Vec<u8>,
}

/// Everything the app bundles from the `assets` folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    /// Each icon the tokens name, once, in name order.
    pub icons: Vec<Bundled>,
    /// Every font file, each under its family folder with a name that needs no escaping in a URL.
    pub fonts: Vec<Bundled>,
    /// The checked sound tokens the sounds were read with.
    pub sound_tokens: Sounds,
    /// Each sound `sounds.toml` names, once, in the order first named.
    pub sounds: Vec<SoundFile>,
}

/// What the app bundles from the `assets` folder.
///
/// # Errors
///
/// The tokens do not load, an icon they name has no file, a font folder cannot be read, or a named sound is missing
/// or invalid.
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
    let sound_tokens = Sounds::shipped().map_err(|e| e.to_string())?;
    let sounds = sound_files(assets, &sound_tokens)?;
    Ok(Bundle {
        icons,
        fonts,
        sound_tokens,
        sounds,
    })
}

/// Each file `sounds` names, once in the order first named, read from the `assets` folder and checked; other files
/// there are left alone.
///
/// # Errors
///
/// A named file is missing or is not a shipped sound in `sounds`' format; the error names it.
pub fn sound_files(assets: &Path, sounds: &Sounds) -> Result<Vec<SoundFile>, String> {
    let read = |name: &str| {
        let at = |why: String| format!("{SOUND_DIR}/{name}: {why}");
        let path = assets.join(SOUND_DIR).join(name);
        // The size comes first, so a huge file is refused before it fills memory.
        let len = std::fs::metadata(&path)
            .map_err(|e| at(e.to_string()))?
            .len();
        wav::check_size(len, &sounds.format).map_err(|e| at(e.to_string()))?;
        let bytes = std::fs::read(&path).map_err(|e| at(e.to_string()))?;
        let samples = wav::samples(&bytes, &sounds.format).map_err(|e| at(e.to_string()))?;
        let (name, samples) = (name.to_owned(), samples.to_vec());
        Ok(SoundFile { name, samples })
    };
    sounds.files().into_iter().map(read).collect()
}

/// Font `file` under its family folder, both names made URL-safe.
fn font(file: PathBuf) -> Bundled {
    let part = |p: Option<&std::ffi::OsStr>| p.unwrap_or_default().to_string_lossy().into_owned();
    let family = part(file.parent().and_then(Path::file_name));
    let name = font_name(&format!("{family}/{}", part(file.file_name())));
    Bundled { name, file }
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

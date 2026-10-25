//! Where the words live: the word packs beside the program, and in a debug build also where `cargo xtask words` writes
//! them; the learned words stay sealed in a folder under the data folder.

use kx_module_api::ports::PACK_DIR;
use std::path::{Path, PathBuf};

/// The folder under the data folder that holds the sealed learned words and emails.
pub const SEALED_DIR: &str = "words";

/// The workspace root, from this crate's folder; only a debug build reads under it.
#[cfg(debug_assertions)]
macro_rules! root {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
    };
}

/// The words tool's manifest, which names the folder its packs go to under the workspace root.
#[cfg(debug_assertions)]
const WORDS_TOML: &str = include_str!(concat!(root!(), "/xtask/words.toml"));
/// The manifest's table that names that folder.
#[cfg(debug_assertions)]
const TABLE: &str = "text";
/// The key in that table.
#[cfg(debug_assertions)]
const KEY: &str = "out";

/// The folders the packs are read from, the first first: `PACK_DIR` beside the program, then in a debug build the
/// folder `cargo xtask words` writes to.
#[must_use]
pub fn pack_dirs(exe_dir: &Path) -> Vec<PathBuf> {
    std::iter::once(exe_dir.join(PACK_DIR))
        .chain(built())
        .collect()
}

/// The folder `cargo xtask words` writes the packs to, as its manifest names it.
#[cfg(debug_assertions)]
fn built() -> Option<PathBuf> {
    let manifest: toml::Table = WORDS_TOML.parse().ok()?;
    let out = manifest.get(TABLE)?.get(KEY)?.as_str()?;
    Some(Path::new(root!()).join(out))
}

/// None: a release build reads the packs only beside the program.
#[cfg(not(debug_assertions))]
fn built() -> Option<PathBuf> {
    None
}

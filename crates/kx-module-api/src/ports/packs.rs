//! Word packs (ADR-0032): each language's seed, read straight from disk. Packs hold no user data, so the port needs
//! no capability.

use super::sealed::RESERVED_NAMES;
use crate::{Capability, ServiceId, ServiceKey};
use thiserror::Error;

mod index;
pub use index::{IndexError, PackIndex};

/// The extension of a pack file, after its language's code: `fr.kxw`.
pub const PACK_EXT: &str = "kxw";
/// The largest pack a platform maps and a writer makes, 64 MiB.
pub const MAX_PACK_BYTES: u64 = 64 * 1024 * 1024;
/// The largest pack index a platform maps and `PackIndex::read` takes, 64 KiB: far more than a code per language.
pub const MAX_INDEX_BYTES: u64 = 64 * 1024;
/// The shortest language code: ISO 639-1.
pub const MIN_ISO_LEN: usize = 2;
/// The longest language code: ISO 639-2 or 639-3, as layouts give a language with no two-letter code.
pub const MAX_ISO_LEN: usize = 3;

/// True when `iso` is a language code a pack may carry and name its file after: `MIN_ISO_LEN` to `MAX_ISO_LEN`
/// lower-case ASCII letters, and no device name (`RESERVED_NAMES`).
#[must_use]
pub fn fits_iso(iso: &str) -> bool {
    let sized = (MIN_ISO_LEN..=MAX_ISO_LEN).contains(&iso.len());
    sized && iso.bytes().all(|b| b.is_ascii_lowercase()) && !RESERVED_NAMES.contains(&iso)
}

/// The file name of language `iso`'s pack, `<iso>.<PACK_EXT>`; `None` unless `fits_iso`, so no path leaves its folder.
#[must_use]
pub fn pack_file(iso: &str) -> Option<String> {
    fits_iso(iso).then(|| format!("{iso}.{PACK_EXT}"))
}

/// The file the build writes beside the packs, naming each language it shipped and the codes it serves (`PackIndex`).
pub const PACK_INDEX: &str = "packs.txt";
/// The folder beside the program that holds the packs, their index and credits; the stage fills it and the app reads it.
pub const PACK_DIR: &str = "words";

/// One open pack's bytes, read-only; dropping it closes the pack.
pub trait PackView: Send + Sync {
    /// The whole pack, unchanged while the view lives. A system that cannot block writers copies the file, or maps
    /// only from folders that only the system can write.
    fn bytes(&self) -> &[u8];
}

/// Why a pack could not be opened.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum PackError {
    /// The file is there but cannot be read.
    #[error("the word pack cannot be read")]
    Unreadable,
}

/// Opens the shipped word packs; the platform provides it.
pub trait WordPacks: Send + Sync {
    /// The pack named `iso`, as `pack_of` names it; `Ok(None)` when there is none or `iso` fails `fits_iso`.
    ///
    /// # Errors
    /// `Unreadable` when the pack is there but cannot be read.
    fn open(&self, iso: &str) -> Result<Option<Box<dyn PackView>>, PackError>;

    /// The code of the pack the build shipped for layout language `iso` (`LangLabel::iso`), as the index beside the
    /// packs resolves it (`PackIndex::pack_of`); `None` if it shipped none, `iso` fails `fits_iso`, or no index reads.
    fn pack_of(&self, iso: &str) -> Option<String>;
}

/// The word packs service; it needs no capability.
pub struct WordPacksKey;

impl ServiceKey for WordPacksKey {
    type Api = dyn WordPacks;
    const ID: ServiceId = ServiceId::new("word-packs");
    const CAPABILITY: Option<Capability> = None;
}

#[cfg(test)]
mod tests;

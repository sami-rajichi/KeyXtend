//! Word packs (ADR-0032): each language's seed, read straight from disk. Packs hold no user data, so the port needs
//! no capability.

use crate::{Capability, ServiceId, ServiceKey};
use thiserror::Error;

/// One open pack's bytes, read-only; dropping it closes the pack.
pub trait PackView: Send + Sync {
    /// The whole pack.
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
    /// The pack of language `iso`, as `LangLabel::iso` names it; `Ok(None)` when the language has no pack.
    ///
    /// # Errors
    /// `Unreadable` when the pack is there but cannot be read.
    fn open(&self, iso: &str) -> Result<Option<Box<dyn PackView>>, PackError>;
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

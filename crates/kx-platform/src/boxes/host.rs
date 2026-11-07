//! The vault job on the neutral host: each `BoxRun` call as one vault ask, within the deadline.

use super::{ask, reply};
use crate::worker::{Host, Spawner, VaultFrames};
use kx_module_api::ports::{BoxError, BoxPart, KeepassImport, Opened, Padlock};
use kx_module_api::{Redacted, Secret};

impl<S: Spawner<VaultFrames>> Host<S, VaultFrames> {
    /// Makes a padlock whose identity `password` locks at work factor `work`.
    pub(super) fn padlock(&mut self, password: Secret, work: u8) -> Result<Padlock, BoxError> {
        self.call(|id| ask::padlock(id, password, work), reply::padlock)
    }

    /// Seals `parts` to `padlock`.
    pub(super) fn seal(
        &mut self,
        padlock: Padlock,
        parts: Vec<BoxPart>,
    ) -> Result<Redacted<Vec<u8>>, BoxError> {
        self.call(|id| ask::seal(id, padlock, parts), reply::sealed)
    }

    /// Opens the box `bytes` with `password`.
    pub(super) fn open(&mut self, bytes: Vec<u8>, password: Secret) -> Result<Opened, BoxError> {
        self.call(|id| ask::open(id, bytes, password), reply::opened)
    }

    /// Whether `password` opens `padlock`.
    pub(super) fn check(&mut self, padlock: Padlock, password: Secret) -> Result<bool, BoxError> {
        self.call(|id| ask::check(id, padlock, password), reply::checked)
    }

    /// The entries of the `KeePass` file `bytes`, opened with `password`.
    pub(super) fn keepass(
        &mut self,
        bytes: Vec<u8>,
        password: Secret,
    ) -> Result<KeepassImport, BoxError> {
        self.call(|id| ask::keepass(id, bytes, password), reply::entries)
    }
}

#[cfg(test)]
mod tests;

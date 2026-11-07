//! Randomness: the system's random bytes, for entry ids and made passwords; it needs no capability.
//! A port, as modules make no OS calls and a seeded fake keeps tests deterministic.

use crate::{Capability, ServiceId, ServiceKey};
use thiserror::Error;

/// The system gave no random bytes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("the system gave no random bytes")]
pub struct RandomError;

/// Fills buffers with random bytes; the platform provides it from the OS, and tests use a seeded fake.
pub trait Random: Send + Sync {
    /// Fills all of `buf` with random bytes.
    ///
    /// # Errors
    /// `RandomError` when the system gives none.
    fn fill(&self, buf: &mut [u8]) -> Result<(), RandomError>;
}

/// The randomness service; it needs no capability.
pub struct RandomKey;

impl ServiceKey for RandomKey {
    type Api = dyn Random;
    const ID: ServiceId = ServiceId::new("random");
    const CAPABILITY: Option<Capability> = None;
}

#[cfg(test)]
mod tests;

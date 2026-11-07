//! Sealed files for learned data and the clipboard history's shelf: XChaCha20-Poly1305 under a random data key that
//! the OS guard wraps (ADR-0010, ADR-0034, spec §9.1); a hidden run keeps them in memory alone (ADR-0037), and
//! `OsRandom` gives the OS's random bytes. The crate never logs, and holds every secret only as `Zeroizing`.

#![forbid(unsafe_code)]

mod consts;
mod files;
mod guard;
mod memory;
mod random;
mod seal;
mod sealer;
mod shelf;
#[cfg(test)]
mod testing;

pub use consts::KEY_LEN;
pub use files::SealedFiles;
pub use guard::{GuardError, KeyGuard};
pub use memory::{MemoryShelf, MemoryStore};
pub use random::OsRandom;
pub use seal::{CryptoError, open, seal};
pub use shelf::{DiskRoom, SealedShelfFiles};
pub use zeroize::Zeroizing;

use std::sync::{Mutex, MutexGuard, PoisonError};

/// Locks `m`; no step here leaves a key or a blob half changed, so a poisoned lock is still used.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

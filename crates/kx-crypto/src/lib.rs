//! Sealed files for learned data and the clipboard history's shelf: XChaCha20-Poly1305 under a random data key that
//! the OS guard wraps (ADR-0010, ADR-0034, spec §9.1). The crate never logs, and holds every secret only as `Zeroizing`.

#![forbid(unsafe_code)]

mod consts;
mod files;
mod guard;
mod seal;
mod sealer;
mod shelf;
#[cfg(test)]
mod testing;

pub use consts::KEY_LEN;
pub use files::SealedFiles;
pub use guard::{GuardError, KeyGuard};
pub use seal::{CryptoError, open, seal};
pub use shelf::{DiskRoom, SealedShelfFiles};
pub use zeroize::Zeroizing;

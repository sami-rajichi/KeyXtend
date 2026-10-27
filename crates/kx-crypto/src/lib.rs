//! Sealed files for learned data: XChaCha20-Poly1305 under a random data key that the OS guard wraps
//! (ADR-0010, spec §9.1). The crate never logs, and holds every secret only as `Zeroizing`.

#![forbid(unsafe_code)]

mod consts;
mod files;
mod guard;
mod seal;

pub use consts::KEY_LEN;
pub use files::SealedFiles;
pub use guard::{GuardError, KeyGuard};
pub use seal::{CryptoError, open, seal};
pub use zeroize::Zeroizing;

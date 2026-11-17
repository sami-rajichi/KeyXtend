//! The SHA-256 a download is checked by, fed with each byte as it is written to the part.

use kx_ipc::SHA256_BYTES;
use sha2::{Digest, Sha256};

/// A running SHA-256 over a part's bytes, from its first byte.
#[derive(Clone, Default)]
pub struct Print(Sha256);

impl Print {
    /// Adds the next `bytes`.
    pub fn add(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    /// The fingerprint of every byte added.
    pub fn done(self) -> [u8; SHA256_BYTES] {
        self.0.finalize().into()
    }
}

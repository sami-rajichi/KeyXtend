//! The OS guard of the data key: the platform wraps the key so only this user on this PC unwraps it.

use thiserror::Error;
use zeroize::Zeroizing;

/// Why the guard could not wrap or unwrap the key.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum GuardError {
    /// The system's key service cannot be reached.
    #[error("the system key service is unavailable")]
    Unavailable,
    /// The system's key service declined, for example for a key from another user.
    #[error("the system key service refused the key")]
    Refused,
}

/// Wraps the data key with the OS key (DPAPI, Keychain or Secret Service); the platform implements it and never shows a
/// prompt.
pub trait KeyGuard: Send + Sync {
    /// The wrapped form of `key`, safe to keep in a file; it may be an opaque handle where the OS keeps the key itself,
    /// as Keychain and libsecret do.
    ///
    /// # Errors
    /// `Unavailable` without the key service, `Refused` when it declines.
    fn wrap(&self, key: &[u8]) -> Result<Vec<u8>, GuardError>;

    /// The key that `wrap` turned into `wrapped`.
    ///
    /// # Errors
    /// `Unavailable` without the key service, `Refused` when `wrapped` is not this user's.
    fn unwrap(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, GuardError>;
}

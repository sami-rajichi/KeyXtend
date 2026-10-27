//! The data key: unwrapped from the key file, or made, wrapped and written there on the first save.
//! A key file the guard cannot unwrap is never deleted or replaced.

use super::disk::{self, Read};
use crate::consts::{KEY_LEN, MAX_KEY_FILE_BYTES};
use crate::guard::KeyGuard;
use kx_module_api::ports::SealedError;
use std::fs;
use std::io;
use std::path::Path;
use zeroize::Zeroizing;

/// The data key, on the heap so moves never copy it, and wiped when dropped.
pub(super) type DataKey = Box<Zeroizing<[u8; KEY_LEN]>>;

/// A zeroed key to fill in place.
#[allow(
    clippy::unnecessary_box_returns,
    reason = "the box keeps moves from copying the key"
)]
fn blank() -> DataKey {
    Box::new(Zeroizing::new([0; KEY_LEN]))
}

/// Whether the key file is there.
pub(super) fn present(path: &Path) -> Result<bool, SealedError> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(SealedError::Io),
    }
}

/// The key in the key file at `path`, or `None` when there is no key file.
///
/// A key file the guard refuses, or that holds no key of the right length, is `Locked`.
pub(super) fn read(path: &Path, guard: &dyn KeyGuard) -> Result<Option<DataKey>, SealedError> {
    let wrapped = match disk::read(path, MAX_KEY_FILE_BYTES).map_err(|_| SealedError::Io)? {
        Read::Missing => return Ok(None),
        Read::TooBig => return Err(SealedError::Locked),
        Read::Bytes(bytes) => bytes,
    };
    let plain = guard.unwrap(&wrapped).map_err(|_| SealedError::Locked)?;
    if plain.len() != KEY_LEN {
        return Err(SealedError::Locked);
    }
    let mut key = blank();
    key.copy_from_slice(&plain);
    Ok(Some(key))
}

/// Makes a random key, and writes it wrapped to the key file at `path`.
pub(super) fn make(path: &Path, guard: &dyn KeyGuard) -> Result<DataKey, SealedError> {
    let mut key = blank();
    getrandom::fill(&mut key[..]).map_err(|_| SealedError::Io)?;
    let wrapped = guard.wrap(&key[..]).map_err(|_| SealedError::Locked)?;
    disk::write_whole(path, &wrapped).map_err(|_| SealedError::Io)?;
    Ok(key)
}

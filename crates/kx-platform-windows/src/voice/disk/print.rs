//! A file's SHA-256 by Windows' CNG, read in pieces so a model of gigabytes never sits in memory whole. A link or
//! junction is never followed, no other program may write the file while it is read, and a size change is refused.

use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use kx_module_api::ports::{Fingerprint, MAX_SOUND_BYTES};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{
    BCRYPT_HASH_HANDLE, BCRYPT_SHA256_ALG_HANDLE, BCryptCreateHash, BCryptDestroyHash,
    BCryptFinishHash, BCryptHashData,
};
use windows::Win32::Storage::FileSystem::{
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ,
};

/// Bytes read at a time: the frames' piece size, a buffer the process already budgets for.
const PIECE: usize = MAX_SOUND_BYTES;

/// The SHA-256 of every byte of `file`; `InvalidInput` for a link or a folder, `InvalidData` when its size changes.
pub(super) fn sha256(file: &Path) -> io::Result<[u8; Fingerprint::BYTES]> {
    let (mut file, size) = open(file)?;
    let hash = Hash::new()?;
    let (mut piece, mut read) = (vec![0; PIECE], 0_u64);
    loop {
        let got = file.read(&mut piece)?;
        if got == 0 {
            break;
        }
        hash.add(&piece[..got])?;
        read = read.saturating_add(u64::try_from(got).unwrap_or(u64::MAX));
    }
    if read != size || file.metadata()?.len() != size {
        return Err(io::ErrorKind::InvalidData.into());
    }
    hash.finish()
}

/// Opens `file` itself, never a link's target, sharing it only with readers and deleters; it and its size now.
fn open(file: &Path) -> io::Result<(File, u64)> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_DELETE.0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0)
        .open(file)?;
    let meta = file.metadata()?;
    if !meta.file_type().is_file() {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    Ok((file, meta.len()))
}

/// A running hash, destroyed once when dropped.
struct Hash(BCRYPT_HASH_HANDLE);

impl Hash {
    /// A fresh SHA-256 from the system's shared provider.
    #[allow(unsafe_code, reason = "Makes a hash object of our own.")]
    fn new() -> io::Result<Self> {
        let mut made = BCRYPT_HASH_HANDLE::default();
        // SAFETY: the pseudo-handle names a provider Windows keeps; it makes its own hash object and writes `made`.
        ok(unsafe { BCryptCreateHash(BCRYPT_SHA256_ALG_HANDLE, &raw mut made, None, None, 0) })?;
        Ok(Self(made))
    }

    /// Adds `bytes` to the hash.
    #[allow(unsafe_code, reason = "Feeds our own hash object.")]
    fn add(&self, bytes: &[u8]) -> io::Result<()> {
        // SAFETY: our live hash object, and a slice that outlives the call.
        ok(unsafe { BCryptHashData(self.0, bytes, 0) })
    }

    /// The hash of every byte added.
    #[allow(unsafe_code, reason = "Reads our own hash object.")]
    fn finish(self) -> io::Result<[u8; Fingerprint::BYTES]> {
        let mut out = [0; Fingerprint::BYTES];
        // SAFETY: our live hash object, and an output of exactly the hash's size.
        ok(unsafe { BCryptFinishHash(self.0, &mut out, 0) })?;
        Ok(out)
    }
}

impl Drop for Hash {
    #[allow(unsafe_code, reason = "Destroys our own hash object.")]
    fn drop(&mut self) {
        // SAFETY: made by `new` and destroyed once, here.
        let _ = unsafe { BCryptDestroyHash(self.0) };
    }
}

/// `status` as a result, keeping Windows' code.
fn ok(status: NTSTATUS) -> io::Result<()> {
    status.ok().map_err(io::Error::from)
}

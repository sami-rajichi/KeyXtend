//! The model files port (ADR-0043): the fixed drives and their room, the models folder, a file's fingerprint, and a
//! row's download into a part file by the worker's `fetch` job. Only `Network` reaches it, for downloads the user
//! asked for, and no path or name prints.

mod fetch;

pub use fetch::{FetchAsk, FetchNote, FetchRun, PART_EXT, host_ok, link_named};

use super::clip::SEPARATORS;
use super::probe_ok;
use super::sealed::names_device;
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use kx_ipc::FetchFault;
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// True for an absolute drive path (`probe_ok`) in UTF-8 within the frames' `MAX_PATH_BYTES`, with no control
/// character, no part made only of dots and spaces (read as `.` or `..`), no device name (`names_device`) and no
/// colon after the drive's, which names a stream.
#[must_use]
pub fn path_ok(path: &Path) -> bool {
    let fits = |text: &&str| text.len() <= kx_ipc::MAX_PATH_BYTES;
    let Some(text) = path.to_str().filter(fits) else {
        return false;
    };
    let bytes = text.as_bytes();
    let dots = |part: &&str| !part.is_empty() && part.bytes().all(|b| b" .".contains(&b));
    let mut parts = text.split(|c: char| u8::try_from(c).is_ok_and(|b| SEPARATORS.contains(&b)));
    let odd = parts.any(|part| dots(&part) || names_device(part));
    let stream = bytes
        .get(DRIVE_LEN..)
        .is_some_and(|rest| rest.contains(&b':'));
    let control = text.chars().any(char::is_control);
    probe_ok(path) && !odd && !stream && !control
}

/// Bytes in a drive's prefix, such as `C:`, whose colon is the only one a path may hold.
const DRIVE_LEN: usize = 2;

/// A fixed drive and its room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drive {
    /// Its root folder.
    pub root: Redacted<PathBuf>,
    /// Its name to show, such as its letter and label.
    pub name: Redacted<String>,
    /// Bytes free on it.
    pub free: u64,
}

/// A file a folder holds, with a model's extension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFile {
    /// Its file name.
    pub name: Redacted<String>,
    /// Its size in bytes.
    pub bytes: u64,
}

/// What one folder holds: its subfolders, to pick a folder, and its model files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FolderScan {
    /// Its subfolders' names, A to Z, without hidden ones or links.
    pub folders: Vec<Redacted<String>>,
    /// Its files with one of the asked extensions, A to Z.
    pub files: Vec<ModelFile>,
}

/// The base a fingerprint is spelled in.
const HEX_RADIX: u32 = 16;
/// Hex digits per byte.
const HEX_DIGITS: usize = 2;

/// A file's SHA-256, which the models table trusts.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint([u8; Self::BYTES]);

impl Fingerprint {
    /// Bytes in a SHA-256, as the frames carry it.
    pub const BYTES: usize = kx_ipc::SHA256_BYTES;

    /// Wraps a SHA-256's bytes.
    #[must_use]
    pub const fn new(bytes: [u8; Self::BYTES]) -> Self {
        Self(bytes)
    }

    /// The fingerprint `text` spells in hex, in either case; `None` for any other text.
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        let digits = text.as_bytes();
        let (pairs, rest) = digits.as_chunks::<HEX_DIGITS>();
        let sized = pairs.len() == Self::BYTES && rest.is_empty();
        if !sized || !digits.iter().all(u8::is_ascii_hexdigit) {
            return None;
        }
        let mut bytes = [0; Self::BYTES];
        for (byte, pair) in bytes.iter_mut().zip(pairs) {
            let pair = std::str::from_utf8(pair).ok()?;
            *byte = u8::from_str_radix(pair, HEX_RADIX).ok()?;
        }
        Some(Self(bytes))
    }

    /// The SHA-256's bytes, as the frames carry them.
    #[must_use]
    pub const fn bytes(&self) -> [u8; Self::BYTES] {
        self.0
    }

    /// The fingerprint in lower-case hex, as the table writes it.
    #[must_use]
    pub fn hex(&self) -> String {
        self.0.iter().fold(String::new(), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
    }
}

/// Shows the hex, which is public.
impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Fingerprint")
            .field(&format_args!("{}", self.hex()))
            .finish()
    }
}

/// Why a folder, a file or a download failed; no variant holds a path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ModelError {
    /// The ask's value of this name is outside its range.
    #[error("the ask's {0} is outside its range")]
    Range(&'static str),
    /// The path is not an absolute drive path, or climbs or names a stream (`path_ok`).
    #[error("the path is not a plain drive path")]
    BadPath,
    /// The folder, the file or its drive is gone.
    #[error("the folder or its drive is gone")]
    Missing,
    /// The folder cannot be written.
    #[error("the folder cannot be written")]
    ReadOnly,
    /// The drive has less room than the file and `FetchAsk::min_free_mb`.
    #[error("the drive has too little room")]
    Full,
    /// Another error of the file system.
    #[error("the file could not be read or written")]
    Io,
    /// The network failed.
    #[error("the download failed")]
    Net,
    /// No byte came for `FetchAsk::stall_s`.
    #[error("the download stalled")]
    Stalled,
    /// An address or a redirect is not https, or names a host the row does not list.
    #[error("the address or a redirect is not allowed")]
    Host,
    /// The file has another size than its row's, so the part was deleted.
    #[error("the downloaded file has the wrong size")]
    Size,
    /// The file has another fingerprint than its row's, so the part was deleted.
    #[error("the downloaded file has the wrong fingerprint")]
    Print,
    /// Every address answered an error.
    #[error("the server refused the download")]
    Server,
    /// The download was cancelled, so the part was deleted.
    #[error("the download was cancelled")]
    Cancelled,
    /// The fetch job failed inside.
    #[error("the download job failed")]
    Internal,
    /// The fetch worker ended or answered badly.
    #[error("the download worker ended")]
    Gone,
}

/// Each fault the `fetch` job sends, as the error the voice module shows.
impl From<FetchFault> for ModelError {
    fn from(fault: FetchFault) -> Self {
        match fault {
            FetchFault::BadSize => Self::Size,
            FetchFault::BadHash => Self::Print,
            FetchFault::Redirect | FetchFault::NotHttps => Self::Host,
            FetchFault::Stall => Self::Stalled,
            FetchFault::DiskFull => Self::Full,
            FetchFault::Disk => Self::Io,
            FetchFault::Net => Self::Net,
            FetchFault::Server => Self::Server,
            FetchFault::Cancelled => Self::Cancelled,
            FetchFault::Internal => Self::Internal,
        }
    }
}

/// Lists, makes, reads and fetches model files; the platform provides it.
pub trait ModelFiles: Send + Sync {
    /// The fixed drives, with their room, so the folder's offer takes the one with the most.
    fn drives(&self) -> Vec<Drive>;

    /// Makes `folder` and the folders above it when missing.
    ///
    /// # Errors
    /// `BadPath` unless `path_ok`, `Missing` without its drive, `ReadOnly` when it cannot be written, else `Io`.
    fn make(&self, folder: &Path) -> Result<(), ModelError>;

    /// What `folder` holds: its subfolders and its files whose extension is one of `exts`, in any case.
    ///
    /// # Errors
    /// `BadPath` unless `path_ok`, `Missing` when it is gone, else `Io`.
    fn scan(&self, folder: &Path, exts: &[String]) -> Result<FolderScan, ModelError>;

    /// The SHA-256 of `file`, read whole; it may take seconds, so call it off the module's thread.
    ///
    /// # Errors
    /// `BadPath` unless `path_ok`, `Missing` when it is gone, else `Io`.
    fn fingerprint(&self, file: &Path) -> Result<Fingerprint, ModelError>;

    /// Starts downloading `ask` into its part file, resuming a part there; `tell` hears its notes on any thread.
    ///
    /// # Errors
    /// `Range` when `ask.check` refuses it, `Full` under its room, `Missing` or `ReadOnly` for its folder, and `Gone`
    /// when the download job cannot start.
    fn fetch(
        &self,
        ask: FetchAsk,
        tell: Box<dyn Fn(FetchNote) + Send>,
    ) -> Result<Box<dyn FetchRun>, ModelError>;

    /// Deletes `file` at once; a missing file is no error.
    ///
    /// # Errors
    /// `BadPath` unless `path_ok`, `ReadOnly` when it cannot be deleted, else `Io`.
    fn remove(&self, file: &Path) -> Result<(), ModelError>;
}

/// The model files service; it needs `Network`, as it downloads (ADR-0042 decision 8).
pub struct ModelFilesKey;

impl ServiceKey for ModelFilesKey {
    type Api = dyn ModelFiles;
    const ID: ServiceId = ServiceId::new("model-files");
    const CAPABILITY: Option<Capability> = Some(Capability::Network);
}

#[cfg(test)]
mod tests;

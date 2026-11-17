//! The fetch job's frames (ADR-0042 decision 2, ADR-0043 decision 8): the host asks for one model file, from the
//! table's HTTPS addresses in order into a `.part` file, and may pause, resume or cancel it; the job tells its
//! progress, then that the file is checked and named, or one fixed fault. Every download ends with one of those two.

pub(crate) mod ask;
pub(crate) mod note;
pub(crate) mod wire;

use crate::code::Code;
use crate::limits::PART_EXT;

/// Why a download stopped; a fixed reason, never a server's or library's words.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FetchFault {
    /// The file's size is not the one asked; the part is deleted.
    BadSize = 1,
    /// The file's SHA-256 is not the one asked; the part is deleted.
    BadHash = 2,
    /// A redirect named a host the ask does not list.
    Redirect = 3,
    /// An address or a redirect was not HTTPS.
    NotHttps = 4,
    /// No byte came for `stall_s`.
    Stall = 5,
    /// The drive is full.
    DiskFull = 6,
    /// The folder is missing or read-only, or a write failed.
    Disk = 7,
    /// No connection: the name, the connection or TLS failed.
    Net = 8,
    /// No address served, and the first fault was an error status.
    Server = 9,
    /// Cancelled as asked; the part is deleted.
    Cancelled = 10,
    /// The job failed inside.
    Internal = 11,
}

impl Code for FetchFault {
    const ALL: &'static [Self] = &[
        Self::BadSize,
        Self::BadHash,
        Self::Redirect,
        Self::NotHttps,
        Self::Stall,
        Self::DiskFull,
        Self::Disk,
        Self::Net,
        Self::Server,
        Self::Cancelled,
        Self::Internal,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}

/// The partial file a download writes until its size and fingerprint pass: the final path and `PART_EXT`.
#[must_use]
pub fn part_path(path: &str) -> String {
    format!("{path}{PART_EXT}")
}

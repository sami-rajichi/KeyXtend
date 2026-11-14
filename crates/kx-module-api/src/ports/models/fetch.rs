//! A row's download (ADR-0043 decision 8): its addresses and allowed hosts, its size and fingerprint, checked before
//! the part file gets its name, and the run's notes, pause, resume and cancel. The frames' caps bound each field.

use super::{Fingerprint, ModelError};
use crate::Redacted;
use crate::ports::{JobLimits, Link, path_ok};
use kx_ipc::{MAX_HOST_BYTES, MAX_HOSTS, MAX_MODEL_BYTES, MAX_URL_BYTES, MAX_URLS};
use std::ops::RangeInclusive;
use std::path::PathBuf;

/// The suffix a download's part file adds to its file's name until it is checked, as the `fetch` job writes it.
pub use kx_ipc::PART_EXT;

/// One row's download into the models folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchAsk {
    /// The fetch job's limits and waits.
    pub job: JobLimits,
    /// The row's addresses, tried in order, so a mirror is one more; at most `MAX_URLS`.
    pub links: Vec<Link>,
    /// The hosts an address or a redirect may reach, in lower case, at most `MAX_HOSTS`; any other is refused.
    pub hosts: Vec<String>,
    /// The file it becomes once checked: an absolute drive path (`path_ok`), and so is its part file.
    pub file: Redacted<PathBuf>,
    /// Its size in bytes, from its row, at most `MAX_MODEL_BYTES`.
    pub bytes: u64,
    /// Its SHA-256, from its row.
    pub print: Fingerprint,
    /// The room the drive keeps free beside the file, in MB.
    pub min_free_mb: u32,
    /// A download with no byte for this many seconds fails.
    pub stall_s: u16,
}

impl FetchAsk {
    /// `min_free_mb`'s range.
    pub const MIN_FREE_MB: RangeInclusive<u32> = 256..=65_536;
    /// `stall_s`' range, the frames' own.
    pub const STALL_S: RangeInclusive<u16> = kx_ipc::STALL_S;

    /// Refuses an ask with a value outside its range or cap, no address, a host the frames refuse, or a file that is
    /// no plain drive path.
    ///
    /// # Errors
    /// `ModelError::Range` naming the first such value.
    pub fn check(&self) -> Result<(), ModelError> {
        let host =
            |h: &String| (1..=MAX_HOST_BYTES).contains(&h.len()) && kx_ipc::rules::host(h).is_ok();
        let link = |l: &Link| l.expose().len() <= MAX_URL_BYTES;
        let part = self.part().is_some_and(|p| path_ok(&p));
        let shape = [
            (
                (1..=MAX_URLS).contains(&self.links.len()) && self.links.iter().all(link),
                "links",
            ),
            (
                (1..=MAX_HOSTS).contains(&self.hosts.len()) && self.hosts.iter().all(host),
                "hosts",
            ),
            ((1..=MAX_MODEL_BYTES).contains(&self.bytes), "bytes"),
            (path_ok(self.file.expose()) && part, "file"),
        ];
        let odd = shape
            .into_iter()
            .find_map(|(fits, name)| (!fits).then_some(name));
        let own = || first_bad!(self, min_free_mb: Self::MIN_FREE_MB, stall_s: Self::STALL_S);
        let bad = self.job.bad().or(odd).or_else(own);
        bad.map_or(Ok(()), |name| Err(ModelError::Range(name)))
    }

    /// The part file the download writes until it is checked, as the frames' `part_path` names it; `None` for a file
    /// that is not UTF-8, which `check` refuses.
    #[must_use]
    pub fn part(&self) -> Option<PathBuf> {
        let file = self.file.expose().to_str()?;
        Some(PathBuf::from(kx_ipc::part_path(file)))
    }
}

/// What a download tells.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FetchNote {
    /// Bytes written so far, of the file's size.
    Progress {
        /// Written so far, a resumed part's bytes among them.
        done: u64,
        /// The file's size.
        total: u64,
    },
    /// The file was checked and took its name.
    Done,
    /// The download stopped on this fault; a bad size or fingerprint, or a cancel, deleted the part.
    Fault(ModelError),
}

/// A download on the platform's side. Dropping it pauses it.
pub trait FetchRun: Send {
    /// Stops reading and keeps the part.
    fn pause(&mut self);

    /// Goes on from the part's end after `pause`.
    fn resume(&mut self);

    /// Stops the download and deletes its part.
    fn cancel(&mut self);
}

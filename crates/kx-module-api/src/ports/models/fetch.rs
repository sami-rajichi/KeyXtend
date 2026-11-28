//! A row's download (ADR-0043 decision 8): its addresses and allowed hosts, its size and fingerprint, checked before
//! the part file gets its name, and the run's notes, pause, resume and cancel. The frames' caps bound each field.

use super::{Fingerprint, ModelError};
use crate::Redacted;
use crate::ports::{JobLimits, LINK_SCHEME, Link, path_ok};
use kx_ipc::rules::{covers, listed, names};
use kx_ipc::{MAX_HOST_BYTES, MAX_HOSTS, MAX_MODEL_BYTES, MAX_URL_BYTES, MAX_URLS};
use std::ops::RangeInclusive;
use std::path::PathBuf;

/// The suffix a download's part file adds to its file's name until it is checked, as the `fetch` job writes it.
pub use kx_ipc::PART_EXT;

/// Ends a link's host and starts its path.
const SLASH: char = '/';

/// True for a host a download may list, within `MAX_HOST_BYTES`: a plain lower-case host, or `.` and a domain of two
/// names or more, which takes each host under it (`kx_ipc::rules::listed`).
#[must_use]
pub fn host_ok(entry: &str) -> bool {
    entry.len() <= MAX_HOST_BYTES && listed(entry).is_ok()
}

/// True when the text between `link`'s scheme and its path is a plain entry of `hosts` itself (`rules::names`); a host
/// only under a `.` entry, a port, a user name or a query there never passes.
#[must_use]
pub fn link_named(hosts: &[String], link: &str) -> bool {
    let host = link
        .strip_prefix(LINK_SCHEME)
        .and_then(|rest| rest.split_once(SLASH));
    host.is_some_and(|(host, _)| hosts.iter().any(|h| names(h, host)))
}

/// One row's download into the models folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchAsk {
    /// The fetch job's limits and waits.
    pub job: JobLimits,
    /// The row's addresses, tried in order, so a mirror is one more; at most `MAX_URLS`.
    pub links: Vec<Link>,
    /// The hosts an address or a redirect may reach, in lower case, at most `MAX_HOSTS`; any other is refused.
    pub hosts: Vec<String>,
    /// The file it becomes once checked: an absolute path (`path_ok`), and so is its part file.
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

    /// Refuses an ask with a value outside its range or cap, a host `host_ok` refuses, no address or one `link_named`
    /// refuses, or a file that is no plain absolute path.
    ///
    /// # Errors
    /// `ModelError::Range` naming the first such value.
    pub fn check(&self) -> Result<(), ModelError> {
        let link =
            |l: &Link| l.expose().len() <= MAX_URL_BYTES && link_named(&self.hosts, l.expose());
        let part = self.part().is_some_and(|p| path_ok(&p));
        let shape = [
            (
                (1..=MAX_HOSTS).contains(&self.hosts.len())
                    && self.hosts.iter().all(|h| host_ok(h)),
                "hosts",
            ),
            (
                (1..=MAX_URLS).contains(&self.links.len()) && self.links.iter().all(link),
                "links",
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

    /// True when a redirect of its download may reach `host`, by name or under a `.` entry, as the fetch's own rule
    /// decides; its links need `link_named`.
    #[must_use]
    pub fn reaches(&self, host: &str) -> bool {
        self.hosts.iter().any(|h| covers(h, host))
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

//! What the host sends the fetch job: download one model file, pause, resume or cancel it, or goodbye.

use super::wire::{HOST, HOSTS, PATH, SIZE, URL, URLS};
use crate::code::Kind;
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::limits::{HTTPS, SHA256_BYTES, STALL_S};
use crate::rules::{self, ok_if, within};
use crate::wire::{Payload, Put, Sink, Size, Spec};
use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};

/// A message from the host to the fetch job; it borrows its fields when written and owns them when read.
#[derive(Clone, PartialEq, Eq)]
pub enum FetchAsk<'a> {
    /// Download one file into its `part_path`, then check it and give it `path`.
    Get {
        /// The download's id, which its notes repeat.
        id: u32,
        /// The table's addresses, tried in order; each starts with `HTTPS`.
        urls: Vec<Cow<'a, str>>,
        /// The only hosts an address or a redirect may name.
        hosts: Vec<Cow<'a, str>>,
        /// The final path; the file gets it only once its size and fingerprint pass.
        path: Cow<'a, str>,
        /// The file's size in bytes, within 1 and `MAX_MODEL_BYTES`.
        size: u64,
        /// The file's SHA-256.
        sha256: [u8; SHA256_BYTES],
        /// Seconds without a byte before it stops, within `STALL_S`.
        stall_s: u16,
    },
    /// Stop reading and keep the part.
    Pause {
        /// The download's id.
        id: u32,
    },
    /// Go on from the part's end.
    Resume {
        /// The download's id.
        id: u32,
    },
    /// Stop and delete the part.
    Cancel {
        /// The download's id.
        id: u32,
    },
    /// End: the fetch job exits after this frame.
    Bye,
}

/// The kinds a fetch ask may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::Fetch,
    Kind::FetchPause,
    Kind::FetchResume,
    Kind::FetchCancel,
    Kind::FetchBye,
];

impl FetchAsk<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<FetchAsk<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let ask = match head.kind {
            Kind::Fetch => read_get(&mut p, id)?,
            Kind::FetchPause => FetchAsk::Pause { id },
            Kind::FetchResume => FetchAsk::Resume { id },
            Kind::FetchCancel => FetchAsk::Cancel { id },
            Kind::FetchBye if id == 0 => FetchAsk::Bye,
            Kind::FetchBye => return Err(IpcError::Inconsistent),
            _ => return Err(IpcError::BadKind),
        };
        p.end()?;
        Ok(ask)
    }

    /// Writes this ask as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, `TooLong` or `Inconsistent`, before any byte is written, for an ask the job would refuse; `Io`
    /// when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let mut size = Size::default();
        self.put(&mut size)?;
        write_head(w, self.kind(), self.id(), size.0)?;
        self.put(&mut Put(w))?;
        w.flush()?;
        Ok(())
    }

    /// Runs this ask's payload through `s`.
    fn put(&self, s: &mut dyn Sink) -> Result<(), IpcError> {
        let Self::Get {
            urls,
            hosts,
            path,
            size,
            sha256,
            stall_s,
            ..
        } = self
        else {
            return Ok(());
        };
        s.count(URLS, urls.len())?;
        urls.iter().try_for_each(|u| put_url(s, u))?;
        s.count(HOSTS, hosts.len())?;
        hosts.iter().try_for_each(|h| put_host(s, h))?;
        s.field(PATH, path.as_bytes())?;
        rules::path(path)?;
        s.u64(within(*size, &SIZE)?)?;
        s.raw(sha256)?;
        s.u16(within(*stall_s, &STALL_S)?)
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Get { .. } => Kind::Fetch,
            Self::Pause { .. } => Kind::FetchPause,
            Self::Resume { .. } => Kind::FetchResume,
            Self::Cancel { .. } => Kind::FetchCancel,
            Self::Bye => Kind::FetchBye,
        }
    }

    /// The download this acts on; goodbye's is 0.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Get { id, .. }
            | Self::Pause { id }
            | Self::Resume { id }
            | Self::Cancel { id } => *id,
            Self::Bye => 0,
        }
    }
}

/// Refuses an address that is not visible ASCII starting with `HTTPS`.
fn url(text: &str) -> Result<(), IpcError> {
    ok_if(text.starts_with(HTTPS) && text.bytes().all(|b| b.is_ascii_graphic()))
}

/// Refuses a host that is not lower-case letters, digits, dots and hyphens.
fn host(text: &str) -> Result<(), IpcError> {
    let plain = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-';
    ok_if(text.bytes().all(plain))
}

/// Sends an address.
fn put_url(s: &mut dyn Sink, text: &str) -> Result<(), IpcError> {
    s.field(URL, text.as_bytes())?;
    url(text)
}

/// Sends a host.
fn put_host(s: &mut dyn Sink, text: &str) -> Result<(), IpcError> {
    s.field(HOST, text.as_bytes())?;
    host(text)
}

/// Reads `n` texts of `spec`, each passing `rule`.
fn read_list<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    spec: Spec,
    rule: fn(&str) -> Result<(), IpcError>,
    n: usize,
) -> Result<Vec<Cow<'static, str>>, IpcError> {
    (0..n)
        .map(|_| {
            let text = p.text(spec)?;
            rule(&text)?;
            Ok(Cow::Owned(text))
        })
        .collect()
}

/// Reads a download's fields.
fn read_get<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    id: u32,
) -> Result<FetchAsk<'static>, IpcError> {
    let n = p.count(URLS)?;
    let urls = read_list(p, URL, url, n)?;
    let n = p.count(HOSTS)?;
    let hosts = read_list(p, HOST, host, n)?;
    let path = p.text(PATH)?;
    rules::path(&path)?;
    let size = within(p.u64()?, &SIZE)?;
    let sha256 = p.array::<SHA256_BYTES>()?;
    let stall_s = within(p.u16()?, &STALL_S)?;
    let path = Cow::Owned(path);
    Ok(FetchAsk::Get {
        id,
        urls,
        hosts,
        path,
        size,
        sha256,
        stall_s,
    })
}

impl fmt::Debug for FetchAsk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id();
        match self {
            Self::Get {
                urls,
                hosts,
                size,
                stall_s,
                ..
            } => f
                .debug_struct("Get")
                .field("id", &id)
                .field("urls", &urls.len())
                .field("hosts", &hosts.len())
                .field("size", size)
                .field("stall_s", stall_s)
                .finish_non_exhaustive(),
            Self::Pause { .. } => f.debug_struct("Pause").field("id", &id).finish(),
            Self::Resume { .. } => f.debug_struct("Resume").field("id", &id).finish(),
            Self::Cancel { .. } => f.debug_struct("Cancel").field("id", &id).finish(),
            Self::Bye => f.write_str("Bye"),
        }
    }
}

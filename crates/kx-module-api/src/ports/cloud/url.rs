//! A service's address (ADR-0047 decision 6): an https link whose host is a plain lower-case host with at most a port,
//! so no user name, fragment or other reading of its text can reach another host. The rule is the frames' own
//! (`kx_ipc::rules::https`), so the `speech-net` job takes every address this type holds.

use crate::ports::Link;
use kx_ipc::rules::{covers, https, https_host, https_origin};
use kx_ipc::{HTTPS, MAX_URL_BYTES};

/// A checked HTTPS address within the frames' `MAX_URL_BYTES`: `LINK_SCHEME`, a plain lower-case host and maybe a port,
/// then a path and query of link characters (`Link`). It prints redacted, as a typed address is the user's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpsUrl(Link);

impl HttpsUrl {
    /// The longest address it holds: the frames' own cap.
    pub const MAX_BYTES: usize = MAX_URL_BYTES;

    /// `text` with its scheme and host in lower case, as both ignore case (RFC 3986), so a typed `HTTPS://Host.com/A`
    /// can be taken; the path and query keep their case, and a text that is no https address stays as it is.
    #[must_use]
    pub fn lowered(text: &str) -> String {
        let lower = text.to_ascii_lowercase();
        if !lower.starts_with(HTTPS) {
            return text.to_owned();
        }
        // Lowering keeps every byte's place, so the host ends at the same place in both texts.
        let end = HTTPS.len() + https_host(&lower).len();
        let mut out = lower[..end].to_owned();
        out.push_str(&text[end..]);
        out
    }

    /// `text` as an address; `None` for anything else, an `http://` one among them.
    #[must_use]
    pub fn new(text: &str) -> Option<Self> {
        let fits = text.len() <= MAX_URL_BYTES && https(text).is_ok();
        let link = Link::new(text.to_owned()).ok().filter(|_| fits)?;
        Some(Self(link))
    }

    /// True when `text`'s scheme, host and port alone make an address, so a refusal of `text` lies in its path or query.
    #[must_use]
    pub fn has_host(text: &str) -> bool {
        Self::new(https_origin(text)).is_some()
    }

    /// The host, in lower case, the only one its requests may reach.
    #[must_use]
    pub fn host(&self) -> &str {
        https_host(self.0.expose())
    }

    /// True when its host is `entry`, or under `entry` when that starts with `.` (`rules::covers`), as a typed Azure
    /// address must be under its service's domain.
    #[must_use]
    pub fn is_on(&self, entry: &str) -> bool {
        covers(entry, self.host())
    }

    /// The whole address, for the job that sends to it.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose()
    }
}

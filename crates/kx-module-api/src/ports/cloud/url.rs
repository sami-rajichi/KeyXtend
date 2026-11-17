//! A service's address (ADR-0047 decision 6): an https link whose host is a plain lower-case host with at most a port,
//! so no user name, fragment or other reading of its text can reach another host. The rule is the frames' own
//! (`kx_ipc::rules::https`), so the `speech-net` job takes every address this type holds.

use crate::ports::Link;
use kx_ipc::MAX_URL_BYTES;
use kx_ipc::rules::{https, https_host};

/// A checked HTTPS address within the frames' `MAX_URL_BYTES`: `LINK_SCHEME`, a plain lower-case host and maybe a port,
/// then a path and query of link characters (`Link`). It prints redacted, as a typed address is the user's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpsUrl(Link);

impl HttpsUrl {
    /// `text` as an address; `None` for anything else, an `http://` one among them.
    #[must_use]
    pub fn new(text: &str) -> Option<Self> {
        let fits = text.len() <= MAX_URL_BYTES && https(text).is_ok();
        let link = Link::new(text.to_owned()).ok().filter(|_| fits)?;
        Some(Self(link))
    }

    /// The host, in lower case, the only one its requests may reach.
    #[must_use]
    pub fn host(&self) -> &str {
        https_host(self.0.expose())
    }

    /// The whole address, for the job that sends to it.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose()
    }
}

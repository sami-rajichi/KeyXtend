//! A service's address (ADR-0047 decision 6): an https link whose host is a plain lower-case host with at most a port,
//! so no user name, fragment or other reading of its text can reach another host.

use crate::ports::{LINK_SCHEME, Link};
use kx_ipc::rules::{UNDER, listed};
use kx_ipc::{MAX_HOST_BYTES, MAX_URL_BYTES};

/// What ends an address's host and port: its path or its query.
const HOST_END: [char; 2] = ['/', '?'];
/// Parts a host from its port.
const PORT_MARK: char = ':';
/// Marks a user name or a fragment, which an address never holds.
const NOT_IN: [char; 2] = ['@', '#'];

/// A checked HTTPS address within the frames' `MAX_URL_BYTES`: `LINK_SCHEME`, a plain lower-case host and maybe a port,
/// then a path and query of link characters (`Link`). It prints redacted, as a typed address is the user's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpsUrl(Link);

impl HttpsUrl {
    /// `text` as an address; `None` for anything else, an `http://` one among them.
    #[must_use]
    pub fn new(text: &str) -> Option<Self> {
        let fits = text.len() <= MAX_URL_BYTES && !text.contains(NOT_IN);
        let link = Link::new(text.to_owned()).ok().filter(|_| fits)?;
        host_ok(authority(link.expose())).then_some(Self(link))
    }

    /// The host, in lower case, the only one its requests may reach.
    #[must_use]
    pub fn host(&self) -> &str {
        let auth = authority(self.0.expose());
        auth.rsplit_once(PORT_MARK).map_or(auth, |(host, _)| host)
    }

    /// The whole address, for the job that sends to it.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose()
    }
}

/// The host and port of `link`, between its scheme and its path or query.
fn authority(link: &str) -> &str {
    let rest = link.strip_prefix(LINK_SCHEME).unwrap_or_default();
    rest.split(HOST_END).next().unwrap_or_default()
}

/// True for a plain lower-case host within `MAX_HOST_BYTES` (`rules::listed`), then maybe `:` and a port from 1.
fn host_ok(auth: &str) -> bool {
    let (host, port) = match auth.rsplit_once(PORT_MARK) {
        Some((host, port)) => (host, Some(port)),
        None => (auth, None),
    };
    let digits = |p: &str| p.bytes().all(|b| b.is_ascii_digit());
    let port_ok = port.is_none_or(|p| digits(p) && p.parse::<u16>().is_ok_and(|n| n > 0));
    let plain = !host.starts_with(UNDER) && host.len() <= MAX_HOST_BYTES && listed(host).is_ok();
    plain && port_ok
}

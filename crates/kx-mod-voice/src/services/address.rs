//! Where a row's requests go (ADR-0047 decision 6, ruling 2026-11-17): the row's own address on its own host, or the
//! address the user types, an HTTPS host name under the row's domain when it names one. The key's test goes to the
//! same host, as `speech-net` sees one address only.

use kx_module_api::ports::{HttpsUrl, LINK_SCHEME, host_ok};

use super::{Service, ServiceError};

/// The table's word for an address the user types.
const TYPED: &str = "typed";
/// What starts a test that is a path under the typed host.
const PATH: char = '/';
/// What ends an address's host and port: its path or its query.
const HOST_END: [char; 2] = ['/', '?'];
/// A plain host a typed row's test path is checked under, as the real one is typed later.
const PROBE: &str = "probe.invalid";

/// Where a row's pieces go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    /// The row's own address, on its own host.
    Fixed(HttpsUrl),
    /// The user types it: an HTTPS host name, under `under` when the row names a domain (Azure), any for Custom.
    Typed {
        /// The `.` domain or host the typed host must be on; `None` takes any HTTPS host name.
        under: Option<String>,
    },
}

/// The key's test: a call that needs the key and sends no sound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyTest {
    /// The row has none, so the first dictation tests the key.
    None,
    /// The row's own test address, on its own host.
    At(HttpsUrl),
    /// A path under the typed address's host.
    Under(String),
}

/// The address `text` names: "typed", or HTTPS on exactly the row's `host`.
pub(super) fn address(text: &str, host: Option<&str>) -> Result<Address, ServiceError> {
    if text == TYPED {
        return match host {
            None => Ok(Address::Typed { under: None }),
            Some(entry) if host_ok(entry) => Ok(Address::Typed {
                under: Some(entry.to_owned()),
            }),
            Some(_) => Err(ServiceError::Host),
        };
    }
    let url = HttpsUrl::new(text).ok_or(ServiceError::Address)?;
    if host == Some(url.host()) {
        Ok(Address::Fixed(url))
    } else {
        Err(ServiceError::Host)
    }
}

/// The test `text` names: HTTPS on the fixed address's host, or a path under a typed one.
pub(super) fn test(text: Option<&str>, address: &Address) -> Result<KeyTest, ServiceError> {
    let Some(text) = text else {
        return Ok(KeyTest::None);
    };
    match address {
        Address::Fixed(url) => {
            let call = HttpsUrl::new(text).ok_or(ServiceError::Test)?;
            if call.host() == url.host() {
                Ok(KeyTest::At(call))
            } else {
                Err(ServiceError::Host)
            }
        }
        Address::Typed { .. } => {
            let probe = format!("{LINK_SCHEME}{PROBE}{text}");
            let path = text.starts_with(PATH) && HttpsUrl::new(&probe).is_some();
            if path {
                Ok(KeyTest::Under(text.to_owned()))
            } else {
                Err(ServiceError::Test)
            }
        }
    }
}

impl Service {
    /// Where the pieces go: the row's own address, or `typed` when it is HTTPS on a host name the row takes; `None` for
    /// any other typed text, such as an `http://` or numeric address (edge 13).
    #[must_use]
    pub fn url(&self, typed: &str) -> Option<HttpsUrl> {
        match &self.address {
            Address::Fixed(url) => Some(url.clone()),
            Address::Typed { under } => HttpsUrl::new(typed)
                .filter(|url| under.as_deref().is_none_or(|entry| url.is_on(entry))),
        }
    }

    /// The key's test address, under `typed` for a typed row; `None` when the row has none or `typed` is refused.
    #[must_use]
    pub fn test_url(&self, typed: &str) -> Option<HttpsUrl> {
        match &self.test {
            KeyTest::None => None,
            KeyTest::At(url) => Some(url.clone()),
            KeyTest::Under(path) => {
                let base = self.url(typed)?;
                HttpsUrl::new(&format!("{}{path}", origin(base.expose())))
            }
        }
    }
}

/// The scheme, host and port of an address `HttpsUrl` took, without its path or query.
fn origin(url: &str) -> &str {
    let rest = url.get(LINK_SCHEME.len()..).unwrap_or_default();
    let end = rest.find(HOST_END).unwrap_or(rest.len());
    url.get(..LINK_SCHEME.len() + end).unwrap_or(url)
}

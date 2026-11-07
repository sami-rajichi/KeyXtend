//! The browser link port (ADR-0041 decision 5): a checked https link, handed to the desktop shell so the user's browser
//! opens it without our rights. Only `OpenLink` reaches it, and a link may hold selected text, so it never prints.

use crate::{Capability, Redacted, ServiceId, ServiceKey};
use thiserror::Error;

/// The one scheme a link may start with, in lower case.
pub const LINK_SCHEME: &str = "https://";
/// The longest link, in bytes: a safety ceiling under every platform's command-line limit (32,767 characters on
/// Windows), never a user setting.
pub const MAX_LINK_BYTES: usize = 16 * 1024;
/// The marks a link may hold besides ASCII letters and digits: RFC 3986's unreserved and reserved ones, and `%`.
const LINK_MARKS: &str = "-._~:/?#[]@!$&'()*+,;=%";

/// Why a link was refused or not opened; no variant holds the link.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum LinkError {
    /// It does not start with `LINK_SCHEME` and a host.
    #[error("the link is not an https link")]
    NotHttps,
    /// It holds a character no link may hold: a space, a quote, a control, a hidden mark or any non-ASCII one.
    #[error("the link holds a character a link may not hold")]
    BadChar,
    /// It is longer than `MAX_LINK_BYTES`.
    #[error("the link is too long")]
    TooLong,
    /// No desktop shell or browser can open links.
    #[error("no browser can open the link")]
    Unavailable,
    /// The shell refused, with its error code.
    #[error("the system refused to open the link (code {0})")]
    Refused(i32),
}

/// A checked link: `LINK_SCHEME`, a host, then only RFC 3986 characters, at most `MAX_LINK_BYTES`; it prints redacted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link(Redacted<String>);

impl Link {
    /// Checks and wraps `text`.
    ///
    /// # Errors
    /// `TooLong` past the cap, `NotHttps` without the scheme and a host, `BadChar` for any other character.
    pub fn new(text: String) -> Result<Self, LinkError> {
        if text.len() > MAX_LINK_BYTES {
            return Err(LinkError::TooLong);
        }
        let host = text
            .strip_prefix(LINK_SCHEME)
            .and_then(|rest| rest.chars().next());
        if !host.is_some_and(|c| c.is_ascii_alphanumeric()) {
            return Err(LinkError::NotHttps);
        }
        let fits = |c: char| c.is_ascii_alphanumeric() || LINK_MARKS.contains(c);
        if !text.chars().all(fits) {
            return Err(LinkError::BadChar);
        }
        Ok(Self(Redacted::new(text)))
    }

    /// The link's text, for the adapter that hands it to the shell.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose()
    }
}

/// Opens links in the user's default browser, which runs as the user's own program; the platform provides it.
pub trait WebLink: Send + Sync {
    /// Hands `link` to the desktop shell; it may wait on the shell, so call it off the UI thread.
    ///
    /// # Errors
    /// `Unavailable` with no shell or browser, `Refused` when the shell says no.
    fn open(&self, link: &Link) -> Result<(), LinkError>;
}

/// The browser link service; it needs `OpenLink`.
pub struct WebLinkKey;

impl ServiceKey for WebLinkKey {
    type Api = dyn WebLink;
    const ID: ServiceId = ServiceId::new("web-link");
    const CAPABILITY: Option<Capability> = Some(Capability::OpenLink);
}

#[cfg(test)]
mod tests;

//! Why a request failed, in fixed words, and the fetch or cloud fault each error becomes.

use kx_ipc::{FetchFault, NetFault};

/// Why a request failed; it never holds a server's or the system's words.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, thiserror::Error)]
pub enum NetError {
    /// An address or a redirect is not a plain HTTPS address.
    #[error("the address is not a plain HTTPS address")]
    NotHttps,
    /// An address or a redirect names a host the download does not list.
    #[error("the address names a host the download does not list")]
    Unlisted,
    /// The server redirected more than `MAX_HOPS` times.
    #[error("the server redirected too many times")]
    Hops,
    /// The server answered a status that is neither the file nor a redirect.
    #[error("the server answered status {0}")]
    Status(u16),
    /// A partial answer does not start where the request asked.
    #[error("the server answered another range than the one asked")]
    Range,
    /// The server's answer could not be read: a bad header, or a redirect without its address.
    #[error("the server's answer could not be read")]
    Server,
    /// No byte came within the stall time.
    #[error("no byte came within the stall time")]
    Stall,
    /// The name, the connection or TLS failed.
    #[error("the connection failed")]
    Net,
    /// A call's header would break its head, or a GET carries a body; nothing was sent.
    #[error("a header would break the request")]
    Header,
    /// The call was stopped before its body was all sent, as its run was cancelled.
    #[error("the request was stopped")]
    Stopped,
    /// The system failed inside.
    #[error("the system's HTTP client failed")]
    Internal,
}

impl From<NetError> for FetchFault {
    fn from(e: NetError) -> Self {
        match e {
            NetError::NotHttps => Self::NotHttps,
            NetError::Unlisted => Self::Redirect,
            NetError::Hops | NetError::Status(_) | NetError::Range | NetError::Server => {
                Self::Server
            }
            NetError::Stall => Self::Stall,
            NetError::Net => Self::Net,
            NetError::Header | NetError::Stopped | NetError::Internal => Self::Internal,
        }
    }
}

impl From<NetError> for NetFault {
    fn from(e: NetError) -> Self {
        match e {
            NetError::NotHttps | NetError::Header => Self::Range,
            NetError::Unlisted
            | NetError::Hops
            | NetError::Status(_)
            | NetError::Range
            | NetError::Server => Self::Strange,
            NetError::Stall => Self::Timeout,
            NetError::Net => Self::Network,
            NetError::Stopped | NetError::Internal => Self::Gone,
        }
    }
}

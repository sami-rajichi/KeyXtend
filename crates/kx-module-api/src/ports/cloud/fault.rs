//! Why a cloud run, its ask or a key's test failed (ADR-0047 decision 4): what the service answered, the network, or
//! the job. No variant holds the key, the sound or a reply's text.

use std::time::Duration;
use thiserror::Error;

/// A cloud fault; after any of them the recording is kept, so Try again and the local model stay open.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum CloudFault {
    /// The ask's value of this name is outside its range.
    #[error("the ask's {0} is outside its range")]
    Range(&'static str),
    /// The service refused the key: wrong, revoked or missing.
    #[error("the service refused the key")]
    Key,
    /// The account has no credit left.
    #[error("the account has no credit left")]
    Credit,
    /// The service asks for a step on its site first, such as a verification, a payment method or a spending cap.
    #[error("the service asks for a step on its site")]
    Account,
    /// The free allowance is used up.
    #[error("the free allowance is used up")]
    Allowance {
        /// How long until it renews, from the reply, when the service tells.
        renews_in: Option<Duration>,
    },
    /// Too many requests, still after one wait.
    #[error("the service had too many requests")]
    Rate,
    /// A piece was too big for the service.
    #[error("a piece was too big for the service")]
    TooBig,
    /// The service's model does not take the language code, such as a regional one.
    #[error("the service does not take this language")]
    Language,
    /// The service is not offered in the user's country or region.
    #[error("the service is not offered in this country or region")]
    Region,
    /// The key test's reply marks a management key, which manages keys but never transcribes.
    #[error("the key is a management key, which cannot transcribe")]
    Admin,
    /// The service sent no text.
    #[error("the service sent no text")]
    Empty,
    /// The reply was not the shape's: a hotspot's sign-in page, a redirect to another host, or one past its cap.
    #[error("the service sent a reply it should not")]
    Strange,
    /// The network failed, or the service is down.
    #[error("the network failed")]
    Network,
    /// The service gave no answer within `net_wait_s`.
    #[error("the service took too long")]
    Timeout,
    /// The `speech-net` job could not start, ended or answered badly.
    #[error("the cloud worker ended")]
    Gone,
}

impl CloudFault {
    /// The fault's plain name, letters only, which a log line may hold; never a value it carries.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Range(_) => "Range",
            Self::Key => "Key",
            Self::Credit => "Credit",
            Self::Account => "Account",
            Self::Allowance { .. } => "Allowance",
            Self::Rate => "Rate",
            Self::TooBig => "TooBig",
            Self::Language => "Language",
            Self::Region => "Region",
            Self::Admin => "Admin",
            Self::Empty => "Empty",
            Self::Strange => "Strange",
            Self::Network => "Network",
            Self::Timeout => "Timeout",
            Self::Gone => "Gone",
        }
    }
}

//! The `speech-net` job's frames (ADR-0047 decisions 3 to 6): the host sends a dictation's pieces to one cloud service,
//! their sound in parts after them, or tests a key, and may cancel either; the job tells each piece's text in order,
//! then that it is done, or one fault. The key and the sound travel in buffers wiped when dropped, and `Debug` never
//! shows them, an address, a model or a text.

pub(crate) mod ask;
mod body;
pub(crate) mod code;
pub(crate) mod note;
mod show;
pub(crate) mod wire;

use crate::limits::piece_bytes;
use crate::wiped::Wiped;
use code::{FaultCode, ShapeCode};
use std::borrow::Cow;

/// One call to a service: where it goes, how it carries the key and how long it waits; the job holds no table.
#[derive(Clone, PartialEq, Eq)]
pub struct NetCall<'a> {
    /// The request's shape, with a form's field names.
    pub shape: NetShape<'a>,
    /// The service's address: plain HTTPS naming the only host the call may reach (`rules::https`).
    pub url: Cow<'a, str>,
    /// The header that carries the key.
    pub header: NetHeader<'a>,
    /// The user's key: visible ASCII within `MAX_KEY_BYTES`.
    pub key: Wiped<'a>,
    /// The longest wait for one answer in seconds, within `NET_WAIT_S`.
    pub wait_s: u16,
}

/// The request shape a service row names; the job holds one adapter per shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetShape<'a> {
    /// The `OpenAI`-style upload form, with the row's field names.
    Form(NetForm<'a>),
    /// `ElevenLabs`' own form: the row's language field and fixed fields, and nothing else.
    Eleven(NetOwn<'a>),
    /// Gemini's single request, with the sound inline.
    Gemini,
    /// `AssemblyAI`'s Sync request: a JSON `config` part, then the sound, with the model under its own header.
    Assembly {
        /// The name of the header that carries the model (`rules::token`).
        model_header: Cow<'a, str>,
    },
}

impl NetShape<'_> {
    /// This shape's code.
    #[must_use]
    pub const fn code(&self) -> ShapeCode {
        match self {
            Self::Form(_) => ShapeCode::Form,
            Self::Eleven(_) => ShapeCode::Eleven,
            Self::Gemini => ShapeCode::Gemini,
            Self::Assembly { .. } => ShapeCode::Assembly,
        }
    }
}

/// How an upload form names its fields; it sends the model, the language, the fixed fields, then the sound. Its
/// `paywall`, `region` and `admin` are rules for reading the service's replies, never form fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetForm<'a> {
    /// The language field's name (`rules::token`).
    pub lang: Cow<'a, str>,
    /// The language goes as a list of one, its name followed by `[]`.
    pub list: bool,
    /// Fields sent as they are after the language, in order, at most `MAX_FIXED`.
    pub fixed: Vec<NetField<'a>>,
    /// A 403 asks the account to pay, as a missing payment method or a spending cap does: the account's step, not a
    /// bad key.
    pub paywall: bool,
    /// The error code a reply gives where the service is not offered in the user's country or region (`rules::token`),
    /// such as `OpenAI`'s `unsupported_country_region_territory`; `None` for none.
    pub region: Option<Cow<'a, str>>,
    /// Key-test reply fields (`rules::field_path`), at most `MAX_ADMIN`; any one true marks a management key.
    pub admin: Vec<Cow<'a, str>>,
}

/// `ElevenLabs`' own form: it sends the model, the language under `lang`, the fixed fields, then the sound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetOwn<'a> {
    /// The language field's name (`rules::token`).
    pub lang: Cow<'a, str>,
    /// Fields sent as they are after the language, in order, at most `MAX_FIXED`.
    pub fixed: Vec<NetField<'a>>,
}

/// A field a form always sends, such as `OpenRouter`'s keep-nothing `provider`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetField<'a> {
    /// Its name (`rules::token`).
    pub name: Cow<'a, str>,
    /// Its value (`rules::form_value`), at most `MAX_NAME_BYTES`.
    pub value: Cow<'a, str>,
}

/// The header that carries the key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetHeader<'a> {
    /// `Authorization: Bearer` and the key.
    Bearer,
    /// The key alone under this header's name (`rules::token`).
    Named(Cow<'a, str>),
}

/// A service row's limits on one request, each within its `NET_*` range.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NetLimits {
    /// The most megabytes of raw sound a request carries.
    pub max_mb: u32,
    /// The most seconds of sound a request carries.
    pub max_s: u16,
    /// The most requests a minute.
    pub per_min: u16,
    /// The most requests at once.
    pub at_once: u8,
}

impl NetLimits {
    /// The most bytes one piece may hold within these limits (`piece_bytes`).
    #[must_use]
    pub fn piece_bytes(&self) -> usize {
        piece_bytes(self.max_mb, self.max_s)
    }
}

/// What a run sends besides its call: the model, the language, the row's limits, the tries and each piece's size.
#[derive(Clone, PartialEq, Eq)]
pub struct NetRun<'a> {
    /// The model's id (`rules::model`); `None` sends none, which `ShapeCode::needs_model` shapes refuse.
    pub model: Option<Cow<'a, str>>,
    /// The model's code for the language in front (`rules::token`).
    pub lang: Cow<'a, str>,
    /// The row's limits.
    pub limits: NetLimits,
    /// Tries per piece before its fault is told, within `NET_TRIES`.
    pub tries: u8,
    /// Each piece's size in bytes, in order: whole samples within `limits.piece_bytes`, at most `MAX_RECORD_BYTES`
    /// together; the sound follows in `Sound` asks.
    pub sizes: Vec<u32>,
}

/// Why a run or a key's test stopped; a fixed reason, never a service's words. The host tells each as the
/// `CloudFault` of the same name; `CloudFault::Range` for an ask is also the host's own, refused before any frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NetFault {
    /// The service refused the key: wrong, revoked or missing.
    Key,
    /// The account has no credit left.
    Credit,
    /// The service asks for a step on its site first, such as a verification, a payment method or a spending cap.
    Account,
    /// The free allowance is used up.
    Allowance {
        /// Seconds until it renews, within `NET_RENEW_S`, when the service tells.
        renews_s: Option<u32>,
    },
    /// Too many requests, still after one wait.
    Rate,
    /// A piece was too big for the service.
    TooBig,
    /// The service's model does not take the language code.
    Language,
    /// The service is not offered in the user's country or region.
    Region,
    /// The key test's reply marks a management key, which manages keys but never transcribes.
    Admin,
    /// The service sent no text.
    Empty,
    /// The reply was not the shape's: a sign-in page, a redirect to another host, or one past `MAX_NET_REPLY_BYTES`.
    Strange,
    /// The network failed, or the service is down.
    Network,
    /// The service gave no answer within the call's wait.
    Timeout,
    /// The shape refused a value the frames let through, such as a `/` in a model id it puts in its address.
    Range,
    /// The job failed inside, reached its memory cap, or got an ask out of step.
    Gone,
}

impl NetFault {
    /// The fault of `code`; `renews_s` counts only for the allowance.
    pub(crate) const fn of(code: FaultCode, renews_s: Option<u32>) -> Self {
        match code {
            FaultCode::Key => Self::Key,
            FaultCode::Credit => Self::Credit,
            FaultCode::Account => Self::Account,
            FaultCode::Allowance => Self::Allowance { renews_s },
            FaultCode::Rate => Self::Rate,
            FaultCode::TooBig => Self::TooBig,
            FaultCode::Language => Self::Language,
            FaultCode::Region => Self::Region,
            FaultCode::Admin => Self::Admin,
            FaultCode::Empty => Self::Empty,
            FaultCode::Strange => Self::Strange,
            FaultCode::Network => Self::Network,
            FaultCode::Timeout => Self::Timeout,
            FaultCode::Range => Self::Range,
            FaultCode::Gone => Self::Gone,
        }
    }

    /// This fault's code.
    pub(crate) const fn code(self) -> FaultCode {
        match self {
            Self::Key => FaultCode::Key,
            Self::Credit => FaultCode::Credit,
            Self::Account => FaultCode::Account,
            Self::Allowance { .. } => FaultCode::Allowance,
            Self::Rate => FaultCode::Rate,
            Self::TooBig => FaultCode::TooBig,
            Self::Language => FaultCode::Language,
            Self::Region => FaultCode::Region,
            Self::Admin => FaultCode::Admin,
            Self::Empty => FaultCode::Empty,
            Self::Strange => FaultCode::Strange,
            Self::Network => FaultCode::Network,
            Self::Timeout => FaultCode::Timeout,
            Self::Range => FaultCode::Range,
            Self::Gone => FaultCode::Gone,
        }
    }
}

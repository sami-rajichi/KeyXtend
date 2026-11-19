//! A service row's request (ADR-0047 decision 5): its shape, the upload form's field names and the header that carries
//! the key, each refused when it could break a request's head or body.

use crate::ports::TAG_LEN;
pub use kx_ipc::MAX_FIXED;
use kx_ipc::ShapeCode;
use kx_ipc::rules::{form_value, token};

/// The longest fixed field's value, in bytes: the frames' cap on a name.
pub const MAX_FIELD_VALUE: usize = kx_ipc::MAX_NAME_BYTES;

/// The request shape a service row names; `speech-net` holds one adapter per shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudShape {
    /// The `OpenAI`-style upload form, with the row's field names: Groq, `OpenAI`, Mistral, Together, Lemonfox,
    /// `DeepInfra`, `OpenRouter` and the Custom row.
    Form(FormFields),
    /// `ElevenLabs`' own form: the row's language field and fixed fields, and nothing else.
    Eleven(OwnFields),
    /// Gemini's single request, with the sound inline.
    Gemini,
}

impl CloudShape {
    /// True when a form's names and values fit; Gemini names no field.
    #[must_use]
    pub fn fits(&self) -> bool {
        match self {
            Self::Form(form) => form.fits(),
            Self::Eleven(own) => own.fits(),
            Self::Gemini => true,
        }
    }

    /// True when a run must name a model, as the frames' code says: `ElevenLabs` and Gemini have no default one.
    #[must_use]
    pub const fn needs_model(&self) -> bool {
        self.code().needs_model()
    }

    /// The frames' code for this shape.
    pub(super) const fn code(&self) -> ShapeCode {
        match self {
            Self::Form(_) => ShapeCode::Form,
            Self::Eleven(_) => ShapeCode::Eleven,
            Self::Gemini => ShapeCode::Gemini,
        }
    }
}

/// How an upload form names its fields; it sends the model, the language, the fixed fields, then the sound last, as
/// Together asks the model before the file. Its `paywall` and `region` are rules for reading the replies, never fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormFields {
    /// The language field's name, such as `language`.
    pub lang: String,
    /// The language goes as a list of one, its name followed by `[]`, as `OpenAI`'s `languages[]`.
    pub list: bool,
    /// Fields sent as they are after the language, in order, at most `MAX_FIXED`.
    pub fixed: Vec<FormField>,
    /// A 403 asks the account to pay, as Together's missing payment method or `OpenRouter`'s spending cap: the
    /// account's step, not a bad key.
    pub paywall: bool,
    /// The error code a reply gives where the service is not offered in the user's country or region, a plain name,
    /// such as `OpenAI`'s `unsupported_country_region_territory`; `None` for none.
    pub region: Option<String>,
}

impl FormFields {
    /// True when every name and the region code are plain names and every value fits.
    #[must_use]
    pub fn fits(&self) -> bool {
        let region = self.region.as_deref().is_none_or(name_ok);
        name_ok(&self.lang) && fixed_fit(&self.fixed) && region
    }
}

/// `ElevenLabs`' own form: it sends the model, the language under `lang`, the fixed fields, then the sound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnFields {
    /// The language field's name, such as `language_code`.
    pub lang: String,
    /// Fields sent as they are after the language, in order, at most `MAX_FIXED`.
    pub fixed: Vec<FormField>,
}

impl OwnFields {
    /// True when every name is a plain name and every value fits.
    #[must_use]
    pub fn fits(&self) -> bool {
        name_ok(&self.lang) && fixed_fit(&self.fixed)
    }
}

/// True when `fixed` holds at most `MAX_FIXED` fields, each fitting.
fn fixed_fit(fixed: &[FormField]) -> bool {
    fixed.len() <= MAX_FIXED && fixed.iter().all(FormField::fits)
}

/// A field a form always sends, such as `OpenRouter`'s `provider` with `{"zdr":true}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormField {
    /// Its name, a plain name.
    pub name: String,
    /// Its value: visible ASCII and spaces, at most `MAX_FIELD_VALUE` bytes.
    pub value: String,
}

impl FormField {
    /// True for a plain name and a value of visible ASCII and spaces within `MAX_FIELD_VALUE` (`rules::form_value`).
    fn fits(&self) -> bool {
        let value = self.value.len() <= MAX_FIELD_VALUE && form_value(&self.value).is_ok();
        name_ok(&self.name) && value
    }
}

/// The header that carries the key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyHeader {
    /// `Authorization: Bearer` and the key, as the upload form's services take it.
    Bearer,
    /// The key alone under this header's name, such as `xi-api-key` or `Ocp-Apim-Subscription-Key`.
    Named(String),
}

impl KeyHeader {
    /// True for `Bearer` or a plain header name.
    #[must_use]
    pub fn fits(&self) -> bool {
        match self {
            Self::Bearer => true,
            Self::Named(name) => name_ok(name),
        }
    }
}

/// True for a name within `TAG_LEN` of ASCII letters, digits, `-` and `_` (`rules::token`), which no request's head
/// misreads.
fn name_ok(name: &str) -> bool {
    TAG_LEN.contains(&name.len()) && token(name).is_ok()
}

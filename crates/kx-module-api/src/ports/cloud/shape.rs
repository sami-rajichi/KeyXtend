//! A service row's request (ADR-0047 decision 5): its shape, the upload form's field names and the header that carries
//! the key, each refused when it could break a request's head or body.

use crate::ports::TAG_LEN;

/// The most fixed fields a form adds, such as `OpenRouter`'s keep-nothing `provider`.
pub const MAX_FIXED: usize = 4;
/// The longest fixed field's value, in bytes: the frames' cap on a name.
pub const MAX_FIELD_VALUE: usize = kx_ipc::MAX_NAME_BYTES;
/// The marks a field's or a header's name may hold besides ASCII letters and digits.
const NAME_MARKS: &[u8] = b"-_";
/// The one byte a fixed field's value may hold besides visible ASCII.
const SPACE: u8 = b' ';

/// The request shape a service row names; `speech-net` holds one adapter per shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudShape {
    /// The `OpenAI`-style upload form, with the row's field names: Groq, `OpenAI`, Mistral, Together, Lemonfox,
    /// `DeepInfra`, `OpenRouter` and the Custom row.
    Form(FormFields),
    /// Azure's fast transcription: one upload with a settings part, at the address the user types.
    Azure,
    /// `ElevenLabs`' own form.
    Eleven,
    /// Gemini's single request, with the sound inline.
    Gemini,
}

impl CloudShape {
    /// True when a form's names and values fit; the other shapes name no field.
    pub(super) fn fits(&self) -> bool {
        match self {
            Self::Form(form) => form.fits(),
            Self::Azure | Self::Eleven | Self::Gemini => true,
        }
    }
}

/// How an upload form names its fields; it sends the model, the language, the fixed fields, then the sound last, as
/// Together asks the model before the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormFields {
    /// The language field's name, such as `language`.
    pub lang: String,
    /// The language goes as a list of one, its name followed by `[]`, as `OpenAI`'s `languages[]`.
    pub list: bool,
    /// Fields sent as they are after the language, in order, at most `MAX_FIXED`.
    pub fixed: Vec<FormField>,
}

impl FormFields {
    /// True when every name is a plain name and every value fits.
    fn fits(&self) -> bool {
        let fixed = self.fixed.len() <= MAX_FIXED && self.fixed.iter().all(FormField::fits);
        name_ok(&self.lang) && fixed
    }
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
    /// True for a plain name and a value of visible ASCII and spaces within `MAX_FIELD_VALUE`.
    fn fits(&self) -> bool {
        let plain = |b: u8| b.is_ascii_graphic() || b == SPACE;
        let value = self.value.len() <= MAX_FIELD_VALUE && self.value.bytes().all(plain);
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
    pub(super) fn fits(&self) -> bool {
        match self {
            Self::Bearer => true,
            Self::Named(name) => name_ok(name),
        }
    }
}

/// True for a name within `TAG_LEN` of ASCII letters, digits and `NAME_MARKS`, which no request's head misreads.
fn name_ok(name: &str) -> bool {
    let plain = |b: u8| b.is_ascii_alphanumeric() || NAME_MARKS.contains(&b);
    TAG_LEN.contains(&name.len()) && name.bytes().all(plain)
}

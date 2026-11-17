//! A row's request shape and a form's fields: the shapes `speech-net` knows by name, a form's language field and its
//! fixed fields, each asking only for plain text or JSON without timings (ruling 2026-11-17).

use kx_module_api::ports::{CloudShape, FormField, FormFields, MAX_FIXED};
use serde::Deserialize;

use super::ServiceError;

/// The shapes `speech-net` holds an adapter for (ADR-0047 decision 5).
#[derive(Copy, Clone, PartialEq, Eq)]
enum Kind {
    /// The `OpenAI`-style upload form, with the row's fields.
    Form,
    /// Azure's fast transcription.
    Azure,
    /// `ElevenLabs`' own form.
    Eleven,
    /// Gemini's single request.
    Gemini,
}

/// Each shape's name in the table, and whether its request carries the sound as Base64 text, a third larger.
const SHAPES: [(&str, Kind, bool); 4] = [
    ("form", Kind::Form, false),
    ("azure", Kind::Azure, false),
    ("eleven", Kind::Eleven, false),
    ("gemini", Kind::Gemini, true),
];

/// What a form's fixed fields may ask about the reply.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Reply {
    /// Fields that name the reply's format.
    formats: Vec<String>,
    /// The formats they may ask for.
    plain: Vec<String>,
    /// Fields that ask for timings.
    timings: Vec<String>,
    /// The values that ask for none.
    untimed: Vec<String>,
}

impl Reply {
    /// True unless `field` asks for a format that is not plain, or for timings.
    fn allows(&self, field: &FormField) -> bool {
        let names = |list: &[String]| list.contains(&field.name);
        let takes = |list: &[String]| list.contains(&field.value);
        let format = !names(&self.formats) || takes(&self.plain);
        let timing = !names(&self.timings) || takes(&self.untimed);
        format && timing
    }
}

/// A form's fields as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawForm {
    /// The language field's name.
    lang: String,
    /// The language goes as a list of one.
    list: bool,
    /// Fields sent as they are, in order.
    #[serde(default)]
    fixed: Vec<RawField>,
}

/// A fixed field as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawField {
    /// Its name.
    name: String,
    /// Its value.
    value: String,
}

impl RawField {
    /// The field as the port holds it.
    pub(super) fn field(&self) -> FormField {
        FormField {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

/// The shape `name` names, with `form`'s fields for a form, whose fixed fields and `keep` must fit one request.
pub(super) fn shape(
    name: &str,
    form: Option<&RawForm>,
    reply: &Reply,
    keep: Option<&FormField>,
) -> Result<CloudShape, ServiceError> {
    let kind = SHAPES
        .iter()
        .find(|(n, ..)| *n == name)
        .map(|(_, kind, _)| *kind)
        .ok_or(ServiceError::Shape)?;
    match (kind, form, keep) {
        (Kind::Form, Some(form), _) => fields(form, reply, keep),
        (Kind::Azure, None, None) => Ok(CloudShape::Azure),
        (Kind::Eleven, None, None) => Ok(CloudShape::Eleven),
        (Kind::Gemini, None, None) => Ok(CloudShape::Gemini),
        _ => Err(ServiceError::Form),
    }
}

/// A form's checked fields: plain names, values that fit, at most `MAX_FIXED` with `keep`, and a plain reply.
pub(super) fn fields(
    form: &RawForm,
    reply: &Reply,
    keep: Option<&FormField>,
) -> Result<CloudShape, ServiceError> {
    let fixed: Vec<FormField> = form.fixed.iter().map(RawField::field).collect();
    let mut sent = fixed.clone();
    sent.extend(keep.cloned());
    let whole = CloudShape::Form(FormFields {
        lang: form.lang.clone(),
        list: form.list,
        fixed: sent.clone(),
    });
    if !whole.fits() || sent.len() > MAX_FIXED {
        return Err(ServiceError::Form);
    }
    if !sent.iter().all(|f| reply.allows(f)) {
        return Err(ServiceError::Reply);
    }
    Ok(CloudShape::Form(FormFields {
        lang: form.lang.clone(),
        list: form.list,
        fixed,
    }))
}

/// True when shape `name` carries the sound as Base64 text.
pub(super) fn base64(name: &str) -> bool {
    SHAPES.iter().any(|(n, _, b64)| *n == name && *b64)
}

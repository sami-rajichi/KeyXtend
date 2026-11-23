//! A row's request shape and a form's fields: the shapes `speech-net` knows by name, a form's language field and its
//! fixed fields, each asking only for plain text or JSON without timings (ruling 2026-11-17).

use kx_module_api::ports::{CloudShape, FormField, FormFields, KeyHeader, MAX_FIXED, OwnFields};
use serde::Deserialize;

use super::ServiceError;

/// The shapes `speech-net` holds an adapter for (ADR-0047 decision 5, three since ADR-0050, four since ADR-0051).
#[derive(Copy, Clone, PartialEq, Eq)]
enum Kind {
    /// The `OpenAI`-style upload form, with the row's fields.
    Form,
    /// `ElevenLabs`' own form.
    Eleven,
    /// Gemini's single request.
    Gemini,
    /// `AssemblyAI`'s Sync request.
    Assembly,
}

/// Each shape's name in the table, and whether its request carries the sound as Base64 text, a third larger.
const SHAPES: [(&str, Kind, bool); 4] = [
    ("form", Kind::Form, false),
    ("eleven", Kind::Eleven, false),
    ("gemini", Kind::Gemini, true),
    ("assembly", Kind::Assembly, false),
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

/// What every form of a row sends beside its own fields, and the rules its reply keeps.
pub(super) struct Sent<'a> {
    /// What a form may ask about its reply.
    reply: &'a Reply,
    /// The row's keep-nothing field, sent while its switch is on.
    pub(super) keep: Option<&'a FormField>,
    /// A 403 is the row's paywall.
    paywall: bool,
    /// The error code by which a reply says the service is not offered in the user's country or region.
    region: Option<&'a str>,
    /// The key test's reply fields, any one true marking a management key.
    admin: &'a [String],
}

impl<'a> Sent<'a> {
    /// What a row with `keep`, `paywall`, `region` and `admin` sends, under `reply`'s rules.
    pub(super) fn new(
        reply: &'a Reply,
        keep: Option<&'a FormField>,
        paywall: bool,
        region: Option<&'a str>,
        admin: &'a [String],
    ) -> Self {
        Self {
            reply,
            keep,
            paywall,
            region,
            admin,
        }
    }

    /// True when the row adds no keep field and no reply rule, which only an upload form takes.
    fn bare(&self) -> bool {
        self.keep.is_none() && !self.paywall && self.region.is_none() && self.admin.is_empty()
    }
}

/// The shape `name` names, with `form`'s fields for an upload form or `ElevenLabs`' own, whose fixed fields and keep
/// field must fit one request; Gemini takes no form, `ElevenLabs` no list, and `AssemblyAI` only its plain
/// `model_header`, naming neither `key` nor a header the request sets. Only an upload form takes a keep field, a
/// paywall, a region code or reply paths.
pub(super) fn shape(
    name: &str,
    form: Option<&RawForm>,
    model_header: Option<&str>,
    key: &KeyHeader,
    sent: &Sent<'_>,
) -> Result<CloudShape, ServiceError> {
    let kind = SHAPES
        .iter()
        .find(|(n, ..)| *n == name)
        .map(|(_, kind, _)| *kind)
        .ok_or(ServiceError::Shape)?;
    match (kind, form, model_header, sent.bare()) {
        (Kind::Form, Some(form), None, _) => fields(form, sent),
        (Kind::Eleven, Some(form), None, true) if !form.list => {
            own(form, sent).map(CloudShape::Eleven)
        }
        (Kind::Gemini, None, None, true) => Ok(CloudShape::Gemini),
        (Kind::Assembly, None, Some(name), true) => {
            let model_header = name.to_owned();
            let shape = CloudShape::Assembly { model_header };
            let fits = |s: &CloudShape| s.fits() && s.fits_beside(key);
            Some(shape).filter(fits).ok_or(ServiceError::Form)
        }
        _ => Err(ServiceError::Form),
    }
}

/// An upload form's checked fields as its shape (`checked`).
pub(super) fn fields(form: &RawForm, sent: &Sent<'_>) -> Result<CloudShape, ServiceError> {
    checked(form, sent).map(CloudShape::Form)
}

/// `ElevenLabs`' own checked fields: its language field and fixed fields, as an upload form's are checked.
fn own(form: &RawForm, sent: &Sent<'_>) -> Result<OwnFields, ServiceError> {
    let FormFields { lang, fixed, .. } = checked(form, sent)?;
    Ok(OwnFields { lang, fixed })
}

/// A form's checked fields: plain names, values that fit, at most `MAX_FIXED` with the keep field, and a plain reply;
/// it carries the row's paywall, its region code, a plain name, and its reply paths, plain names parted by dots.
fn checked(form: &RawForm, sent: &Sent<'_>) -> Result<FormFields, ServiceError> {
    let fixed: Vec<FormField> = form.fixed.iter().map(RawField::field).collect();
    let mut all = fixed.clone();
    all.extend(sent.keep.cloned());
    let shape = |fixed| FormFields {
        lang: form.lang.clone(),
        list: form.list,
        fixed,
        paywall: sent.paywall,
        region: sent.region.map(str::to_owned),
        admin: sent.admin.to_vec(),
    };
    if !shape(all.clone()).fits() || all.len() > MAX_FIXED {
        return Err(ServiceError::Form);
    }
    if !all.iter().all(|f| sent.reply.allows(f)) {
        return Err(ServiceError::Reply);
    }
    Ok(shape(fixed))
}

/// True when shape `name` carries the sound as Base64 text.
pub(super) fn base64(name: &str) -> bool {
    SHAPES.iter().any(|(n, _, b64)| *n == name && *b64)
}

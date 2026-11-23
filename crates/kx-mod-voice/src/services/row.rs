//! One row of the services table: its checks in order, and what a request to it takes: its shape for a model, its key
//! header, limits and names.

use kx_module_api::ports::{CloudLimits, CloudShape, FormField, HttpsUrl, KeyHeader, ServiceName};
use serde::Deserialize;
use thiserror::Error;

use super::{
    Address, Facts, KeyTest, Model, ModelError, Rules, address, facts, form, limits, model,
};
use crate::plain::one_line;
use crate::services::FactError;

/// The table's word for the `Authorization: Bearer` header.
const BEARER: &str = "bearer";

/// Why a row was refused, naming the rule it breaks.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ServiceError {
    /// The name is empty or holds a hidden mark, or is missing on a row other than the one Custom row.
    #[error(
        "its name is empty, holds a hidden mark, or is missing on a row other than the one custom row"
    )]
    Name,
    /// The shape is not one `speech-net` knows.
    #[error("its shape is not one speech-net knows")]
    Shape,
    /// A form's fields or a model header are missing, odd or too many, given to a shape without them, or a model header
    /// names one the request sets.
    #[error(
        "its form fields or model header are missing, odd, too many, misplaced, or name a header the request sets"
    )]
    Form,
    /// A fixed field asks for a reply that is not plain, or for timings (ruling 2026-11-17).
    #[error("a fixed field asks for timings or a reply that is not plain text or JSON")]
    Reply,
    /// The address is not HTTPS on a host name.
    #[error("its address is not https on a host name")]
    Address,
    /// The address or the test reaches a host other than the row's own, or a typed row's domain is no host entry.
    #[error("its address or test reaches a host other than its own")]
    Host,
    /// The test is not HTTPS, or not a path under a typed host.
    #[error("its key test is not https, or not a path under a typed host")]
    Test,
    /// The key header is neither bearer nor a plain header name.
    #[error("its key header is neither bearer nor a plain header name")]
    Header,
    /// The page where keys are made is missing on a named row, or is not HTTPS.
    #[error("its keys page is missing or not https")]
    Keys,
    /// A limit is outside the frames' range, or the service's cap is zero.
    #[error("a limit is outside the frames' range")]
    Limits,
    /// A piece leaves no room for the WAV header and the form under the service's own cap.
    #[error("a piece leaves no room for the wav header and the form under the service's cap")]
    Room,
    /// It offers no model and no Other line, or more models than the table allows.
    #[error("it offers no model and no other line, or more models than the table allows")]
    Models,
    /// A model breaks a rule.
    #[error("its model {id} is refused: {why}")]
    Model {
        /// The model's id.
        id: String,
        /// The rule it breaks.
        why: ModelError,
    },
    /// A fact breaks a rule.
    #[error("its fact {0} is refused: {1}")]
    Fact(&'static str, FactError),
    /// The terms number is zero.
    #[error("its terms number is zero")]
    Terms,
}

/// A row as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one on-or-off field of a row as the table writes it."
)]
pub(super) struct RawService {
    /// The id `services`, `picks` and the key's blob use.
    pub(super) id: String,
    /// The shown name; none for Custom.
    #[serde(default)]
    name: Option<String>,
    /// The request shape's name.
    shape: String,
    /// A form's fields.
    #[serde(default)]
    form: Option<form::RawForm>,
    /// The header that carries the model, on `AssemblyAI`'s shape.
    #[serde(default)]
    model_header: Option<String>,
    /// The field sent while the keep-nothing switch is on.
    #[serde(default)]
    keep: Option<form::RawField>,
    /// A 403 is the form's paywall: a missing payment method or a spending cap, so it tells the account's step.
    #[serde(default)]
    paywall: bool,
    /// The error code by which a form's reply says the service is not offered in the user's country or region.
    #[serde(default)]
    region: Option<String>,
    /// A form's key-test reply fields, each a dotted path; any one true marks a management key.
    #[serde(default)]
    admin: Vec<String>,
    /// The only host, or the domain a typed host must be under.
    #[serde(default)]
    host: Option<String>,
    /// Where the pieces go, or "typed".
    address: String,
    /// The key's test address or path.
    #[serde(default)]
    test: Option<String>,
    /// The header that carries the key.
    header: String,
    /// The page where keys are made.
    #[serde(default)]
    keys: Option<String>,
    /// The service's cap and the pieces' limits.
    limits: limits::RawLimits,
    /// An Other model line takes a typed id.
    #[serde(default)]
    other: bool,
    /// The terms number.
    terms: u16,
    /// Shown on the Cloud page.
    offered: bool,
    /// No real key has checked the connection yet.
    untested: bool,
    /// The models offered.
    #[serde(default)]
    models: Vec<model::RawModel>,
    /// The card's facts.
    facts: facts::RawFacts,
}

/// A checked row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Service {
    /// The id `services`, `picks` and the key's blob use.
    pub id: String,
    /// The shown name; `None` for Custom, which the ui names in the user's language.
    pub name: Option<String>,
    /// The request shape, with the row's form fields.
    pub shape: CloudShape,
    /// The field a form adds while the keep-nothing switch is on; its presence gives the row that switch.
    pub keep: Option<FormField>,
    /// Where the pieces go.
    pub address: Address,
    /// The key's test.
    pub test: KeyTest,
    /// The header that carries the key.
    pub header: KeyHeader,
    /// The page where keys are made; `None` for Custom.
    pub keys: Option<HttpsUrl>,
    /// The pieces' limits.
    pub limits: CloudLimits,
    /// The models offered, at most the table's count.
    pub models: Vec<Model>,
    /// An Other model line takes a typed id, sent with the keyboard's language code as it is.
    pub other: bool,
    /// Raised when its training, retention or region fact changes, so the user confirms again.
    pub terms: u16,
    /// Shown on the Cloud page.
    pub offered: bool,
    /// No real key has checked the connection yet.
    pub untested: bool,
    /// The card's facts with their day and source.
    pub facts: Facts,
}

impl Service {
    /// Checks `raw` against `rules` and keeps it.
    pub(super) fn checked(raw: RawService, rules: &Rules<'_>) -> Result<Self, ServiceError> {
        let address = address::address(&raw.address, raw.host.as_deref())?;
        name_ok(raw.name.as_deref(), &address)?;
        let keep = raw.keep.as_ref().map(form::RawField::field);
        let region = raw.region.as_deref();
        let sent = form::Sent::new(rules.reply, keep.as_ref(), raw.paywall, region, &raw.admin);
        let header = header(&raw.header)?;
        let model_header = raw.model_header.as_deref();
        let shape = form::shape(&raw.shape, raw.form.as_ref(), model_header, &header, &sent)?;
        let test = address::test(raw.test.as_deref(), &address)?;
        let named = raw.name.is_some();
        let keys = keys_page(raw.keys.as_deref(), named)?;
        let limits = limits::checked(&raw.limits, form::base64(&raw.shape), rules.room)?;
        let row = model::Row {
            shape: &shape,
            sent: &sent,
            other: raw.other,
        };
        let models = model::checked(raw.models, &row, rules)?;
        let facts = facts::checked(raw.facts, named)?;
        if raw.terms == 0 {
            return Err(ServiceError::Terms);
        }
        Ok(Self {
            id: raw.id,
            name: raw.name,
            shape,
            keep,
            address,
            test,
            header,
            keys,
            limits,
            models,
            other: raw.other,
            terms: raw.terms,
            offered: raw.offered,
            untested: raw.untested,
            facts,
        })
    }

    /// The listed model `id` names.
    #[must_use]
    pub fn model(&self, id: &str) -> Option<&Model> {
        self.models.iter().find(|m| m.id == id)
    }

    /// The name a line shows: the row's own, or Custom's in the user's language.
    #[must_use]
    pub fn shown(&self) -> ServiceName {
        self.name
            .clone()
            .map_or(ServiceName::Custom, ServiceName::Named)
    }

    /// The request's shape for `model` (a typed one is `None`): the model's own form, such as `OpenAI`'s `languages[]`
    /// list, else the row's, with the keep-nothing field last while `keep` is on.
    #[must_use]
    pub fn shape_for(&self, model: Option<&Model>, keep: bool) -> CloudShape {
        let mut shape = model
            .and_then(|m| m.shape.clone())
            .unwrap_or_else(|| self.shape.clone());
        if let (CloudShape::Form(form), Some(field), true) = (&mut shape, &self.keep, keep) {
            form.fixed.push(field.clone());
        }
        shape
    }
}

/// Fine for a one-line name, or none on the Custom row: typed, on any host name.
fn name_ok(name: Option<&str>, address: &Address) -> Result<(), ServiceError> {
    let custom = matches!(address, Address::Typed { under: None });
    let good = name.map_or(custom, one_line);
    if good {
        Ok(())
    } else {
        Err(ServiceError::Name)
    }
}

/// The HTTPS page where keys are made, which a named row must give and Custom has none of.
fn keys_page(text: Option<&str>, named: bool) -> Result<Option<HttpsUrl>, ServiceError> {
    match text {
        Some(text) => HttpsUrl::new(text).map(Some).ok_or(ServiceError::Keys),
        None if named => Err(ServiceError::Keys),
        None => Ok(None),
    }
}

/// The header `text` names: bearer, or a plain header's name.
fn header(text: &str) -> Result<KeyHeader, ServiceError> {
    let header = if text == BEARER {
        KeyHeader::Bearer
    } else {
        KeyHeader::Named(text.to_owned())
    };
    if header.fits() {
        Ok(header)
    } else {
        Err(ServiceError::Header)
    }
}

//! A row's models (design "Connections and services"): at most the table's count, each with its id, shown name and the
//! code it wants per keyboard language (its own list or a table's set), maybe its own form (`OpenAI`'s `languages[]` beside `whisper-1`'s `language`,
//! ruling 2026-11-17), whether the request names it, and whether every host of it keeps the voice.

use std::collections::BTreeMap;

use kx_module_api::ports::{CloudAsk, CloudShape, FormField, fits_iso};
use serde::Deserialize;
use thiserror::Error;

use super::{LangCodes, Rules, ServiceError, form};
use crate::models::one_line;

/// Why a model was refused, in the order the rules are checked.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ModelError {
    /// The id is not one the frames take.
    #[error("its id is not one the frames take")]
    Id,
    /// Another model of the row has the id.
    #[error("another model has its id")]
    Twice,
    /// The shown name is empty or holds a hidden mark.
    #[error("its name is empty or holds a hidden mark")]
    Name,
    /// It names no language, a language that is no ISO 639 code, or a code the frames refuse.
    #[error("its languages are missing or malformed")]
    Langs,
    /// It names a language set the table does not have.
    #[error("its language set is not in the table")]
    Set,
    /// Its own form is odd, too many with the keep field, or on a shape without a form.
    #[error("its own form is odd or on a shape without a form")]
    Form,
    /// Its own form asks for timings or a reply that is not plain text or JSON (ruling 2026-11-17).
    #[error("its own form asks for timings or a reply that is not plain text or json")]
    Reply,
    /// The request would name no model on a shape that needs one.
    #[error("it is unnamed on a shape that needs a model")]
    Unnamed,
    /// It is marked as kept on a row without the keep-nothing switch.
    #[error("it is marked as kept on a row without the keep-nothing switch")]
    Keeps,
}

/// A model as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawModel {
    /// The id the request and `picks` name.
    id: String,
    /// The shown name.
    name: String,
    /// Each keyboard language and the code the service wants for it, or the name of the table's set that holds them.
    langs: RawLangs,
    /// Its own form, in place of the row's.
    #[serde(default)]
    form: Option<form::RawForm>,
    /// The request names no model.
    #[serde(default)]
    unnamed: bool,
    /// Every host of it keeps the voice.
    #[serde(default)]
    keeps: bool,
}

/// A model's languages as written: the name of one of the table's sets, or its own list.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawLangs {
    /// The name of a set in the table's `langs`.
    Set(String),
    /// Its own list.
    Own(LangCodes),
}

/// A checked model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    /// The id the request and `picks` name.
    pub id: String,
    /// The shown name.
    pub name: String,
    /// Each keyboard language (ISO 639) and the code the service wants for it.
    pub langs: LangCodes,
    /// Its own form, in place of the row's.
    pub shape: Option<CloudShape>,
    /// The request names no model, as Lemonfox's and Azure classic's take none.
    pub unnamed: bool,
    /// Every host of it keeps the voice, so the keep-nothing switch makes it unavailable.
    pub keeps: bool,
}

impl Model {
    /// The code the service wants for keyboard language `lang`, if the model knows it.
    #[must_use]
    pub fn code(&self, lang: &str) -> Option<&str> {
        self.langs.get(lang).map(String::as_str)
    }
}

/// What a row's models are checked against.
pub(super) struct Row<'a> {
    /// The row's shape.
    pub(super) shape: &'a CloudShape,
    /// The row's keep-nothing field.
    pub(super) keep: Option<&'a FormField>,
    /// The row has an Other model line.
    pub(super) other: bool,
}

/// The row's models: at most `max_models`, at least one without an Other line, each checked and with its own id.
pub(super) fn checked(
    raw: Vec<RawModel>,
    row: &Row<'_>,
    rules: &Rules<'_>,
) -> Result<Vec<Model>, ServiceError> {
    if raw.len() > rules.max_models || (raw.is_empty() && !row.other) {
        return Err(ServiceError::Models);
    }
    let mut models: Vec<Model> = Vec::with_capacity(raw.len());
    for one in raw {
        let id = one.id.clone();
        let fail = |why| ServiceError::Model {
            id: id.clone(),
            why,
        };
        if models.iter().any(|m| m.id == id) {
            return Err(fail(ModelError::Twice));
        }
        models.push(one.checked(row, rules).map_err(fail)?);
    }
    Ok(models)
}

impl RawModel {
    /// Checks the model against its row and keeps it.
    fn checked(self, row: &Row<'_>, rules: &Rules<'_>) -> Result<Model, ModelError> {
        let langs = self.langs.codes(rules.sets)?;
        let pair = |(lang, code): (&String, &String)| fits_iso(lang) && CloudAsk::lang_fits(code);
        let checks = [
            (CloudAsk::model_fits(&self.id), ModelError::Id),
            (one_line(&self.name), ModelError::Name),
            (
                !langs.is_empty() && langs.iter().all(pair),
                ModelError::Langs,
            ),
            (
                !(self.unnamed && row.shape.needs_model()),
                ModelError::Unnamed,
            ),
            (!self.keeps || row.keep.is_some(), ModelError::Keeps),
        ];
        if let Some(why) = checks
            .into_iter()
            .find_map(|(ok, why)| (!ok).then_some(why))
        {
            return Err(why);
        }
        let shape = self.own_shape(row, rules)?;
        Ok(Model {
            id: self.id,
            name: self.name,
            langs,
            shape,
            unnamed: self.unnamed,
            keeps: self.keeps,
        })
    }

    /// Its own form as a request shape, on a row whose shape is a form; a reply it may not ask for keeps its reason.
    fn own_shape(
        &self,
        row: &Row<'_>,
        rules: &Rules<'_>,
    ) -> Result<Option<CloudShape>, ModelError> {
        match (&self.form, row.shape) {
            (None, _) => Ok(None),
            (Some(form), CloudShape::Form(_)) => match form::fields(form, rules.reply, row.keep) {
                Ok(shape) => Ok(Some(shape)),
                Err(ServiceError::Reply) => Err(ModelError::Reply),
                Err(_) => Err(ModelError::Form),
            },
            (Some(_), _) => Err(ModelError::Form),
        }
    }
}

impl RawLangs {
    /// The codes: its own list, or the table's set it names.
    fn codes(&self, sets: &BTreeMap<String, LangCodes>) -> Result<LangCodes, ModelError> {
        match self {
            Self::Own(codes) => Ok(codes.clone()),
            Self::Set(name) => sets.get(name).cloned().ok_or(ModelError::Set),
        }
    }
}

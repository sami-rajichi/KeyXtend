//! A row's models (design "Connections and services"): at most the table's count, each with its id, shown name and the
//! code it wants per keyboard language, maybe its own form (`OpenAI`'s `languages[]` beside `whisper-1`'s `language`,
//! ruling 2026-11-17), whether the request names it, and whether every host of it keeps the voice.

use std::collections::BTreeMap;

use kx_module_api::ports::{CloudAsk, CloudShape, FormField, fits_iso};
use serde::Deserialize;
use thiserror::Error;

use super::{Rules, ServiceError, form};
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
    /// Its own form is odd, too many with the keep field, or on a shape without a form.
    #[error("its own form is odd or on a shape without a form")]
    Form,
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
    /// Each keyboard language and the code the service wants for it.
    langs: BTreeMap<String, String>,
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

/// A checked model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    /// The id the request and `picks` name.
    pub id: String,
    /// The shown name.
    pub name: String,
    /// Each keyboard language (ISO 639) and the code the service wants for it.
    pub langs: BTreeMap<String, String>,
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
        let pair = |(lang, code): (&String, &String)| fits_iso(lang) && CloudAsk::lang_fits(code);
        let langs = !self.langs.is_empty() && self.langs.iter().all(pair);
        let checks = [
            (CloudAsk::model_fits(&self.id), ModelError::Id),
            (one_line(&self.name), ModelError::Name),
            (langs, ModelError::Langs),
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
        let shape = match (&self.form, row.shape) {
            (None, _) => None,
            (Some(form), CloudShape::Form(_)) => {
                Some(form::fields(form, rules.reply, row.keep).map_err(|_| ModelError::Form)?)
            }
            (Some(_), _) => return Err(ModelError::Form),
        };
        Ok(Model {
            id: self.id,
            name: self.name,
            langs: self.langs,
            shape,
            unnamed: self.unnamed,
            keeps: self.keeps,
        })
    }
}

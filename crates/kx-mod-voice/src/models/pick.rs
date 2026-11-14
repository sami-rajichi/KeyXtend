//! The user's pick per language (`picks`): a row that knows the language or a model file by name. Anything else, such
//! as a row no longer in the table, takes the language's default (plan edge case 19).

use std::collections::BTreeMap;

use super::{Row, Table, file_ok};

/// What a pick names.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Pick<'a> {
    /// A row of the table.
    Row(&'a Row),
    /// A model file in the folder, by its name; an other file unless its fingerprint is a row's.
    File(&'a str),
}

impl Table {
    /// What `value` names for `lang`: a row that knows it, or a model file's name; `None` for anything else.
    #[must_use]
    pub fn pick<'a>(&'a self, lang: &str, value: &'a str) -> Option<Pick<'a>> {
        if let Some(row) = self.get(value) {
            return row.code(lang).is_some().then_some(Pick::Row(row));
        }
        file_ok(value, &self.exts).then_some(Pick::File(value))
    }

    /// The model used for `lang`: the user's pick when it is good, else the language's default; `None` when no model
    /// knows the language.
    #[must_use]
    pub fn used<'a>(&'a self, lang: &str, picks: &'a BTreeMap<String, String>) -> Option<Pick<'a>> {
        let picked = picks.get(lang).and_then(|value| self.pick(lang, value));
        picked.or_else(|| self.default_for(lang).map(Pick::Row))
    }
}

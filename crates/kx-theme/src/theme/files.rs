//! The themes' parts as files: each must parse alone and hold only its own table, and an error names the part at
//! fault, never the joined text.

use serde::Deserialize;

use super::{ID, Theme, parse, twice};
use crate::ThemeError;
use crate::common::Common;

/// One part of `tokens/themes/`: its name (`common` or a theme's id), its file for errors and its text.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Part {
    pub(crate) name: &'static str,
    pub(crate) file: &'static str,
    pub(crate) text: &'static str,
}

/// The tables a part may hold, as the joined file names them; a stray one is refused at its line in the part.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Own {
    #[serde(default)]
    theme: Vec<toml::Table>,
    common: Option<toml::Table>,
}

/// The themes in `joined`, the text of `parts` in order, the shared colours first. An error names the part that does
/// not hold only its own table, else the theme its reason starts with, else the shared colours.
pub(crate) fn load(parts: &[Part], joined: &str) -> Result<(Vec<Theme>, Common), ThemeError> {
    for (at, p) in parts.iter().enumerate() {
        own(p, at == 0, &parts[..at]).map_err(|why| ThemeError { file: p.file, why })?;
    }
    parse(joined).map_err(|why| ThemeError {
        file: file_of(parts, &why),
        why,
    })
}

/// Whether part `p` parses alone and holds only its own table: `[common]` when it is `first`, else one `[[theme]]`
/// whose id is its name, which none of the `earlier` parts has.
fn own(p: &Part, first: bool, earlier: &[Part]) -> Result<(), String> {
    let Own { theme, common } = toml::from_str(p.text).map_err(|e| e.to_string())?;
    let name = p.name;
    if first {
        let only = theme.is_empty() && common.is_some();
        return only
            .then_some(())
            .ok_or_else(|| format!("{name}: this part holds the [common] table and nothing else"));
    }
    if earlier.iter().any(|q| q.name == name) {
        return Err(twice(name));
    }
    let id = match theme.as_slice() {
        [t] => t.get(ID).and_then(toml::Value::as_str),
        _ => None,
    };
    (common.is_none() && id == Some(name))
        .then_some(())
        .ok_or_else(|| {
            format!("{name}: this part holds one [[theme]] with {ID} = \"{name}\" and nothing else")
        })
}

/// The file of the part whose name starts `why` before a `:` or `.`, else the first part's; none for no parts.
fn file_of(parts: &[Part], why: &str) -> &'static str {
    let names = |p: &&Part| {
        let rest = why.strip_prefix(p.name);
        rest.is_some_and(|r| r.starts_with([':', '.']))
    };
    parts
        .iter()
        .find(names)
        .or(parts.first())
        .map_or("", |p| p.file)
}

#[cfg(test)]
mod tests;

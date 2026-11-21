//! The themes' parts as files: each must parse alone, and an error names the part at fault, never the joined text.

use super::{Theme, parse};
use crate::ThemeError;
use crate::common::Common;

/// One part of `tokens/themes/`: its name (`common` or a theme's id), its file for errors and its text.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Part {
    pub(crate) name: &'static str,
    pub(crate) file: &'static str,
    pub(crate) text: &'static str,
}

/// The themes in `joined`, the text of `parts` in order; an error names the part that does not parse alone, else the
/// theme its reason starts with, else the shared colours, which come first.
pub(crate) fn load(parts: &[Part], joined: &str) -> Result<(Vec<Theme>, Common), ThemeError> {
    for p in parts {
        let why = |e: toml::de::Error| ThemeError {
            file: p.file,
            why: e.to_string(),
        };
        toml::from_str::<toml::Table>(p.text).map_err(why)?;
    }
    parse(joined).map_err(|why| ThemeError {
        file: file_of(parts, &why),
        why,
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

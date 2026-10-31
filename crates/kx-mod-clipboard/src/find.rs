//! Search and filters (spec §7 "Search line"): a filter picks kinds, and the query, folded as the entries' search
//! text is, matches anywhere in it, ignoring case, accents, harakat, tatweel and the alef forms.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipKind, Filter};
use kx_module_api::text::fold_any;

use crate::entry::{Entry, flat};

/// What the list shows: the entries of a filter's kinds whose search text holds the folded query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Find {
    /// The kinds shown.
    filter: Filter,
    /// The query, folded.
    query: Redacted<String>,
}

impl Find {
    /// The find for `filter` and the search line `query`: on one line, folded, without the blanks around it.
    #[must_use]
    pub fn new(filter: Filter, query: &str) -> Self {
        let line: String = flat(query).collect();
        let query = fold_any(&line).trim().to_owned();
        Self {
            filter,
            query: Redacted::new(query),
        }
    }

    /// True when the list shows `entry`.
    #[must_use]
    pub fn keeps(&self, entry: &Entry) -> bool {
        let found = entry.search.expose().contains(self.query.expose().as_str());
        shows(self.filter, entry.kind) && found
    }

    /// The entries of `all` the list shows, in their order.
    #[must_use]
    pub fn list<'a>(&self, all: &'a [Entry]) -> Vec<&'a Entry> {
        all.iter().filter(|e| self.keeps(e)).collect()
    }
}

/// True when `filter` shows entries of `kind`: Text shows links too.
#[must_use]
pub fn shows(filter: Filter, kind: ClipKind) -> bool {
    match filter {
        Filter::All => true,
        Filter::Text => matches!(kind, ClipKind::Text | ClipKind::Link),
        Filter::Images => kind == ClipKind::Image,
        Filter::Files => kind == ClipKind::File,
        Filter::Media => kind == ClipKind::Media,
    }
}

#[cfg(test)]
mod tests;

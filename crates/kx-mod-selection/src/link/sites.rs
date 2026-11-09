//! The compiled-in translate sites (`sites.toml`): each one's id, name, link pattern, limits and language codes,
//! checked once. A bad table turns Translate off; it never panics.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use kx_module_api::ports::{Link, MAX_LINK_BYTES};
use serde::Deserialize;
use thiserror::Error;

use super::{TAG_CHARS, tag_ok};

/// The place in a pattern for the target's code.
pub(super) const TO: &str = "{to}";
/// The place in a pattern for the encoded text.
pub(super) const TEXT: &str = "{text}";

/// The table, read and checked at its first use.
static TABLE: LazyLock<Result<Sites, SiteError>> =
    LazyLock::new(|| Sites::parse(include_str!("../../sites.toml")));

/// The compiled-in sites, read and checked once.
///
/// # Errors
/// The table's first fault; Translate is then off.
pub fn sites() -> Result<&'static Sites, &'static SiteError> {
    TABLE.as_ref()
}

/// Why a site table was refused.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SiteError {
    /// The text is not a table of sites.
    #[error("the site table does not parse: {0}")]
    Parse(String),
    /// It lists no site.
    #[error("the site table lists no site")]
    Empty,
    /// A site's id is empty or another site's.
    #[error("a site id is empty or listed twice")]
    Id,
    /// A pattern is not an https link holding `{to}` and `{text}` once each.
    #[error("a site's pattern is not an https link with its two places once each")]
    Pattern,
    /// A whole tag, the fallback or an alias is not a language tag.
    #[error("a site's whole tag, fallback or alias is not a language tag")]
    Tag,
    /// A site takes no character.
    #[error("a site's limit is zero")]
    Limit,
    /// `link_max` passes the port's cap or leaves no room for text.
    #[error("a site's link_max passes the link cap or leaves no room for text")]
    Room,
}

/// One translate site.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    /// The id `translate_site` names.
    pub id: String,
    /// The site's name, as the consent question shows it.
    pub name: String,
    /// The page's link, with `{to}` and `{text}` once each.
    pub(super) pattern: String,
    /// The most characters the site takes.
    pub limit: u16,
    /// Tags the site takes whole.
    pub(super) whole: Vec<String>,
    /// The target when no language is set or usable.
    pub(super) fallback: String,
    /// The most bytes of the whole encoded link.
    pub(super) link_max: usize,
    /// Tags the site knows by another code, such as `nb` for `no`.
    pub(super) alias: BTreeMap<String, String>,
}

/// The table's rows, in order.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rows {
    /// The sites.
    sites: Vec<Site>,
}

/// The checked sites, in the table's order; the first is the default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sites(Vec<Site>);

impl Sites {
    /// Reads and checks a site table.
    ///
    /// # Errors
    /// The first fault found.
    pub fn parse(text: &str) -> Result<Self, SiteError> {
        let rows: Rows = toml::from_str(text).map_err(|e| SiteError::Parse(e.to_string()))?;
        if rows.sites.is_empty() {
            return Err(SiteError::Empty);
        }
        for (at, site) in rows.sites.iter().enumerate() {
            let twice = rows.sites.iter().take(at).any(|s| s.id == site.id);
            if site.id.is_empty() || twice {
                return Err(SiteError::Id);
            }
            site.check()?;
        }
        Ok(Self(rows.sites))
    }

    /// The site `id` names.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Site> {
        self.0.iter().find(|s| s.id == id)
    }

    /// The first site, the default.
    #[must_use]
    pub fn first(&self) -> Option<&Site> {
        self.0.first()
    }
}

impl Site {
    /// The pattern without its two places, which must be a good link with room for a tag and some text.
    pub(super) fn bare(&self) -> String {
        self.pattern.replacen(TO, "", 1).replacen(TEXT, "", 1)
    }

    /// The row's own checks: the pattern, the tags, the limit and the room.
    fn check(&self) -> Result<(), SiteError> {
        let once = |mark: &str| self.pattern.matches(mark).count() == 1;
        let bare = self.bare();
        if !once(TO) || !once(TEXT) || Link::new(bare.clone()).is_err() {
            return Err(SiteError::Pattern);
        }
        let aliases = self.alias.iter().flat_map(|(from, to)| [from, to]);
        let mut tags = self.whole.iter().chain([&self.fallback]).chain(aliases);
        if !tags.all(|t| tag_ok(t)) {
            return Err(SiteError::Tag);
        }
        if self.limit == 0 {
            return Err(SiteError::Limit);
        }
        if self.link_max > MAX_LINK_BYTES || bare.len().saturating_add(TAG_CHARS) >= self.link_max {
            return Err(SiteError::Room);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

//! Where each source comes from: its download URL, or the archive it is unzipped from.

use super::Manifest;
use crate::devtools::paths::SEP;

/// Where a source comes from.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Origin<'a> {
    /// Downloaded from this URL.
    Url(String),
    /// Unzipped from this source.
    Zip(&'a str),
}

impl Manifest {
    /// Where the source `rel` comes from; `None` for a path that is not a source.
    pub(crate) fn origin(&self, rel: &str) -> Option<Origin<'_>> {
        let source = self.sources.get(rel)?;
        if let Some(zip) = &source.zip {
            return Some(Origin::Zip(zip));
        }
        let base = self.bases.get(source.base.as_ref()?)?;
        let name = rel.rsplit(SEP).next()?;
        let dir = source.dir.as_deref().unwrap_or_default();
        Some(Origin::Url(format!("{base}{dir}{name}")))
    }

    /// The last folder of the base the source `rel` is downloaded from, such as a dated dump's date.
    pub(crate) fn base_folder(&self, rel: &str) -> Option<&str> {
        let base = self.bases.get(self.sources.get(rel)?.base.as_ref()?)?;
        base.trim_end_matches(SEP).rsplit(SEP).next()
    }
}

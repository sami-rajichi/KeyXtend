//! The user's model per language (`[voice] picks`): a map that prints a typed Other id hidden, as `ServiceOn` hides
//! the typed lines.

use std::collections::BTreeMap;
use std::collections::btree_map::Iter;
use std::fmt;
use std::ops::{Deref, DerefMut};

use kx_module_api::Redacted;
use serde::{Deserialize, Serialize};

use crate::cloud::{PICK_MARK, pick_row};

/// The picks by language: a row id or file name, or a service's id and model joined by `PICK_MARK`.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Picks(pub BTreeMap<String, String>);

impl Deref for Picks {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Picks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<(String, String)> for Picks {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[(String, String); N]> for Picks {
    fn from(pairs: [(String, String); N]) -> Self {
        Self(pairs.into())
    }
}

impl<'a> IntoIterator for &'a Picks {
    type Item = (&'a String, &'a String);
    type IntoIter = Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Shows each language's pick; a typed Other id, which no row lists, is the user's own.
impl fmt::Debug for Picks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(lang, pick)| (lang, Shown(pick))))
            .finish()
    }
}

/// One pick as `Picks` prints it.
struct Shown<'a>(&'a str);

impl fmt::Debug for Shown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match pick_row(self.0) {
            Some((row, typed, None)) => {
                write!(f, "\"{}{PICK_MARK}{:?}\"", row.id, Redacted::new(typed))
            }
            _ => fmt::Debug::fmt(self.0, f),
        }
    }
}

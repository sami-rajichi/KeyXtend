//! What a `send` key tells word prediction when it goes out alone (ADR-0033); a key without `tell`, or one sent with
//! latches held, tells a break.

use super::{BoardSpec, KeySpec, RowItem};
use kx_module_api::Redacted;
use kx_module_api::ports::{Piece, Scan};
use serde::Deserialize;
use std::collections::HashMap;

/// What a `send` key tells when it goes out alone, as `board.toml` writes it with `tell`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tell {
    /// One character erased, as Backspace does.
    Erase,
    /// The word ends whole and a new line or field starts, as after Enter or Tab.
    Advance,
    /// The text the key types, such as the space bar's space.
    Text(String),
}

impl Tell {
    /// The piece word prediction hears.
    #[must_use]
    pub fn piece(&self) -> Piece {
        match self {
            Self::Erase => Piece::Erase,
            Self::Advance => Piece::Advance,
            Self::Text(t) => Piece::Text(Redacted::new(t.clone())),
        }
    }
}

impl BoardSpec {
    /// The named keys that tell more than a break, by id.
    #[must_use]
    pub fn tells(&self) -> HashMap<String, Tell> {
        let told = self
            .named()
            .filter_map(|k| Some((k.id.clone(), k.tell.clone()?)));
        told.collect()
    }

    /// The scan code of the first key that tells an erase, which a picked word erases with.
    #[must_use]
    pub fn erase_key(&self) -> Option<Scan> {
        let erases = |k: &&KeySpec| k.tell == Some(Tell::Erase);
        self.named().find(erases).and_then(|k| k.sc)
    }

    /// The named keys of the main rows, row by row.
    pub(super) fn named(&self) -> impl Iterator<Item = &KeySpec> {
        self.rows.iter().flatten().filter_map(|i| match i {
            RowItem::Key(k) => Some(k),
            RowItem::Chars(_) => None,
        })
    }
}

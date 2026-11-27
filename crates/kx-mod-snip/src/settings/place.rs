//! How `[snip] editor` and `editor_full` read and write: a whole rectangle once set, and an empty table before.

use kx_module_api::ports::Rect;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A place as the settings write it, for the module's own writes.
#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct Written(#[serde(serialize_with = "serialize")] pub(crate) Option<Rect>);

/// What the table may hold; any other table does not read.
#[derive(Deserialize)]
#[serde(untagged)]
enum Held {
    /// A whole rectangle.
    Placed(Rect),
    /// An empty table: not placed yet.
    Unplaced(Empty),
}

/// A table with no key.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}

/// Writes the rectangle, or an empty table while there is none, as the defaults hold it.
#[allow(
    clippy::ref_option,
    reason = "serde's `with` hands the field by reference"
)]
pub(super) fn serialize<S: Serializer>(place: &Option<Rect>, out: S) -> Result<S::Ok, S::Error> {
    match place {
        Some(rect) => rect.serialize(out),
        None => out.serialize_map(Some(0))?.end(),
    }
}

/// Reads a whole rectangle, or `None` from an empty table.
pub(super) fn deserialize<'de, D: Deserializer<'de>>(input: D) -> Result<Option<Rect>, D::Error> {
    Ok(match Held::deserialize(input)? {
        Held::Placed(rect) => Some(rect),
        Held::Unplaced(Empty {}) => None,
    })
}

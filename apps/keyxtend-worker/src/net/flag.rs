//! A key test's reply read for a row's flags, such as `OpenRouter`'s `data.is_management_key`: `true` at a field's
//! dotted path, found in one streaming pass that holds none of the reply. A body that is not one JSON object flags
//! nothing.

use super::reply::opens_object;
use kx_ipc::rules::FIELD_MARK;
use serde::de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::borrow::Cow;
use std::fmt;

/// True when any of `paths` holds `true` in `bytes`.
pub fn any(bytes: &[u8], paths: &[Cow<'_, str>]) -> bool {
    paths.iter().any(|path| flagged(bytes, path))
}

/// True when `path` holds `true` in `bytes`; a missing field, any other value or a body that is not one JSON object
/// is false.
pub fn flagged(bytes: &[u8], path: &str) -> bool {
    if !opens_object(bytes) {
        return false;
    }
    let names: Vec<&str> = path.split(FIELD_MARK).collect();
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let found = Seek(&names).deserialize(&mut de);
    found.is_ok_and(|yes| yes && de.end().is_ok())
}

/// Seeks the value under its names, one object level each, skipping everything else unread.
struct Seek<'a>(&'a [&'a str]);

impl<'de> DeserializeSeed<'de> for Seek<'_> {
    type Value = bool;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<bool, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seek<'_> {
    type Value = bool;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<bool, E> {
        Ok(v && self.0.is_empty())
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<bool, E> {
        Ok(false)
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<bool, E> {
        Ok(false)
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<bool, E> {
        Ok(false)
    }

    fn visit_str<E: de::Error>(self, _: &str) -> Result<bool, E> {
        Ok(false)
    }

    fn visit_unit<E: de::Error>(self) -> Result<bool, E> {
        Ok(false)
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<bool, S::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(false)
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<bool, M::Error> {
        let (name, rest) = match self.0 {
            [name, rest @ ..] => (Some(*name), rest),
            [] => (None, self.0),
        };
        let mut found = false;
        while let Some(here) = map.next_key_seed(Named(name))? {
            if here {
                found |= map.next_value_seed(Seek(rest))?;
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(found)
    }
}

/// Reads an object's key: true when it is this name.
struct Named<'a>(Option<&'a str>);

impl<'de> DeserializeSeed<'de> for Named<'_> {
    type Value = bool;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<bool, D::Error> {
        d.deserialize_str(self)
    }
}

impl Visitor<'_> for Named<'_> {
    type Value = bool;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a name")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<bool, E> {
        Ok(self.0 == Some(v))
    }
}

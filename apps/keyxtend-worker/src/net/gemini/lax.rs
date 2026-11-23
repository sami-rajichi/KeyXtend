//! Lenient JSON reading for Gemini's reply [N10]: a value of another shape than expected reads as its type's empty
//! value, never as a failed parse, so a field Google adds or changes cannot hide the transcript.

use super::super::reply::Wiped;
use serde::Deserialize;
use serde::de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;
use zeroize::Zeroizing;

/// A type read from any JSON value: what it takes from a string, a list or an object, and its default otherwise.
pub trait Lax: Default {
    /// The value a string gives.
    ///
    /// # Errors
    /// None by default; a type's own reading may fail.
    fn of_str<E: de::Error>(_: &str) -> Result<Self, E> {
        Ok(Self::default())
    }

    /// The value a list gives, its items skipped unread by default.
    ///
    /// # Errors
    /// When the JSON itself is broken.
    fn of_seq<'de, A: SeqAccess<'de>>(mut seq: A) -> Result<Self, A::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(Self::default())
    }

    /// The value an object gives, its entries skipped unread by default.
    ///
    /// # Errors
    /// When the JSON itself is broken.
    fn of_map<'de, A: MapAccess<'de>>(mut map: A) -> Result<Self, A::Error> {
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        Ok(Self::default())
    }
}

/// Reads a `T` from whatever value comes.
struct LaxVisitor<T>(PhantomData<T>);

impl<'de, T: Lax> Visitor<'de> for LaxVisitor<T> {
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<T, E> {
        Ok(T::default())
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<T, E> {
        Ok(T::default())
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<T, E> {
        Ok(T::default())
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<T, E> {
        Ok(T::default())
    }

    fn visit_unit<E: de::Error>(self) -> Result<T, E> {
        Ok(T::default())
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
        T::of_str(v)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<T, A::Error> {
        T::of_seq(seq)
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
        T::of_map(map)
    }
}

/// A `T` read leniently from `d`.
///
/// # Errors
/// When the JSON itself is broken.
pub fn lax<'de, D: Deserializer<'de>, T: Lax>(d: D) -> Result<T, D::Error> {
    d.deserialize_any(LaxVisitor(PhantomData))
}

/// A JSON string read straight into a wiped buffer; none for any other value.
#[derive(Default)]
pub struct Text(pub Option<Wiped>);

impl Lax for Text {
    fn of_str<E: de::Error>(v: &str) -> Result<Self, E> {
        Ok(Self(Some(Wiped(Zeroizing::new(v.to_owned())))))
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        lax(d)
    }
}

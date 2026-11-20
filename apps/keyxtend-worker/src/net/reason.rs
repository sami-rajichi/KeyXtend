//! A JSON error reply as the status tables read it, taken in one streaming pass that keeps only what they ask: the codes
//! where a `Coded` says, and the words under some names within `SAYING_DEPTH`. Nothing else of the reply is held.

use super::reply::Coded;
use serde::de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// How many levels of objects and lists the words are looked for in: a list of `loc` inside a `detail` item, inside
/// `detail`, inside `message`, inside the reply.
const SAYING_DEPTH: usize = 5;
/// What starts a JSON object.
const OBJECT: u8 = b'{';

/// What an error reply says.
#[derive(Default)]
pub struct Reason {
    /// The texts where the reply's `Coded` names its code.
    codes: Vec<String>,
    /// The texts under the words' names, each after a space, in lower case.
    words: String,
}

impl Reason {
    /// What `bytes` says: its codes where `at` says and its words under `names`; nothing for a body that is not a JSON
    /// object.
    pub fn read(bytes: &[u8], at: &Coded, names: &[&str]) -> Self {
        let mut reason = Self::default();
        if bytes.trim_ascii_start().first() != Some(&OBJECT) {
            return reason;
        }
        let walk = Walk {
            at,
            names,
            depth: Some(SAYING_DEPTH),
            code: Code::Top,
            into: &mut reason,
        };
        let mut de = serde_json::Deserializer::from_slice(bytes);
        if walk.deserialize(&mut de).and_then(|()| de.end()).is_err() {
            return Self::default();
        }
        reason.words.make_ascii_lowercase();
        reason
    }

    /// True when its code is exactly one of `codes`, in any case.
    pub fn coded(&self, codes: &[&str]) -> bool {
        let is = |s: &String| codes.iter().any(|c| c.eq_ignore_ascii_case(s));
        self.codes.iter().any(is)
    }

    /// True when its words hold one of `words`, given in lower case.
    pub fn says(&self, words: &[&str]) -> bool {
        words.iter().any(|w| self.words.contains(w))
    }
}

/// Where a value sits on the way to a code: the reply, its `Coded` object, or a name its code may sit under.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Code {
    Top,
    Object,
    Name,
    Off,
}

/// One value of the reply, read for what `into` keeps: words while `depth` lasts, and a code at its place.
struct Walk<'a, 'r> {
    at: &'a Coded,
    names: &'a [&'a str],
    /// The levels the words may still go down; `None` once the value is under no word's name.
    depth: Option<usize>,
    code: Code,
    into: &'r mut Reason,
}

impl<'de> DeserializeSeed<'de> for Walk<'_, '_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        if self.depth.is_none() && self.code == Code::Off {
            return d.deserialize_ignored_any(IgnoredAny).map(drop);
        }
        d.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Walk<'_, '_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<(), E> {
        if self.depth.is_some() {
            self.into.words.push(' ');
            self.into.words.push_str(v);
        }
        if self.code == Code::Name {
            self.into.codes.push(v.to_owned());
        }
        Ok(())
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        let depth = self.depth.and_then(|d| d.checked_sub(1));
        loop {
            let (at, names, into) = (self.at, self.names, &mut *self.into);
            let item = Walk {
                at,
                names,
                depth,
                code: Code::Off,
                into,
            };
            if seq.next_element_seed(item)?.is_none() {
                return Ok(());
            }
        }
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let deeper = self.depth.and_then(|d| d.checked_sub(1));
        while let Some(key) = map.next_key::<String>()? {
            let depth = deeper.filter(|_| self.names.contains(&key.as_str()));
            let code = match self.code {
                Code::Top if key == self.at.object => Code::Object,
                Code::Object if self.at.names.contains(&key.as_str()) => Code::Name,
                _ => Code::Off,
            };
            let (at, names, into) = (self.at, self.names, &mut *self.into);
            map.next_value_seed(Walk {
                at,
                names,
                depth,
                code,
                into,
            })?;
        }
        Ok(())
    }
}

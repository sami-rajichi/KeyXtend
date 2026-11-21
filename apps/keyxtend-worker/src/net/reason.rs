//! A JSON error reply as the status tables read it, in one streaming pass that keeps only what they ask: the codes
//! where a `Coded` says, the words under some names within `SAYING_DEPTH`, and texts under some names inside the coded
//! object. Nothing else of the reply is held.

use super::reply::{Coded, opens_object};
use serde::de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// How many levels of objects and lists the words are looked for in: a list of `loc` inside a `detail` item, inside
/// `detail`, inside `message`, inside the reply.
const SAYING_DEPTH: usize = 5;

/// What an error reply says.
#[derive(Default)]
pub struct Reason {
    /// The texts where the reply's `Coded` names its code.
    codes: Vec<String>,
    /// The texts under the words' names, each after a space, in lower case.
    words: String,
    /// The names whose texts inside the coded object are kept.
    keep: &'static [&'static str],
    /// Each kept text in its own case, after its name's place in `keep`.
    kept: Vec<(usize, String)>,
}

impl Reason {
    /// What `bytes` says: its codes where `at` says and its words under `names`; nothing for a body that is not a JSON
    /// object.
    pub fn read(bytes: &[u8], at: &Coded, names: &[&str]) -> Self {
        Self::keeping(bytes, at, names, &[])
    }

    /// As `read`, also keeping the texts under `keep`'s names at any depth inside the coded object, in the same pass.
    /// A value of another shape keeps nothing and loses nothing beside it.
    pub fn keeping(
        bytes: &[u8],
        at: &Coded,
        names: &[&str],
        keep: &'static [&'static str],
    ) -> Self {
        let mut reason = Self {
            keep,
            ..Self::default()
        };
        if !opens_object(bytes) {
            return Self::default();
        }
        let walk = Walk {
            at,
            names,
            depth: Some(SAYING_DEPTH),
            code: Code::Top,
            slot: None,
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

    /// The texts kept under `name`, in the reply's order; none for a name it was not asked to keep.
    pub fn kept<'s>(&'s self, name: &str) -> impl Iterator<Item = &'s str> {
        let slot = self.keep.iter().position(|k| *k == name);
        let under = move |(at, _): &&(usize, String)| Some(*at) == slot;
        self.kept
            .iter()
            .filter(under)
            .map(|(_, text)| text.as_str())
    }
}

/// Where a value sits on the way to a code: the reply, its `Coded` object, a name its code may sit under, deeper
/// inside that object, or off the way.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Code {
    Top,
    Object,
    Name,
    Inside,
    Off,
}

impl Code {
    /// Where a value under one at `self` sits, other than the coded object or a code's name.
    const fn below(self) -> Self {
        match self {
            Self::Object | Self::Name | Self::Inside => Self::Inside,
            Self::Top | Self::Off => Self::Off,
        }
    }
}

/// One value of the reply, read for what `into` keeps: words while `depth` lasts, a code at its place, and a text
/// under a kept name.
struct Walk<'a, 'r> {
    at: &'a Coded,
    names: &'a [&'a str],
    /// The levels the words may still go down; `None` once the value is under no word's name.
    depth: Option<usize>,
    code: Code,
    /// The place in `keep` of the name the value sits under, inside the coded object.
    slot: Option<usize>,
    into: &'r mut Reason,
}

impl<'a> Walk<'a, '_> {
    /// The walk of a value under this one.
    fn under(&mut self, depth: Option<usize>, code: Code, slot: Option<usize>) -> Walk<'a, '_> {
        Walk {
            at: self.at,
            names: self.names,
            depth,
            code,
            slot,
            into: self.into,
        }
    }

    /// True when nothing under this value is read: no words, no code and no kept name can sit there.
    fn idle(&self) -> bool {
        let unread = match self.code {
            Code::Off => true,
            Code::Inside => self.into.keep.is_empty(),
            Code::Top | Code::Object | Code::Name => false,
        };
        self.depth.is_none() && unread
    }
}

impl<'de> DeserializeSeed<'de> for Walk<'_, '_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        if self.idle() {
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
        if let Some(slot) = self.slot {
            self.into.kept.push((slot, v.to_owned()));
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

    fn visit_seq<A: SeqAccess<'de>>(mut self, mut seq: A) -> Result<(), A::Error> {
        let (depth, code) = (self.depth.and_then(|d| d.checked_sub(1)), self.code.below());
        loop {
            if seq
                .next_element_seed(self.under(depth, code, None))?
                .is_none()
            {
                return Ok(());
            }
        }
    }

    fn visit_map<A: MapAccess<'de>>(mut self, mut map: A) -> Result<(), A::Error> {
        let deeper = self.depth.and_then(|d| d.checked_sub(1));
        let keep = self.into.keep;
        while let Some(key) = map.next_key::<String>()? {
            let depth = deeper.filter(|_| self.names.contains(&key.as_str()));
            let code = match self.code {
                Code::Top if key == self.at.object => Code::Object,
                Code::Object if self.at.names.contains(&key.as_str()) => Code::Name,
                other => other.below(),
            };
            let inside = matches!(code, Code::Name | Code::Inside);
            let slot = keep.iter().position(|k| *k == key).filter(|_| inside);
            map.next_value_seed(self.under(depth, code, slot))?;
        }
        Ok(())
    }
}

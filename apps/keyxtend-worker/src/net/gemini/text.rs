//! Gemini's transcript in its reply [N1]: `output_text` as its SDK reads it, else its steps' content texts in order, as
//! the sample's raw reply places them, each read into a wiped string. A reply with neither is not Gemini's, and steps
//! with no text are an empty transcript.

use super::super::reply::{self, Wiped};
use serde::Deserialize;
use serde::de::value::MapAccessDeserializer;
use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;
use zeroize::Zeroizing;

/// A reply: its transcript in one place or the other; other fields are skipped unread.
#[derive(Deserialize)]
struct Said {
    #[serde(default)]
    output_text: Option<Wiped>,
    #[serde(default)]
    steps: Option<Vec<Step>>,
}

/// One step of the interaction.
#[derive(Deserialize)]
struct Step {
    #[serde(default)]
    content: Parts,
}

/// A step's content: a list of parts, or one part.
#[derive(Default)]
struct Parts(Vec<Part>);

/// One part of a step's content; one with no text, such as the sound sent, adds none.
#[derive(Deserialize)]
struct Part {
    #[serde(default)]
    text: Option<Wiped>,
}

/// Reads `Parts` from a list or from one object.
struct PartsVisitor;

impl<'de> Visitor<'de> for PartsVisitor {
    type Value = Parts;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a part or a list of parts")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Parts, A::Error> {
        let mut parts = Vec::new();
        while let Some(part) = seq.next_element()? {
            parts.push(part);
        }
        Ok(Parts(parts))
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Parts, A::Error> {
        Part::deserialize(MapAccessDeserializer::new(map)).map(|p| Parts(vec![p]))
    }
}

impl<'de> Deserialize<'de> for Parts {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(PartsVisitor)
    }
}

/// The transcript a JSON reply object holds; none for a reply of neither shape, or when memory runs out.
pub fn said(bytes: &[u8]) -> Option<Zeroizing<String>> {
    if !reply::opens_object(bytes) {
        return None;
    }
    let said: Said = serde_json::from_slice(bytes).ok()?;
    if let Some(text) = said.output_text {
        return Some(text.0);
    }
    let parts = said.steps?.into_iter().flat_map(|s| s.content.0);
    let texts: Vec<Wiped> = parts.filter_map(|p| p.text).collect();
    joined(&texts)
}

/// `texts` joined in order into one wiped string sized once.
fn joined(texts: &[Wiped]) -> Option<Zeroizing<String>> {
    let mut all = Zeroizing::new(String::new());
    all.try_reserve_exact(texts.iter().map(|t| t.0.len()).sum())
        .ok()?;
    for t in texts {
        all.push_str(&t.0);
    }
    Some(all)
}

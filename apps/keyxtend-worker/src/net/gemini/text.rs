//! Gemini's transcript in its reply [N1][N10]: `output_text`, read alone so no other field can fail it, else the texts
//! of the model's output steps in order, each read into a wiped string. Steps or content of other shapes add nothing.

use super::super::reply::{self, Wiped};
use super::lax::{Lax, Text, lax};
use serde::Deserialize;
use serde::de::{Deserializer, IgnoredAny, MapAccess, SeqAccess};
use zeroize::Zeroizing;

/// The type of a step the model wrote [N10].
const MODEL_OUTPUT: &str = "model_output";
/// The names a step and a part hold.
const TYPE: &str = "type";
const CONTENT: &str = "content";
const TEXT: &str = "text";

/// A reply's `output_text`, the only field read.
#[derive(Deserialize)]
struct Output {
    #[serde(default)]
    output_text: Text,
}

/// A reply's steps, the only field read.
#[derive(Deserialize)]
struct Steps {
    #[serde(default)]
    steps: Option<Vec<Step>>,
}

/// One part of a step's content: its text, when it holds one.
#[derive(Default)]
struct Part(Option<Wiped>);

impl Lax for Part {
    fn of_map<'de, A: MapAccess<'de>>(mut map: A) -> Result<Self, A::Error> {
        let mut text = None;
        while let Some(name) = map.next_key::<String>()? {
            if name == TEXT {
                text = map.next_value::<Text>()?.0;
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(Self(text))
    }
}

/// A step's content: its parts' texts in order, from a list of parts or one part.
#[derive(Default)]
struct Parts(Vec<Wiped>);

impl Lax for Parts {
    fn of_seq<'de, A: SeqAccess<'de>>(mut seq: A) -> Result<Self, A::Error> {
        let mut texts = Vec::new();
        while let Some(part) = seq.next_element::<Part>()? {
            texts.extend(part.0);
        }
        Ok(Self(texts))
    }

    fn of_map<'de, A: MapAccess<'de>>(map: A) -> Result<Self, A::Error> {
        Ok(Self(Part::of_map(map)?.0.into_iter().collect()))
    }
}

/// One step: its content's texts when the model wrote it, else none.
#[derive(Default)]
struct Step(Vec<Wiped>);

impl Lax for Step {
    fn of_map<'de, A: MapAccess<'de>>(mut map: A) -> Result<Self, A::Error> {
        let (mut model, mut texts) = (false, Vec::new());
        while let Some(name) = map.next_key::<String>()? {
            match name.as_str() {
                TYPE => {
                    let kind = map.next_value::<Text>()?.0;
                    model = kind.is_some_and(|k| k.0.eq_ignore_ascii_case(MODEL_OUTPUT));
                }
                CONTENT => texts = map.next_value::<Parts>()?.0,
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(Self(if model { texts } else { Vec::new() }))
    }
}

impl<'de> Deserialize<'de> for Part {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        lax(d)
    }
}

impl<'de> Deserialize<'de> for Parts {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        lax(d)
    }
}

impl<'de> Deserialize<'de> for Step {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        lax(d)
    }
}

/// The transcript a JSON reply object holds; none for a reply of neither shape, or when memory runs out.
pub fn said(bytes: &[u8]) -> Option<Zeroizing<String>> {
    if !reply::opens_object(bytes) {
        return None;
    }
    let output: Output = serde_json::from_slice(bytes).ok()?;
    if let Some(text) = output.output_text.0 {
        return Some(text.0);
    }
    let steps: Steps = serde_json::from_slice(bytes).ok()?;
    let texts: Vec<Wiped> = steps.steps?.into_iter().flat_map(|s| s.0).collect();
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

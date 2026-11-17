//! JSON for a view that holds a dictated text: written into a buffer sized first, so it never grows and leaves no
//! copy behind, and wiped when dropped. Qt's own copy of it is beyond our reach.

use super::NULL;
use kx_module_api::Zeroizing;
use serde::{Serialize, Serializer};
use std::io;

/// Counts the bytes written to it, to size the buffer first.
struct Count(usize);

impl io::Write for Count {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 = self.0.saturating_add(buf.len());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// `value` as JSON in a buffer wiped on drop and sized once; `null` when it cannot be written.
pub(crate) fn json(value: &impl Serialize) -> Zeroizing<String> {
    written(value).unwrap_or_else(|| Zeroizing::new(NULL.to_owned()))
}

/// `value` written twice: once to count its bytes, then into a wiped buffer of exactly that size.
fn written(value: &impl Serialize) -> Option<Zeroizing<String>> {
    let mut size = Count(0);
    serde_json::to_writer(&mut size, value).ok()?;
    let mut bytes = Zeroizing::new(Vec::with_capacity(size.0));
    serde_json::to_writer(&mut *bytes, value).ok()?;
    let text = String::from_utf8(std::mem::take(&mut *bytes));
    // JSON is always UTF-8; the bytes of a refusal are wiped all the same.
    text.map_err(|e| drop(Zeroizing::new(e.into_bytes())))
        .ok()
        .map(Zeroizing::new)
}

/// Writes a wiped text as a plain JSON string, for `serialize_with`.
pub(crate) fn text<S: Serializer>(text: &Zeroizing<String>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(text)
}

/// Writes a wiped text that may be missing as a plain JSON string or `null`, for `serialize_with`.
#[expect(
    clippy::ref_option,
    reason = "serde's serialize_with hands the field by reference"
)]
pub(crate) fn maybe_text<S: Serializer>(
    text: &Option<Zeroizing<String>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match text {
        Some(text) => s.serialize_some(text.as_str()),
        None => s.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json as value;
    use std::collections::BTreeMap;

    #[test]
    fn the_json_is_the_plain_one_in_a_buffer_sized_once() {
        let view = value!({"text": "مرحبا, “quoted”\n", "n": [1, 2.5, null], "masked": false});
        let got = json(&view);
        assert_eq!(got.as_str(), serde_json::to_string(&view).unwrap());
        assert_eq!(got.capacity(), got.len(), "never grown, so no copy is left");
    }

    #[test]
    fn a_value_json_cannot_write_gives_null() {
        let keyed: BTreeMap<(u8, u8), u8> = BTreeMap::from([((1, 2), 3)]);
        assert_eq!(json(&keyed).as_str(), NULL);
    }

    /// A view row holding wiped texts, one of them missing.
    #[derive(Serialize)]
    struct Row {
        #[serde(serialize_with = "text")]
        text: Zeroizing<String>,
        #[serde(serialize_with = "maybe_text")]
        some: Option<Zeroizing<String>>,
        #[serde(serialize_with = "maybe_text")]
        none: Option<Zeroizing<String>>,
    }

    #[test]
    fn wiped_texts_write_as_plain_strings_and_a_missing_one_as_null() {
        let row = Row {
            text: Zeroizing::new("مرحبا".to_owned()),
            some: Some(Zeroizing::new("bonjour".to_owned())),
            none: None,
        };
        let want = r#"{"text":"مرحبا","some":"bonjour","none":null}"#;
        assert_eq!(json(&row).as_str(), want);
    }
}

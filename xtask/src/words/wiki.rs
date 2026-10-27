//! Reads Wikipedia's search-index dumps (`*_content-*.json.bz2`, Elasticsearch's bulk format) as a stream: the lines
//! alternate an action and a document, and the document's `text` field holds the article as plain text.

use std::io::Read;

use serde::Deserialize;

use super::lines::Lines;

/// Lines come in pairs: an action, then its document.
const PAIR: u64 = 2;

/// The one field read from a document; serde skips the rest.
#[derive(Deserialize)]
struct Document {
    /// The article as plain text.
    text: Option<String>,
}

/// Gives `each` the text of each document of the bzip2 stream `reader` until `max_bytes` of text were given;
/// returns how many bytes were given. The first document must have a `text` field, or nothing is given.
pub(crate) fn read(
    reader: impl Read,
    max_bytes: u64,
    each: &mut dyn FnMut(&str),
) -> Result<u64, String> {
    let mut lines = Lines::new(reader);
    let (mut given, mut line) = (0, 0_u64);
    while given < max_bytes {
        let Some(buf) = lines.line()? else {
            break;
        };
        line += 1;
        if line % PAIR == 1 {
            continue;
        }
        let doc: Document = serde_json::from_slice(buf).map_err(|e| format!("line {line}: {e}"))?;
        let Some(text) = doc.text else {
            if line == PAIR {
                return Err("the first document has no text field; is this a content dump?".into());
            }
            continue;
        };
        given += u64::try_from(text.len()).unwrap_or(u64::MAX);
        each(&text);
    }
    Ok(given)
}

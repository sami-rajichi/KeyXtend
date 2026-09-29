//! Unicode text on the clipboard: reads capped in size, and writes that carry marker formats.
#![cfg(windows)]

use windows::Win32::System::Ole::CF_UNICODETEXT;

use crate::clip::{Open, empty, format_id, open, put, with_data};

/// Most UTF-16 units one character takes.
const UNITS_PER_CHAR: usize = 2;
/// Every write by kx-gates carries this format holding 0, so Windows never syncs test texts to the cloud.
const CLOUD_OFF: &str = "CanUploadToCloudClipboard";

/// The text before the first NUL; `Err` past `max_chars` characters.
fn text_of(units: &[u16], max_chars: usize) -> Result<String, String> {
    let limit = max_chars.saturating_mul(UNITS_PER_CHAR);
    let scan = &units[..units.len().min(limit.saturating_add(1))];
    let len = scan.iter().position(|&u| u == 0).unwrap_or(scan.len());
    let text = (len <= limit).then(|| String::from_utf16_lossy(&scan[..len]));
    match text {
        Some(t) if t.chars().count() <= max_chars => Ok(t),
        _ => Err(format!(
            "the clipboard holds more than {max_chars} characters"
        )),
    }
}

/// The clipboard text, or `None` when it holds none; opens it within `timeout_ms`.
pub fn read_text(
    timeout_ms: u64,
    poll_ms: u64,
    max_chars: usize,
) -> Result<Option<String>, String> {
    unicode(&open(timeout_ms, poll_ms)?, max_chars)
}

/// The text on the open clipboard, or `None` when it holds none; only the capped start is read.
pub fn unicode(open: &Open, max_chars: usize) -> Result<Option<String>, String> {
    let limit = max_chars.saturating_mul(UNITS_PER_CHAR).saturating_add(1);
    with_data(open, u32::from(CF_UNICODETEXT.0), |bytes| {
        let units: Vec<u16> = (bytes.as_chunks::<2>().0.iter().take(limit))
            .map(|b| u16::from_ne_bytes(*b))
            .collect();
        text_of(&units, max_chars)
    })
    .transpose()
}

/// Puts `text` on the clipboard, or leaves it empty for `None`.
pub fn write_text(text: Option<&str>, timeout_ms: u64, poll_ms: u64) -> Result<(), String> {
    write_marked(text, &[], timeout_ms, poll_ms)
}

/// Like `write_text`, plus each format in `marks` holding the number 0, in one clipboard change.
pub fn write_marked(
    text: Option<&str>,
    marks: &[&str],
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    let open = open(timeout_ms, poll_ms)?;
    empty(&open)?;
    let Some(text) = text else {
        return Ok(());
    };
    let units: Vec<u8> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_ne_bytes)
        .collect();
    put(&open, u32::from(CF_UNICODETEXT.0), &units)?;
    for m in marks.iter().chain([&CLOUD_OFF]) {
        put(&open, format_id(m)?, &0u32.to_ne_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn text_stops_at_the_first_nul() {
        assert_eq!(text_of(&units("ab\0cd"), 2), Ok("ab".to_string()));
        assert_eq!(text_of(&units("لا€"), 3), Ok("لا€".to_string()));
        assert_eq!(text_of(&units("\u{1F600}"), 1), Ok("\u{1F600}".to_string()));
    }

    #[test]
    fn text_past_the_cap_is_an_error() {
        assert!(text_of(&units("abc"), 2).is_err());
        assert!(text_of(&units("abcdefgh\0"), 2).is_err());
    }
}

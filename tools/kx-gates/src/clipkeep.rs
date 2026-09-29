//! Keeps the owner's clipboard around a test: every format saved raw first, and put back after.
#![cfg(windows)]

use windows::Win32::System::Ole::{
    CF_HDROP, CF_LOCALE, CF_OEMTEXT, CF_TEXT, CF_UNICODETEXT, CLIPBOARD_FORMAT,
};

use crate::clip::{self, FIRST_NAMED};
use crate::cliplisten::{HISTORY_FLAG, excluded};
use crate::config::GatesConfig;

/// Standard formats held as plain memory, so they copy byte for byte (adapter table).
const RAW_STANDARD: [CLIPBOARD_FORMAT; 5] =
    [CF_TEXT, CF_OEMTEXT, CF_UNICODETEXT, CF_LOCALE, CF_HDROP];

/// Every format of a clipboard content, saved as raw bytes to put back later.
pub struct Saved(Vec<(u32, Vec<u8>)>);

/// True when `format` is plain memory we can copy raw: a text or file-list format, or any registered one such as HTML.
fn raw_ok(format: u32) -> bool {
    format >= FIRST_NAMED || RAW_STANDARD.iter().any(|f| u32::from(f.0) == format)
}

/// Saves every format; refuses a private copy, pictures and other handles, and more than `max_bytes`.
pub fn save(timeout_ms: u64, poll_ms: u64, max_bytes: usize) -> Result<Saved, String> {
    let open = clip::open(timeout_ms, poll_ms)?;
    // Putting a password back would restart its life: the password manager's auto-clear would miss it.
    if excluded(&clip::names(&open), clip::dword(&open, HISTORY_FLAG)) {
        return Err(
            "the clipboard holds a private copy, such as a password; try again once it clears"
                .to_string(),
        );
    }
    let (mut saved, mut left) = (Vec::new(), max_bytes);
    for format in clip::formats(&open) {
        if !raw_ok(format) {
            return Err(format!(
                "the clipboard holds format {format}, which we could not put back"
            ));
        }
        // The size is checked before the copy, so a huge clipboard is never duplicated.
        match clip::with_data(&open, format, |b| (b.len() <= left).then(|| b.to_vec())) {
            None => {}
            Some(None) => return Err(format!("the clipboard holds more than {max_bytes} bytes")),
            Some(Some(data)) => {
                left -= data.len();
                saved.push((format, data));
            }
        }
    }
    Ok(Saved(saved))
}

/// Puts back what `save` kept, as one clipboard change.
pub fn restore(saved: &Saved, timeout_ms: u64, poll_ms: u64) -> Result<(), String> {
    let open = clip::open(timeout_ms, poll_ms)?;
    clip::empty(&open)?;
    saved
        .0
        .iter()
        .try_for_each(|(f, data)| clip::put(&open, *f, data))
}

/// Runs `f` with the owner's clipboard saved first and put back after; a failed put-back is an error too.
pub fn keep<T>(cfg: &GatesConfig, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let t = &cfg.timing;
    let saved = save(t.read_wait_ms, t.poll_ms, cfg.probes.keep_max_bytes)
        .map_err(|e| format!("could not save the clipboard first: {e}"))?;
    let got = f();
    match (got, restore(&saved, t.read_wait_ms, t.poll_ms)) {
        (got, Ok(())) => got,
        (Ok(_), Err(r)) => Err(format!("the clipboard was not put back: {r}")),
        (Err(e), Err(r)) => Err(format!("{e}; the clipboard was not put back: {r}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Ole::{
        CF_BITMAP, CF_ENHMETAFILE, CF_METAFILEPICT, CF_OWNERDISPLAY, CF_PALETTE,
    };

    #[test]
    fn memory_formats_can_be_saved_raw() {
        for f in [CF_UNICODETEXT, CF_TEXT, CF_LOCALE, CF_HDROP] {
            assert!(raw_ok(u32::from(f.0)), "{}", f.0);
        }
        assert!(raw_ok(FIRST_NAMED), "a registered format such as HTML");
    }

    #[test]
    fn picture_and_handle_formats_cannot() {
        let handles = [
            CF_BITMAP,
            CF_METAFILEPICT,
            CF_PALETTE,
            CF_ENHMETAFILE,
            CF_OWNERDISPLAY,
        ];
        for f in handles {
            assert!(!raw_ok(u32::from(f.0)), "{}", f.0);
        }
        assert!(
            !raw_ok(0x0200) && !raw_ok(0x0300),
            "private and GDI object ranges"
        );
    }
}

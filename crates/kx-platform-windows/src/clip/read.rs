//! Reading one copy: the markers first, then a clear and the size, and only then the parts. The app is named after
//! the clipboard closes, so no other app waits on our file reads.

use kx_module_api::Redacted;
use kx_module_api::ports::{AppRef, ClipConfig, ClipError, ClipNote, ClipPart, ClipWait};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Ole::{CF_DIB, CF_DIBV5, CF_HDROP, CF_UNICODETEXT, DROPEFFECT_COPY};

use super::open::{self, Open};
use super::parts::{Formats, Kind, UNITS_PER_CHAR, Want, blank, text_of, until_nul};
use super::{Marks, Table, app, hdrop};

/// What one look at the clipboard saw, before its app is named.
enum Seen {
    Marked,
    Cleared,
    TooBig(u64),
    Copied(Vec<ClipPart>),
}

impl Seen {
    /// The note to tell, with the app that made the copy.
    fn note(self, app: Option<AppRef>) -> ClipNote {
        match self {
            Self::Marked => ClipNote::Marked { app },
            Self::Cleared => ClipNote::Cleared { app },
            Self::TooBig(bytes) => ClipNote::TooBig { app, bytes },
            Self::Copied(parts) => ClipNote::Copied { parts, app },
        }
    }
}

/// The format ids of `table` and Windows' standard formats.
///
/// # Errors
/// `Refused` when Windows will not register a format.
pub(crate) fn ids(table: &Table) -> Result<Formats, ClipError> {
    Ok(Formats {
        text: u32::from(CF_UNICODETEXT.0),
        html: open::format_id(&table.html)?,
        rtf: open::format_id(&table.rtf)?,
        png: open::format_id(&table.png)?,
        dib_v5: u32::from(CF_DIBV5.0),
        dib: u32::from(CF_DIB.0),
        files: u32::from(CF_HDROP.0),
        effect: open::format_id(&table.drop_effect)?,
        copy: DROPEFFECT_COPY.0,
    })
}

/// What the clipboard holds now, opened for `owner`, a window of this thread; `Busy` past `cfg`'s wait.
pub(crate) fn note(owner: HWND, cfg: &ClipConfig, marks: &Marks, formats: &Formats) -> ClipNote {
    let Ok(open) = open::open_for(owner, cfg.wait) else {
        return ClipNote::Busy;
    };
    let maker = app::owner();
    let seen = look(&open, cfg.max_bytes, marks, formats);
    drop(open);
    seen.note(app::of(maker))
}

/// The markers first, then an empty clipboard and the summed size, and only then the parts. A blank text is dropped,
/// and alone it makes a clear.
fn look(open: &Open, max_bytes: u64, marks: &Marks, formats: &Formats) -> Seen {
    if marks.on(open) {
        return Seen::Marked;
    }
    let held = open::formats(open);
    if held.is_empty() {
        return Seen::Cleared;
    }
    let wants = formats.pick(&held);
    let sizes = wants.iter().filter_map(|w| open::size(open, w.format));
    let bytes = sizes.fold(0, u64::saturating_add);
    if bytes > max_bytes {
        return Seen::TooBig(bytes);
    }
    let mut blanked = false;
    let mut parts: Vec<ClipPart> = wants.iter().filter_map(|&w| part(open, w)).collect();
    parts.retain(|p| {
        let gone = matches!(p, ClipPart::Text(t) if blank(t.expose()));
        blanked |= gone;
        !gone
    });
    if parts.is_empty() && blanked {
        Seen::Cleared
    } else {
        Seen::Copied(parts)
    }
}

/// The part `want` names; `None` when it cannot be read or holds nothing.
fn part(open: &Open, want: Want) -> Option<ClipPart> {
    let text = |b: &[u8]| Some(until_nul(b).to_vec()).filter(|t| !t.is_empty());
    Some(match want.kind {
        Kind::Text => ClipPart::Text(Redacted::new(unicode(open, usize::MAX).ok().flatten()?)),
        Kind::Html => ClipPart::Html(Redacted::new(open::with_data(open, want.format, text)??)),
        Kind::Rtf => ClipPart::Rtf(Redacted::new(open::with_data(open, want.format, text)??)),
        Kind::Image(codec) => ClipPart::Image {
            codec,
            bytes: Redacted::new(open::with_data(open, want.format, <[u8]>::to_vec)?),
        },
        Kind::Files => {
            let paths = open::with_data(open, want.format, hdrop::decode)??;
            ClipPart::Files(Redacted::new(Some(paths).filter(|p| !p.is_empty())?))
        }
    })
}

/// The text on the open clipboard, or `None` when it holds none; only the start up to `max_chars` is read.
///
/// # Errors
/// `Io` past `max_chars` characters.
fn unicode(open: &Open, max_chars: usize) -> Result<Option<String>, ClipError> {
    let limit = max_chars.saturating_mul(UNITS_PER_CHAR).saturating_add(1);
    let text = open::with_data(open, u32::from(CF_UNICODETEXT.0), |bytes| {
        let units = bytes.as_chunks::<2>().0.iter().take(limit);
        text_of(
            &units.map(|b| u16::from_ne_bytes(*b)).collect::<Vec<_>>(),
            max_chars,
        )
    });
    match text {
        None => Ok(None),
        Some(None) => Err(ClipError::Io),
        Some(t) => Ok(t),
    }
}

/// The clipboard's text for a test tool, or `None` when it holds none; past `max_chars` characters only the start is
/// read, and it is refused.
///
/// # Errors
/// `Busy` or `Refused` as `open::open` says, and `Io` past the cap.
pub fn read_text(wait: ClipWait, max_chars: usize) -> Result<Option<String>, ClipError> {
    unicode(&open::open(wait)?, max_chars)
}

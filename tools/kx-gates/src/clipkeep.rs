//! Keeps the owner's clipboard around a test: every format saved raw first, and put back after.
//! The clipboard calls are the product's (`kx_platform_windows::clip`), and every gate write is refused in tests.
#![cfg(windows)]

use kx_module_api::ports::{ClipError, ClipWait};
use kx_platform_windows::clip::{self, open, open::FIRST_NAMED};
use windows::Win32::System::Ole::{
    CF_HDROP, CF_LOCALE, CF_OEMTEXT, CF_TEXT, CF_UNICODETEXT, CLIPBOARD_FORMAT,
};

use crate::config::{GatesConfig, Timing};

/// Standard formats held as plain memory, so they copy byte for byte (adapter table).
const RAW_STANDARD: [CLIPBOARD_FORMAT; 5] =
    [CF_TEXT, CF_OEMTEXT, CF_UNICODETEXT, CF_LOCALE, CF_HDROP];

/// Every format of a clipboard content, saved as raw bytes to put back later.
pub struct Saved(Vec<(u32, Vec<u8>)>);

/// What a test build says when a gate test reaches a clipboard writer.
#[cfg(test)]
const REFUSED: &str =
    "a gate test reached a real clipboard writer; the owner's clipboard stays as it is";
/// A wait that never waits, for tests that stop before the clipboard.
#[cfg(test)]
pub(crate) const NOW: ClipWait = ClipWait {
    open_ms: 0,
    poll_ms: 0,
};

/// Refuses a clipboard write in a test build, as `keys` refuses real input, so `cargo test` never overwrites the
/// owner's clipboard; every gate clipboard writer calls it first.
pub(crate) fn live() {
    #[cfg(test)]
    panic!("{REFUSED}");
}

/// True when `write` stops at `live`'s refusal, so nothing reached the clipboard.
#[cfg(test)]
pub(crate) fn refused(write: impl FnOnce() + std::panic::UnwindSafe) -> bool {
    let Err(why) = std::panic::catch_unwind(write) else {
        return false;
    };
    let text = why.downcast_ref::<String>().map(String::as_str);
    text.or_else(|| why.downcast_ref::<&str>().copied()) == Some(REFUSED)
}

/// The product's text write (`clip::write_text`), refused in a test build.
pub(crate) fn write_text(
    text: Option<&str>,
    zero: &[&str],
    wait: ClipWait,
) -> Result<(), ClipError> {
    live();
    clip::write_text(text, zero, wait)
}

/// The product's clipboard wait from the gates' timing: open within `read_wait_ms`, trying every `poll_ms`.
pub fn wait(t: &Timing) -> ClipWait {
    ClipWait {
        open_ms: ms(t.read_wait_ms),
        poll_ms: ms(t.poll_ms),
    }
}

/// `v` milliseconds as the product's wait holds them, capped at `u32::MAX`.
fn ms(v: u64) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// True when `format` is plain memory we can copy raw: a text or file-list format, or any registered one such as HTML.
fn raw_ok(format: u32) -> bool {
    format >= FIRST_NAMED || RAW_STANDARD.iter().any(|f| u32::from(f.0) == format)
}

/// Saves every format; refuses a private copy, pictures and other handles, and more than `max_bytes`.
pub fn save(wait: ClipWait, max_bytes: usize) -> Result<Saved, String> {
    let open = open::open(wait).map_err(|e| e.to_string())?;
    let read =
        |f, left: usize| open::with_data(&open, f, |b| (b.len() <= left).then(|| b.to_vec()));
    let (has, dword) = clip::probes(&open);
    keep_from(has, dword, open::formats(&open), max_bytes, read)
}

/// What `save` keeps of a clipboard holding `formats`, each read by `read` while it fits what is left. It refuses a
/// private copy, which `has` a presence marker (`clip::private_by`), as putting a password back would restart its life
/// past its manager's auto-clear. Number markers by `dword`, such as our own put-back's, are kept byte for byte.
fn keep_from(
    has: impl Fn(&str) -> bool,
    dword: impl Fn(&str) -> Option<u32>,
    formats: Vec<u32>,
    max_bytes: usize,
    read: impl Fn(u32, usize) -> Option<Option<Vec<u8>>>,
) -> Result<Saved, String> {
    if clip::private_by(has, dword) {
        return Err(
            "the clipboard holds a private copy, such as a password; try again once it clears"
                .to_string(),
        );
    }
    let (mut saved, mut left) = (Vec::new(), max_bytes);
    for format in formats {
        if !raw_ok(format) {
            return Err(format!(
                "the clipboard holds format {format}, which we could not put back"
            ));
        }
        // The size is checked before the copy, so a huge clipboard is never duplicated.
        match read(format, left) {
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
pub fn restore(saved: &Saved, wait: ClipWait) -> Result<(), String> {
    live();
    let put = || {
        let open = open::open(wait)?;
        open::empty(&open)?;
        put_all(saved, |f, data| open::put(&open, f, data))
    };
    put().map_err(|e| e.to_string())
}

/// Puts each format `saved` holds with `put`, in order and byte for byte, stopping at the first refused.
fn put_all<E>(saved: &Saved, mut put: impl FnMut(u32, &[u8]) -> Result<(), E>) -> Result<(), E> {
    saved.0.iter().try_for_each(|(f, data)| put(*f, data))
}

/// Puts `saved` back with `put` when dropped before `done`, as when the test panics.
struct Back<S, P: Fn(&S) -> Result<(), String>> {
    saved: S,
    put: P,
    done: bool,
}

impl<S, P: Fn(&S) -> Result<(), String>> Drop for Back<S, P> {
    fn drop(&mut self) {
        if !self.done {
            let _ = (self.put)(&self.saved);
        }
    }
}

/// Runs `f`, then puts `saved` back with `put`, even when `f` panics; a failed put-back is an error too.
fn kept<S, T>(
    saved: S,
    put: impl Fn(&S) -> Result<(), String>,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let mut back = Back {
        saved,
        put,
        done: false,
    };
    let got = f();
    back.done = true;
    match (got, (back.put)(&back.saved)) {
        (got, Ok(())) => got,
        (Ok(_), Err(r)) => Err(format!("the clipboard was not put back: {r}")),
        (Err(e), Err(r)) => Err(format!("{e}; the clipboard was not put back: {r}")),
    }
}

/// Runs `f` with the owner's clipboard saved first and put back after, even on a panic; a failed put-back is an
/// error too.
pub fn keep<T>(cfg: &GatesConfig, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let wait = wait(&cfg.timing);
    let saved = save(wait, cfg.probes.keep_max_bytes)
        .map_err(|e| format!("could not save the clipboard first: {e}"))?;
    kept(saved, |s| restore(s, wait), f)
}

/// Runs `put` only when the clipboard's sequence number moved from `seq` to `now`, so an untouched clipboard is never
/// written again.
fn put_if(seq: u32, now: u32, put: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    if now == seq { Ok(()) } else { put() }
}

/// `keep` for a run that writes nothing: the clipboard goes back only when something changed it meanwhile.
pub fn keep_changed<T>(
    cfg: &GatesConfig,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let wait = wait(&cfg.timing);
    let saved = save(wait, cfg.probes.keep_max_bytes)
        .map_err(|e| format!("could not save the clipboard first: {e}"))?;
    let seq = open::sequence();
    kept(
        saved,
        |s| put_if(seq, open::sequence(), || restore(s, wait)),
        f,
    )
}

#[cfg(test)]
mod tests;

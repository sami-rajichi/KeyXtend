//! The product's notice banners G23 meets: the hidden-run banner ends the run with an error, any other notice at the
//! start is answered by its last button, and the first backup offer after the first save is answered Later. Each
//! answer is a real click, so a banner covers nothing.
#![cfg(windows)]

use kx_module_api::{NoticeAsk, keys};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Accessibility::UIA_ButtonControlTypeId;
use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;

use crate::apps::Ctx;
use crate::clippanel::has_area;
use crate::ftl::{self, Piece};
use crate::g23::live::{self, kept, poll_read};
use crate::keymap::{Reader, centre};
use crate::win::sleep_ms;
use crate::{mouse, outline, product, win};

/// The first backup offer's buttons, as the vault raises it; a test reads the vault's source to keep them so.
pub const OFFER_ASK: NoticeAsk = NoticeAsk::YES_LATER;

/// Why G23 stops on a hidden run.
pub const HIDDEN_RUN: &str = "the test build started as a hidden run, so Quick-fill is off and saves nothing: start \
                              kx-gates from a plain terminal, not from inside another app";

/// True when `name` reads as `shape`: its words in order, and each variable some text where it stands. A variable
/// takes the first place its next words fit.
pub fn fits(shape: &[Piece], name: &str) -> bool {
    let (mut rest, mut open) = (name, false);
    for piece in shape {
        match piece {
            Piece::Var(_) => open = true,
            Piece::Words(w) if open => {
                // The variable takes at least its first character.
                let first = rest.chars().next().map_or(0, char::len_utf8);
                let Some(at) = rest.get(first..).and_then(|r| r.find(w.as_str())) else {
                    return false;
                };
                rest = &rest[first + at + w.len()..];
                open = false;
            }
            Piece::Words(w) => match rest.strip_prefix(w.as_str()) {
                Some(r) => rest = r,
                None => return false,
            },
        }
    }
    if open {
        !rest.is_empty()
    } else {
        rest.is_empty()
    }
}

/// Refuses a run whose shown `names` hold the hidden-run banner in any of its language `shapes`.
pub fn normal(names: &[String], shapes: &[Vec<Piece>]) -> Result<(), String> {
    let banner = |n: &String| shapes.iter().any(|s| fits(s, n));
    if names.iter().any(banner) {
        return Err(HIDDEN_RUN.to_string());
    }
    Ok(())
}

/// Checks the shown `names` with `normal`, then lets `next` answer the notice the banner shows, until it shows none:
/// at most `most` notices, as the banner shows one at a time. `next` is true when it answered one.
fn wade(
    shapes: &[Vec<Piece>],
    most: usize,
    mut names: impl FnMut() -> Result<Vec<String>, String>,
    mut next: impl FnMut() -> Result<bool, String>,
) -> Result<(), String> {
    for _ in 0..=most {
        normal(&names()?, shapes)?;
        if !next()? {
            return Ok(());
        }
    }
    Err(format!("more than {most} notices showed at the start"))
}

/// The names in window `w` while it shows; none once it went.
fn names_kept(reader: &Reader, w: HWND) -> Result<Option<Vec<String>>, String> {
    kept(live::names(reader, w), || live::shown(w))
}

/// The banner among `windows`, each with its title: the first titled as one of `titles`, as kx-ui titles it.
fn banner_of<W: Copy>(windows: &[(W, String)], titles: &[String]) -> Option<W> {
    outline::titled(windows, titles).first().copied()
}

/// The notice banner's window while it shows a notice. Only it is read, as the keyboard redraws under other reads.
fn banner(titles: &[String]) -> Option<HWND> {
    let named: Vec<(HWND, String)> = product::shown()
        .into_iter()
        .map(|w| (w, win::title(w)))
        .collect();
    banner_of(&named, titles)
}

/// The names on the banner now; none while it is hidden.
fn banner_names(reader: &Reader, titles: &[String]) -> Result<Vec<String>, String> {
    banner(titles).map_or(Ok(Vec::new()), |w| {
        Ok(names_kept(reader, w)?.unwrap_or_default())
    })
}

/// How long to wait before a click on the banner, which answers nothing as the second click of a double click: the
/// system's double-click time and one poll more.
fn past_double(double_ms: u32, poll_ms: u64) -> u64 {
    u64::from(double_ms).saturating_add(poll_ms)
}

/// The system's double-click time, in ms.
#[allow(unsafe_code, reason = "A plain system call with no arguments.")]
fn double_click_ms() -> u32 {
    // SAFETY: it takes nothing and only reads a system setting.
    unsafe { GetDoubleClickTime() }
}

/// Clicks the middle of box `r` of window `w` with the real pointer, unless something covers it there.
fn click_in(w: HWND, r: &RECT, what: &str) -> Result<(), String> {
    let at = centre(r);
    if win::root_at(at) != w {
        return Err(format!("something covers the notice's {what} button"));
    }
    mouse::click_at(at)
}

/// True once a banner that read `before` moved on after its answer: it no longer `shows`, it went, or it reads
/// otherwise, as the next notice does; a hidden window may still read as it did.
fn moved_on(
    shows: bool,
    read: impl FnOnce() -> Result<Option<Vec<String>>, String>,
    before: &[String],
) -> Result<bool, String> {
    if !shows {
        return Ok(true);
    }
    Ok(read()?.is_none_or(|n| n != before))
}

/// Answers the notice banner `w` shows with its last button, No or OK as it never mirrors, once past the last answer's
/// double-click time, and waits up to `[g23] notice_wait_ms` for that notice to go: the banner hides or shows the next.
fn answer_last(ctx: &Ctx, reader: &Reader, w: HWND) -> Result<(), String> {
    let (wait, poll) = (ctx.cfg.g23.notice_wait_ms, ctx.cfg.timing.poll_ms);
    sleep_ms(past_double(double_click_ms(), poll));
    let before = names_kept(reader, w)?.unwrap_or_default();
    let buttons = live::items(reader, w, UIA_ButtonControlTypeId)?;
    let last = buttons
        .iter()
        .filter(|k| has_area(&k.rect))
        .max_by_key(|k| k.rect.right)
        .ok_or("the notice banner shows no button")?;
    click_in(w, &last.rect, "last")?;
    let read = || names_kept(reader, w);
    let went = || Ok(moved_on(live::shown(w), read, &before)?.then_some(()));
    poll_read(wait, poll, went)?
        .ok_or_else(|| format!("a notice stayed {wait} ms after its answer"))
}

/// Ends with `HIDDEN_RUN` when the product shows its hidden-run banner, in any shipped language, now or behind other
/// notices: each of those is answered first, at most one per notice key.
pub fn check_normal(ctx: &Ctx, reader: &Reader) -> Result<(), String> {
    let shapes = ftl::shapes(keys::HIDDEN_RUN)?;
    let titles = ftl::every(&ctx.cfg.g23.notice_title)?;
    let names = || banner_names(reader, &titles);
    let next = || banner(&titles).map_or(Ok(false), |w| answer_last(ctx, reader, w).map(|()| true));
    wade(&shapes, keys::ALL_KEYS.len(), names, next)
}

/// The banner's window while one of `texts` is its name for the notice it shows.
fn banner_with(
    reader: &Reader,
    titles: &[String],
    texts: &[String],
) -> Result<Option<HWND>, String> {
    let Some(w) = banner(titles) else {
        return Ok(None);
    };
    let holds = names_kept(reader, w)?.is_some_and(|n| n.iter().any(|s| texts.contains(s)));
    Ok(holds.then_some(w))
}

/// Answers the first backup offer Later with a real click once it shows within `[g23] notice_wait_ms`, and waits for
/// it to go: true when it was answered, false when it never showed.
pub fn answer_offer(ctx: &Ctx, reader: &Reader) -> Result<bool, String> {
    let (wait, poll) = (ctx.cfg.g23.notice_wait_ms, ctx.cfg.timing.poll_ms);
    let (asks, laters) = (ftl::every(keys::VAULT_OFFER)?, ftl::every(OFFER_ASK.no)?);
    let titles = ftl::every(&ctx.cfg.g23.notice_title)?;
    let shown = || banner_with(reader, &titles, &asks);
    let Some(hwnd) = poll_read(wait, poll, shown)? else {
        return Ok(false);
    };
    let buttons = live::items(reader, hwnd, UIA_ButtonControlTypeId)?;
    let later = buttons
        .iter()
        .find(|k| laters.contains(&k.name) && has_area(&k.rect))
        .ok_or("the backup offer shows no Later button")?;
    click_in(hwnd, &later.rect, "Later")?;
    let gone = || Ok(shown()?.is_none().then_some(()));
    poll_read(wait, poll, gone)?
        .map(|()| true)
        .ok_or_else(|| "the backup offer stayed after Later".to_string())
}

#[cfg(test)]
mod tests;

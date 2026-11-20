//! One clip dictated into target-window by clicks: the clip's layout in front, a card's wait when one is picked, the
//! Dictation key, the bar, Stop once the clip has played, then the typed text read until it settles. The foreground is
//! watched all through; the text is only counted, never kept or printed.
#![cfg(windows)]

use std::time::{Duration, Instant};

use kx_mod_keyboard::Role;
use kx_module_api::Redacted;

use super::bar::{self, BarLook};
use super::front::Watch;
use super::line::Waits;
use super::settle::Settle;
use super::{Case, Said, why};
use crate::apps::{Ctx, Opened};
use crate::g27type::Typist;
use crate::keymap::Reader;
use crate::ownkeys::{self, Real};
use crate::product::{self, Keyboard};
use crate::win::{self, ms_since, sleep_ms};
use crate::{board, langkey, langs};

/// What a dictation gave, before the foreground's count joins it.
struct Heard {
    bar: isize,
    look: BarLook,
    top_class: Option<String>,
    first_ms: Option<u64>,
    text: Redacted<String>,
}

/// Dictates `case` through the copy `pid` into the box `t` types for: the language's line.
pub(super) fn dictate(
    ctx: &Ctx,
    pid: u32,
    t: &Typist,
    case: &Case,
    waits: &Waits,
) -> Result<Said, String> {
    let (g, poll) = (&ctx.cfg.g22_product, ctx.cfg.timing.poll_ms);
    let id = g
        .layout(&case.lang)
        .ok_or("the clip's language has no layout")?;
    to_layout(t.app(), id, g.layout_wait_ms, poll)?;
    // Watched from here, as a card's check runs during the wait and may bring a window to the front.
    let watch = Watch::start(Duration::from_millis(poll));
    // A card serves only once its check of this language's model passed, and that check follows the language in front.
    sleep_ms(product::follow_ms(poll)?.saturating_add(waits.before_ms));
    let heard = listen(ctx, pid, t, case, waits);
    let fronts = watch.finish();
    let target = win::key(t.app().hwnd);
    let away = |e| format!("{e}; other windows in front: {:?}", fronts.away(target));
    let heard = heard.map_err(away)?;
    Ok(Said {
        lang: case.lang.clone(),
        clip_ms: case.clip_ms,
        first_ms: heard.first_ms,
        wrong: case.clip.wrong(heard.text.expose(), &case.lang),
        look: heard.look,
        front: fronts.count(target, heard.bar),
        top_class: heard.top_class,
    })
}

/// Clicks Dictation, waits for the bar and the clip, clicks Stop and reads the text once it settles.
fn listen(ctx: &Ctx, pid: u32, t: &Typist, case: &Case, waits: &Waits) -> Result<Heard, String> {
    let (g, poll) = (&ctx.cfg.g22_product, ctx.cfg.timing.poll_ms);
    let (kb, reader, stop_id) = (Keyboard::find()?, Reader::new()?, g.stop_id());
    let voice = board::id_of(Role::Voice)?;
    ownkeys::click(&Real, t.app(), &|| kb.click(&voice))?;
    let find = || bar::find(&reader, pid, kb.hwnd, &stop_id);
    let shown = crate::g23::live::poll_read(g.bar_wait_ms, poll, find)?;
    let (bar, _) = shown.ok_or_else(|| format!("no bar showed within {} ms", g.bar_wait_ms))?;
    let (at, (look, top_class)) = (Instant::now(), bar::look(bar, kb.hwnd)?);
    let play = waits.open_ms + u64::from(case.clip_ms) + ctx.cfg.g22_bench.tail_ms;
    sleep_ms(play.saturating_sub(ms_since(at)));
    let (now, stop) = find()?.ok_or("Stop went before its click")?;
    bar::stop(t.app(), now, stop)?;
    let stopped = Instant::now();
    let settles = settled(t, g.settle_ms, waits.text, poll);
    let (text, first) = settles.map_err(|e| why::with_bar(&e, &reader, now))?;
    let first_ms = first.map(|f| ms(f.saturating_duration_since(stopped)));
    Ok(Heard {
        bar: win::key(bar),
        look,
        top_class,
        first_ms,
        text,
    })
}

/// The box's text once it settles for `settle_ms`, read every `poll_ms` for up to `wait`, and when text first showed.
fn settled(
    t: &Typist,
    settle_ms: u64,
    wait: Duration,
    poll_ms: u64,
) -> Result<(Redacted<String>, Option<Instant>), String> {
    let mut settle = Settle::new(Duration::from_millis(settle_ms));
    let wait_ms = ms(wait);
    let read = || {
        let text = t.box_text().ok()?;
        settle.feed(text.expose(), Instant::now()).then_some(text)
    };
    let text = win::poll_until(wait_ms, poll_ms, read);
    let text = text.ok_or_else(|| format!("no typed text settled within {wait_ms} ms"))?;
    Ok((text, settle.first()))
}

/// Puts layout `id` in front of `app`'s box and waits up to `wait_ms` until it types in it.
fn to_layout(app: &Opened, id: u16, wait_ms: u64, poll_ms: u64) -> Result<(), String> {
    let h = langkey::installed_for(id).ok_or_else(|| format!("no installed layout {id:04X}"))?;
    langs::ask(app.hwnd, h)?;
    let took = || (langs::layout_of(app.hwnd) == Some(h)).then_some(());
    win::poll_until(wait_ms, poll_ms, took)
        .ok_or_else(|| format!("target-window did not take layout {id:04X} within {wait_ms} ms"))
}

/// `d` in whole ms.
fn ms(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

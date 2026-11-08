//! Translate up to its consent (Ruling S): the first click on the Arabic line's Translate asks with
//! `notice.translate-ask`, its Not now opens nothing, as no browser starts, and a second click asks again. The real
//! open sends text to the site, so only the owner's Yes in `translate-read` makes it.
#![cfg(windows)]

use kx_module_api::ports::SelChip;
use kx_module_api::{NoticeAsk, keys};
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::UIA_ButtonControlTypeId;

use crate::apps::Opened;
use crate::clippanel::has_area;
use crate::ftl::{self, Piece};
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::Lines;
use crate::g19chips::procs;
use crate::g23::live::{self, poll_read};
use crate::g23::notice::{banner, click_in, double_click_ms, fits, names_kept, past_double};
use crate::win::sleep_ms;

/// The rounds: the first click asks, and so does the next after Not now.
pub const ROUNDS: usize = 2;
/// The acts the step takes: each round's drag and Translate.
pub const ACTS: usize = 2 * ROUNDS;
/// The waits with no browser started: one per Not now.
pub const QUIET: usize = ROUNDS;

/// The banner's window once it asks Translate's question within `strip_wait_ms`.
fn asked(gate: &Gate, titles: &[String], shapes: &[Vec<Piece>]) -> Result<Option<HWND>, String> {
    let found = || asked_now(gate, titles, shapes);
    poll_read(gate.g().strip_wait_ms, gate.ctx.cfg.timing.poll_ms, found)
}

/// Answers the question on banner `w` Not now with a real click once past the double-click time, and waits up to
/// `gone_wait_ms` for it to go.
fn not_now(gate: &Gate, w: HWND, titles: &[String], shapes: &[Vec<Piece>]) -> Result<bool, String> {
    let poll = gate.ctx.cfg.timing.poll_ms;
    sleep_ms(past_double(double_click_ms(), poll));
    let laters = ftl::every(NoticeAsk::YES_LATER.no)?;
    let buttons = live::items(&gate.reader, w, UIA_ButtonControlTypeId)?;
    let later = (buttons.iter())
        .find(|k| laters.contains(&k.name) && has_area(&k.rect))
        .ok_or("Translate's question shows no Not now button")?;
    click_in(w, &later.rect, "Not now")?;
    let gone = || Ok(asked_now(gate, titles, shapes)?.is_none().then_some(()));
    Ok(poll_read(gate.g().gone_wait_ms, poll, gone)?.is_some())
}

/// The banner's window while it asks Translate's question now.
fn asked_now(
    gate: &Gate,
    titles: &[String],
    shapes: &[Vec<Piece>],
) -> Result<Option<HWND>, String> {
    let Some(w) = banner(titles) else {
        return Ok(None);
    };
    let names = names_kept(&gate.reader, w)?.unwrap_or_default();
    Ok(names
        .iter()
        .any(|n| shapes.iter().any(|s| fits(s, n)))
        .then_some(w))
}

/// One round: the Arabic line dragged, Translate clicked, the question answered Not now, and no browser started.
fn round(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    at: usize,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let (titles, shapes) = (
        ftl::every(&gate.ctx.cfg.g23.notice_title)?,
        ftl::shapes(keys::TRANSLATE_ASK)?,
    );
    let before = Gate::before();
    gate.acts
        .drag(app, lines.start(Line::Arabic), lines.end(Line::Arabic))?;
    let (case, shown) = gate.case(&format!("arabic_{at}"), &before, &gate.g().editable_chips)?;
    cases.push(case);
    let Some(s) = shown else {
        return Ok(json!({ "ok": false, "error": "no strip" }));
    };
    let running = procs::now();
    let (focus, _) = gate.tap(&s, SelChip::Translate)?;
    let ask = asked(gate, &titles, &shapes)?;
    let later = ask.map_or(Ok(false), |w| not_now(gate, w, &titles, &shapes))?;
    sleep_ms(gate.g().quiet_ms);
    let started = procs::launches(&running, &procs::now(), &gate.g().browsers).len();
    let ok = focus && ask.is_some() && later && started == 0;
    Ok(
        json!({ "asked": ask.is_some(), "not_now": later, "browsers_started": started, "focus": focus, "ok": ok }),
    )
}

/// Both rounds, with the consent the fresh data folder starts with; `cases` gathers each round's selection.
pub fn run(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let mut rounds = Vec::new();
    for at in 0..ROUNDS {
        rounds.push(round(gate, app, lines, at, cases)?);
    }
    let ok = !gate.sel.translate_ok && rounds.iter().all(|r| r["ok"] == true);
    Ok(json!({ "rounds": rounds, "consent_at_start": gate.sel.translate_ok, "ok": ok }))
}

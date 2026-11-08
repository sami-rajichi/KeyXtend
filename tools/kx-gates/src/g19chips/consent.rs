//! Translate up to its consent (Ruling S): the first click on the Arabic line's Translate asks with
//! `notice.translate-ask`, its Not now opens nothing, and a second click asks again. "Nothing" is watched through
//! `quiet_ms`: no browser starts, no browser window appears, the window in front stays, and the question never asks
//! again on its own. The gate stops before a click if the copy's consent is already given, as the click would open
//! the site; the real open is only the owner's Yes in `translate-read`.
#![cfg(windows)]

use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use kx_mod_selection::{ID, SPEC, SelSettings};
use kx_module_api::ports::SelChip;
use kx_module_api::{NoticeAsk, keys};
use kx_platform_windows::window::foreground;
use kx_settings::{peek, peek_section};
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::UIA_ButtonControlTypeId;

use crate::apps::{Ctx, Opened};
use crate::clippanel::has_area;
use crate::ftl::{self, Piece};
use crate::g19chips::docs::Line;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::Lines;
use crate::g19chips::{addr, procs};
use crate::g23::live::{self, poll_read};
use crate::g23::notice::{banner, click_in, double_click_ms, fits, names_kept, past_double};
use crate::keymap::Reader;
use crate::product;
use crate::win::{self, ms_since, sleep_ms};

/// The rounds: the first click asks, and so does the next after Not now.
pub const ROUNDS: usize = 2;
/// The acts the step takes: each round's drag and Translate.
pub const ACTS: usize = 2 * ROUNDS;
/// The waits with nothing opened: one per Not now.
pub const QUIET: usize = ROUNDS;

/// The question's titles and the shapes of its text in every shipped language.
pub struct Ask {
    titles: Vec<String>,
    shapes: Vec<Vec<Piece>>,
}

impl Ask {
    /// Translate's question, as `ctx`'s banner title and the shipped texts name it.
    pub fn new(ctx: &Ctx) -> Result<Self, String> {
        Ok(Self {
            titles: ftl::every(&ctx.cfg.g23.notice_title)?,
            shapes: ftl::shapes(keys::TRANSLATE_ASK)?,
        })
    }

    /// The banner's window while it asks the question now.
    pub fn shows(&self, reader: &Reader) -> Result<Option<HWND>, String> {
        let Some(w) = banner(&self.titles) else {
            return Ok(None);
        };
        let names = names_kept(reader, w)?.unwrap_or_default();
        let fit = names.iter().any(|n| self.shapes.iter().any(|s| fits(s, n)));
        Ok(fit.then_some(w))
    }
}

/// What the quiet wait after Not now saw, poll by poll.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Calm {
    /// Browsers started anew, at the most seen in one poll.
    launched: usize,
    /// Browser windows that were not there before the click.
    windows: usize,
    /// Another window came in front.
    front_moved: bool,
    /// The question showed again with no click.
    asked: bool,
}

impl Calm {
    /// One poll's sight, kept with the polls before it.
    pub fn note(&mut self, launched: usize, windows: usize, front_same: bool, asked: bool) {
        self.launched = self.launched.max(launched);
        self.windows = self.windows.max(windows);
        self.front_moved |= !front_same;
        self.asked |= asked;
    }

    /// True when nothing opened and the question asked once.
    pub fn ok(&self) -> bool {
        self.launched == 0 && self.windows == 0 && !self.front_moved && !self.asked
    }

    /// The result line's part.
    fn json(self) -> Value {
        json!({
            "browsers_started": self.launched, "browser_windows": self.windows, "front_moved": self.front_moved,
            "asked_again": self.asked, "ok": self.ok(),
        })
    }
}

/// Refuses a click on Translate while the consent is given, as it would open the site at once.
pub fn may_click(translate_ok: bool) -> Result<(), String> {
    if translate_ok {
        return Err(
            "Translate's consent is given on this copy, so the gate stops before a click opens the site"
                .into(),
        );
    }
    Ok(())
}

/// The consent in settings file `file`, which `exists` on disk: the saved one, the default while no file exists yet,
/// and an error when a file exists but does not read, so the gate stops.
pub fn consent_in(exists: bool, file: Option<&toml::Table>) -> Result<bool, String> {
    if exists && file.is_none() {
        return Err(
            "the copy's settings file does not read, so the gate stops before Translate".into(),
        );
    }
    let s: SelSettings =
        peek_section(file, ID, &SPEC).map_err(|e| format!("the selection settings: {e}"))?;
    Ok(s.translate_ok)
}

/// The consent of the copy at `program`, from its own settings file.
fn consent_of(program: &Path) -> Result<bool, String> {
    let files = product::files(program)?;
    consent_in(files.settings.exists(), peek(&files).as_ref())
}

/// Answers the question on banner `w` Not now with a real click once past the double-click time, and waits up to
/// `gone_wait_ms` for it to go.
fn not_now(gate: &Gate, w: HWND, ask: &Ask) -> Result<bool, String> {
    let poll = gate.ctx.cfg.timing.poll_ms;
    sleep_ms(past_double(double_click_ms(), poll));
    let laters = ftl::every(NoticeAsk::YES_LATER.no)?;
    let buttons = live::items(&gate.reader, w, UIA_ButtonControlTypeId)?;
    let later = (buttons.iter())
        .find(|k| laters.contains(&k.name) && has_area(&k.rect))
        .ok_or("Translate's question shows no Not now button")?;
    click_in(w, &later.rect, "Not now")?;
    let gone = || Ok(ask.shows(&gate.reader)?.is_none().then_some(()));
    Ok(poll_read(gate.g().gone_wait_ms, poll, gone)?.is_some())
}

/// Watches through `quiet_ms` after Not now: browsers started since `running`, browser windows not in `windows`,
/// the window in front against `front`, and the question showing again.
fn calm(
    gate: &Gate,
    running: &[procs::Proc],
    windows: &HashSet<isize>,
    front: HWND,
    ask: &Ask,
) -> Result<Calm, String> {
    let (g, poll) = (gate.g(), gate.ctx.cfg.timing.poll_ms);
    let (mut calm, from) = (Calm::default(), Instant::now());
    loop {
        let launched = procs::launches(running, &procs::now(), &g.browsers).len();
        let now: Vec<isize> = addr::windows(&g.browsers)
            .into_iter()
            .map(win::key)
            .collect();
        let asked = ask.shows(&gate.reader)?.is_some();
        calm.note(
            launched,
            procs::fresh(windows, &now),
            foreground() == front,
            asked,
        );
        if ms_since(from) >= g.quiet_ms {
            return Ok(calm);
        }
        sleep_ms(poll);
    }
}

/// One round: the Arabic line dragged, Translate clicked, the question answered Not now, and nothing opened.
fn round(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    at: usize,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let ask = Ask::new(gate.ctx)?;
    let before = Gate::before();
    gate.acts
        .drag(app, lines.start(Line::Arabic), lines.end(Line::Arabic))?;
    let (case, shown) = gate.case(&format!("arabic_{at}"), &before, &gate.g().editable_chips)?;
    cases.push(case);
    let Some(s) = shown else {
        return Ok(json!({ "ok": false, "error": "no strip" }));
    };
    may_click(consent_of(&gate.program)?)?;
    let browsers = &gate.g().browsers;
    let (running, windows): (_, HashSet<isize>) = (
        procs::now(),
        addr::windows(browsers).into_iter().map(win::key).collect(),
    );
    let (focus, _) = gate.tap(app, &s, SelChip::Translate)?;
    let front = foreground();
    let asked = poll_read(gate.g().strip_wait_ms, gate.ctx.cfg.timing.poll_ms, || {
        ask.shows(&gate.reader)
    })?;
    let later = asked.map_or(Ok(false), |w| not_now(gate, w, &ask))?;
    let calm = calm(gate, &running, &windows, front, &ask)?;
    let ok = focus && asked.is_some() && later && calm.ok();
    Ok(
        json!({ "asked": asked.is_some(), "not_now": later, "calm": calm.json(), "focus": focus, "ok": ok }),
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
    let ok = rounds.iter().all(|r| r["ok"] == true);
    Ok(json!({ "rounds": rounds, "ok": ok }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_settings_file_that_shows_but_does_not_read_stops_the_gate() {
        let e = consent_in(true, None).expect_err("unread");
        assert!(e.contains("stops"), "{e}");
        assert_eq!(
            consent_in(false, None),
            Ok(false),
            "no file yet: the default"
        );
        let given: toml::Table = format!("[{}]\ntranslate_ok = true\n", ID.as_str())
            .parse()
            .expect("TOML");
        assert_eq!(consent_in(true, Some(&given)), Ok(true));
    }

    #[test]
    fn a_click_is_refused_while_the_consent_is_given() {
        assert_eq!(may_click(false), Ok(()));
        let e = may_click(true).expect_err("given");
        assert!(e.contains("stops"), "{e}");
    }

    #[test]
    fn not_now_passes_only_when_nothing_opened_in_any_poll_and_the_question_asked_once() {
        let mut calm = Calm::default();
        calm.note(0, 0, true, false);
        assert!(calm.ok());
        for (launched, windows, same, asked) in [
            (1, 0, true, false),
            (0, 1, true, false),
            (0, 0, false, false),
            (0, 0, true, true),
        ] {
            let mut c = calm;
            c.note(launched, windows, same, asked);
            c.note(0, 0, true, false);
            assert!(
                !c.ok(),
                "{launched} {windows} {same} {asked}: a later quiet poll never hides it"
            );
        }
    }
}

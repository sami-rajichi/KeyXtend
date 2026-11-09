//! Properties over any run of focuses, clicks, acts, the keyboard's answers, open ends, typed pieces, consent answers
//! and waits on the fake clock: one act per round is in flight, each ends once within `edit_ms` as a tick or a
//! notice, an old round never ticks a newer strip, and no page opens without a Yes within `consent_s`.

use std::time::Duration;

use kx_module_api::keys;
use kx_module_api::ports::{ChipAct, ChipsView, EditAsk, EditDone, LinkError, SelChip, SelKind};
use kx_module_api::{Clock, Mono};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use toml::Value;

use super::super::{Busy, Msg};
use super::rig::Rig;
use super::spots::{caret, sel};
use super::taps::{Heard, release};
use crate::settings::TRANSLATE_OK_KEY;

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 40;
/// The longest wait in one step, in ms, past the default `edit_ms`.
const WAIT_MS: u64 = 7_000;
/// The lines a selection sits on, few so that boxes look alike.
const LINES: [i32; 2] = [40, 80];
/// How often an act names the strip before the one shown.
const OLD: f64 = 0.2;
/// The selection's text, so Translate has a page to open: "hello world".
const TEXT: &str = "مرحبا بالعالم";

/// One thing the actor meets.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// Focus moves to a new box of this kind holding a selection on a line, or to a bare caret.
    Focus(Option<SelKind>, usize),
    /// A press and release in another app.
    Click,
    /// A click on this keycap of the strip shown last, or of the one before it.
    Act(SelChip, bool),
    /// The keyboard answers one of the edits it has not answered yet, yes or no.
    Done(usize, bool),
    /// The held opens end, the browser taking them or not.
    Opened(bool),
    /// The keyboard typed a piece, a break or not.
    Typed(bool),
    /// The user answers the last question, yes or no.
    Answered(bool),
    /// Time passes, in ms.
    Tick(u64),
    /// Time passes beyond the consent's wait.
    Late,
}

fn step() -> impl Strategy<Value = Step> {
    let kind = proptest::option::of(select(vec![
        SelKind::Editable,
        SelKind::ReadOnly,
        SelKind::Terminal,
    ]));
    prop_oneof![
        2 => (kind, 0..LINES.len()).prop_map(|(k, l)| Step::Focus(k, l)),
        1 => Just(Step::Click),
        4 => (select(SelChip::ALL.to_vec()), proptest::bool::weighted(OLD)).prop_map(|(c, old)| Step::Act(c, old)),
        3 => (any::<usize>(), any::<bool>()).prop_map(|(at, ok)| Step::Done(at, ok)),
        2 => any::<bool>().prop_map(Step::Opened),
        1 => any::<bool>().prop_map(Step::Typed),
        3 => any::<bool>().prop_map(Step::Answered),
        2 => (0..=WAIT_MS).prop_map(Step::Tick),
        1 => Just(Step::Late),
    ]
}

/// What the run has seen: the edits the keyboard still owes an answer, the last question's time and whether it was
/// answered, and whether a Yes came in time.
#[derive(Default)]
struct Model {
    pending: Vec<EditAsk>,
    edits: usize,
    asked: Option<(Mono, bool)>,
    consented: bool,
}

impl Model {
    /// Takes `step` on `rig`, as the keyboard, the browser, the user or the clock would.
    fn take(&mut self, rig: &mut Rig, step: Step) {
        match step {
            Step::Focus(kind, line) => {
                let y = LINES[line];
                rig.focus(kind.map_or_else(|| caret(y), |k| sel(k, y)));
            }
            Step::Click => rig.click(),
            Step::Act(chip, old) => {
                if let Some(round) = live(rig).or_else(|| last(rig)) {
                    let round = if old { round.saturating_sub(1) } else { round };
                    rig.send(Msg::Act(ChipAct { round, chip }));
                }
            }
            Step::Done(at, ok) if !self.pending.is_empty() => {
                let ask = self.pending.remove(at % self.pending.len());
                rig.send(Msg::Done(EditDone {
                    round: ask.round,
                    ok,
                }));
            }
            Step::Done(..) => {}
            Step::Opened(ok) => {
                rig.web.set_error((!ok).then_some(LinkError::Unavailable));
                rig.release_opens();
            }
            Step::Typed(broke) => rig.send(Msg::Typed(broke)),
            Step::Answered(yes) => self.answer(rig, yes),
            Step::Tick(ms) => rig.advance(Duration::from_millis(ms)),
            Step::Late => rig.advance(
                rig.actor
                    .settings
                    .consent()
                    .saturating_add(Duration::from_millis(1)),
            ),
        }
    }

    /// The user answers the last question: a Yes counts once, and only within `consent_s` of it.
    fn answer(&mut self, rig: &mut Rig, yes: bool) {
        let Some(notice) = rig.question() else {
            return;
        };
        if let Some((at, answered)) = self.asked.as_mut() {
            let timely = rig.clock.now().saturating_since(*at) <= rig.actor.settings.consent();
            self.consented |= yes && timely && !*answered;
            *answered = true;
        }
        rig.send(Msg::Answered { notice, yes });
    }

    /// Notes the edits asked and a question raised in the step that published `new`.
    fn note(&mut self, rig: &Rig, new: &[Heard]) {
        let edits = rig.edits();
        self.pending.extend_from_slice(&edits[self.edits..]);
        self.edits = edits.len();
        if new
            .iter()
            .any(|h| matches!(h, Heard::Notice(n) if n.key == keys::TRANSLATE_ASK))
        {
            self.asked = Some((rig.clock.now(), false));
        }
    }
}

/// The round of the live strip, if one shows untouched.
fn live(rig: &Rig) -> Option<u32> {
    match rig.view() {
        ChipsView::Shown {
            round, acted: None, ..
        } => Some(round),
        _ => None,
    }
}

/// The round of the last strip shown, hidden since or not.
fn last(rig: &Rig) -> Option<u32> {
    let views = rig.views();
    views.iter().rev().find_map(|v| match v {
        ChipsView::Shown { round, .. } => Some(*round),
        ChipsView::Hidden => None,
    })
}

/// Whether `heard` says an act did not go.
fn failed(heard: &Heard) -> bool {
    matches!(heard, Heard::Notice(n) if n.key == keys::KEYS_NOT_SENT || n.key == keys::TRANSLATE_FAILED)
}

/// Whether `heard` is a tick, or the tick of `busy` when one is named.
fn ticks(heard: &Heard, busy: Option<&Busy>) -> bool {
    let Heard::View(ChipsView::Shown {
        round,
        acted: Some(chip),
        ..
    }) = heard
    else {
        return false;
    };
    busy.is_none_or(|b| b.round == *round && b.chip == *chip)
}

/// At most one act per round is in flight, and each is due to end within `edit_ms` from now.
fn check_busy(rig: &Rig) -> Result<(), TestCaseError> {
    let (busy, now, edit) = (&rig.actor.busy, rig.clock.now(), rig.actor.settings.edit());
    for (at, b) in busy.iter().enumerate() {
        prop_assert!(
            busy[..at].iter().all(|o| o.round != b.round),
            "two acts in round {}",
            b.round
        );
        prop_assert!(
            b.until > now && b.until.saturating_since(now) <= edit,
            "{b:?} outlived edit_ms"
        );
    }
    Ok(())
}

/// Each act that left the list in `step` ended as its tick or a notice, or with its strip already gone; an answer,
/// an open's end or a wait tells no more ends than it took.
fn check_ends(
    before: &[Busy],
    rig: &Rig,
    live: Option<u32>,
    new: &[Heard],
    step: Step,
) -> Result<(), TestCaseError> {
    let ended: Vec<&Busy> = before
        .iter()
        .filter(|b| !rig.actor.busy.contains(b))
        .collect();
    let told = new.iter().filter(|h| failed(h)).count();
    for b in &ended {
        let ticked = new.iter().any(|h| ticks(h, Some(b)));
        prop_assert!(
            ticked || told > 0 || live != Some(b.round),
            "{b:?} ended unseen: {new:?}"
        );
    }
    if matches!(
        step,
        Step::Done(..) | Step::Opened(_) | Step::Tick(_) | Step::Late
    ) {
        let signals = told + new.iter().filter(|h| ticks(h, None)).count();
        prop_assert!(signals <= ended.len(), "an act ended twice: {new:?}");
    }
    Ok(())
}

/// Every tick lands on the live strip of its own round, never on a newer one.
fn check_ticks(views: &[ChipsView]) -> Result<(), TestCaseError> {
    let mut live = None;
    for view in views {
        match view {
            ChipsView::Shown {
                round, acted: None, ..
            } => live = Some(*round),
            ChipsView::Shown { round, .. } => {
                prop_assert_eq!(live, Some(*round), "a tick on another strip");
                live = None;
            }
            ChipsView::Hidden => live = None,
        }
    }
    Ok(())
}

/// Runs `steps` on a fresh rig whose opens wait to be ended, checking after each one; when `saved`, the user said Yes
/// to Translate in an earlier run, so its clicks open at once.
fn run(saved: bool, steps: &[Step]) -> Result<(), TestCaseError> {
    // Opens an earlier run left held end now, into that run's gone actor.
    release();
    let mut rig = Rig::build(|h| h.put(TRANSLATE_OK_KEY, Value::Boolean(saved)));
    let mut model = Model {
        consented: saved,
        ..Model::default()
    };
    rig.watch.set_clip(true);
    rig.watch.set_text(TEXT);
    rig.hold_opens();
    for &step in steps {
        let (before, heard, live) = (rig.actor.busy.clone(), rig.count(), live(&rig));
        model.take(&mut rig, step);
        let new = rig.heard().split_off(heard);
        check_busy(&rig)?;
        check_ends(&before, &rig, live, &new, step)?;
        model.note(&rig, &new);
        prop_assert!(
            rig.web.opened().is_empty() || model.consented,
            "a page opened without a timely Yes"
        );
    }
    check_ticks(&rig.views())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Any run keeps one act per round in flight, ends each once within `edit_ms` as a tick or a notice, never ticks
    /// a newer strip for an old round, and opens no page before a timely Yes.
    #[test]
    fn acts_end_once_in_time_on_their_own_strip_and_pages_wait_for_a_timely_yes(
        saved in any::<bool>(),
        steps in vec(step(), 1..=MAX_STEPS),
    ) {
        run(saved, &steps)?;
    }
}

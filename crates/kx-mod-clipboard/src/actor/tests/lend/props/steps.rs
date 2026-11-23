//! The steps the lent paste's property test draws, the texts they copy, how often each comes, and how the run does
//! each.

use std::time::Duration;

use kx_module_api::ports::{
    AppRef, ClipAct, ClipError, ClipNote, ClipTake, ClipText, MAX_FIELD_CHARS, PasteText, Pasted,
};
use kx_module_api::{Clock, ModuleId, Redacted, Zeroizing};
use proptest::prelude::*;

use super::super::super::ears::Heard;
use super::super::skip::VAULT;
use super::super::{SAID, WAIT_MS};
use super::Run;
use crate::actor::Msg;
use crate::fixture::{app, text};
use crate::settings::ClipSettings;

/// The share of answers that say the paste went out.
const OK_SHARE: f64 = 0.8;
/// The module whose Paste takes the clipboard's text.
const TAKER: ModuleId = ModuleId::new("voice");
/// The text copied.
pub(super) const COPY: &str = "x";
/// The text a password manager copies with a private marker.
pub(super) const PRIVATE: &str = "p";
/// The text a skipped app copies, unmarked.
pub(super) const VAULTED: &str = "v";
/// The text a password manager copies marked by number markers alone, which the presence check lets through.
pub(super) const NUMBERED: &str = "n";
/// The text an app copies and a take reads before the watch tells it.
const KEY: &str = "k";

/// One thing that can happen around a lent paste.
#[derive(Clone, Debug)]
pub(super) enum Step {
    /// Another module asks for its text pasted.
    Lend,
    /// The keyboard answers the last request, or an older one when `back`.
    Answer { back: bool, ok: bool },
    /// Moves the clock on by this many ms.
    Advance(u64),
    /// The user copies `COPY`.
    Copy,
    /// A skipped app copies `VAULTED`.
    Skipped,
    /// A password manager copies `PRIVATE` with a private marker.
    Private,
    /// A password manager copies `NUMBERED`, marked by number markers alone.
    Numbered,
    /// The watch tells a marked copy.
    Marked,
    /// The user clears the clipboard.
    Clear,
    /// The next text read is refused as busy.
    Busy,
    /// Opens the panel, ticks the top row and clicks Insert.
    Insert,
    /// The voice key page's or Quick-fill's Paste takes the clipboard's text.
    Take,
    /// An app copies `KEY`, and a take reads it before the watch tells it.
    EarlyTake,
    /// The watch tells the copy an early take read, if still due; it always comes before the next note.
    Late,
    /// Opens the panel and deletes every entry.
    Delete,
}

impl Step {
    /// True when the watch tells a note for this step.
    pub(super) const fn notes(&self) -> bool {
        matches!(
            self,
            Self::Copy
                | Self::Skipped
                | Self::Private
                | Self::Numbered
                | Self::Marked
                | Self::Clear
                | Self::Late
        )
    }
}

/// Any step; the clock steps hit the gap and the paste wait most.
pub(super) fn step() -> impl Strategy<Value = Step> {
    let s = ClipSettings::from_defaults().unwrap();
    let ms = prop_oneof![
        2 => Just(s.paste_gap_ms),
        2 => Just(s.paste_wait_ms),
        1 => 0..s.paste_wait_ms,
    ];
    prop_oneof![
        4 => Just(Step::Lend),
        4 => (any::<bool>(), prop::bool::weighted(OK_SHARE)).prop_map(|(back, ok)| Step::Answer { back, ok }),
        3 => ms.prop_map(|n| Step::Advance(u64::from(n))),
        2 => Just(Step::Copy),
        2 => Just(Step::Marked),
        1 => Just(Step::Private),
        1 => Just(Step::Numbered),
        1 => Just(Step::Skipped),
        1 => Just(Step::Clear),
        1 => Just(Step::Busy),
        1 => Just(Step::Insert),
        1 => Just(Step::Take),
        1 => Just(Step::EarlyTake),
        1 => Just(Step::Late),
        1 => Just(Step::Delete),
    ]
}

impl Run {
    /// Does `step`.
    pub(super) fn apply(&mut self, step: &Step) {
        let rig = &mut self.rig;
        match step {
            Step::Lend => self.lend(),
            Step::Answer { back, ok } => {
                let last = rig.pastes().last().map_or(0, |p| p.round);
                let round = last.wrapping_sub(u32::from(*back));
                rig.send(Msg::Pasted(Pasted { round, ok: *ok }));
            }
            Step::Advance(ms) => rig.advance(Duration::from_millis(*ms)),
            Step::Copy => self.copy(COPY, None),
            Step::Skipped => self.copy(VAULTED, Some(app(VAULT))),
            Step::Private => {
                rig.fakes.clip.set_marked(Some(PRIVATE));
                rig.push(ClipNote::Marked { app: None });
            }
            Step::Numbered => {
                rig.fakes.clip.set_text(Some(NUMBERED));
                rig.push(ClipNote::Marked { app: None });
            }
            Step::Marked => rig.push(ClipNote::Marked { app: None }),
            Step::Clear => {
                rig.fakes.clip.set_text(None);
                rig.push(ClipNote::Cleared { app: None });
            }
            Step::Busy => rig.fakes.clip.fail_text(ClipError::Busy),
            Step::Insert | Step::Delete => self.click(matches!(step, Step::Insert)),
            Step::Take => self.take(),
            Step::EarlyTake => {
                rig.fakes.clip.set_text(Some(KEY));
                self.take();
                self.late = true;
            }
            Step::Late => {
                self.late = false;
                rig.copy(vec![text(KEY)], None);
            }
        }
    }

    /// The clipboard holds `said`, copied by `by`, and the watch tells it.
    fn copy(&mut self, said: &str, by: Option<AppRef>) {
        self.rig.fakes.clip.set_text(Some(said));
        self.rig.copy(vec![text(said)], by);
    }

    /// Asks for `SAID` pasted on the next round, as the voice module does.
    fn lend(&mut self) {
        self.asks += 1;
        let text = Redacted::new(Zeroizing::new(SAID.to_owned()));
        let ask = PasteText {
            round: self.asks,
            text,
            wait_ms: WAIT_MS,
        };
        let now = self.rig.fakes.clock.now();
        self.rig.send(Msg::Lend(ask, now));
    }

    /// `TAKER`'s Paste takes the clipboard's text, which then counts as gone unless an entry still holds it.
    fn take(&mut self) {
        let take = ClipTake::new(TAKER, 0, MAX_FIELD_CHARS);
        self.rig.send(Msg::Take(take));
        let answer = self.rig.all().into_iter().rev().find_map(|h| match h {
            Heard::Taken(t) => Some(t.text),
            _ => None,
        });
        if let Some(ClipText::Text(read)) = answer {
            self.gone.push(read.expose().as_str().to_owned());
        }
    }

    /// Opens the panel unless it is open, then clicks Insert on the top row, or Delete all and Yes.
    fn click(&mut self, insert: bool) {
        let rig = &mut self.rig;
        if rig.panels().last() != Some(&true) {
            rig.open();
        }
        if insert {
            rig.tick_row(0);
            rig.send(Msg::Act(ClipAct::Insert));
        } else {
            rig.send(Msg::Act(ClipAct::DeleteAll));
            rig.send(Msg::Act(ClipAct::Confirm(true)));
        }
    }
}

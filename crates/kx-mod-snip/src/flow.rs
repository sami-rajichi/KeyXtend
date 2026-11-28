//! The capture flow (design "What the user sees"): picking, the strip, the countdown, recording, the close question and
//! the picture editor, on an injected `Clock`. Pure: inputs in, steps out; the actor runs them on ports and the bus.

mod edit;
mod ends;
mod frame;
mod io;
mod keep;
mod pick;
mod rec;
mod stage;

pub use io::{Call, In, Out, Outcome, Timed, Video, What};
pub use stage::{Count, Edit, Film, Held, Picking, Rec, Recorded, Reel, Sounds, Stage, Strip};

use std::sync::Arc;
use std::time::Duration;

use kx_module_api::ports::{Frozen, Keep, SnipLit};
use kx_module_api::{Clock, Mono};

use crate::SnipSettings;
use keep::{Busy, Files};

/// The countdown's step, as `countdown_s` counts whole seconds.
const SECOND: Duration = Duration::from_secs(1);

/// What follows the close question's Save or Don't save.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum After {
    /// The snip ends.
    End,
    /// The snip ends and a new one starts, as the Snip key asked in the editor (edge 22).
    Start,
}

/// The tick of a Copy or Save: when it goes, and whether the snip ends then.
#[derive(Copy, Clone, Debug)]
struct Due {
    at: Mono,
    /// The strip's snip, or the one the question's Save closed, ends with the tick.
    ends: bool,
}

/// The capture state machine: one snip at a time, and nothing kept unless the user asks.
pub struct Flow {
    clock: Arc<dyn Clock>,
    /// The settings the running snip started with, or the next one starts with.
    settings: SnipSettings,
    /// Settings that came during a snip, for the next one.
    next: Option<Box<SnipSettings>>,
    stage: Stage,
    frozen: Option<Frozen>,
    /// The close question shows, with what follows its answer.
    ask: Option<After>,
    busy: Option<Busy>,
    /// The snip's changes since it was taken.
    changes: u32,
    /// The changes counted at its last Copy or Save.
    kept_at: Option<u32>,
    files: Files,
    /// The last video number given.
    videos: u32,
    /// The Snip key is lit.
    lit: bool,
    /// The keyboard stepped aside.
    aside: bool,
    /// The freeze waits for the ui's answer that the keyboard stepped aside, until then.
    stepping: Option<Mono>,
    /// The tick of a Copy or Save.
    due: Option<Due>,
    /// The theme's frame handle click area across, in logical pixels.
    hit_px: u16,
}

impl Flow {
    /// A flow with no snip, reading time from `clock`.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, settings: SnipSettings) -> Self {
        Self {
            clock,
            settings,
            next: None,
            stage: Stage::Idle,
            frozen: None,
            ask: None,
            busy: None,
            changes: 0,
            kept_at: None,
            files: Files::default(),
            videos: 0,
            lit: false,
            aside: false,
            stepping: None,
            due: None,
            hit_px: 0,
        }
    }

    /// Takes one input and gives the steps to carry out, in order.
    #[must_use]
    pub fn on(&mut self, input: In) -> Vec<Out> {
        let mut out = Vec::new();
        match input {
            In::Key => self.key(&mut out),
            In::Esc => self.esc(&mut out),
            In::Aside => self.stepped(&mut out),
            In::Bare => self.bare(&mut out),
            In::Frozen(answer) => self.froze(answer, &mut out),
            In::Bar(key) => self.bar(key, &mut out),
            In::Spot(at) => self.spot(at),
            In::HitPx(px) => self.hit_px = px,
            In::Handle(handle) => self.pick_up(handle),
            In::Strip(key) => self.strip(key, &mut out),
            In::Record(key) => self.rec_key(key, &mut out),
            In::Note { video, note } => self.note(video, note, &mut out),
            In::Edit(key) => self.edit_key(key, &mut out),
            In::Changed => self.drew(),
            In::Flat { keep, shot } => self.flat(keep, shot, &mut out),
            In::Ask(key) => self.answer(key, &mut out),
            In::Kept(outcome) => self.kept_answer(outcome, &mut out),
            In::Timing => self.timing(),
            In::Settings(settings) => self.set(settings),
            In::Tick => self.tick(&mut out),
            In::Quit => self.end(&mut out),
        }
        out
    }

    /// When the freeze stops waiting for the keyboard, the next countdown second or a tick's fade is due, for
    /// `In::Tick`.
    #[must_use]
    pub fn deadline(&self) -> Option<Mono> {
        match &self.stage {
            Stage::Count(count) => Some(count.next),
            _ => self.stepping.or(self.due.map(|due| due.at)),
        }
    }

    /// Where the snip stands.
    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    /// The frozen screens, from the freeze until Record, Edit or the end.
    #[must_use]
    pub fn frozen(&self) -> Option<&Frozen> {
        self.frozen.as_ref()
    }

    /// The close question shows over the strip or the editor.
    #[must_use]
    pub fn asking(&self) -> bool {
        self.ask.is_some()
    }

    /// The Snip key's light now, as last told.
    #[must_use]
    pub fn lit(&self) -> SnipLit {
        if self.lit { SnipLit::On } else { SnipLit::Off }
    }

    fn now(&self) -> Mono {
        self.clock.now()
    }

    /// No Copy or Save waits and no question shows, so the strip's and the editor's keys act.
    fn free(&self) -> bool {
        self.busy.is_none() && self.ask.is_none()
    }

    /// The snip was copied or saved since its last change.
    fn kept(&self) -> bool {
        self.kept_at == Some(self.changes)
    }

    /// A snip just taken: unchanged and not kept.
    fn fresh(&mut self) {
        (self.changes, self.kept_at) = (0, None);
    }

    /// A new video's number.
    fn video(&mut self) -> Video {
        self.videos = self.videos.wrapping_add(1);
        Video(self.videos)
    }

    /// New settings: at once while no snip runs, else for the next snip.
    fn set(&mut self, settings: Box<SnipSettings>) {
        if matches!(self.stage, Stage::Idle) {
            self.settings = *settings;
        } else {
            self.next = Some(settings);
        }
    }

    /// A freeze no answer came for, the countdown's seconds that passed, and a tick whose time came: it ends its snip,
    /// or fades where the snip stays open.
    fn tick(&mut self, out: &mut Vec<Out>) {
        let now = self.now();
        if self.stepping.is_some_and(|until| now >= until) {
            self.stepped(out);
        }
        self.count_down(now, out);
        let Some(due) = self.due.filter(|due| now >= due.at) else {
            return;
        };
        self.due = None;
        if due.ends {
            self.end(out);
        } else if let Some(tick) = self.lasting() {
            *tick = None;
        }
    }

    /// The tick of a window that stays open after its Copy or Save: the editor's, or a recorded video's strip's.
    fn lasting(&mut self) -> Option<&mut Option<Keep>> {
        match &mut self.stage {
            Stage::Edit(edit) => Some(&mut edit.done),
            Stage::Strip(strip) if matches!(strip.held, Held::Video(_)) => Some(&mut strip.done),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;

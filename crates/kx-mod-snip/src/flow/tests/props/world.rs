//! The world the flow's steps act on: the unsaved part's videos, the copied ones, each run, the Copy or Save asked,
//! the keyboard and the light. It answers as the ports would and checks each step as it applies it.

mod notes;

use std::collections::{BTreeMap, BTreeSet};

use super::super::*;
use super::{Step, clicked, freeze, settings_of};
use proptest::prelude::*;

/// A recording run as the host keeps it.
#[derive(Copy, Clone, Debug, Default)]
struct Run {
    /// Its last note was told.
    ended: bool,
    /// Cancel was called.
    cancelled: bool,
}

/// The rig and what the ports did.
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact the property checks: open, frozen, editor left, changed and kept."
)]
pub(super) struct World {
    pub(super) rig: Rig,
    /// The videos in the unsaved part.
    unsaved: BTreeSet<Video>,
    /// The videos the user copied, which may stay.
    copied: BTreeSet<Video>,
    runs: BTreeMap<Video, Run>,
    /// The Copy or Save asked, or Play's re-timing as `None`, not yet answered, with the video it copies, if any.
    asked: Option<(Option<Keep>, Option<Video>)>,
    /// A snip runs, from the key's light to `End`.
    open: bool,
    /// The snip asked its freeze.
    froze: bool,
    aside: Option<SnipAside>,
    /// The step began in the editor, whose windows may still show.
    editing: bool,
    lit: SnipLit,
    /// Nothing changed since the Copy or Save asked last.
    clean: bool,
    /// The snip held was copied or saved since its last change.
    kept: bool,
}

impl World {
    pub(super) fn new(rig: Rig) -> Self {
        Self {
            rig,
            unsaved: BTreeSet::new(),
            copied: BTreeSet::new(),
            runs: BTreeMap::new(),
            asked: None,
            open: false,
            froze: false,
            aside: None,
            editing: false,
            lit: SnipLit::Off,
            clean: true,
            kept: false,
        }
    }

    /// No Copy or Save waits and no question shows.
    pub(super) fn free(&self) -> bool {
        self.asked.is_none() && !self.rig.flow.asking()
    }

    pub(super) fn kept(&self) -> bool {
        self.kept
    }

    /// Feeds `step`, answering as the ports would, and applies the steps that come out.
    pub(super) fn act(&mut self, step: &Step) -> Result<Vec<Out>, TestCaseError> {
        let before = self.rig.stage().clone();
        let outs = match self.input(step) {
            Some(input) => self.rig.on(input),
            None => Vec::new(),
        };
        self.follow(step, &before);
        self.editing = matches!(before, Stage::Edit(_));
        for out in &outs {
            self.apply(step, out)?;
        }
        Ok(outs)
    }

    /// Whenever no snip runs, nothing unchosen is left, every run is over, and the keyboard and light are back.
    pub(super) fn check_idle(&self) -> Result<(), TestCaseError> {
        if self.rig.stage() != &Stage::Idle {
            return Ok(());
        }
        prop_assert!(!self.open, "idle inside a snip");
        let left: Vec<_> = self.unsaved.difference(&self.copied).collect();
        prop_assert!(left.is_empty(), "left in unsaved: {left:?}");
        let live = self.runs.values().any(|r| !r.ended && !r.cancelled);
        prop_assert!(!live, "a run records after the snip ended");
        prop_assert_eq!(self.lit, SnipLit::Off);
        prop_assert!(
            matches!(self.aside, None | Some(SnipAside::Back | SnipAside::Stay)),
            "{:?}",
            self.aside
        );
        Ok(())
    }

    /// The input `step` makes, after the ports' side of it; `None` when it has nothing to act on.
    fn input(&mut self, step: &Step) -> Option<In> {
        Some(match step {
            Step::Key => In::Key,
            Step::Esc => In::Esc,
            Step::Aside => In::Aside,
            Step::Bare => In::Bare,
            Step::Freeze(froze) => In::Frozen(freeze(*froze)),
            Step::Bar(key) => In::Bar(*key),
            Step::Spot(at) => In::Spot(*at),
            Step::HitPx(px) => In::HitPx(*px),
            Step::Handle(handle) => In::Handle(*handle),
            Step::Strip(key) => In::Strip(*key),
            Step::Record(key) => In::Record(*key),
            Step::Note(said) => return self.note(*said),
            Step::Late => return self.late(),
            Step::Stale(said) => return self.stale(*said),
            Step::Settings(keyboard, count, mic) => {
                In::Settings(Box::new(settings_of(*keyboard, *count, *mic)))
            }
            Step::Edit(key) => In::Edit(*key),
            Step::Changed => In::Changed,
            Step::Flat(keep) => In::Flat {
                keep: *keep,
                shot: flat(),
            },
            Step::Ask(key) => In::Ask(*key),
            Step::Answer(n) => In::Kept(self.answer(*n)),
            Step::Timing => In::Timing,
            Step::Wait(by) => {
                self.rig.clock.advance(*by);
                In::Tick
            }
            Step::Quit => In::Quit,
            Step::Snap(_) | Step::Shoot | Step::Open => return None,
        })
    }

    /// The asked Copy or Save's outcome; a copied video joins the copied ones.
    fn answer(&mut self, n: u8) -> Outcome {
        let outcome = match n {
            0 => Outcome::Done,
            1 => Outcome::Cancelled,
            _ => Outcome::Failed(SnipNote::SaveFailed),
        };
        let asked = self.asked.take();
        if let Some((Some(keep), clip)) = asked.filter(|_| outcome == Outcome::Done) {
            self.kept = self.clean;
            if let (Keep::Copy, Some(video)) = (keep, clip) {
                self.copied.insert(video);
            }
        }
        outcome
    }

    /// Follows what the step changed in the snip held: a new snip is unkept, and so is the editor after a change and a
    /// recorded video after a new speed.
    fn follow(&mut self, step: &Step, before: &Stage) {
        let after = self.rig.stage();
        let taken = matches!(after, Stage::Strip(_)) && !matches!(before, Stage::Strip(_));
        let video = matches!(before, Stage::Strip(s) if matches!(s.held, Held::Video(_)));
        let sped = video && matches!(step, Step::Strip(StripKey::Speed)) && after != before;
        let changed = sped || matches!(before, Stage::Edit(_)) && matches!(step, Step::Changed);
        if changed {
            self.clean = false;
        }
        if taken || changed {
            self.kept = false;
        }
    }

    /// Applies one step out, checking it.
    fn apply(&mut self, step: &Step, out: &Out) -> Result<(), TestCaseError> {
        match out {
            Out::Lit(lit) => self.light(*lit)?,
            Out::Aside(aside) => self.aside = Some(*aside),
            Out::Freeze => self.freeze(step)?,
            Out::End => {
                prop_assert!(self.open, "an end with no snip");
                // A video whose Copy is in flight may reach the clipboard, so it stays as a copied one.
                if let Some((Some(Keep::Copy), Some(video))) = self.asked.take() {
                    self.copied.insert(video);
                }
                self.open = false;
            }
            Out::Record { video, .. } => {
                prop_assert!(
                    self.runs.insert(*video, Run::default()).is_none(),
                    "{video:?} again"
                );
                self.unsaved.insert(*video);
            }
            Out::Run { video, call } => self.call(*video, *call)?,
            Out::Keep { keep, what } => self.keep(step, *keep, what)?,
            Out::Delete(video) => {
                prop_assert!(!self.copied.contains(video), "deleted copied {video:?}");
                self.unsaved.remove(video);
            }
            Out::Play(video) => self.play(step, *video)?,
            Out::Retime { video, timed } => self.retime(step, *video, *timed)?,
            Out::Note(_) | Out::Panel | Out::Edit(_) => {}
        }
        Ok(())
    }

    /// The key's light: lit, a snip starts, one at a time, with no step aside yet.
    fn light(&mut self, lit: SnipLit) -> Result<(), TestCaseError> {
        if lit == SnipLit::On {
            prop_assert!(!self.open, "a second snip started");
            (self.open, self.froze, self.kept, self.aside) = (true, false, false, None);
        }
        self.lit = lit;
        Ok(())
    }

    /// The freeze, once a snip: after a step aside or a stay, only on the ui's answer or the deadline's tick, and
    /// never in the step that left the editor.
    fn freeze(&mut self, step: &Step) -> Result<(), TestCaseError> {
        prop_assert!(
            self.open && !self.froze,
            "a freeze outside a snip, or twice"
        );
        prop_assert!(!self.editing, "froze while the editor may show: {step:?}");
        let waits = matches!(
            self.aside,
            Some(SnipAside::Bubble | SnipAside::Hidden | SnipAside::Stay)
        );
        let answered = matches!(step, Step::Aside | Step::Wait(_));
        prop_assert!(
            !waits || answered,
            "froze before the keyboard stepped aside: {step:?}"
        );
        self.froze = true;
        Ok(())
    }

    /// A call on a run; Cancel deletes the video unless its end was told.
    fn call(&mut self, video: Video, call: Call) -> Result<(), TestCaseError> {
        let Some(run) = self.runs.get_mut(&video) else {
            return Err(TestCaseError::fail(format!("{call:?} on no run {video:?}")));
        };
        if call == Call::Cancel {
            run.cancelled = true;
            if !run.ended {
                self.unsaved.remove(&video);
            }
        }
        Ok(())
    }

    /// A Copy or Save asked: only by its own click, one at a time, of a video still there.
    fn keep(&mut self, step: &Step, keep: Keep, what: &What) -> Result<(), TestCaseError> {
        prop_assert_eq!(clicked(step), Some(keep), "{:?} asked by {:?}", keep, step);
        prop_assert!(self.asked.is_none(), "a second keep while one waits");
        let clip = match what {
            What::Picture(_) => None,
            What::Video { video, timed } => {
                prop_assert!(self.unsaved.contains(video), "kept a gone {video:?}");
                let into = timed.map(|t| t.into);
                if let Some(copy) = into {
                    self.unsaved.insert(copy);
                }
                Some(into.unwrap_or(*video))
            }
        };
        (self.asked, self.clean) = (Some((Some(keep), clip)), true);
        Ok(())
    }

    /// Play: only by its click, or by the answer that its copy is made, of a video still there.
    fn play(&self, step: &Step, video: Video) -> Result<(), TestCaseError> {
        let by = matches!(step, Step::Strip(StripKey::Play) | Step::Answer(_));
        prop_assert!(by, "played {:?} by {:?}", video, step);
        prop_assert!(self.unsaved.contains(&video), "played {video:?}");
        Ok(())
    }

    /// Play's re-timing: only by its click, one at a time, of a recording still there, into a new copy.
    fn retime(&mut self, step: &Step, video: Video, timed: Timed) -> Result<(), TestCaseError> {
        prop_assert!(
            matches!(step, Step::Strip(StripKey::Play)),
            "re-timed by {step:?}"
        );
        prop_assert!(self.asked.is_none(), "a re-timing while a keep waits");
        prop_assert!(self.unsaved.contains(&video), "re-timed a gone {video:?}");
        prop_assert!(self.unsaved.insert(timed.into), "{:?} again", timed.into);
        self.asked = Some((None, None));
        Ok(())
    }
}

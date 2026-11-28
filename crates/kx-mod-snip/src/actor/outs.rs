//! The flow's steps, carried out in order: the light and the keyboard's step aside, the freeze, notes, the panel, the
//! recordings, the editor, Copy and Save, Play, deletes, and the end, which drops the editor and the frozen pixels.

use kx_module_api::ports::{FreezeError, Frozen, FrozenScreen, Shot, ShotCap, SnipAside, SnipNote};

use super::limits::NO_JOB;
use super::{Actor, Back, Job, Msg};
use crate::edit::Canvas;
use crate::flow::{In, Out, Video};

impl Actor {
    /// Carries out one step of the flow.
    pub(super) fn out(&mut self, out: Out) {
        match out {
            Out::Lit(lit) => self.publish(&lit),
            Out::Aside(aside) => self.step_aside(aside),
            Out::Freeze => self.freeze(),
            Out::Note(note) => self.show_note(note),
            Out::Panel => self.open_panel(),
            Out::Record {
                video,
                part,
                mic,
                sound,
            } => self.record(video, part, mic, sound),
            Out::Run { video, call } => self.call(video, call),
            Out::Edit(shot) => self.open_editor(shot),
            Out::Keep { keep, what } => self.keep(keep, what),
            Out::Play(video) => self.play(video),
            Out::Retime { video, timed } => self.retime(video, timed),
            Out::Delete(video) => self.delete(video),
            Out::End => self.ended(),
        }
    }

    /// Carries out a job's answer.
    pub(super) fn answer(&mut self, back: Back) {
        match back {
            Back::Facts(facts) => self.seen.facts = facts,
            Back::Frozen { n, answer } => self.frozen(n, answer),
            Back::Started { video, run } => self.started(video, run),
            Back::Called {
                video,
                run,
                call,
                done,
            } => self.called(video, run, call, done),
            Back::Timing => self.timing(),
            Back::Kept(kept) => self.kept(kept),
            Back::Folder(answer) => self.folder_picked(answer),
            Back::Note(note) => self.show_note(note),
        }
    }

    /// The job started writing its copy at a speed: a Save's dialog has answered, so Snip's windows go back on top.
    fn timing(&mut self) {
        self.keeping.saving = false;
        self.step(In::Timing);
    }

    /// The keyboard steps aside or comes back; a step aside first tells the view as a new round, so the ui hides
    /// Snip's windows before it answers.
    fn step_aside(&mut self, aside: SnipAside) {
        if aside != SnipAside::Back {
            let page = self.page();
            self.told.ask(page, &self.ports.bus);
        }
        self.publish(&aside);
    }

    /// A new snip: the unsaved part is swept, then it keeps the settings now for its whole run, notes its date and
    /// time, reads what the system can record, and freezes the screens in a job, counted so an older snip's answer is
    /// dropped.
    fn freeze(&mut self) {
        self.sweep();
        self.held = self.settings.clone();
        self.stamp = Some(self.ports.calendar.now());
        self.read_facts();
        let Some(cap) = ShotCap::new(self.held.shot_cap_mb) else {
            return self.step(In::Frozen(Err(FreezeError::TooBig)));
        };
        self.freezes = self.freezes.wrapping_add(1);
        let (n, freeze, back) = (
            self.freezes,
            self.ports.freeze.clone(),
            self.ports.back.clone(),
        );
        let job: Job = Box::new(move || {
            let answer = freeze.freeze(cap);
            let _ = back.send(Msg::Back(Back::Frozen { n, answer }));
        });
        if !(self.ports.spawn)(job) {
            self.step(In::Frozen(Err(FreezeError::Refused(NO_JOB))));
        }
    }

    /// Freeze `n`'s answer, whose screens the editor's remembered place must meet; an older freeze's is dropped. The
    /// time since the Snip key goes to the debug log, in ms.
    fn frozen(&mut self, n: u32, frozen: Result<Frozen, FreezeError>) {
        if n != self.freezes {
            return;
        }
        if let Some(at) = self.keyed.take() {
            let ms = self.now().saturating_since(at).as_millis();
            tracing::debug!(
                ms = u64::try_from(ms).unwrap_or(u64::MAX),
                "the freeze answered after the Snip key"
            );
        }
        if let Ok(frozen) = &frozen {
            self.seen.screens = frozen.screens.iter().map(FrozenScreen::rect).collect();
        }
        self.step(In::Frozen(frozen));
    }

    /// The editor opens on picture `shot` with its model, Undo's steps held for the snip.
    fn open_editor(&mut self, shot: Shot) {
        let (width, height) = (shot.width(), shot.height());
        let (undo_max, min) = (self.held.undo_max, self.held.frame_min_px);
        self.canvas = Canvas::new(width, height, undo_max, min, self.fit.clone());
        self.picture = Some(shot);
    }

    /// Plays `video` in the user's own player, off the actor; a refusal is logged by kind and shows its note.
    fn play(&mut self, video: Video) {
        let Some(file) = self.files.file(video) else {
            tracing::warn!("a video to play was not found");
            return self.show_note(SnipNote::PlayFailed);
        };
        let (player, back) = (self.ports.player.clone(), self.ports.back.clone());
        let job = Box::new(move || {
            if let Err(error) = player.play(&file) {
                tracing::warn!(%error, "the video did not play");
                let _ = back.send(Msg::Back(Back::Note(SnipNote::PlayFailed)));
            }
        });
        if !(self.ports.spawn)(job) {
            tracing::warn!("the video's player job did not start");
            self.show_note(SnipNote::PlayFailed);
        }
    }

    /// The snip ended: the editor's model, picture and videos are forgotten, and the next snip reads the settings anew.
    fn ended(&mut self) {
        (self.canvas, self.picture, self.stamp, self.keyed) = (None, None, None, None);
        self.seen.low = false;
        self.held = self.settings.clone();
        if let Some(file) = self.keeping.clear() {
            self.remove(&file);
        }
        self.files.forget();
    }
}

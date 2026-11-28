//! Copy and Save, and Play at a speed: one at a time, of a picture, a recording or the copy each speed makes once, then
//! the answer's tick and end, the close question's answers, and the snip's videos, which go at its end unless copied.

use kx_module_api::ports::{AskKey, Keep, Speed, StripKey};

use super::{After, Due, Flow, Held, Out, Outcome, Reel, Stage, Timed, Video, What};

/// A re-timed copy a job writes, reused at that speed once made.
#[derive(Copy, Clone, Debug)]
pub(super) struct Making {
    timed: Timed,
    /// It is being written now, which its strip key shows.
    now: bool,
}

impl Making {
    /// The copy `timed`, not yet being written.
    const fn new(timed: Timed) -> Self {
        Self { timed, now: false }
    }
}

/// A Copy or Save asked and not yet answered.
#[derive(Copy, Clone, Debug)]
pub(super) struct Asked {
    keep: Keep,
    /// The changes counted when it was asked.
    rev: u32,
    /// The video it puts on the clipboard, if any.
    clip: Option<Video>,
    /// The re-timed copy it writes first, if any.
    made: Option<Making>,
    /// What follows once done, when the question asked it.
    after: Option<After>,
}

/// A job asked and not yet answered: a Copy or Save, or the re-timed copy Play opens.
#[derive(Copy, Clone, Debug)]
pub(super) enum Busy {
    Keep(Asked),
    Play(Making),
}

impl Busy {
    /// The re-timed copy it writes, if any.
    const fn made(self) -> Option<Making> {
        match self {
            Self::Keep(asked) => asked.made,
            Self::Play(made) => Some(made),
        }
    }

    /// That copy, to mark it being written.
    const fn making(&mut self) -> Option<&mut Making> {
        match self {
            Self::Keep(asked) => asked.made.as_mut(),
            Self::Play(made) => Some(made),
        }
    }
}

/// The videos a snip made in the unsaved part, each with whether it was copied, and the re-timed copies written.
#[derive(Debug, Default)]
pub(super) struct Files {
    made: Vec<(Video, bool)>,
    /// Each speed's copy once written, which Play, Copy and Save reuse.
    timed: Vec<Timed>,
}

impl Files {
    /// A video made.
    pub(super) fn add(&mut self, video: Video) {
        self.made.push((video, false));
    }

    /// A video put on the clipboard, which stays in the unsaved part for `copied_days`.
    fn copied(&mut self, video: Video) {
        for (made, copied) in &mut self.made {
            *copied |= *made == video;
        }
    }

    /// The copy written at `speed`, if any.
    fn at(&self, speed: Speed) -> Option<Video> {
        self.timed.iter().find(|t| t.speed == speed).map(|t| t.into)
    }

    /// Deletes each video not copied, and forgets them all.
    pub(super) fn drop_all(&mut self, out: &mut Vec<Out>) {
        self.timed.clear();
        let gone = self.made.drain(..).filter(|(_, copied)| !copied);
        out.extend(gone.map(|(video, _)| Out::Delete(video)));
    }
}

impl Flow {
    /// Copies or saves the strip's snip; the question's Save names what follows.
    pub(super) fn keep(&mut self, keep: Keep, after: Option<After>, out: &mut Vec<Out>) {
        if let Some(what) = self.what() {
            self.ask_keep(keep, what, after, out);
        }
    }

    /// What the strip keeps; a video at another speed keeps its re-timed copy.
    fn what(&mut self) -> Option<What> {
        let Stage::Strip(strip) = &self.stage else {
            return None;
        };
        let reel = match &strip.held {
            Held::Picture(shot) => return Some(What::Picture(shot.clone())),
            Held::Video(reel) => *reel,
            Held::Ready(_) => return None,
        };
        let (video, timed) = self.at_speed(reel);
        Some(What::Video { video, timed })
    }

    /// The video `reel` plays or keeps, with the copy to write first: the recording at 1×, else the copy made at its
    /// speed, else the recording and a new copy.
    fn at_speed(&mut self, reel: Reel) -> (Video, Option<Timed>) {
        let speed = reel.speed;
        if speed == Speed::PLAIN {
            return (reel.clip.video, None);
        }
        if let Some(copy) = self.files.at(speed) {
            return (copy, None);
        }
        let into = self.video();
        self.files.add(into);
        (reel.clip.video, Some(Timed { speed, into }))
    }

    /// Play: the video at the speed picked, its copy written first while the keys wait.
    pub(super) fn play(&mut self, reel: Reel, out: &mut Vec<Out>) {
        let (video, timed) = self.at_speed(reel);
        let Some(timed) = timed else {
            return out.push(Out::Play(video));
        };
        self.busy = Some(Busy::Play(Making::new(timed)));
        out.push(Out::Retime { video, timed });
    }

    /// The copy asked started being written, which a Save's dialog comes before.
    pub(super) fn timing(&mut self) {
        if let Some(made) = self.busy.as_mut().and_then(Busy::making) {
            made.now = true;
        }
    }

    /// The strip key that waits while the copy at its speed is written: Play, Copy or Save.
    #[must_use]
    pub fn waits(&self) -> Option<StripKey> {
        let busy = self.busy?;
        busy.made().filter(|made| made.now)?;
        Some(match busy {
            Busy::Play(_) => StripKey::Play,
            Busy::Keep(Asked {
                keep: Keep::Copy, ..
            }) => StripKey::Copy,
            Busy::Keep(Asked {
                keep: Keep::Save, ..
            }) => StripKey::Save,
        })
    }

    /// Asks for `what`'s Copy or Save, noting the changes so far, the video it copies and the copy it writes.
    pub(super) fn ask_keep(
        &mut self,
        keep: Keep,
        what: What,
        after: Option<After>,
        out: &mut Vec<Out>,
    ) {
        let (clip, made) = match &what {
            What::Picture(_) => (None, None),
            What::Video { video, timed } => (Some(timed.map_or(*video, |t| t.into)), *timed),
        };
        self.busy = Some(Busy::Keep(Asked {
            keep,
            rev: self.changes,
            clip,
            made: made.map(Making::new),
            after,
        }));
        out.push(Out::Keep { keep, what });
    }

    /// The answer: a copy written is kept for reuse, then done keeps a Copy or Save or plays Play's copy; a cancelled
    /// dialog comes back, and a failure notes.
    pub(super) fn kept_answer(&mut self, outcome: Outcome, out: &mut Vec<Out>) {
        let Some(busy) = self.busy.take() else {
            return;
        };
        match outcome {
            Outcome::Done => {
                self.files.timed.extend(busy.made().map(|made| made.timed));
                match busy {
                    Busy::Keep(asked) => self.done(asked, out),
                    Busy::Play(made) => out.push(Out::Play(made.timed.into)),
                }
            }
            Outcome::Cancelled => {}
            Outcome::Failed(note) => out.push(Out::Note(note)),
        }
    }

    /// Kept as asked: it ticks, then a picture's strip or the snip the question closed ends, while the editor and a
    /// recorded video's strip stay. A change during the dialog leaves it unkept and open; the Snip key's Save starts
    /// the new snip at once.
    fn done(&mut self, asked: Asked, out: &mut Vec<Out>) {
        let keep = asked.keep;
        self.kept_at = Some(asked.rev);
        if let (Keep::Copy, Some(video)) = (keep, asked.clip) {
            self.files.copied(video);
        }
        let after = asked.after.filter(|_| self.kept());
        if after == Some(After::Start) {
            return self.finish(After::Start, out);
        }
        let stays = self.lasting().is_some();
        let tick = match &mut self.stage {
            Stage::Strip(strip) => &mut strip.done,
            Stage::Edit(edit) => &mut edit.done,
            _ => return,
        };
        *tick = Some(keep);
        let at = self.now().saturating_add(self.settings.shown());
        let ends = !stays || after.is_some();
        self.due = Some(Due { at, ends });
    }

    /// At an end, a video whose Copy is in flight stays as a copied one, as the clipboard may hold it.
    pub(super) fn spare_copy(&mut self) {
        if let Some(Busy::Keep(Asked {
            keep: Keep::Copy,
            clip: Some(video),
            ..
        })) = self.busy
        {
            self.files.copied(video);
        }
    }

    /// An answer to the close question: Back, Don't save, or Save, which in the editor waits for its flat picture.
    pub(super) fn answer(&mut self, key: AskKey, out: &mut Vec<Out>) {
        let Some(after) = self.ask else {
            return;
        };
        match key {
            AskKey::Back => self.ask = None,
            AskKey::Drop => self.finish(after, out),
            AskKey::Save if !matches!(self.stage, Stage::Edit(_)) => {
                self.ask = None;
                self.keep(Keep::Save, Some(after), out);
            }
            AskKey::Save => {}
        }
    }
}

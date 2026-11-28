//! Copy and Save, and the copy Play asks at a speed, one at a time in a job (`media`): the clipboard tried again while
//! busy (edge 5), and a delete of a video the job uses waits for its end. A video marked copied that never reached the
//! clipboard goes once the snip or the module ends, unless an earlier Copy put it there.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{ClipPart, FileKind, Keep, Shot, SnipFile, SnipNote, SpeedAsk};

use super::files::remove;
use super::media::{Goal, Kit};
use super::ports::Using;
use super::{Actor, Answer, Back, Job, Kept};
use crate::flow::{In, Outcome, Timed, Video, What};

/// The videos a job in flight uses, and the deletes the actor handed it.
#[derive(Debug, Default)]
struct Hold {
    /// The job ended; a delete is the actor's again.
    over: bool,
    /// Delete the recording once done.
    source: bool,
    /// Delete the re-timed copy once made.
    into: bool,
}

/// A Copy or Save in flight.
struct Flight {
    source: Option<Video>,
    into: Option<Video>,
    hold: Arc<Mutex<Hold>>,
}

/// A busy clipboard's next try: the parts, when, and the video they copy.
struct Retry {
    parts: Vec<ClipPart>,
    at: Mono,
    copied: Option<SnipFile>,
}

/// The Copy or Save in flight, a busy clipboard's next try, and the videos on the clipboard.
#[derive(Default)]
pub(super) struct Keeping {
    flight: Option<Flight>,
    retry: Option<Retry>,
    /// The videos a Copy put on the clipboard since the module started, which no end deletes, as a paste needs them.
    copied: Vec<SnipFile>,
    /// The tries again so far.
    again: u8,
    /// A Save's dialog shows until its job answers, or starts writing its copy at a speed.
    pub(super) saving: bool,
}

impl Keeping {
    /// True when the clipboard's next try is due.
    pub(super) fn due(&self, now: Mono) -> bool {
        self.retry.as_ref().is_some_and(|r| now >= r.at)
    }

    /// When the clipboard's next try is due.
    pub(super) fn next(&self) -> Option<Mono> {
        self.retry.as_ref().map(|r| r.at)
    }

    /// The snip ended: no try is left to make; the video a dropped try was to copy, unless a Copy put it on the
    /// clipboard.
    pub(super) fn clear(&mut self) -> Option<SnipFile> {
        self.flight = None;
        let copied = self.retry.take().and_then(|r| r.copied);
        self.lost(copied)
    }

    /// Notes the video `kept` put on the clipboard, if its Copy was done.
    fn heard(&mut self, kept: &Kept) {
        if let (Answer::Done, Some(file)) = (&kept.answer, &kept.copied)
            && !self.copied.contains(file)
        {
            self.copied.push(file.clone());
        }
    }

    /// `video`, marked copied by a Copy that failed, unless a Copy put it on the clipboard.
    fn lost(&self, video: Option<SnipFile>) -> Option<SnipFile> {
        video.filter(|file| !self.copied.contains(file))
    }

    /// Hands the delete of `video` to the job in flight using it; false when none does or it ended.
    pub(super) fn defer(&self, video: Video) -> bool {
        let Some(flight) = &self.flight else {
            return false;
        };
        let (source, into) = (flight.source == Some(video), flight.into == Some(video));
        let mut hold = flight.hold.lock().unwrap_or_else(PoisonError::into_inner);
        if hold.over || !(source || into) {
            return false;
        }
        hold.source |= source;
        hold.into |= into;
        true
    }

    /// True while the job in flight is asked to make the re-timed copy `video`.
    pub(super) fn awaits(&self, video: Video) -> bool {
        self.flight.as_ref().is_some_and(|f| f.into == Some(video))
    }
}

/// Ends a job: it deletes what the actor handed it, and gives back the re-timed copy unless deleted.
fn settle(kit: &Kit, hold: &Mutex<Hold>, source: &SnipFile, kept: &mut Kept) {
    let mut hold = hold.lock().unwrap_or_else(PoisonError::into_inner);
    hold.over = true;
    let gone = |file: &SnipFile| remove(&*kit.folder, &kit.leftovers, file);
    if hold.source {
        gone(source);
    }
    if hold.into
        && let Some(made) = kept.made.take()
    {
        gone(&made);
    }
}

impl Actor {
    /// Copies or saves `what` in a job; the flow hears how it came out.
    pub(super) fn keep(&mut self, keep: Keep, what: What) {
        let note = match keep {
            Keep::Copy => SnipNote::CopyFailed,
            Keep::Save => SnipNote::SaveFailed,
        };
        let job = match what {
            What::Picture(shot) => self.picture_job(keep, shot, note),
            What::Video { video, timed } => self.video_job(Some(keep), video, timed, note),
        };
        self.launch(job, note, keep == Keep::Save);
    }

    /// Writes `video`'s copy for Play in a job, as a Copy at that speed does; the flow hears how it came out.
    pub(super) fn retime(&mut self, video: Video, timed: Timed) {
        let note = SnipNote::PlayFailed;
        let job = self.video_job(None, video, Some(timed), note);
        self.launch(job, note, false);
    }

    /// Starts a Copy, Save or Play job, while `saving` shows the Save dialog; one not made fails with `note`.
    fn launch(&mut self, job: Option<Job>, note: SnipNote, saving: bool) {
        self.keeping.again = 0;
        let Some(job) = job else {
            self.keeping.flight = None;
            return self.step(In::Kept(Outcome::Failed(note)));
        };
        // A job that does not start answers through its reply.
        if (self.ports.spawn)(job) {
            self.keeping.saving = saving;
        }
    }

    /// The job of a picture's Copy or Save; `None` for a Save with no folder to propose.
    fn picture_job(&self, keep: Keep, shot: Shot, note: SnipNote) -> Option<Job> {
        let proposed = match keep {
            Keep::Copy => None,
            Keep::Save => Some(self.proposed(FileKind::Png)?),
        };
        let kit = self.kit();
        let reply = kit.reply(None, None, note);
        Some(Box::new(move || {
            let answer = kit.keep_picture(keep, &shot, proposed.as_ref());
            reply.send(Back::Kept(Kept::of(answer)));
        }))
    }

    /// The job of a video's Copy or Save, or of Play's copy when `keep` is `None`, held as in flight; `None` for a
    /// video not known or no folder to propose. The recording stays out of every sweep until the job ends.
    fn video_job(
        &mut self,
        keep: Option<Keep>,
        video: Video,
        timed: Option<Timed>,
        note: SnipNote,
    ) -> Option<Job> {
        let source = self.files.file(video)?;
        let goal = match keep {
            Some(Keep::Copy) => Goal::Copy,
            Some(Keep::Save) => Goal::Save(self.proposed(FileKind::Mp4)?),
            None => Goal::Play,
        };
        let speed = timed.map(|t| self.speed_ask(&source, t));
        let hold = Arc::new(Mutex::new(Hold::default()));
        let (into, kit, job_hold) = (timed.map(|t| t.into), self.kit(), Arc::clone(&hold));
        self.keeping.flight = Some(Flight {
            source: Some(video),
            into,
            hold,
        });
        // A dropped re-timed Copy names no video: the recording is not what it puts on the clipboard.
        let copied = (matches!(goal, Goal::Copy) && timed.is_none()).then(|| source.clone());
        let (reply, using) = (
            kit.reply(into, copied, note),
            Using::new(&kit.leftovers, source.clone()),
        );
        Some(Box::new(move || {
            let mut kept = kit.keep_video(&source, speed.as_ref(), &goal);
            let _made = kept
                .made
                .clone()
                .map(|made| Using::new(&kit.leftovers, made));
            settle(&kit, &job_hold, &source, &mut kept);
            kept.into = into;
            reply.send(Back::Kept(kept));
            drop(using);
        }))
    }

    /// The re-timing of `source` a Copy or Save asks first.
    fn speed_ask(&self, source: &SnipFile, timed: Timed) -> SpeedAsk {
        SpeedAsk {
            from: source.clone(),
            speed: timed.speed,
            min_free_mb: self.held.min_free_mb,
            job_mb: self.held.record_mb,
            waits: self.held.record_waits(),
        }
    }

    /// The Copy or Save came out: a re-timed copy is mapped, a busy clipboard waits for another try, else the flow
    /// hears.
    pub(super) fn kept(&mut self, kept: Kept) {
        (self.keeping.flight, self.keeping.saving) = (None, false);
        self.keeping.heard(&kept);
        self.copied_into(kept.into, kept.made);
        let outcome = match kept.answer {
            Answer::Done => Outcome::Done,
            Answer::Cancelled => Outcome::Cancelled,
            Answer::Failed(note) => Outcome::Failed(note),
            Answer::Busy(parts) if self.keeping.again < self.held.copy_again => {
                self.keeping.again += 1;
                let pause = Duration::from_millis(u64::from(self.held.copy_again_ms));
                let at = self.now().saturating_add(pause);
                let copied = kept.copied;
                self.keeping.retry = Some(Retry { parts, at, copied });
                return;
            }
            Answer::Busy(_) => Outcome::Failed(SnipNote::ClipBusy),
        };
        self.step(In::Kept(outcome));
    }

    /// A Copy or Save that came out once the actor stopped: a re-timed copy is mapped, and a video marked copied goes
    /// unless this Copy or an earlier one put it on the clipboard, as the flow counted it copied.
    pub(super) fn kept_late(&mut self, kept: Kept) {
        self.keeping.heard(&kept);
        let failed = !matches!(kept.answer, Answer::Done);
        let lost = self.keeping.lost(kept.copied.filter(|_| failed));
        if let Some(file) = &lost {
            self.remove(file);
        }
        let made = kept.made.filter(|made| lost.as_ref() != Some(made));
        self.copied_into(kept.into, made);
    }

    /// Tries the busy clipboard again.
    pub(super) fn write_again(&mut self) {
        let Some(Retry { parts, copied, .. }) = self.keeping.retry.take() else {
            return;
        };
        let kit = self.kit();
        let reply = kit.reply(None, copied.clone(), SnipNote::CopyFailed);
        let job: Job = Box::new(move || {
            let answer = kit.write(parts);
            reply.send(Back::Kept(Kept {
                copied,
                ..Kept::of(answer)
            }));
        });
        // A job that does not start answers through its reply.
        (self.ports.spawn)(job);
    }

    /// The ports a job works through, and the snip's clipboard wait.
    fn kit(&self) -> Kit {
        let ports = &self.ports;
        Kit {
            recorder: ports.recorder.clone(),
            dialogs: ports.dialogs.clone(),
            clip: ports.clip.clone(),
            pictures: ports.pictures.clone(),
            folder: ports.folder.clone(),
            leftovers: ports.leftovers.clone(),
            back: ports.back.clone(),
            wait: self.held.copy_wait(),
            limits: self.held.pick_limits(),
            encode: self.held.encode_waits(),
        }
    }
}

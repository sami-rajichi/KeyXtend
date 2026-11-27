//! What a Copy, Save or Play job does off the actor: the Save dialog's place (edge 6), a picture's PNG made in the
//! media worker with a memory cap and a deadline sized from its pixels, then its `CF_DIB` built here, a video re-timed
//! first and put on the clipboard as a file reference, and one kept write to the clipboard.

use std::sync::Arc;
use std::sync::mpsc::Sender;

use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskError, ClipError, ClipPart, ClipWait, ClipboardWriter, EncodeAsk, FileAsk, ImageCodec, Keep,
    MB, PickLimits, PictureConfig, PictureError, Pictures, RecordError, Recorder, SaveError,
    SaveTo, Shot, SnipFile, SnipFolder, SnipNote, SpeedAsk,
};

use super::reply::Reply;
use super::{Answer, Back, Kept, Leftovers, Msg, dib};
use crate::EncodeWaits;
use crate::flow::Video;

/// The ports a Copy or Save job works through, and the snip's clipboard wait, dialogs' limits and PNG waits.
#[derive(Clone)]
pub(super) struct Kit {
    pub recorder: Arc<dyn Recorder>,
    pub dialogs: Arc<dyn FileAsk>,
    pub clip: Arc<dyn ClipboardWriter>,
    pub pictures: Arc<dyn Pictures>,
    pub folder: Arc<dyn SnipFolder>,
    pub leftovers: Leftovers,
    pub back: Sender<Msg>,
    pub wait: ClipWait,
    pub limits: PickLimits,
    pub encode: EncodeWaits,
}

/// Where a video job's file goes once re-timed, if asked.
pub(super) enum Goal {
    /// The clipboard, as a file reference.
    Copy,
    /// The place the Save dialog gives, proposed at this file.
    Save(SnipFile),
    /// Nowhere: Play opens the copy written.
    Play,
}

impl Kit {
    /// The answer a job owes: failed with `note` unless it sends another, naming the copy `into` and the video
    /// `copied`.
    pub(super) fn reply(
        &self,
        into: Option<Video>,
        copied: Option<SnipFile>,
        note: SnipNote,
    ) -> Reply {
        let fallback = Kept {
            into,
            copied,
            ..Kept::of(Answer::Failed(note))
        };
        Reply::new(self.back.clone(), Back::Kept(fallback))
    }

    /// Puts `parts` on the clipboard, kept by both histories; a busy clipboard hands them back for another try.
    pub(super) fn write(&self, parts: Vec<ClipPart>) -> Answer {
        match self.clip.write_kept(&parts, self.wait) {
            Ok(()) => Answer::Done,
            Err(ClipError::Busy) => Answer::Busy(parts),
            Err(error) => {
                tracing::warn!(%error, "the snip was not copied");
                Answer::Failed(SnipNote::CopyFailed)
            }
        }
    }

    /// Makes `shot`'s PNG in a session of its own, dropped once done so the worker's memory goes back.
    fn png(&self, shot: &Shot) -> Result<Redacted<Vec<u8>>, PictureError> {
        let ask = EncodeAsk { shot: shot.clone() };
        let mut run = self.pictures.open(config(&ask, self.encode));
        run.encode(ask)
    }

    /// A picture's clipboard parts: its PNG, then its `CF_DIB`, built once the PNG is back so the bitmap never waits in
    /// memory while the worker encodes; without a PNG the bitmap goes alone.
    fn picture(&self, shot: &Shot) -> Vec<ClipPart> {
        let png = self.png(shot);
        let bitmap = ClipPart::Image {
            codec: ImageCodec::Dib,
            bytes: dib::of(shot),
        };
        match png {
            Ok(bytes) => {
                let codec = ImageCodec::Png;
                vec![ClipPart::Image { codec, bytes }, bitmap]
            }
            Err(error) => {
                tracing::warn!(%error, "no PNG was made, so the bitmap is copied alone");
                vec![bitmap]
            }
        }
    }

    /// Shows the Save dialog at `proposed`; its answer, or how the job ends without one: a refused place says to
    /// choose another.
    fn ask(&self, proposed: &SnipFile) -> Result<Box<dyn SaveTo>, Answer> {
        match self.dialogs.save(proposed, &self.limits) {
            Ok(Some(to)) => Ok(to),
            Ok(None) => Err(Answer::Cancelled),
            Err(error) => {
                tracing::warn!(%error, "the Save dialog gave no place");
                Err(Answer::Failed(refused_or(&error, SnipNote::SaveFailed)))
            }
        }
    }

    /// A picture's Copy or Save.
    pub(super) fn keep_picture(
        &self,
        keep: Keep,
        shot: &Shot,
        proposed: Option<&SnipFile>,
    ) -> Answer {
        let Some(proposed) = proposed.filter(|_| keep == Keep::Save) else {
            return self.write(self.picture(shot));
        };
        let to = match self.ask(proposed) {
            Ok(to) => to,
            Err(answer) => return answer,
        };
        match self.png(shot) {
            Ok(png) => saved(to.png(&png)),
            Err(error) => {
                tracing::warn!(%error, "the picture's PNG was not made");
                Answer::Failed(SnipNote::SaveFailed)
            }
        }
    }

    /// A video's Copy, Save or Play, re-timed first when asked; the copy made and the video marked copied come back.
    pub(super) fn keep_video(
        &self,
        source: &SnipFile,
        speed: Option<&SpeedAsk>,
        goal: &Goal,
    ) -> Kept {
        let to = match goal {
            Goal::Save(proposed) => match self.ask(proposed) {
                Ok(to) => Some(to),
                Err(answer) => return Kept::of(answer),
            },
            Goal::Copy | Goal::Play => None,
        };
        let made = match speed.map(|ask| self.retime(ask)).transpose() {
            Ok(made) => made,
            Err(answer) => return Kept::of(answer),
        };
        let file = made.as_ref().unwrap_or(source).clone();
        let (answer, copied) = match (to, goal) {
            (Some(to), _) => (saved(to.copy(&file)), None),
            (None, Goal::Copy) => {
                self.mark(&file);
                (self.write(vec![file_part(&file)]), Some(file))
            }
            (None, _) => (Answer::Done, None),
        };
        Kept {
            made,
            copied,
            ..Kept::of(answer)
        }
    }

    /// Writes the re-timed copy `ask` says, telling the actor it started; a full drive notes low room, any other error
    /// that it failed.
    fn retime(&self, ask: &SpeedAsk) -> Result<SnipFile, Answer> {
        let _ = self.back.send(Msg::Back(Back::Timing));
        self.recorder.speed(ask).map_err(|error| {
            tracing::warn!(%error, "the video was not re-timed");
            let full = error == RecordError::Full;
            Answer::Failed(if full {
                SnipNote::DiskLow
            } else {
                SnipNote::SpeedFailed
            })
        })
    }

    /// Marks `video` copied before it goes on the clipboard, so the sweep keeps it its days; a mark not made only
    /// warns.
    fn mark(&self, video: &SnipFile) {
        if let Err(error) = self.folder.mark_copied(video) {
            tracing::warn!(%error, "the copied video was not marked, so the next sweep deletes it");
        }
    }
}

/// The note for a dialog or a write that failed.
fn saved(done: Result<(), SaveError>) -> Answer {
    match done {
        Ok(()) => Answer::Done,
        Err(error) => {
            tracing::warn!(%error, "the snip could not be saved there");
            Answer::Failed(SnipNote::SaveFailed)
        }
    }
}

/// The note for a Save or folder dialog that gave no place: choose another for a refused place, else `failed`.
pub(super) fn refused_or(error: &AskError, failed: SnipNote) -> SnipNote {
    if *error == AskError::BadPath {
        SnipNote::PlaceRefused
    } else {
        failed
    }
}

/// A video on the clipboard as a file reference, so it pastes into chats, mail and folders.
pub(super) fn file_part(file: &SnipFile) -> ClipPart {
    ClipPart::Files(Redacted::new(vec![file.expose().to_path_buf()]))
}

/// The media worker's session for `ask` by `waits`: its memory cap and deadline grow with the pixels, as an Encode
/// takes seconds; each Encode opens its own session, so its first restart wait is its longest.
fn config(ask: &EncodeAsk, waits: EncodeWaits) -> PictureConfig {
    let (width, height) = (ask.shot.width(), ask.shot.height());
    let pixels = u64::from(width) * u64::from(height);
    let raw_mb = u64::try_from(ask.shot.bgrx().expose().len()).unwrap_or(u64::MAX) / MB;
    let more = u64::from(waits.mb_ms).saturating_mul(raw_mb);
    let deadline_ms = u32::try_from(more).map_or(u32::MAX, |m| waits.ms.saturating_add(m));
    PictureConfig {
        deadline_ms,
        idle_s: waits.idle_s,
        memory_mb: ask.memory_mb(),
        max_pixels: pixels,
        restart_ms: waits.restart_ms,
        restart_max_ms: waits.restart_ms,
    }
}

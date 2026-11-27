//! A re-timed copy (edge 20) in a record job of its own: its source checked to lie in the unsaved part, the room
//! checked first, a deadline by the source's size as a copy cannot be cancelled, and a copy that failed deleted.

use super::{WorkerRecorder, error, folder_error, ms, plain, retry, scrap, times, unsent};
use crate::worker::{Launch, RecordFrames, Spawner, Split, Stream, Told, Wait};
use kx_ipc::{RecordAsk as Ask, RecordNote as Note};
use kx_module_api::ports::{MB, RecordError, RecordWaits, SnipFile, SnipFolder, SpeedAsk};
use std::fs;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

/// A copy's stream thread.
const THREAD: &str = "kx-speed";

/// What a copy's job answered, and whether the folder still held a copy that failed.
type Copied = (Result<(), RecordError>, bool);

/// Writes `ask`'s copy into a new video of the unsaved part, then says goodbye to its job.
pub(super) fn copy<S: Spawner<RecordFrames, Worker: Split<RecordFrames>>>(
    recorder: &WorkerRecorder<S>,
    ask: &SpeedAsk,
) -> Result<SnipFile, RecordError> {
    ask.check()?;
    let unsaved = recorder.unsaved()?;
    let listed = recorder.folder.unsaved().map_err(folder_error)?;
    if !listed.iter().any(|u| u.file == ask.from) {
        return Err(RecordError::BadPath);
    }
    recorder.room(ask.min_free_mb)?;
    let to = recorder.fresh(&unsaved)?;
    let make = frame(ask, &to)?;
    let (answer, heard) = mpsc::channel();
    let hear = settle(Arc::clone(&recorder.folder), to.clone(), answer);
    let (spawner, clock) = (Arc::clone(&recorder.spawner), Arc::clone(&recorder.clock));
    let launch = Launch::record(ask.job_mb);
    let stream = Stream::open(THREAD, spawner, clock, times(&ask.waits), launch, hear);
    let size = fs::metadata(ask.from.expose()).map_or(0, |m| m.len());
    let wait = Wait::Once(deadline(&ask.waits, size));
    let (copied, held) = match stream.begin(make, wait) {
        Ok(_) => heard.recv().unwrap_or((Err(RecordError::Gone), true)),
        Err(e) => (Err(unsent(e)), false),
    };
    stream.stop();
    if held {
        retry(&*recorder.folder, &to);
    }
    copied.map(|()| to)
}

/// Answers the copy once, at its end, its fault or a lost job; a copy that failed is deleted first, and one the
/// folder still held is said to be.
fn settle(
    folder: Arc<dyn SnipFolder>,
    to: SnipFile,
    answer: Sender<Copied>,
) -> impl FnMut(Told<Note>) + Send {
    let mut answer = Some(answer);
    move |told| {
        let copied = match told {
            Told::Note(Note::End { .. }) => Ok(()),
            Told::Note(Note::Fault { fault, .. }) => Err(error(fault)),
            Told::Note(_) => return,
            Told::Lost(_) | Told::Idle => Err(RecordError::Gone),
        };
        let Some(answer) = answer.take() else {
            return;
        };
        let held = copied.is_err() && !scrap(&*folder, &to);
        let _ = answer.send((copied, held));
    }
}

/// The Speed frame of `ask` into `to`, for the id the stream gives it.
fn frame(
    ask: &SpeedAsk,
    to: &SnipFile,
) -> Result<impl FnOnce(u32) -> Ask<'static> + Send + 'static, RecordError> {
    let (from, to, speed) = (plain(&ask.from)?, plain(to)?, ask.speed.times());
    Ok(move |id| Ask::Speed {
        id,
        from,
        speed,
        to,
    })
}

/// A copy's deadline for a source of `bytes`: `speed_ms`, and `speed_mb_ms` for each MB begun.
pub(super) fn deadline(waits: &RecordWaits, bytes: u64) -> Duration {
    let per = ms(waits.speed_mb_ms);
    let begun = u32::try_from(bytes.div_ceil(MB)).unwrap_or(u32::MAX);
    ms(waits.speed_ms).saturating_add(per.saturating_mul(begun))
}

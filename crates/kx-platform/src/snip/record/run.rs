//! One recording on a stream of its own: its start waits for the job's first note, a switch for its answer, Stop and
//! Cancel go to the job, and a drop says goodbye, deleting a cancelled video the job kept.

use super::route::Route;
use super::{WorkerRecorder, lock, ms, plain, times, unsent};
use crate::ear::Ear;
use crate::worker::{Launch, RecordFrames, Spawner, Split, Stream, Told, Wait};
use kx_ipc::{Area, RecordAsk as Ask, RecordNote as Note};
use kx_module_api::ports::{
    MicPass, MicSeal, RecordAsk, RecordError, RecordNote, RecordRun, SnipFile, Source,
};
use std::sync::{Arc, Mutex, Weak, mpsc};

/// A run's stream thread.
const THREAD: &str = "kx-record";

/// A recording the worker `S` runs.
pub(super) struct WorkerRun<S: Spawner<RecordFrames, Worker: Split<RecordFrames>>> {
    stream: Stream<S, RecordFrames>,
    id: u32,
    file: SnipFile,
    /// Held by the stream's thread alone, so a wait on it ends when that thread does.
    route: Weak<Mutex<Route>>,
    ear: Arc<Ear<RecordNote>>,
    seal: MicSeal,
    /// The wait each later ask sets: a tick while it records, the end's time once stopped.
    wait: Wait,
    /// The end's time, which Stop sets.
    end: Wait,
}

impl<S: Spawner<RecordFrames, Worker: Split<RecordFrames>>> WorkerRun<S> {
    /// Starts the checked `ask` into `file` in a job of its own and waits for its first note; a start that failed
    /// says goodbye at once.
    pub(super) fn start(
        recorder: &WorkerRecorder<S>,
        ask: &RecordAsk,
        file: SnipFile,
        tell: Box<dyn Fn(RecordNote) + Send>,
    ) -> Result<Self, RecordError> {
        let make = frame(ask, &file)?;
        let (started, heard) = mpsc::channel();
        let (folder, clock) = (Arc::clone(&recorder.folder), Arc::clone(&recorder.clock));
        let switch = ms(ask.waits.switch_ms);
        let route = Route::new(file.clone(), folder, clock, started, switch);
        let route = Arc::new(Mutex::new(route));
        let ear = Arc::new(Ear::new(tell));
        let (weak, hear) = (Arc::downgrade(&route), relay(route, Arc::clone(&ear)));
        let (spawner, clock) = (Arc::clone(&recorder.spawner), Arc::clone(&recorder.clock));
        let launch = Launch::record(ask.job_mb);
        let stream = Stream::open(THREAD, spawner, clock, times(&ask.waits), launch, hear);
        let (first, each) = (ms(ask.waits.start_ms), ms(ask.waits.tick_ms));
        let wait = Wait::Shift {
            first,
            each,
            then: each,
        };
        let id = stream.begin(make, wait).map_err(unsent)?;
        if let Err(e) = heard.recv().unwrap_or(Err(RecordError::Gone)) {
            stream.stop();
            if let Some(route) = weak.upgrade() {
                lock(&route).leave();
            }
            return Err(e);
        }
        let (seal, wait) = (recorder.seal.clone(), Wait::Each(each));
        let end = Wait::Each(ms(ask.waits.end_ms));
        Ok(Self {
            stream,
            id,
            file,
            route: weak,
            ear,
            seal,
            wait,
            end,
        })
    }

    /// True once the run ended, or its stream's thread did.
    fn ended(&self) -> bool {
        self.route.upgrade().is_none_or(|r| lock(&r).ended())
    }

    /// Says goodbye once the run ended, so no watch an ask set just after its end waits on a job that is done.
    fn done(&self) {
        if self.ended() {
            self.stream.bye();
        }
    }

    /// Switches `source` and waits for the job's answer.
    fn switch(&mut self, source: Source, on: bool) -> Result<(), RecordError> {
        let (answer, heard) = mpsc::channel();
        let waits = self
            .route
            .upgrade()
            .is_some_and(|r| lock(&r).wait(source, answer));
        if !waits {
            return Err(RecordError::Gone);
        }
        let id = self.id;
        let ask = match source {
            Source::Mic => Ask::Mic { id, on },
            Source::Sound => Ask::Sound { id, on },
        };
        self.stream.ask(ask, self.wait).map_err(unsent)?;
        let answer = heard.recv().unwrap_or(Err(RecordError::Gone));
        self.done();
        answer
    }
}

impl<S: Spawner<RecordFrames, Worker: Split<RecordFrames>>> RecordRun for WorkerRun<S> {
    fn file(&self) -> &SnipFile {
        &self.file
    }

    fn mic(&mut self, on: Option<MicPass>) -> Result<(), RecordError> {
        if on.as_ref().is_some_and(|pass| !self.seal.opens(pass)) {
            return Err(RecordError::BadPass);
        }
        self.switch(Source::Mic, on.is_some())
    }

    fn sound(&mut self, on: bool) -> Result<(), RecordError> {
        self.switch(Source::Sound, on)
    }

    fn stop(&mut self) -> Result<(), RecordError> {
        if self.ended() {
            return Err(RecordError::Gone);
        }
        self.wait = self.end;
        let stop = Ask::Stop { id: self.id };
        let sent = self.stream.ask(stop, self.wait).map_err(unsent);
        self.done();
        sent
    }

    /// Tells nothing more, and the job deletes the video; one the job cannot delete, or an end not yet heard, the
    /// route deletes. The Cancel is posted, so no caller waits behind the stream, even from inside the callback.
    fn cancel(&mut self) {
        self.ear.close();
        let tell = self.route.upgrade().is_some_and(|r| lock(&r).cancel());
        if tell {
            self.stream.post(Ask::Cancel { id: self.id });
        }
    }
}

impl<S: Spawner<RecordFrames, Worker: Split<RecordFrames>>> Drop for WorkerRun<S> {
    /// Tells nothing more and says goodbye, which stops a live run and keeps its video, unless it was cancelled.
    fn drop(&mut self) {
        self.ear.close();
        self.stream.bye();
        let route = Weak::clone(&self.route);
        self.stream.later(move || {
            if let Some(route) = route.upgrade() {
                lock(&route).leave();
            }
        });
    }
}

/// What the stream tells, through `route`, to `ear`, the route unlocked first; a note the closed ear refused goes back.
fn relay(route: Arc<Mutex<Route>>, ear: Arc<Ear<RecordNote>>) -> impl FnMut(Told<Note>) + Send {
    move |told| {
        let note = lock(&route).heard(told);
        if !ear.tell(note) {
            lock(&route).unheard();
        }
    }
}

/// The Start frame `ask` is sent as, into `file`, for the id the stream gives it.
fn frame(
    ask: &RecordAsk,
    file: &SnipFile,
) -> Result<impl FnOnce(u32) -> Ask<'static> + Send + 'static, RecordError> {
    let file = plain(file)?;
    let rect = ask.part.rect();
    let area = Area {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: rect.h,
    };
    let (fps, mic, sound) = (ask.fps, ask.mic.is_some(), ask.sound);
    let (max_min, min_free_mb) = (ask.max_min, ask.min_free_mb);
    Ok(move |id| Ask::Start {
        id,
        area,
        fps,
        mic,
        sound,
        max_min,
        min_free_mb,
        file,
    })
}

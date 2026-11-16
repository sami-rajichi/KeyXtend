//! One dictation on the dictation job: its asks watched as `job::waits` says, its recording kept in the job until
//! Close, and Again in a new job when the model differs, the recording moved in wiped pieces through the host. A
//! recording a new job could not take stays parked here, wiped at Close, so Again can try once more.

use super::frame::setup;
use super::job::{Key, Live, extra_mb, waits};
use super::note::unsent;
use super::router::{self, Out, Router, State, Tell};
use super::{NO_MODEL, Shared, lock};
use crate::worker::{Spawner, Split, Stream, VoiceFrames, Wait};
use kx_ipc::{MAX_RECORD_BYTES, MAX_SOUND_PIECES, Setup, SoundPiece, VoiceAsk, Wiped};
use kx_module_api::SecretBytes;
use kx_module_api::ports::{JobLimits, SpeechAsk, SpeechError, SpeechRun};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// A run on the dictation job; dropping it closes it.
pub(super) struct WorkerRun<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> {
    shared: Arc<Shared<S>>,
    /// Its number in the router.
    run: u64,
    /// The id of its newest ask on the job, which Stop, Cancel, Keep and Close name.
    id: u32,
    /// The limits of its newest ask, whose waits the next asks take.
    limits: JobLimits,
    /// A recording a new job did not take yet.
    parked: Option<Vec<SecretBytes>>,
    closed: bool,
}

impl<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> WorkerRun<S> {
    /// Opens a run as the checked `ask` says, on the job its model needs; other runs' recordings go, as a Start
    /// wipes them.
    pub(super) fn open(
        shared: Arc<Shared<S>>,
        ask: &SpeechAsk,
        tell: Tell,
    ) -> Result<Self, SpeechError> {
        let setup = setup(ask, &shared.assets, shared.hardware.cores())?;
        let (key, w) = (Key::of(ask), waits(&ask.job));
        let run = lock(&shared.router).add(tell);
        let others = lock(&shared.router).wipe(Some(run));
        router::tell(others);
        let mut live = shared.live();
        let (stream, _) = shared.stream_for(&mut live, &key, 0);
        let start = move |id| VoiceAsk::Start { id, setup };
        let wait = Wait::Shift {
            first: w.start,
            each: w.ask,
            then: w.write,
        };
        let begun = begin(&shared.router, stream, run, start, wait);
        drop(live);
        let id = begun.inspect_err(|_| drop(lock(&shared.router).close(run)))?;
        let (limits, parked, closed) = (ask.job, None, false);
        Ok(Self {
            shared,
            run,
            id,
            limits,
            parked,
            closed,
        })
    }

    /// Where the run stands; a closed run holds nothing.
    fn state(&self) -> State {
        let state = lock(&self.shared.router).state(self.run);
        state.unwrap_or(State {
            held: false,
            dead: false,
        })
    }

    /// Sends `ask` on the run's job, watched as `wait` says; a worker found gone kills the run.
    fn send(&self, ask: VoiceAsk<'static>, wait: Wait) -> Result<(), SpeechError> {
        let live = self.shared.live();
        let sent = live.stream.as_ref().map(|stream| stream.ask(ask, wait));
        drop(live);
        let error = match sent {
            Some(Ok(())) => return Ok(()),
            Some(Err(e)) => unsent(e),
            None => SpeechError::Gone,
        };
        if matches!(error, SpeechError::Gone | SpeechError::Timeout) {
            lock(&self.shared.router).kill(self.run);
        }
        Err(error)
    }

    /// Asks for the held recording and gathers its pieces as the router hands them over; a job that sends more than
    /// the longest recording fails as `Internal`, so a broken job cannot fill the app's memory.
    fn collect(&self, live: &Live<S>) -> Result<Vec<SecretBytes>, SpeechError> {
        let stream = live.stream.as_ref().ok_or(SpeechError::Gone)?;
        let (to, kept) = mpsc::channel();
        lock(&self.shared.router).keep(self.run, to);
        let ask = VoiceAsk::Keep { id: self.id };
        let wait = Wait::Each(waits(&self.limits).ask);
        stream.ask(ask, wait).map_err(unsent)?;
        let (mut pieces, mut held) = (Vec::new(), 0);
        loop {
            let (bytes, last) = kept.recv().map_err(|_| SpeechError::Gone)??;
            held += bytes.expose().len();
            pieces.push(bytes);
            if last {
                return Ok(pieces);
            }
            if pieces.len() >= MAX_SOUND_PIECES || held > MAX_RECORD_BYTES {
                return Err(SpeechError::Internal);
            }
        }
    }

    /// Writes the held recording again on the same job, as a fresh run id.
    fn again_here(
        &mut self,
        live: &Live<S>,
        setup: Setup<'static>,
        wait: Wait,
    ) -> Result<(), SpeechError> {
        let stream = live.stream.as_ref().ok_or(SpeechError::Gone)?;
        let again = move |id| VoiceAsk::Again { id, setup };
        self.id = begin(&self.shared.router, stream, self.run, again, wait)?;
        Ok(())
    }

    /// Moves the recording to a new job for `key`, then writes it there; a recording the job did not take is parked.
    /// The other runs whose recordings went with the old job go in `wiped`, told once the job is unlocked.
    fn again_moved(
        &mut self,
        live: &mut Live<S>,
        ask: &SpeechAsk,
        setup: Setup<'static>,
        wait: Wait,
        wiped: &mut Out,
    ) -> Result<(), SpeechError> {
        let pieces = match self.parked.take() {
            Some(parked) => parked,
            None => self.collect(live)?,
        };
        let held = pieces.iter().map(|p| p.expose().len()).sum();
        let extra = extra_mb(held, ask.record_max_s);
        *wiped = lock(&self.shared.router).wipe(Some(self.run));
        let (stream, _) = self.shared.stream_for(live, &Key::of(ask), extra);
        match hand_in(&self.shared.router, stream, self.run, &pieces, setup, wait) {
            Ok(id) => {
                self.id = id;
                Ok(())
            }
            Err(error) => {
                self.parked = Some(pieces);
                Err(error)
            }
        }
    }
}

impl<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> SpeechRun for WorkerRun<S> {
    fn stop(&mut self) -> Result<(), SpeechError> {
        if self.closed {
            return Ok(());
        }
        if self.state().dead {
            return Err(SpeechError::Gone);
        }
        let wait = Wait::Each(waits(&self.limits).write);
        self.send(VoiceAsk::Stop { id: self.id }, wait)
    }

    fn cancel(&mut self) -> Result<(), SpeechError> {
        if self.closed {
            return Ok(());
        }
        if self.state().dead {
            return Err(SpeechError::Gone);
        }
        let wait = Wait::Once(waits(&self.limits).ask);
        self.send(VoiceAsk::Cancel { id: self.id }, wait)
    }

    fn again(&mut self, ask: SpeechAsk) -> Result<(), SpeechError> {
        ask.check()?;
        if ask.model.is_none() {
            return Err(SpeechError::Range(NO_MODEL));
        }
        if !self.state().held && self.parked.is_none() {
            return Err(SpeechError::NoSound);
        }
        let setup = setup(&ask, &self.shared.assets, self.shared.hardware.cores())?;
        let wait = Wait::Each(waits(&ask.job).write);
        let (shared, mut wiped) = (Arc::clone(&self.shared), Out::new());
        let mut live = shared.live();
        let here = live.key == Some(Key::of(&ask)) && self.parked.is_none();
        let written = if here {
            self.again_here(&live, setup, wait)
        } else {
            self.again_moved(&mut live, &ask, setup, wait, &mut wiped)
        };
        drop(live);
        router::tell(wiped);
        written?;
        self.limits = ask.job;
        Ok(())
    }

    fn keep(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        if let Some(parked) = &self.parked {
            return Ok(parked.clone());
        }
        let state = self.state();
        if !state.held {
            return Err(SpeechError::NoSound);
        }
        if state.dead {
            return Err(SpeechError::Gone);
        }
        let shared = Arc::clone(&self.shared);
        let live = shared.live();
        self.collect(&live)
    }

    /// Ends the run at once: no note comes after it returns, and the job wipes the recording.
    fn close(&mut self) {
        if std::mem::replace(&mut self.closed, true) {
            return;
        }
        self.parked = None;
        let hearer = lock(&self.shared.router).close(self.run);
        let wait = Wait::Once(waits(&self.limits).ask);
        let _ = self.send(VoiceAsk::Close { id: self.id }, wait);
        if let Some(hearer) = hearer {
            router::hush(&hearer);
        }
    }
}

impl<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>> Drop for WorkerRun<S> {
    fn drop(&mut self) {
        self.close();
    }
}

/// Begins the ask `make` builds on `stream`, its fresh id given to `run` before any note of it can come.
fn begin<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    router: &Arc<Mutex<Router>>,
    stream: &Stream<S, VoiceFrames>,
    run: u64,
    make: impl FnOnce(u32) -> VoiceAsk<'static> + Send + 'static,
    wait: Wait,
) -> Result<u32, SpeechError> {
    let (given, mapped) = (Arc::new(Mutex::new(None)), Arc::clone(router));
    let seen = Arc::clone(&given);
    let mapping = move |id| {
        lock(&mapped).map(run, id);
        *lock(&seen) = Some(id);
        make(id)
    };
    stream.begin(mapping, wait).map_err(|e| {
        if let Some(id) = lock(&given).take() {
            lock(router).unmap(id);
        }
        unsent(e)
    })
}

/// Hands `pieces` to a new job as `run`'s, each piece a wiped copy sent unwatched, then asks it to write them.
fn hand_in<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    router: &Arc<Mutex<Router>>,
    stream: &Stream<S, VoiceFrames>,
    run: u64,
    pieces: &[SecretBytes],
    setup: Setup<'static>,
    wait: Wait,
) -> Result<u32, SpeechError> {
    let piece = |index: usize| -> Result<SoundPiece<'static>, SpeechError> {
        let bytes = pieces.get(index).ok_or(SpeechError::NoSound)?;
        Ok(SoundPiece {
            index: u16::try_from(index).map_err(|_| SpeechError::Internal)?,
            last: index + 1 == pieces.len(),
            bytes: Wiped::Kept(bytes.expose().clone()),
        })
    };
    let head = piece(0)?;
    let id = begin(router, stream, run, move |id| sound(id, head), Wait::Off)?;
    for index in 1..pieces.len() {
        stream
            .ask(sound(id, piece(index)?), Wait::Off)
            .map_err(unsent)?;
    }
    let again = VoiceAsk::Again { id, setup };
    stream.ask(again, wait).map_err(unsent)?;
    Ok(id)
}

/// A piece of a recording handed in as run `id`'s.
const fn sound(id: u32, piece: SoundPiece<'static>) -> VoiceAsk<'static> {
    VoiceAsk::Sound { id, piece }
}

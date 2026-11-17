//! One dictation on the dictation job: its asks watched as `job::waits` says, its recording kept in the job until Close
//! or parked here when a new job could not take it, and Again in a new job when the model differs. A listen-only run's
//! speech comes back from the router once its Stop ended.

use super::begin::{begin, hand_in};
use super::frame::setup;
use super::job::{Key, Live, extra_mb, waits};
use super::note::unsent;
use super::router::{self, Out, State, Tell};
use super::{NO_MODEL, Shared, lock};
use crate::worker::{Spawner, Split, VoiceFrames, Wait};
use kx_ipc::{MAX_RECORD_BYTES, MAX_SOUND_PIECES, Setup, VoiceAsk};
use kx_module_api::SecretBytes;
use kx_module_api::ports::{JobLimits, SpeechAsk, SpeechError, SpeechRun};
use std::sync::Arc;
use std::sync::mpsc;

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
    /// It listens with no model, so its Stop hands its speech over.
    alone: bool,
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
        let (limits, parked, alone, closed) = (ask.job, None, ask.model.is_none(), false);
        Ok(Self {
            shared,
            run,
            id,
            limits,
            parked,
            alone,
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
        (self.limits, self.alone) = (ask.job, false);
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

    fn pieces(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        if !self.alone {
            return Err(SpeechError::NoSound);
        }
        lock(&self.shared.router).spoken(self.run)
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

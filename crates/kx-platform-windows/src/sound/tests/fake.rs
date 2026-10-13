//! A fake system that records the speaker's calls and answers as the test scripts it.

use std::collections::{HashSet, VecDeque};
use std::ops::RangeInclusive;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::sound::{Fault, Os, Warn};

/// A handle the fake made.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Part {
    Engine,
    Master,
    Voice(u32),
}

/// What the fake system was asked, in order.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Call {
    Make(Part),
    Gone(Part),
    Start,
    Stop,
    Play(u32, f32),
    Warn(Warn, Option<i32>),
}

/// The rates the fake's voices take.
pub(super) const RATES: RangeInclusive<u32> = 1_000..=48_000;

#[derive(Default)]
pub(super) struct State {
    calls: Vec<Call>,
    /// Faults the next engine starts give; empty means they start.
    pub starts: VecDeque<Fault>,
    /// Faults the next mastering voices give; empty means they are made.
    pub masters: VecDeque<Fault>,
    /// Faults the next plays give; empty means they play.
    pub plays: VecDeque<Fault>,
    /// Voices still playing.
    pub busy: HashSet<u32>,
    /// Voices made so far, which number them.
    made: u32,
    /// Blocks the next engine: it says it entered, then waits for the test to open it.
    pub gate: Option<(Sender<()>, Receiver<()>)>,
    /// Hears each play.
    pub heard: Option<Sender<()>>,
}

/// The fake system, shared by the test and the speaker thread.
#[derive(Clone, Default)]
pub(super) struct World(Arc<Mutex<State>>);

impl World {
    pub(super) fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The calls so far, forgotten.
    pub(super) fn take(&self) -> Vec<Call> {
        std::mem::take(&mut self.state().calls)
    }

    fn note(&self, call: Call) {
        self.state().calls.push(call);
    }

    fn make(&self, part: Part) -> Handle {
        self.note(Call::Make(part));
        Handle(self.clone(), part)
    }
}

/// A fake engine or voice, which notes when it is gone.
pub(super) struct Handle(World, Part);

impl Handle {
    fn voice(&self) -> u32 {
        if let Part::Voice(n) = self.1 { n } else { 0 }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.0.note(Call::Gone(self.1));
    }
}

impl Os for World {
    type Engine = Handle;
    type Master = Handle;
    type Voice = Handle;

    const RATES: RangeInclusive<u32> = RATES;

    fn engine(&self) -> Result<Handle, Fault> {
        let gate = self.state().gate.take();
        if let Some((entered, open)) = gate {
            let _ = entered.send(());
            let _ = open.recv();
        }
        Ok(self.make(Part::Engine))
    }

    fn master(&self, _: &Handle) -> Result<Handle, Fault> {
        let fault = self.state().masters.pop_front();
        fault.map_or_else(|| Ok(self.make(Part::Master)), Err)
    }

    fn voice(&self, _: &Handle, _: u32) -> Result<Handle, Fault> {
        let made = {
            let mut state = self.state();
            state.made += 1;
            state.made
        };
        Ok(self.make(Part::Voice(made)))
    }

    fn start(&self, _: &Handle) -> Result<(), Fault> {
        self.note(Call::Start);
        let fault = self.state().starts.pop_front();
        fault.map_or(Ok(()), Err)
    }

    fn stop(&self, _: &Handle) {
        self.note(Call::Stop);
    }

    fn play(&self, voice: &Handle, _: &'static [u8], gain: f32) -> Result<(), Fault> {
        self.note(Call::Play(voice.voice(), gain));
        let heard = self.state().heard.clone();
        if let Some(heard) = heard {
            let _ = heard.send(());
        }
        let mut state = self.state();
        if let Some(fault) = state.plays.pop_front() {
            return Err(fault);
        }
        state.busy.insert(voice.voice());
        Ok(())
    }

    fn busy(&self, voice: &Handle) -> bool {
        self.state().busy.contains(&voice.voice())
    }

    fn warn(&self, warn: Warn, code: Option<i32>) {
        self.note(Call::Warn(warn, code));
    }
}

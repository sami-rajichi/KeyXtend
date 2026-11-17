//! The engine over a speech library: one thread of its own takes every library call in order, so the parts load before
//! any device list or model, a run's pieces are written while listening goes on, and the job's thread never waits. One
//! job serves one model file; a dropped writer stops its run, aborting the call in flight.

mod desk;

use super::Tell;
use super::check::Exam;
use super::engine::{Engine, Writer};
use super::library::{Library, Stop};
use super::wipe::Buf;
use desk::{Desk, Order};
use kx_ipc::{Model, Target, VoiceFault, VoiceNote};
use std::borrow::Cow;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

/// The name of the engine's thread.
const THREAD: &str = "voice-engine";

/// What the engine's thread takes, in order.
pub enum Cmd {
    /// A run starts: its model loads unless held.
    Begin(Order),
    /// A run's next piece.
    Piece(Stop, Buf<i16>),
    /// A run's end: the sound heard in ms, and when Stop came.
    End(Stop, u32, Instant),
    /// The devices ask.
    Devices(u32, Tell),
    /// A facts ask on a file.
    Facts(u32, PathBuf, Tell),
    /// A card's check.
    Check(Exam),
}

/// An engine over the library `L`, whose thread starts on first use.
pub struct Local<L: Library> {
    tx: Option<Sender<Cmd>>,
    /// The library and the real cores, until the thread takes them.
    idle: Option<(L, usize)>,
    /// The model file this job serves, once a run named one.
    file: Option<String>,
}

impl<L: Library> Local<L> {
    /// An engine over `lib` on a processor of `cores` real cores.
    pub fn new(lib: L, cores: usize) -> Self {
        Self {
            tx: None,
            idle: Some((lib, cores)),
            file: None,
        }
    }

    /// Hands `cmd` to the engine's thread, starting it first.
    fn send(&mut self, cmd: Cmd) -> Result<(), VoiceFault> {
        if self.tx.is_none() {
            let (lib, cores) = self.idle.take().ok_or(VoiceFault::Internal)?;
            let (tx, rx) = mpsc::channel();
            thread::Builder::new()
                .name(THREAD.to_owned())
                .spawn(move || serve(Desk::new(lib, cores), &rx))
                .map_err(|_| VoiceFault::Internal)?;
            self.tx = Some(tx);
        }
        let tx = self.tx.as_ref().ok_or(VoiceFault::Internal)?;
        tx.send(cmd).map_err(|_| VoiceFault::Internal)
    }

    /// Refuses a model this library does not read, or whose file is missing.
    fn readable(model: &Model<'_>) -> Result<(), VoiceFault> {
        if model.runner != L::RUNNER {
            return Err(VoiceFault::Runner);
        }
        exists(&model.path)
    }
}

impl<L: Library> Engine for Local<L> {
    fn load(
        &mut self,
        id: u32,
        model: &Model<'_>,
        tell: Tell,
    ) -> Result<Box<dyn Writer>, VoiceFault> {
        Self::readable(model)?;
        if self.file.as_ref().is_some_and(|f| *f != *model.path) {
            return Err(VoiceFault::OutOfStep);
        }
        let (stop, file) = (Stop::default(), model.path.to_string());
        let model = owned(model);
        self.send(Cmd::Begin(Order {
            id,
            model,
            tell,
            stop: stop.clone(),
        }))?;
        self.file = Some(file);
        let tx = self.tx.clone().ok_or(VoiceFault::Internal)?;
        Ok(Box::new(Pen { tx, stop }))
    }

    fn devices(&mut self, id: u32, tell: Tell) -> Result<(), VoiceFault> {
        self.send(Cmd::Devices(id, tell))
    }

    fn facts(&mut self, id: u32, path: &str, tell: Tell) -> Result<(), VoiceFault> {
        exists(path)?;
        self.send(Cmd::Facts(id, PathBuf::from(path), tell))
    }

    fn check(
        &mut self,
        id: u32,
        model: &Model<'_>,
        clip: &str,
        tell: Tell,
    ) -> Result<(), VoiceFault> {
        Self::readable(model)?;
        exists(clip)?;
        if model.target == Target::Cpu {
            return Err(VoiceFault::OutOfStep);
        }
        let (model, clip) = (owned(model), PathBuf::from(clip));
        self.send(Cmd::Check(Exam {
            id,
            model,
            clip,
            tell,
        }))
    }
}

/// A run's writer: it hands each piece to the engine's thread and stops the run when dropped.
struct Pen {
    tx: Sender<Cmd>,
    stop: Stop,
}

impl Writer for Pen {
    fn piece(&mut self, sound: Buf<i16>) {
        let _ = self.tx.send(Cmd::Piece(self.stop.clone(), sound));
    }

    fn end(&mut self, heard_ms: u32) {
        let _ = self
            .tx
            .send(Cmd::End(self.stop.clone(), heard_ms, Instant::now()));
    }
}

impl Drop for Pen {
    fn drop(&mut self) {
        self.stop.set();
    }
}

/// The engine's thread: each command in turn until the engine drops. A panic tells its ask an internal fault and drops
/// the run and the model, so the next command starts clean.
fn serve<L: Library>(mut desk: Desk<L>, rx: &Receiver<Cmd>) {
    for cmd in rx {
        let who = desk.who(&cmd);
        if panic::catch_unwind(AssertUnwindSafe(|| desk.take(cmd))).is_err() {
            desk.reset();
            if let Some((id, tell)) = who {
                let fault = VoiceFault::Internal;
                tell.note(VoiceNote::Fault { id, fault });
            }
        }
    }
}

/// `NoFile` unless `path` names a file.
fn exists(path: &str) -> Result<(), VoiceFault> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(VoiceFault::NoFile)
    }
}

/// `model` with fields of its own, for the engine's thread.
fn owned(model: &Model<'_>) -> Model<'static> {
    let target = match &model.target {
        Target::Cpu => Target::Cpu,
        Target::Card { id, need_mb } => Target::Card {
            id: Cow::Owned(id.to_string()),
            need_mb: *need_mb,
        },
    };
    Model {
        path: Cow::Owned(model.path.to_string()),
        lang: Cow::Owned(model.lang.to_string()),
        runner: Cow::Owned(model.runner.to_string()),
        target,
        threads: model.threads,
    }
}

#[cfg(test)]
mod tests;

//! The job's downloads: one at a time, paused, resumed or cancelled by its id. A part a fault left stays for a later
//! resume until a Cancel of its id deletes it, as the platform fake does.

use super::Notes;
use super::get::{End, Load, Step, remove};
use kx_ipc::{FetchAsk, FetchFault, FetchNote, IpcError};
use kx_platform_net::Http;
use std::path::PathBuf;

/// The fetch job's state.
pub struct Job {
    http: Box<dyn Http>,
    /// The live download, reading or paused.
    load: Option<Load>,
    /// The part the last download left on a fault, with its id.
    left: Option<(u32, PathBuf)>,
}

impl Job {
    /// A job with no download, which asks over `http`.
    pub fn new(http: Box<dyn Http>) -> Self {
        Self {
            http,
            load: None,
            left: None,
        }
    }

    /// True while a download reads, so the loop steps it between asks.
    pub fn busy(&self) -> bool {
        self.load.as_ref().is_some_and(|l| !l.paused())
    }

    /// The live download's id.
    pub fn live(&self) -> Option<u32> {
        self.load.as_ref().map(Load::id)
    }

    /// Takes one ask. A download while another lives is refused as `Internal`, unless it repeats the live one's id;
    /// pause, resume and cancel of another id do nothing.
    ///
    /// # Errors
    /// The pipe's `IpcError` when a note cannot be written.
    pub fn take(&mut self, ask: FetchAsk<'static>, out: &mut dyn Notes) -> Result<(), IpcError> {
        let id = ask.id();
        let live = self.live() == Some(id);
        match ask {
            FetchAsk::Get { .. } if self.load.is_none() => self.start(ask, out)?,
            FetchAsk::Get { .. } if !live => {
                let fault = FetchFault::Internal;
                out.tell(FetchNote::Fault { id, fault })?;
            }
            FetchAsk::Cancel { .. } => self.cancel(id, out)?,
            FetchAsk::Pause { .. } | FetchAsk::Resume { .. } => {
                let pause = matches!(ask, FetchAsk::Pause { .. });
                if let Some(load) = self.load.as_mut().filter(|_| live) {
                    if pause { load.pause() } else { load.resume() }
                }
            }
            FetchAsk::Get { .. } | FetchAsk::Bye => {}
        }
        Ok(())
    }

    /// Takes one step of the live download: a read, or one part of reading the part back, asking, or the check.
    ///
    /// # Errors
    /// The pipe's `IpcError` when a note cannot be written.
    pub fn work(&mut self, out: &mut dyn Notes) -> Result<(), IpcError> {
        let Some(load) = self.load.as_mut() else {
            return Ok(());
        };
        let end = match load.step(self.http.as_ref()) {
            Step::Go(None) => return Ok(()),
            Step::Go(Some(note)) => return out.tell(note),
            Step::Fail(fault) => self.load.take().map(|l| l.fail(fault)),
            Step::Full => self.load.take().map(Load::name),
        };
        end.map_or(Ok(()), |end| self.ended(end, out))
    }

    /// Stops the live download, keeping its part when it holds any bytes, as the job ends.
    pub fn end(&mut self) {
        if let Some(load) = self.load.take() {
            drop(load.leave());
        }
    }

    /// Starts download `ask` and tells at once the bytes its part holds, or tells why it cannot start.
    fn start(&mut self, ask: FetchAsk<'static>, out: &mut dyn Notes) -> Result<(), IpcError> {
        self.left = None;
        let note = match Load::open(ask) {
            Ok(mut load) => {
                let note = load.progress();
                self.load = Some(load);
                note
            }
            Err(note) => note,
        };
        out.tell(note)
    }

    /// Cancels download `id`: the live one is told `Cancelled`; a part a fault left is deleted without a note.
    fn cancel(&mut self, id: u32, out: &mut dyn Notes) -> Result<(), IpcError> {
        if self.live() == Some(id) {
            let end = self.load.take().map(|l| l.fail(FetchFault::Cancelled));
            return end.map_or(Ok(()), |end| self.ended(end, out));
        }
        if let Some((_, part)) = self.left.take_if(|(left, _)| *left == id) {
            remove(&part);
        }
        Ok(())
    }

    /// Keeps the part `end` left, then tells its note.
    fn ended(&mut self, end: End, out: &mut dyn Notes) -> Result<(), IpcError> {
        self.left = end.left.map(|part| (end.note.id(), part));
        out.tell(end.note)
    }
}

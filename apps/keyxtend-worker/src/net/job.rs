//! The job's runs and key tests by id. A send or test whose id is live ends both as `Gone`, as the host is out of step;
//! a part for no live run is dropped unread, as a cancelled run's parts may still be in the pipe; an outcome for a run
//! or test no longer live, or for an older one of the same id, is never told.

use super::Notes;
use super::run::Run;
use super::send::{self, Sending};
use super::shape;
use super::{Event, Kit, Said, Told};
use kx_ipc::{IpcError, NetAsk, NetCall, NetFault, NetNote};
use std::borrow::Cow;
use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

/// A key's test sends once: a wrong key is told at once, and a 429 still waits once.
const TEST_TRIES: u8 = 1;
/// The name of each test's thread.
const TESTER: &str = "speech-net-test";

/// A live key test: its serial and the flag that drops it.
struct Test {
    serial: u64,
    dropped: Arc<AtomicBool>,
}

/// The speech job's state.
pub struct Job {
    kit: Kit,
    /// Where request threads tell their outcomes.
    tx: Sender<Event>,
    runs: HashMap<u32, Run>,
    tests: HashMap<u32, Test>,
    /// The last serial given to a run or test.
    serial: u64,
}

impl Job {
    /// A job with nothing live, whose requests go over `kit` and tell on `tx`.
    pub fn new(kit: Kit, tx: Sender<Event>) -> Self {
        Self {
            kit,
            tx,
            runs: HashMap::new(),
            tests: HashMap::new(),
            serial: 0,
        }
    }

    /// Takes one ask; goodbye is the loop's.
    ///
    /// # Errors
    /// The pipe's `IpcError` when a note cannot be written.
    pub fn take(&mut self, ask: NetAsk<'static>, out: &mut dyn Notes) -> Result<(), IpcError> {
        let id = ask.id();
        let fault = match ask {
            NetAsk::Send { call, run, .. } => self.start(id, |serial| Run::new(serial, call, run)),
            NetAsk::Test { call, .. } => self.test(id, call),
            NetAsk::Sound { piece, bytes, .. } => self.sound(id, piece, &bytes),
            NetAsk::Cancel { .. } => {
                self.drop_id(id);
                Ok(())
            }
            NetAsk::Bye => Ok(()),
        };
        match fault {
            Ok(()) => Ok(()),
            Err(fault) => {
                self.drop_id(id);
                out.tell(NetNote::Fault { id, fault })
            }
        }
    }

    /// Takes a request's outcome, telling what it makes due; a stale one is dropped.
    ///
    /// # Errors
    /// The pipe's `IpcError` when a note cannot be written.
    pub fn told(&mut self, told: Told, out: &mut dyn Notes) -> Result<(), IpcError> {
        let Told { id, serial, said } = told;
        match said {
            Said::Piece(index, said) => {
                let Some(run) = self.runs.get_mut(&id).filter(|r| r.serial() == serial) else {
                    return Ok(());
                };
                let (notes, ended) = run.told(id, index, said);
                if ended {
                    self.drop_id(id);
                } else if let Err(fault) = run.launch(id, &self.kit, &self.tx) {
                    self.drop_id(id);
                    return out.tell(NetNote::Fault { id, fault });
                }
                notes.into_iter().try_for_each(|n| out.tell(n))
            }
            Said::Test(said) => {
                if self.tests.get(&id).is_none_or(|t| t.serial != serial) {
                    return Ok(());
                }
                self.tests.remove(&id);
                let note = said.map_or_else(
                    |fault| NetNote::Fault { id, fault },
                    |()| NetNote::Done { id },
                );
                out.tell(note)
            }
        }
    }

    /// Drops every run and test, as the job ends.
    pub fn end(&mut self) {
        let ids: Vec<u32> = self.runs.keys().chain(self.tests.keys()).copied().collect();
        for id in ids {
            self.drop_id(id);
        }
    }

    /// Ids live at once: every run and test, for a panic's fault.
    pub fn live(&self) -> Vec<u32> {
        self.runs.keys().chain(self.tests.keys()).copied().collect()
    }

    /// Starts a run made by `make` under a new serial; a live id is out of step.
    fn start(
        &mut self,
        id: u32,
        make: impl FnOnce(u64) -> Result<Run, NetFault>,
    ) -> Result<(), NetFault> {
        self.taken(id)?;
        let run = make(self.next_serial())?;
        self.runs.insert(id, run);
        Ok(())
    }

    /// Starts a test of `call`'s key on a thread of its own.
    fn test(&mut self, id: u32, call: NetCall<'static>) -> Result<(), NetFault> {
        self.taken(id)?;
        let shape = shape::of(call.shape.code()).ok_or(NetFault::Range)?;
        let dropped = Arc::new(AtomicBool::new(false));
        let serial = self.next_serial();
        let sending = Sending {
            shape,
            call,
            model: None,
            lang: Cow::Borrowed(""),
            tries: TEST_TRIES,
            dropped: dropped.clone(),
        };
        let (kit, tx) = (self.kit.clone(), self.tx.clone());
        let work = move || {
            let said = panic::catch_unwind(AssertUnwindSafe(|| send::test(&kit, &sending)));
            let said = Said::Test(said.unwrap_or(Err(NetFault::Gone)));
            let _ = tx.send(Event::Told(Told { id, serial, said }));
        };
        let name = TESTER.to_owned();
        thread::Builder::new()
            .name(name)
            .spawn(work)
            .map_err(|_| NetFault::Gone)?;
        self.tests.insert(id, Test { serial, dropped });
        Ok(())
    }

    /// Takes a part of run `id`'s piece `piece`, then sends what is full.
    fn sound(&mut self, id: u32, piece: u16, bytes: &[u8]) -> Result<(), NetFault> {
        let Some(run) = self.runs.get_mut(&id) else {
            return Ok(());
        };
        run.sound(piece, bytes)?;
        run.launch(id, &self.kit, &self.tx)
    }

    /// Refuses an id already live, dropping the live one.
    fn taken(&mut self, id: u32) -> Result<(), NetFault> {
        if self.runs.contains_key(&id) || self.tests.contains_key(&id) {
            return Err(NetFault::Gone);
        }
        Ok(())
    }

    /// Drops run or test `id`: nothing more is sent or told of it.
    fn drop_id(&mut self, id: u32) {
        if let Some(run) = self.runs.remove(&id) {
            run.drop_requests();
        }
        if let Some(test) = self.tests.remove(&id) {
            test.dropped.store(true, Ordering::SeqCst);
        }
    }

    /// A serial no run or test had.
    fn next_serial(&mut self) -> u64 {
        self.serial += 1;
        self.serial
    }
}

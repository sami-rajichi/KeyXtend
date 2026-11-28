//! The record job's state: at most one run or copy, on a thread of its own, a run taking its switches, Stop and Cancel
//! as orders between frames. Work that ends hands back its last note once its parts are closed, so the job is free
//! before the host reads it.

use super::{Event, Flow, Notes};
use kx_ipc::{IpcError, RecordAsk, RecordEnd, RecordFault, RecordNote, Switch};
use kx_platform_record::record::{Order, Plan, Studio, record, scrap};
use kx_platform_record::speed::speed;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

/// The name of the thread that does the work.
const WORKER: &str = "record-work";

/// The work going on.
struct Live {
    /// Its run or copy id.
    id: u32,
    /// A run's orders; a copy takes none.
    orders: Option<Sender<Order>>,
    thread: JoinHandle<()>,
}

impl Live {
    /// Sends `order` when this is run `id`; false when it is not, or it already ended.
    fn order(&self, id: u32, order: Order) -> bool {
        let orders = self.orders.as_ref().filter(|_| self.id == id);
        orders.is_some_and(|o| o.send(order).is_ok())
    }
}

/// The job's state.
pub struct Job<S> {
    studio: S,
    tx: Sender<Event>,
    live: Option<Live>,
    /// Goodbye or the end of input came while work was live; the job exits with this once the work ended.
    leaving: Option<Result<(), IpcError>>,
}

impl<S: Studio + Clone> Job<S> {
    /// A job over `studio`'s parts whose work reports on `tx`.
    pub fn new(studio: S, tx: Sender<Event>) -> Self {
        Self {
            studio,
            tx,
            live: None,
            leaving: None,
        }
    }

    /// Takes one event, telling its notes on `out`; an ask that panics answers a failure.
    ///
    /// # Errors
    /// The pipe's `IpcError` when it fails.
    pub fn take(&mut self, event: Event, out: &mut dyn Notes) -> Result<Flow, IpcError> {
        match event {
            Event::Ask(RecordAsk::Bye) => Ok(self.leave(Ok(()))),
            Event::End(end) => Ok(self.leave(end)),
            Event::Ask(ask) => {
                let id = ask.id();
                let asked = panic::catch_unwind(AssertUnwindSafe(|| self.ask(ask)));
                match asked.unwrap_or(Err(RecordFault::Failed)) {
                    Ok(()) => Ok(Flow::Go),
                    Err(fault) => out.tell(RecordNote::Fault { id, fault }).map(|()| Flow::Go),
                }
            }
            Event::Note(note) => out.tell(note).map(|()| Flow::Go),
            Event::Done(id, last) => self.done(id, last, out),
        }
    }

    /// Stops a run still going, keeping its video, and waits for the work to end.
    pub fn end(&mut self) {
        if let Some(live) = self.live.take() {
            live.order(live.id, Order::Stop);
            let _ = live.thread.join();
        }
    }

    /// Goodbye or the end of input: exits now when idle, else stops a run and exits once the work ended.
    fn leave(&mut self, end: Result<(), IpcError>) -> Flow {
        let Some(live) = &self.live else {
            return Flow::Exit(end);
        };
        live.order(live.id, Order::Stop);
        self.leaving = Some(end);
        Flow::Go
    }

    /// The work `id` ended: the job is free, then its last note is told.
    fn done(
        &mut self,
        id: u32,
        last: Option<RecordNote>,
        out: &mut dyn Notes,
    ) -> Result<Flow, IpcError> {
        if let Some(live) = self.live.take_if(|l| l.id == id) {
            let _ = live.thread.join();
        }
        if let Some(note) = last {
            out.tell(note)?;
        }
        if self.live.is_none()
            && let Some(end) = self.leaving.take()
        {
            return Ok(Flow::Exit(end));
        }
        Ok(Flow::Go)
    }

    /// Takes an ask other than goodbye.
    fn ask(&mut self, ask: RecordAsk<'static>) -> Result<(), RecordFault> {
        let id = ask.id();
        match ask {
            RecordAsk::Start { .. } => self.start(plan(ask)?),
            RecordAsk::Speed {
                from, speed, to, ..
            } => self.copy(id, Path::new(from.as_ref()), speed, Path::new(to.as_ref())),
            RecordAsk::Mic { on, .. } => self.switch(id, Switch::Mic, on),
            RecordAsk::Sound { on, .. } => self.switch(id, Switch::Sound, on),
            // A Stop or Cancel that finds no run came after it ended by itself: its video stays, and nothing is told.
            RecordAsk::Stop { .. } => {
                self.order(id, Order::Stop);
                Ok(())
            }
            RecordAsk::Cancel { .. } => {
                self.order(id, Order::Cancel);
                Ok(())
            }
            RecordAsk::Bye => Ok(()),
        }
    }

    /// Sends `order` to run `id`; false when no such run is going.
    fn order(&self, id: u32, order: Order) -> bool {
        self.live.as_ref().is_some_and(|l| l.order(id, order))
    }

    /// Switches a sound of run `id`, which answers once it took effect; refused when no such run is going. A run that
    /// just ended dropped its orders, and its last note, already on its way, settles the switch.
    fn switch(&self, id: u32, which: Switch, on: bool) -> Result<(), RecordFault> {
        let live = self.live.as_ref().filter(|l| l.id == id);
        let orders = live
            .and_then(|l| l.orders.as_ref())
            .ok_or(RecordFault::Failed)?;
        let _ = orders.send(Order::Switch(which, on));
        Ok(())
    }

    /// Starts recording `plan`.
    fn start(&mut self, plan: Plan) -> Result<(), RecordFault> {
        let (orders, rx) = mpsc::channel();
        let (id, file) = (plan.id, plan.file.clone());
        self.begin(id, &file, Some(orders), move |studio, tell| {
            record(studio, &plan, &rx, tell)
        })
    }

    /// Starts copy `id` of `from` into `to`, `times` faster.
    fn copy(&mut self, id: u32, from: &Path, times: u8, to: &Path) -> Result<(), RecordFault> {
        let (from, made) = (from.to_owned(), to.to_owned());
        self.begin(id, to, None, move |studio, _| {
            Some(match speed(studio, &from, &made, times) {
                Ok(length_ms) => RecordNote::End {
                    id,
                    length_ms,
                    end: RecordEnd::Stopped,
                },
                Err(fault) => RecordNote::Fault { id, fault },
            })
        })
    }

    /// Starts `work` on a thread of its own, unless work is live or `file`, the new file it makes, exists. A panic
    /// inside deletes `file` and ends the work as a failure.
    fn begin<F>(
        &mut self,
        id: u32,
        file: &Path,
        orders: Option<Sender<Order>>,
        work: F,
    ) -> Result<(), RecordFault>
    where
        F: FnOnce(&S, &mut dyn FnMut(RecordNote)) -> Option<RecordNote> + Send + 'static,
    {
        if self.live.is_some() || file.exists() {
            return Err(RecordFault::Failed);
        }
        let (studio, tx, file) = (self.studio.clone(), self.tx.clone(), file.to_owned());
        let body = move || {
            let notes = tx.clone();
            let mut tell = |note| {
                let _ = notes.send(Event::Note(note));
            };
            let ran = panic::catch_unwind(AssertUnwindSafe(|| work(&studio, &mut tell)));
            let last = ran.unwrap_or_else(|_| {
                // The unwinding closed the work's parts; `Failed` tells the host to delete a file still there.
                let _ = scrap(&file);
                Some(RecordNote::Fault {
                    id,
                    fault: RecordFault::Failed,
                })
            });
            let _ = tx.send(Event::Done(id, last));
        };
        let spawned = thread::Builder::new().name(WORKER.to_owned()).spawn(body);
        let thread = spawned.map_err(|_| RecordFault::Failed)?;
        self.live = Some(Live { id, orders, thread });
        Ok(())
    }
}

/// The plan a start ask gives; any other ask is refused.
fn plan(ask: RecordAsk<'_>) -> Result<Plan, RecordFault> {
    let RecordAsk::Start {
        id,
        area,
        fps,
        mic,
        sound,
        max_min,
        min_free_mb,
        file,
    } = ask
    else {
        return Err(RecordFault::Failed);
    };
    let file = PathBuf::from(file.as_ref());
    Ok(Plan {
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

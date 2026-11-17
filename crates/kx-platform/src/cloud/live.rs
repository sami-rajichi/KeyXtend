//! A live run or key test, shared by its stream's thread, its timer and its owner: the board, the callback and the way
//! back to its job. Every note is told on the stream's thread, so a timeout never overtakes a text, and nothing here
//! waits on the worker.

use super::board::{Board, Step};
use super::lock;
use super::tick::Wake;
use crate::ear::Ear;
use crate::worker::{NetFrames, Spawner, Split, Stream, Told};
use kx_ipc::{NetAsk, NetNote};
use kx_module_api::Clock;
use kx_module_api::ports::CloudNote;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

/// The callback a run tells its notes to.
pub(super) type Tell = Box<dyn Fn(CloudNote) + Send>;

/// The way back to a run's job, reached without waiting.
pub(super) trait Back: Send + Sync {
    /// Cancels the job's run `id`: its requests are dropped and it tells nothing more.
    fn cancel(&self, id: u32);

    /// Runs `call` on the stream's thread, after the notes it holds.
    fn defer(&self, call: Box<dyn FnOnce() + Send>);

    /// Says goodbye to the job once its run ended, so no process outlives it.
    fn end(&self);
}

impl<S: Spawner<NetFrames, Worker: Split<NetFrames>>> Back for Stream<S, NetFrames> {
    fn cancel(&self, id: u32) {
        self.post(NetAsk::Cancel { id });
    }

    fn defer(&self, call: Box<dyn FnOnce() + Send>) {
        self.later(call);
    }

    fn end(&self) {
        self.bye();
    }
}

/// What a run's stream thread, timer and owner share.
pub(super) struct Live {
    board: Mutex<Board>,
    ear: Ear<CloudNote>,
    clock: Arc<dyn Clock>,
    back: OnceLock<Weak<dyn Back>>,
    /// The job's id for the run, given before its ask is sent.
    id: OnceLock<u32>,
    /// Wakes the run's timer; dropped as the run ends, which ends the timer.
    wake: Mutex<Option<Sender<Wake>>>,
}

impl Live {
    /// A run on `board` that tells `tell`, on `clock`.
    pub(super) fn new(board: Board, tell: Tell, clock: Arc<dyn Clock>) -> Self {
        Self {
            board: Mutex::new(board),
            ear: Ear::new(tell),
            clock,
            back: OnceLock::new(),
            id: OnceLock::new(),
            wake: Mutex::new(None),
        }
    }

    /// Joins the run to its job's way `back` and its timer's `wake`.
    pub(super) fn bind(&self, back: Weak<dyn Back>, wake: Sender<Wake>) {
        let _ = self.back.set(back);
        *lock(&self.wake) = Some(wake);
    }

    /// Notes the job's id for the run.
    pub(super) fn name(&self, id: u32) {
        let _ = self.id.set(id);
    }

    /// Takes what the stream told, on its thread.
    pub(super) fn hear(&self, told: Told<NetNote>) {
        let step = self.board().heard(told, self.clock.now());
        self.apply(step);
    }

    /// Checks the run's due time, on the stream's thread.
    pub(super) fn tick(&self) {
        let step = self.board().tick(self.clock.now());
        self.apply(step);
    }

    /// Starts the next piece's time, as the ask and its sound are all sent.
    pub(super) fn start(&self) {
        let mut board = self.board();
        board.start(self.clock.now());
        let due = board.due();
        drop(board);
        self.wake(Wake::At(due));
    }

    /// True once the run ended.
    pub(super) fn ended(&self) -> bool {
        self.board().ended()
    }

    /// Ends the run as its owner drops it: nothing more is told, the timer ends and the job's run is cancelled.
    pub(super) fn close(&self) {
        self.board().drop_run();
        self.ear.close();
        lock(&self.wake).take();
        self.cancel();
    }

    /// Asks for a check on the stream's thread, from the timer; false once the job is gone.
    pub(super) fn fire(self: Arc<Self>) -> bool {
        let Some(back) = self.back.get().and_then(Weak::upgrade) else {
            return false;
        };
        back.defer(Box::new(move || self.tick()));
        true
    }

    /// Has the timer look at the clock again, then waits until the stream took what that asked, so a test that moved
    /// the clock reads the outcome at once.
    #[cfg(test)]
    pub(super) fn poke(&self) {
        let (looked, seen) = std::sync::mpsc::channel();
        self.wake(Wake::Poke(looked));
        let _ = seen.recv();
        let (marked, flushed) = std::sync::mpsc::channel();
        if let Some(back) = self.back.get().and_then(Weak::upgrade) {
            back.defer(Box::new(move || {
                let _ = marked.send(());
            }));
            let _ = flushed.recv();
        }
    }

    /// Tells `step`'s notes after giving the timer the due time, cancels the job's run if it asks, and lets an ended
    /// run's timer and job go.
    fn apply(&self, step: Step) {
        let due = self.board().due();
        self.wake(Wake::At(due));
        self.ear.tell(step.notes);
        if step.cancel {
            self.cancel();
        }
        if self.ended()
            && lock(&self.wake).take().is_some()
            && let Some(back) = self.back.get().and_then(Weak::upgrade)
        {
            back.end();
        }
    }

    /// Cancels the job's run, without waiting.
    fn cancel(&self) {
        let back = self.back.get().and_then(Weak::upgrade);
        if let (Some(back), Some(&id)) = (back, self.id.get()) {
            back.cancel(id);
        }
    }

    /// Hands `wake` to the timer while the run lives.
    fn wake(&self, wake: Wake) {
        if let Some(timer) = lock(&self.wake).as_ref() {
            let _ = timer.send(wake);
        }
    }

    /// The board, held.
    fn board(&self) -> MutexGuard<'_, Board> {
        lock(&self.board)
    }
}

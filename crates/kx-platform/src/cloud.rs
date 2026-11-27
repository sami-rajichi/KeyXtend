//! `CloudSpeech` over the worker's `speech-net` job (ADR-0047): each run and key test has a job of its own, capped by
//! its sound, its requests in flight and their replies. Texts are taken in order within a cap and a time bound, a
//! cancel never waits on the worker, and keys and sound stay wiped.

mod board;
mod frame;
mod job;
mod live;
mod size;
mod tick;

use crate::lock;
use crate::worker::{NetFrames, Spawner, Split};
use board::Board;
use job::NetJob;
use kx_ipc::{NET_PACE_S, NET_TEST_TRIES, NetAsk};
use kx_module_api::Clock;
use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudNote, CloudRun, CloudSpeech, CloudTest, JobLimits,
};
use live::Tell;
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;
use std::sync::mpsc;

/// The value `Range` names when the job's shape refuses a value the frames let through: the row's request.
pub const SHAPE: &str = "shape";

/// A key's test sends one request, so its job holds one reply.
const TEST_AT_ONCE: u8 = 1;
/// A key's test waits for no turn in the pace: it is its job's only request.
const TEST_PACE_S: u16 = 0;

/// Cloud speech by the worker `S` starts; no job starts before the first `send` or `test`.
pub struct WorkerCloud<S: Spawner<NetFrames, Worker: Split<NetFrames>>> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
    /// The runs and tests opened, so a test can wake their timers after it moved the clock.
    #[cfg(test)]
    lives: Mutex<Vec<std::sync::Weak<live::Live>>>,
}

impl<S: Spawner<NetFrames, Worker: Split<NetFrames>>> WorkerCloud<S> {
    /// Cloud speech over `spawner`, whose jobs read the time from `clock`.
    #[must_use]
    pub fn new(spawner: S, clock: Arc<dyn Clock>) -> Self {
        Self {
            spawner: Arc::new(spawner),
            clock,
            #[cfg(test)]
            lives: Mutex::default(),
        }
    }

    /// A job capped at `job_mb` and `room_mb`, whose notes `board` judges for `tell`.
    fn open(
        &self,
        job: &JobLimits,
        room_mb: u32,
        board: Board,
        tell: Tell,
    ) -> Result<NetJob<S>, CloudFault> {
        let shape = (size::launch(job, room_mb), size::times(job));
        let (spawner, clock) = (Arc::clone(&self.spawner), Arc::clone(&self.clock));
        let opened = NetJob::open(spawner, clock, shape, board, tell)?;
        #[cfg(test)]
        lock(&self.lives).push(Arc::downgrade(opened.live()));
        Ok(opened)
    }

    /// Wakes every live run's timer, which then looks at the clock again.
    #[cfg(test)]
    fn poke(&self) {
        let lives = lock(&self.lives).clone();
        lives
            .iter()
            .filter_map(std::sync::Weak::upgrade)
            .for_each(|l| l.poke());
    }
}

impl<S: Spawner<NetFrames, Worker: Split<NetFrames>>> CloudSpeech for WorkerCloud<S> {
    fn send(&self, ask: CloudAsk, tell: Tell) -> Result<Box<dyn CloudRun>, CloudFault> {
        ask.check()?;
        let CloudAsk {
            job,
            shape,
            url,
            header,
            key,
            model,
            lang,
            limits,
            wait_s,
            tries,
            pieces,
        } = ask;
        let sizes: Vec<usize> = pieces.iter().map(|p| p.expose().len()).collect();
        let longest = sizes.iter().copied().max().unwrap_or_default();
        let room = size::run_mb(sizes.iter().sum(), limits.at_once, longest);
        let call = frame::call(shape, &url, header, key, wait_s);
        let run = frame::run(model, lang, limits, tries, &sizes)?;
        let board = Board::new(sizes, wait_s, tries, NET_PACE_S);
        let job = self.open(&job, room, board, tell)?;
        let id = job.begin(move |id| NetAsk::Send { id, call, run })?;
        job.hand_over(id, pieces);
        Ok(Box::new(job))
    }

    fn test(&self, ask: CloudTest) -> Result<(), CloudFault> {
        ask.check()?;
        let CloudTest {
            job,
            shape,
            url,
            header,
            key,
            wait_s,
        } = ask;
        let call = frame::call(shape, &url, header, key, wait_s);
        let (said, heard) = mpsc::channel();
        let tell: Tell = Box::new(move |note| drop(said.send(note)));
        let board = Board::new(Vec::new(), wait_s, NET_TEST_TRIES, TEST_PACE_S);
        let test = self.open(&job, size::run_mb(0, TEST_AT_ONCE, 0), board, tell)?;
        test.live().start();
        test.begin(move |id| NetAsk::Test { id, call })?;
        let answer = heard.recv().unwrap_or(CloudNote::Fault(CloudFault::Gone));
        drop(test);
        match answer {
            CloudNote::Done => Ok(()),
            CloudNote::Fault(fault) => Err(fault),
            CloudNote::Piece { .. } => Err(CloudFault::Gone),
        }
    }
}

#[cfg(test)]
mod tests;

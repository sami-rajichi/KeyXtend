//! A fake scroll target: the test scripts what each aim and reach returns, and the fake records every turn.

use crate::lock;
use kx_module_api::ports::{
    Aim, Dir, Reach, ScrollConfig, ScrollError, ScrollRun, ScrollTarget, TargetId,
};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};

/// What a caller bug hears when it turns by lines the port forbids.
const BAD_LINES: &str = "a turn needs finite lines of at least 0, as the scroll port says";

/// One turn a run took: which target, which way and how far.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Turn {
    /// The aim's target.
    pub id: TargetId,
    /// The way.
    pub dir: Dir,
    /// How many lines.
    pub lines: f64,
}

/// What one `aim` returns.
type Found = Result<Aim, ScrollError>;

/// What the runs did, and what the test set for them.
#[derive(Default)]
struct Log {
    /// The aims still to come, oldest first.
    aims: VecDeque<Found>,
    /// The latest aim given, which repeats once the queue is empty.
    last: Option<Found>,
    /// Reaches by target and way; a `None` target sets the way for every target.
    reach: HashMap<(Option<TargetId>, Dir), Reach>,
    turns: Vec<Turn>,
    turn_error: Option<ScrollError>,
    /// Targets whose window closed.
    closed: Vec<TargetId>,
    fail: Option<ScrollError>,
    config: Option<ScrollConfig>,
    opens: usize,
    closes: usize,
    aimed: usize,
    read: usize,
}

impl Log {
    /// The next scripted aim, or the latest once they run out; `NoTarget` before any.
    fn aim(&mut self) -> Found {
        self.aimed += 1;
        if let Some(next) = self.aims.pop_front() {
            self.last = Some(next);
        }
        self.last.unwrap_or(Err(ScrollError::NoTarget))
    }

    /// The target's own reach that way, else the way's, else `Unknown`.
    fn reach(&mut self, id: TargetId, dir: Dir) -> Reach {
        self.read += 1;
        let own = self.reach.get(&(Some(id), dir));
        let any = || self.reach.get(&(None, dir));
        own.or_else(any).copied().unwrap_or(Reach::Unknown)
    }
}

/// A scroll target the test scripts, shared by its clones: each `aim` takes the next scripted result, the latest
/// repeating once they run out (`NoTarget` before any), a reach is as set, `Unknown` by default, and a target lives
/// and turns until closed.
///
/// A turn by lines that are not finite or are below zero is a caller bug and panics.
#[derive(Clone, Default)]
pub struct FakeScrollTarget {
    log: Arc<Mutex<Log>>,
}

impl FakeScrollTarget {
    /// A target with nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `found` for a later `aim`, after those already queued.
    pub fn push_aim(&self, found: Result<Aim, ScrollError>) {
        lock(&self.log).aims.push_back(found);
    }

    /// Makes every `reach` towards `dir` return `reach`, unless the target has its own.
    pub fn set_reach(&self, dir: Dir, reach: Reach) {
        lock(&self.log).reach.insert((None, dir), reach);
    }

    /// Makes every `reach` of target `id` towards `dir` return `reach`.
    pub fn set_reach_for(&self, id: TargetId, dir: Dir, reach: Reach) {
        lock(&self.log).reach.insert((Some(id), dir), reach);
    }

    /// Refuses the next open with `error`.
    pub fn fail_next(&self, error: ScrollError) {
        lock(&self.log).fail = Some(error);
    }

    /// Closes target `id`'s window: from now on it is not alive and its turns are `Gone`, recording none.
    pub fn close(&self, id: TargetId) {
        lock(&self.log).closed.push(id);
    }

    /// Refuses every later turn with `error`, recording none, or takes them again with `None`.
    pub fn set_turn_error(&self, error: Option<ScrollError>) {
        lock(&self.log).turn_error = error;
    }

    /// Whether a run is held, from its open until the module drops it.
    #[must_use]
    pub fn running(&self) -> bool {
        let log = lock(&self.log);
        log.opens > log.closes
    }

    /// How many opens succeeded.
    #[must_use]
    pub fn opens(&self) -> usize {
        lock(&self.log).opens
    }

    /// How many runs closed.
    #[must_use]
    pub fn closes(&self) -> usize {
        lock(&self.log).closes
    }

    /// How many aims the runs took, errors too.
    #[must_use]
    pub fn aims(&self) -> usize {
        lock(&self.log).aimed
    }

    /// How many reaches the runs read.
    #[must_use]
    pub fn reaches(&self) -> usize {
        lock(&self.log).read
    }

    /// The config the latest open took, kept after it closes.
    #[must_use]
    pub fn config(&self) -> Option<ScrollConfig> {
        lock(&self.log).config
    }

    /// Every turn taken so far, oldest first.
    #[must_use]
    pub fn turns(&self) -> Vec<Turn> {
        lock(&self.log).turns.clone()
    }

    /// Every turn taken so far, oldest first, and forgets them.
    #[must_use]
    pub fn take_turns(&self) -> Vec<Turn> {
        std::mem::take(&mut lock(&self.log).turns)
    }
}

impl fmt::Debug for FakeScrollTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeScrollTarget")
            .field("running", &self.running())
            .field("opens", &self.opens())
            .field("closes", &self.closes())
            .finish_non_exhaustive()
    }
}

impl ScrollTarget for FakeScrollTarget {
    fn open(&self, config: ScrollConfig) -> Result<Box<dyn ScrollRun>, ScrollError> {
        let mut log = lock(&self.log);
        if let Some(error) = log.fail.take() {
            return Err(error);
        }
        log.config = Some(config);
        log.opens += 1;
        let log = Arc::clone(&self.log);
        Ok(Box::new(FakeRun { log }))
    }
}

/// A live fake run; dropping it closes it.
struct FakeRun {
    log: Arc<Mutex<Log>>,
}

impl ScrollRun for FakeRun {
    fn aim(&mut self) -> Result<Aim, ScrollError> {
        lock(&self.log).aim()
    }

    fn turn(&mut self, aim: &Aim, dir: Dir, lines: f64) -> Result<(), ScrollError> {
        assert!(lines.is_finite() && lines >= 0.0, "{BAD_LINES}: {lines}");
        let mut log = lock(&self.log);
        if log.closed.contains(&aim.id) {
            return Err(ScrollError::Gone);
        }
        if let Some(error) = log.turn_error {
            return Err(error);
        }
        log.turns.push(Turn {
            id: aim.id,
            dir,
            lines,
        });
        Ok(())
    }

    fn reach(&mut self, aim: &Aim, dir: Dir) -> Reach {
        lock(&self.log).reach(aim.id, dir)
    }

    fn alive(&mut self, aim: &Aim) -> bool {
        !lock(&self.log).closed.contains(&aim.id)
    }
}

impl Drop for FakeRun {
    fn drop(&mut self) {
        lock(&self.log).closes += 1;
    }
}

#[cfg(test)]
mod tests;

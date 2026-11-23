//! The job property's model of what lives on an id: a run or a key test under the serial the job gave it, and for a
//! run its parts' cursor, the texts told and its pieces in flight or waiting.

use super::super::super::bench::fault;
use super::Plan;
use super::fill::{FILLED, Kind};
use kx_ipc::{NetFault, NetNote};

/// A live run as the model holds it: how its pieces are answered, their sizes, the parts' cursor, the texts told, and
/// its pieces in flight and full ones waiting, within its requests at once.
pub(super) struct Track {
    pub(super) kind: Kind,
    pub(super) sizes: Vec<u32>,
    pub(super) cursor: (usize, usize),
    pub(super) told: usize,
    pub(super) flying: usize,
    at_once: usize,
    waiting: usize,
}

impl Track {
    /// A run of `plan` with nothing taken, sent or told.
    pub(super) fn new(plan: Plan) -> Self {
        let (kind, sizes, at_once) = (plan.kind, plan.sizes, usize::from(plan.at_once));
        Self {
            kind,
            sizes,
            cursor: (0, 0),
            told: 0,
            flying: 0,
            at_once,
            waiting: 0,
        }
    }

    /// What its current piece still needs; `FILLED` once every piece is full.
    pub(super) fn rest(&self) -> usize {
        let (at, got) = self.cursor;
        self.sizes.get(at).map_or(FILLED, |&s| s as usize - got)
    }

    /// A piece is full and waits for a request of its own.
    pub(super) fn filled(&mut self) {
        self.waiting += 1;
    }

    /// Sends the waiting pieces within its requests at once, as the job's run does; gives how many went.
    pub(super) fn launch(&mut self) -> usize {
        let went = self.waiting.min(self.at_once.saturating_sub(self.flying));
        self.waiting -= went;
        self.flying += went;
        went
    }
}

/// What lives on an id: a run, or a key test whose key works when true.
pub(super) enum Life {
    Run(Track),
    Test(bool),
}

impl Life {
    /// The note that ends it.
    pub(super) fn end(&self, id: u32) -> NetNote {
        match self {
            Self::Test(true) => NetNote::Done { id },
            Self::Test(false) => fault(id, NetFault::Key),
            Self::Run(run) => run.kind.end(id),
        }
    }
}

/// A live run or test and the serial the job gave it, so an older one's outcome is told apart.
pub(super) struct Live {
    pub(super) serial: u64,
    pub(super) life: Life,
}

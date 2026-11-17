//! A run's or a key test's notes as the port tells them: a text only at the next untold index below the piece count,
//! the texts within `MAX_NET_RUN_TEXT_BYTES` together, then one end. A piece past its bound ends as a timeout, and a
//! wrong step from the job ends the run; either way the job's run is cancelled.

use super::frame::{fault, lost};
use super::size::bound;
use crate::worker::Told;
use kx_ipc::{MAX_NET_RUN_TEXT_BYTES, NetNote};
use kx_module_api::ports::{CloudFault, CloudNote};
use kx_module_api::{Mono, Redacted, Zeroizing};

/// What a step tells, and whether the job's run is to be cancelled.
#[derive(Debug, Default)]
pub(super) struct Step {
    /// The notes to tell, in order.
    pub(super) notes: Vec<CloudNote>,
    /// The host ended the run itself, so the job must drop it.
    pub(super) cancel: bool,
}

/// Where a run stands.
pub(super) struct Board {
    /// Each piece's size in bytes; none for a key test.
    sizes: Vec<usize>,
    wait_s: u16,
    tries: u8,
    /// The turn a try may wait in the pace, in seconds.
    pace_s: u16,
    /// The next index to tell.
    told: usize,
    /// The text bytes told so far.
    bytes: usize,
    /// When the next piece, or the end after the last, is late.
    due: Option<Mono>,
    ended: bool,
}

impl Board {
    /// A run of pieces of `sizes`, each sent with `wait_s` and `tries`, waiting up to `pace_s` for each turn; nothing is
    /// late before `start`.
    pub(super) const fn new(sizes: Vec<usize>, wait_s: u16, tries: u8, pace_s: u16) -> Self {
        Self {
            sizes,
            wait_s,
            tries,
            pace_s,
            told: 0,
            bytes: 0,
            due: None,
            ended: false,
        }
    }

    /// Starts the next untold piece's time at `now`: the ask and its sound are all sent.
    pub(super) fn start(&mut self, now: Mono) {
        if !self.ended {
            self.due = Some(self.next_due(now));
        }
    }

    /// When the run is late, while it waits for the job.
    pub(super) const fn due(&self) -> Option<Mono> {
        self.due
    }

    /// True once the run ended: told its end, or dropped.
    pub(super) const fn ended(&self) -> bool {
        self.ended
    }

    /// Takes what the stream told at `now`.
    pub(super) fn heard(&mut self, told: Told<NetNote>, now: Mono) -> Step {
        if self.ended {
            return Step::default();
        }
        match told {
            Told::Note(NetNote::Text { index, text, .. }) => self.text(index, text, now),
            Told::Note(NetNote::Done { .. }) if self.told == self.sizes.len() => {
                self.end(CloudNote::Done, false)
            }
            Told::Note(NetNote::Done { .. }) => self.end(CloudNote::Fault(CloudFault::Gone), true),
            Told::Note(NetNote::Fault { fault: f, .. }) => {
                self.end(CloudNote::Fault(fault(f)), false)
            }
            Told::Lost(gone) => self.end(CloudNote::Fault(lost(gone)), false),
            Told::Idle => self.end(CloudNote::Fault(CloudFault::Gone), false),
        }
    }

    /// Ends the run as a timeout once `now` passes its due time.
    pub(super) fn tick(&mut self, now: Mono) -> Step {
        let late = !self.ended && self.due.is_some_and(|due| due <= now);
        if late {
            self.end(CloudNote::Fault(CloudFault::Timeout), true)
        } else {
            Step::default()
        }
    }

    /// Ends the run as its owner drops it: nothing more is told.
    pub(super) fn drop_run(&mut self) {
        (self.ended, self.due) = (true, None);
    }

    /// Takes piece `index`'s text, in order and within the run's cap, starting the next piece's time.
    fn text(&mut self, index: u16, text: Zeroizing<String>, now: Mono) -> Step {
        if usize::from(index) != self.told || self.told >= self.sizes.len() {
            return self.end(CloudNote::Fault(CloudFault::Gone), true);
        }
        self.bytes = self.bytes.saturating_add(text.len());
        if self.bytes > MAX_NET_RUN_TEXT_BYTES {
            return self.end(CloudNote::Fault(CloudFault::Strange), true);
        }
        self.told += 1;
        self.due = Some(self.next_due(now));
        let text = Redacted::new(text);
        Step {
            notes: vec![CloudNote::Piece { index, text }],
            cancel: false,
        }
    }

    /// Ends the run, telling `note`.
    fn end(&mut self, note: CloudNote, cancel: bool) -> Step {
        self.drop_run();
        Step {
            notes: vec![note],
            cancel,
        }
    }

    /// When the next untold piece, or the end after the last, is late from `now`.
    fn next_due(&self, now: Mono) -> Mono {
        let bytes = self.sizes.get(self.told).copied().unwrap_or_default();
        now.saturating_add(bound(bytes, self.wait_s, self.tries, self.pace_s))
    }
}

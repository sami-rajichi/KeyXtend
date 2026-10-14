//! The wait for our own input: the hand's input waits behind it until the newest batch's first echo, and a full queue
//! lets it pass. Tests read the wait and the room reserved for it here.

use kx_module_api::Mono;

use super::{Host, Trust};
use crate::hook::Os;
use crate::hook::echo::Wait;

impl<O: Os> Host<O> {
    /// True while our input is queued or not back yet.
    pub(super) fn behind(&mut self, now: Mono) -> bool {
        self.waiting(now) || !self.queue.is_empty()
    }

    /// True while our newest batch is not back; a wait past `echo_ms` ends and counts as a lost echo.
    pub(super) fn waiting(&mut self, now: Mono) -> bool {
        let wait = self.echo.check(now);
        if wait == Wait::Lost {
            self.stats.echo_lost();
        }
        wait == Wait::Waiting
    }

    /// The queue is full: events pass in order without the engine, whose view is then stale; counted once per overflow.
    pub(super) fn overflow(&mut self) {
        if self.queue.overflow() {
            self.stats.echo_lost();
        }
        if self.trust == Trust::Whole {
            self.trust = Trust::Stale;
        }
        self.wake();
    }

    /// True when no echo of ours is awaited.
    #[cfg(test)]
    pub(crate) fn is_clear(&self) -> bool {
        self.echo.is_clear()
    }

    /// The room of the lent buffer, the queue, the batch and the inbox.
    #[cfg(test)]
    pub(crate) fn room(&self) -> (usize, usize, usize, usize) {
        let (lent, out) = (self.lent.capacity(), self.out.capacity());
        (lent, self.queue.room(), out, self.inbox.room())
    }
}

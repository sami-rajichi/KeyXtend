//! The inbox the host lends to `SendInput`: Windows runs our hooks inside that call, on this thread and while the host
//! is busy, so their echoes wait here until the send returns.

use std::cell::RefCell;

use crate::hook::Mark;
use crate::hook::inbox::Inbox;

thread_local! {
    /// The inbox lent to this thread's hooks while our send runs; one with no room otherwise.
    static LENT: RefCell<Inbox> = const { RefCell::new(Inbox::none()) };
}

/// Runs `send` with `inbox` lent to this thread's hooks, and takes it back after, even after a panic.
pub(super) fn with_inbox<T>(inbox: &mut Inbox, send: impl FnOnce() -> T) -> T {
    let _lend = Lend::new(inbox);
    send()
}

/// Notes an event with `mark` heard while our send runs: our echo's id in the lent inbox while it has room, else, when
/// the host would `read` it, one more of the user's events.
pub(super) fn hear(mark: Mark, read: bool) {
    let note = |inbox: &mut Inbox| {
        if !inbox.hear(mark.extra) && read {
            inbox.user(mark.injected, mark.extra);
        }
    };
    let _ = LENT.try_with(|slot| slot.try_borrow_mut().map(|mut inbox| note(&mut inbox)));
}

/// The host's inbox, lent until this is dropped.
struct Lend<'a> {
    inbox: &'a mut Inbox,
    /// It went into the slot, so it comes back.
    lent: bool,
}

impl<'a> Lend<'a> {
    fn new(inbox: &'a mut Inbox) -> Self {
        let lent = swap(inbox);
        Self { inbox, lent }
    }
}

impl Drop for Lend<'_> {
    fn drop(&mut self) {
        if self.lent {
            swap(self.inbox);
        }
    }
}

/// Swaps `inbox` with this thread's slot, which moves no echo and allocates nothing; false when the slot is gone or
/// busy.
fn swap(inbox: &mut Inbox) -> bool {
    LENT.try_with(|slot| {
        slot.try_borrow_mut()
            .map(|mut lent| std::mem::swap(&mut *lent, inbox))
            .is_ok()
    })
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;
    use crate::hook::Source;
    use crate::hook::inbox::ECHOES_PER_ACT;
    use crate::keys::{TAG, replay_tag};

    /// The marks of an event with `extra`, injected or the hand's.
    fn mark(injected: bool, extra: usize) -> Mark {
        let time = 0;
        Mark {
            injected,
            extra,
            time,
        }
    }

    /// The marks of our echo of act `id`.
    fn echo(id: u16) -> Mark {
        mark(true, replay_tag(id))
    }

    #[test]
    fn echoes_heard_while_lent_reach_the_inbox_within_its_room_and_none_after() {
        let mut inbox = Inbox::new(1, Source::Physical);
        hear(echo(7), true);
        with_inbox(&mut inbox, || {
            hear(echo(1), true);
            hear(mark(true, TAG), true);
            hear(echo(2), true);
            hear(echo(3), true);
        });
        hear(echo(4), true);
        let heard: Vec<_> = inbox.drain().collect();
        assert_eq!(heard, [1, 2]);
        assert_eq!(inbox.room(), ECHOES_PER_ACT);
        assert_eq!(inbox.take_passed(), 0, "none of them is the hand's");
    }

    #[test]
    fn the_hands_events_heard_while_lent_count_when_the_host_reads_them() {
        let mut inbox = Inbox::new(1, Source::Physical);
        let hand = mark(false, 0);
        hear(hand, true);
        with_inbox(&mut inbox, || {
            hear(hand, true);
            hear(hand, false);
        });
        hear(hand, true);
        assert_eq!(
            inbox.take_passed(),
            1,
            "a wheel and the events outside the send count nothing"
        );
    }

    #[test]
    fn a_panic_inside_the_send_still_takes_the_inbox_back() {
        let mut inbox = Inbox::new(1, Source::Physical);
        let send = || with_inbox(&mut inbox, || panic!("the send panics"));
        assert!(catch_unwind(AssertUnwindSafe(send)).is_err());
        assert_eq!(inbox.room(), ECHOES_PER_ACT, "back, with its room");
        hear(echo(1), true);
        assert_eq!(inbox.drain().count(), 0, "nothing stays lent");
    }
}

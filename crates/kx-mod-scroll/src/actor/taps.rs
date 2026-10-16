//! A pad tap: while still it starts slow, pages our list or says why it cannot; while scrolling it goes fast, turns
//! or stops, settling the owed lines the old way first.

use super::{Actor, Scroll};
use crate::{Change, Glide, Halt};
use kx_module_api::Mono;
use kx_module_api::ports::{Aim, Dir, ScrollPage};

impl Actor {
    /// A pad tap: it restarts the limit's count, then acts on the pad's state.
    pub(super) fn tap(&mut self, dir: Dir) {
        let now = self.clock.now();
        self.stats.taps += 1;
        self.stops.tapped(now);
        if self.pad.scrolling() {
            return self.tap_on(dir, now);
        }
        let Some(found) = self.find() else {
            return;
        };
        match found {
            Ok(aim) if aim.ours => self.page(dir),
            Ok(aim) => self.start(dir, Some(aim), now),
            Err(err) => match Halt::unaimed(err) {
                Some(halt) => self.refuse(halt),
                None => self.start(dir, None, now),
            },
        }
    }

    /// Starts slow towards `dir` at `aim`, unless the target cannot move that way. With no aim yet, as when the OS
    /// refused it, the next tick aims again.
    fn start(&mut self, dir: Dir, aim: Option<Aim>, now: Mono) {
        if let Some(halt) = aim.and_then(|aim| self.blocked(&aim, dir)) {
            return self.refuse(halt);
        }
        // From still, a tap always begins.
        self.pad.tap(dir);
        let glide = Glide::start(self.cfg, self.pad.fast(), now);
        let due = if aim.is_some() {
            now.saturating_add(self.check)
        } else {
            now
        };
        self.scroll = Some(Scroll {
            glide,
            aim: None,
            due,
        });
        self.tell_state();
        if let Some(aim) = aim {
            self.follow(aim);
        }
    }

    /// A tap while scrolling: the same way again goes fast or stops, another way turns slow. The owed lines go the old
    /// way first, then it aims again, and a new way runs a first tap's checks.
    fn tap_on(&mut self, dir: Dir, now: Mono) {
        let old = self.pad.dir();
        let change = self.pad.tap(dir);
        if change == Change::Stopped {
            return self.end(None);
        }
        let new_way = change.new_way();
        let halt = self
            .settle(old, now)
            .or_else(|| self.tap_aim(new_way, now))
            .or_else(|| new_way.then(|| self.blocked_here(dir)).flatten());
        match halt {
            Some(halt) => self.end(Some(halt)),
            None => self.tell_state(),
        }
    }

    /// Settles the lines owed at the old speed and turns them the old way; the glide goes on at the pad's speed now.
    fn settle(&mut self, old: Option<Dir>, now: Mono) -> Option<Halt> {
        let fast = self.pad.fast();
        let owed = self.scroll.as_mut()?.glide.restart(fast, now);
        self.turn(old?, owed)
    }

    /// Aims at a tap while scrolling, which restarts the check's count. A new way's failure says why, as at a first
    /// tap; a faster tap's stops quietly.
    fn tap_aim(&mut self, new_way: bool, now: Mono) -> Option<Halt> {
        self.scroll.as_mut()?.due = now.saturating_add(self.check);
        let err = self.retarget().err()?;
        if new_way {
            Halt::unaimed(err)
        } else {
            Halt::failed(err)
        }
    }

    /// Why the scroll's target cannot move towards `dir`; none with no target yet.
    fn blocked_here(&mut self, dir: Dir) -> Option<Halt> {
        let aim = self.scroll.as_ref()?.aim?;
        self.blocked(&aim, dir)
    }

    /// Why `aim` cannot move towards `dir`: it reads the reach ahead, and the reach back only at an end.
    fn blocked(&mut self, aim: &Aim, dir: Dir) -> Option<Halt> {
        let run = self.run.as_mut()?;
        self.stops
            .blocked(run.reach(aim, dir), || run.reach(aim, dir.opposite()))
    }

    /// Turns our open list one page: back for up and left, on for down and right.
    fn page(&self, dir: Dir) {
        let back = matches!(dir, Dir::Up | Dir::Left);
        self.publish(&ScrollPage { back });
    }
}

//! A tick while scrolling: every `check_ms` the limit, a fresh aim and the reach ahead, then the owed lines turned on
//! the latest target. Only Refused never stops it: those lines drop, so nothing piles up.

use super::Actor;
use crate::Halt;
use kx_module_api::Mono;
use kx_module_api::ports::{Dir, ScrollError};

impl Actor {
    /// One tick at the clock's time; nothing while still.
    pub fn tick(&mut self) {
        let Some(dir) = self.pad.dir() else {
            return;
        };
        let now = self.clock.now();
        self.stats.ticks += 1;
        let halt = self.check(dir, now).or_else(|| self.glide(dir, now));
        if let Some(halt) = halt {
            self.end(Some(halt));
        }
    }

    /// Every `check_ms`: the limit, then a fresh aim, then the reach ahead; none when no check is due.
    fn check(&mut self, dir: Dir, now: Mono) -> Option<Halt> {
        let scroll = self.scroll.as_mut()?;
        if now < scroll.due {
            return None;
        }
        scroll.due = now.saturating_add(self.check);
        self.stops
            .timed_out(now)
            .or_else(|| self.retarget().err().and_then(Halt::failed))
            .or_else(|| self.reached(dir))
    }

    /// Turns the lines owed since the last reckoning.
    fn glide(&mut self, dir: Dir, now: Mono) -> Option<Halt> {
        let lines = self.scroll.as_mut()?.glide.tick(now);
        self.turn(dir, lines)
    }

    /// Turns the latest target `lines` towards `dir`; 0 lines, or no target yet, turn nothing.
    pub(super) fn turn(&mut self, dir: Dir, lines: f64) -> Option<Halt> {
        let aim = self.scroll.as_ref()?.aim?;
        if lines <= 0.0 {
            return None;
        }
        match self.run.as_mut()?.turn(&aim, dir, lines) {
            Ok(()) => {
                self.stats.turns += 1;
                None
            }
            Err(err) => {
                if err == ScrollError::Refused {
                    self.stats.refused += 1;
                }
                Halt::failed(err)
            }
        }
    }

    /// Aims again: a new target or box moves the outline, while our own windows and a refusal keep the last target.
    pub(super) fn retarget(&mut self) -> Result<(), ScrollError> {
        match self.find() {
            Some(Ok(aim)) if !aim.ours => self.follow(aim),
            Some(Err(err)) if err != ScrollError::Refused => return Err(err),
            _ => {}
        }
        Ok(())
    }

    /// Whether the target reached its end towards `dir`.
    fn reached(&mut self, dir: Dir) -> Option<Halt> {
        let aim = self.scroll.as_ref()?.aim?;
        let reach = self.run.as_mut()?.reach(&aim, dir);
        self.stops.reached(reach)
    }
}

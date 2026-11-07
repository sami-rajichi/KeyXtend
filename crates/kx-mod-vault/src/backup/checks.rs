//! Memory checks (design "Memory checks"): each asks a step of `check_days` after a new padlock or the last right
//! answer, and its banner's answer asks again after `later_h`. Each wrong answer uses one of `check_tries`, and after
//! the last a new password is set.

use std::time::Duration;

use kx_module_api::ports::{BoxError, FillNote};

use super::Checks;
use super::flow::{Flow, Goal, Then};
use super::state::{At, Backup, Flowed, Purpose};
use crate::entry::{Line, Stamp};

impl Backup {
    /// Starts the checks over at `wall`, as a new padlock does: the first step's wait and every try.
    pub(super) fn begin_checks(&mut self, wall: Stamp) {
        let due = Some(after(wall, self.step_wait(0)));
        self.kept.checks = Checks {
            due,
            step: 0,
            missed: 0,
        };
        self.raised = false;
    }

    /// The wait of step `step` in `check_days`, the last one repeating.
    fn step_wait(&self, step: u8) -> Duration {
        let steps = &self.cfg.steps;
        let at = usize::from(step).min(steps.len().saturating_sub(1));
        steps.get(at).copied().unwrap_or(self.cfg.later)
    }

    /// The longest a check may wait now: its step's wait, or Later's.
    fn longest(&self) -> Duration {
        self.step_wait(self.kept.checks.step).max(self.cfg.later)
    }

    /// True while checks may ask: they are on, and a padlock is made.
    fn checks_on(&self) -> bool {
        self.checks && self.kept.padlock.is_some()
    }

    /// True while a check's banner asks, so its Check opens the check.
    pub(crate) fn checking(&self) -> bool {
        self.raised && self.checks_on()
    }

    /// True once, when a check is due at `wall` and nothing is `busy`; a missing due time is set, and one past the
    /// longest wait comes nearer.
    pub(crate) fn check_due(&mut self, wall: Stamp, busy: bool) -> bool {
        if !self.checks_on() || self.raised {
            return false;
        }
        let (checks, latest) = (self.kept.checks, after(wall, self.longest()));
        let due = match checks.due {
            Some(due) if due <= latest => due,
            Some(_) => latest,
            None => after(wall, self.step_wait(checks.step)),
        };
        if checks.due != Some(due) {
            self.kept.checks.due = Some(due);
            self.save();
        }
        if busy || due > wall {
            return false;
        }
        self.raised = true;
        true
    }

    /// The wait at `wall` until the next check is due; `None` while checks are off, one asks, or one is due already,
    /// which the next tick raises once nothing is busy.
    pub(crate) fn check_wait(&self, wall: Stamp) -> Option<Duration> {
        if !self.checks_on() || self.raised {
            return None;
        }
        let ahead = self
            .kept
            .checks
            .due?
            .unix_ms()
            .checked_sub(wall.unix_ms())?;
        let ahead = u64::try_from(ahead).ok().filter(|ms| *ms > 0)?;
        Some(Duration::from_millis(ahead).min(self.longest()))
    }

    /// The banner's answer at `wall`, Check or Later: the check asks again after `later_h`, unless it goes right.
    pub(crate) fn later(&mut self, wall: Stamp) {
        if !std::mem::take(&mut self.raised) || self.kept.padlock.is_none() {
            return;
        }
        self.kept.checks.due = Some(after(wall, self.cfg.later));
        self.save();
    }

    /// The check's page: the password, with the tries left, or a new password once none is left.
    pub(crate) fn check(&mut self) -> Flowed {
        (self.asked, self.note) = (None, None);
        if self.tries_left() == 0 {
            return self.renew();
        }
        let line = Line::new(self.cfg.field_chars);
        let goal = Goal::Check;
        self.flow = Some(Flow::Password { line, goal });
        Flowed::Show
    }

    /// The tries the check has left.
    pub(super) fn tries_left(&self) -> u8 {
        self.cfg.tries.saturating_sub(self.kept.checks.missed)
    }

    /// The new password page after the last wrong answer; its padlock rewrites every box.
    fn renew(&mut self) -> Flowed {
        let line = Line::new(self.cfg.field_chars);
        let goal = Goal::New {
            place: None,
            then: Then::Change,
        };
        self.flow = Some(Flow::Password { line, goal });
        self.note = Some(FillNote::TriesUsed);
        Flowed::Show
    }

    /// Takes a check's answer of `round`: right steps on and leaves the pages, and wrong uses a try.
    pub(super) fn checked(
        &mut self,
        round: u32,
        checked: Result<bool, BoxError>,
        at: &At<'_>,
    ) -> Flowed {
        if !matches!(self.answered(round), Some(Purpose::Check)) {
            return Flowed::Stay;
        }
        match checked {
            Ok(true) => self.remembered(Stamp::of(&at.wall)),
            Ok(false) => self.missed(),
            Err(error) => {
                tracing::warn!(%error, "a memory check did not run");
                self.note = Some(FillNote::BoxFailed);
                Flowed::Show
            }
        }
    }

    /// A right answer at `wall`: the next step's wait, every try back, and the Remembered note on the list.
    fn remembered(&mut self, wall: Stamp) -> Flowed {
        let last = self.cfg.steps.len().saturating_sub(1);
        let step = usize::from(self.kept.checks.step)
            .saturating_add(1)
            .min(last);
        let step = u8::try_from(step).unwrap_or(u8::MAX);
        let due = Some(after(wall, self.step_wait(step)));
        self.kept.checks = Checks {
            due,
            step,
            missed: 0,
        };
        self.save();
        (self.flow, self.note) = (None, Some(FillNote::Remembered));
        Flowed::Leave
    }

    /// A wrong answer: one try fewer, and after the last a new password.
    fn missed(&mut self) -> Flowed {
        self.kept.checks.missed = self.kept.checks.missed.saturating_add(1);
        self.save();
        if self.tries_left() == 0 {
            return self.renew();
        }
        self.note = Some(FillNote::WrongPassword);
        Flowed::Show
    }
}

/// The wall time `wait` after `at`.
fn after(at: Stamp, wait: Duration) -> Stamp {
    let ms = i64::try_from(wait.as_millis()).unwrap_or(i64::MAX);
    Stamp::from_unix_ms(at.unix_ms().saturating_add(ms))
}

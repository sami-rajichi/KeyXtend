//! The guard's decisions over one hook's life: which events drop, and when it lets go.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use kx_module_api::Mono;

use super::{End, Kind, MouseGuard, Tally, Verdict, Whose};

/// The guard's decisions over one hook's life: which events drop, and when it lets go.
#[derive(Debug, Clone)]
pub struct Filter {
    /// When the time limit runs out.
    deadline: Mono,
    /// Physical presses that give the mouse back.
    clicks: usize,
    /// How close together they come, when more than one is set.
    window: Duration,
    /// When the latest physical presses came, oldest first; never more than `clicks`, so the hook never allocates.
    presses: VecDeque<Mono>,
    /// Buttons whose physical press the guard dropped.
    held: u8,
    /// Why it let go, once it has; every event then passes.
    out: Option<End>,
    /// What it dropped and passed, shared with the run that reads it live.
    tally: Arc<Tally>,
    /// The test build's `REPLAY` input counts as kx-gates' own, as the gate drives that build.
    replay: bool,
}

impl Filter {
    /// A filter for `cfg` that starts at `start`, after the guard already held the mouse for `used` in this run; it
    /// counts into `tally`, and passes `REPLAY` input as kx-gates' own when `replay`.
    pub fn new(
        cfg: &MouseGuard,
        start: Mono,
        used: Duration,
        tally: Arc<Tally>,
        replay: bool,
    ) -> Self {
        let left = Duration::from_millis(cfg.limit_ms).saturating_sub(used);
        let clicks = usize::from(cfg.escape_clicks);
        Self {
            deadline: start.saturating_add(left),
            clicks,
            window: Duration::from_millis(cfg.escape_window_ms),
            presses: VecDeque::with_capacity(clicks),
            held: 0,
            out: None,
            tally,
            replay,
        }
    }

    /// Why it lets go at `now`, if it does: the time limit, or the escape presses before it.
    pub fn tick(&mut self, now: Mono) -> Option<End> {
        if self.out.is_none() && now >= self.deadline {
            self.let_go(End::Limit);
        }
        self.out
    }

    /// Lets go with `end`, and tells the tally at once, so the gate refuses its next input before the hook thread
    /// even noticed.
    fn let_go(&mut self, end: End) {
        self.out = Some(end);
        self.tally.end(end);
    }

    /// Why it let go, if it already has; unlike `tick`, it decides nothing new.
    pub fn ended(&self) -> Option<End> {
        self.out
    }

    /// True when `whose` input is kx-gates' own: its tagged input, and the test build's replays while a gate drives it.
    fn own(&self, whose: Whose) -> bool {
        match whose {
            Whose::Own => true,
            Whose::Replay => self.replay,
            Whose::Hand => false,
        }
    }

    /// Whether an event at `now` passes: kx-gates' own input does, and so does a physical release whose press came
    /// before the guard, so no button sticks. It counts what it drops and the own input it passes.
    pub fn event(&mut self, whose: Whose, kind: Kind, now: Mono) -> Verdict {
        if self.own(whose) {
            self.tally.pass_one();
            return Verdict::Pass;
        }
        if self.tick(now).is_some() {
            return Verdict::Pass;
        }
        match kind {
            Kind::Down(b) => {
                self.held |= b;
                self.press(now);
            }
            Kind::Up(b) if self.held & b == 0 => return Verdict::Pass,
            Kind::Up(b) => self.held &= !b,
            Kind::Other => {}
        }
        self.tally.drop_one();
        Verdict::Drop
    }

    /// Notes a physical press at `now`; the escape's number of presses within the window lets go at once.
    fn press(&mut self, now: Mono) {
        while let Some(&first) = self.presses.front()
            && now.saturating_since(first) >= self.window
        {
            self.presses.pop_front();
        }
        self.presses.push_back(now);
        if self.presses.len() >= self.clicks {
            self.let_go(End::Owner);
        }
    }

    /// How long until `tick` could let go: the time limit, as the escape presses let go in the event itself.
    pub fn due_in(&self, now: Mono) -> Duration {
        self.deadline.saturating_since(now)
    }
}

#[cfg(test)]
mod tests;

//! The module's actor: it takes the pad's taps, Stop, peeks and presses, glides the target on ticks only while
//! scrolling, and publishes the state, the outline, our list's pages and the notices. A plain struct driven by
//! messages, plus a thread loop.

mod notes;
mod stats;
mod taps;
mod ticks;

use crate::{Glide, GlideCfg, Pad, ScrollSettings, Stops};
use kx_module_api::ports::{
    Aim, Dir, ScrollAimed, ScrollChanged, ScrollConfig, ScrollError, ScrollRun, ScrollTarget,
    TargetId,
};
use kx_module_api::{Bus, Clock, Mono};
use stats::Stats;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

pub(crate) use notes::notice;
pub use stats::STOPPED_LINE;

/// A message for the actor, from the bus or the module; it carries no point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Msg {
    /// A pad arrow was tapped.
    Tap(Dir),
    /// The pad's Stop button was tapped.
    Stop,
    /// The pointer went onto the pad (true) or off it.
    Peek(bool),
    /// A press landed on our windows (true) or elsewhere.
    Pressed(bool),
    /// A module or the UI asks for the state.
    Ask,
    /// The module stops.
    Quit,
}

/// A scroll under way: its glide, its target, and when the next check is due.
struct Scroll {
    glide: Glide,
    /// The latest target; none until an aim succeeds, and never again after.
    aim: Option<Aim>,
    /// When the next aim, reach and limit check is due.
    due: Mono,
}

impl Scroll {
    /// The first aim is still pending, as when the OS refused it, so the tap's checks wait for a target.
    const fn pending(&self) -> bool {
        self.aim.is_none()
    }
}

/// The scroll module's runtime: one per start, on its own thread.
pub struct Actor {
    target: Arc<dyn ScrollTarget>,
    bus: Bus,
    clock: Arc<dyn Clock>,
    /// How the platform finds, reads and moves targets.
    config: ScrollConfig,
    /// The glide's speeds and cap.
    cfg: GlideCfg,
    /// The glide's step period.
    tick_every: Duration,
    /// How often a scroll aims again, reads the reach and checks the limit.
    check_every: Duration,
    /// The platform's run, opened on first need and kept until the module stops.
    run: Option<Box<dyn ScrollRun>>,
    /// The last open failed; until one works, a failure logs nothing more.
    failing: bool,
    pad: Pad,
    stops: Stops,
    /// The scroll under way; some exactly while the pad scrolls.
    scroll: Option<Scroll>,
    /// The pointer is on the pad.
    peek: bool,
    /// The target and the outline it shows; none while it is hidden.
    shown: Option<(TargetId, ScrollAimed)>,
    stats: Stats,
}

impl Actor {
    /// An actor with no run yet; it reads the time from `clock`.
    #[must_use]
    pub fn new(
        target: Arc<dyn ScrollTarget>,
        bus: Bus,
        clock: Arc<dyn Clock>,
        settings: &ScrollSettings,
    ) -> Self {
        let ms = |n: u32| Duration::from_millis(n.into());
        Self {
            target,
            bus,
            clock,
            cfg: GlideCfg::from(settings),
            tick_every: ms(settings.tick_ms),
            check_every: ms(settings.check_ms),
            stops: Stops::from(settings),
            config: settings.scroll_config(),
            run: None,
            failing: false,
            pad: Pad::default(),
            scroll: None,
            peek: false,
            shown: None,
            stats: Stats::default(),
        }
    }

    /// The thread's loop: begins, then handles each message until `Quit`, ticking at each wake while scrolling.
    pub fn run(mut self, inbox: &Receiver<Msg>) {
        self.begin();
        loop {
            let next = match self.wait() {
                Some(wait) => inbox.recv_timeout(wait),
                None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match next {
                Ok(msg) => {
                    if !self.handle(msg) {
                        return;
                    }
                    self.tick();
                }
                Err(RecvTimeoutError::Timeout) => self.tick(),
                Err(RecvTimeoutError::Disconnected) => return self.quit(),
            }
        }
    }

    /// Says the module is live and still.
    pub fn begin(&mut self) {
        self.tell_state();
    }

    /// Handles one message; false after `Quit`, when the actor is done.
    pub fn handle(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::Tap(dir) => self.tap(dir),
            Msg::Stop => {
                if self.pad.scrolling() {
                    self.end(None);
                }
            }
            Msg::Peek(on) => self.peek(on),
            Msg::Pressed(ours) => {
                if self.pad.scrolling()
                    && let Some(halt) = self.stops.pressed(ours)
                {
                    self.end(Some(halt));
                }
            }
            Msg::Ask => self.ask(),
            Msg::Quit => {
                self.quit();
                return false;
            }
        }
        true
    }

    /// How long the loop waits for a message before it ticks: one tick while scrolling, and no limit while still.
    #[must_use]
    pub fn wait(&self) -> Option<Duration> {
        self.pad.scrolling().then_some(self.tick_every)
    }

    /// Lets go of the run, says the module is off with no outline, and logs the counts.
    fn quit(&mut self) {
        self.pad.stop();
        self.scroll = None;
        self.run = None;
        self.shown = None;
        self.publish(&ScrollChanged::OFF);
        self.publish(&ScrollAimed::NONE);
        self.stats.log();
    }

    /// Aims now, opening the run on first need; none when it does not open.
    fn find(&mut self) -> Option<Result<Aim, ScrollError>> {
        let found = self.open()?.aim();
        if found == Err(ScrollError::Refused) {
            self.stats.refused += 1;
        }
        Some(found)
    }

    /// The run, opened on first need; a failed open logs once per streak, and the caller decides whom to tell.
    fn open(&mut self) -> Option<&mut Box<dyn ScrollRun>> {
        if self.run.is_none() {
            match self.target.open(self.config) {
                Ok(run) => (self.run, self.failing) = (Some(run), false),
                Err(error) => {
                    if !std::mem::replace(&mut self.failing, true) {
                        tracing::warn!(%error, "scroll target did not open");
                    }
                    return None;
                }
            }
        }
        self.run.as_mut()
    }
}

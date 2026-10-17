//! G8's scroll part in order: a scroller's reading after one scroll, the hold, `PresentMon`'s frames, the churn with
//! the scroller's scrolls, then its close. While the scroller is kept its own window is the hold, as a fresh hold's
//! target-window start would close it; a fresh hold runs only once no scroller is kept.
#![cfg(windows)]

use crate::apps::{Ctx, Opened};
use crate::g8churn::{self, Churn};
use crate::g8lang::Holds;
use crate::g8scroll::{Scrolled, Scroller};
use crate::proc::Process;

/// What G8's scroll part works through: a scroller on Windows, a fake in tests.
pub trait Steps {
    /// A kept scroller.
    type Pad;
    /// Opens a scroller.
    fn open(&self) -> Result<Self::Pad, String>;
    /// One scroll with `pad`, and the reading after it.
    fn reading(&self, pad: &Self::Pad) -> Result<Scrolled, String>;
    /// The hold through `pad`'s own window.
    fn kept(&self, pad: &Self::Pad) -> Result<(), String>;
    /// The hold through a fresh own window.
    fn fresh(&self) -> Result<(), String>;
    /// The churn, each round scrolling with `pad` when there is one.
    fn churn(&self, pad: Option<&Self::Pad>) -> Result<Churn, String>;
    /// Closes `pad`; what was left open.
    fn close(&self, pad: Self::Pad) -> Vec<String>;
}

/// What the scroll part gave: the reading after a scroll, what ran between, the churn and what the scroller left open.
pub struct Rounds<T> {
    /// The reading after a scroll, or why there is none.
    pub scrolled: Result<Scrolled, String>,
    /// What ran before the churn: `PresentMon`'s frames.
    pub between: T,
    /// The churn.
    pub churn: Result<Churn, String>,
    /// What the scroller's close left open.
    pub left: Vec<String>,
}

/// Runs the scroll part with `s` in order, `between` before the churn; a scroller whose reading failed is closed before
/// the fresh hold, and an error in the hold closes any scroller.
pub fn run<S: Steps, T>(s: &S, between: impl FnOnce() -> T) -> Result<Rounds<T>, String> {
    let (scrolled, pad) = match s.open() {
        Ok(pad) => match s.reading(&pad) {
            Ok(read) => (Ok(read), Some(pad)),
            Err(e) => (Err(format!("{e}; left open: {:?}", s.close(pad))), None),
        },
        Err(e) => (Err(e), None),
    };
    let held = pad.as_ref().map_or_else(|| s.fresh(), |p| s.kept(p));
    if let Err(e) = held {
        return Err(format!("{e}; left open: {:?}", pad.map(|p| s.close(p))));
    }
    let between = between();
    let churn = s.churn(pad.as_ref());
    let left = pad.map(|p| s.close(p)).unwrap_or_default();
    Ok(Rounds {
        scrolled,
        between,
        churn,
        left,
    })
}

/// The scroll part on Windows: `p`'s memory, with `hold` after the reading.
pub struct WinSteps<'a> {
    ctx: &'a Ctx<'a>,
    p: &'a Process,
    hold: &'a dyn Holds<Opened>,
}

impl<'a> WinSteps<'a> {
    /// The scroll part of `ctx` on process `p`, with `hold`.
    pub fn of(ctx: &'a Ctx<'a>, p: &'a Process, hold: &'a dyn Holds<Opened>) -> Self {
        Self { ctx, p, hold }
    }
}

impl<'a> Steps for WinSteps<'a> {
    type Pad = Scroller<'a>;

    fn open(&self) -> Result<Scroller<'a>, String> {
        Scroller::open(self.ctx)
    }

    fn reading(&self, pad: &Scroller<'a>) -> Result<Scrolled, String> {
        pad.reading(self.p)
    }

    fn kept(&self, pad: &Scroller<'a>) -> Result<(), String> {
        self.hold.kept(pad.window())
    }

    fn fresh(&self) -> Result<(), String> {
        self.hold.fresh()
    }

    fn churn(&self, pad: Option<&Scroller<'a>>) -> Result<Churn, String> {
        g8churn::run(self.ctx, self.p, pad)
    }

    fn close(&self, pad: Scroller<'a>) -> Vec<String> {
        pad.close()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::g8scroll::read_as;
    use crate::proc::Memory;

    /// Which step fails in the fake.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Fails {
        Nothing,
        Open,
        Reading,
        Hold,
    }

    /// A scroll part that logs each step in order, failing the one it is told to.
    struct Log {
        fails: Fails,
        steps: RefCell<Vec<&'static str>>,
    }

    impl Log {
        fn new(fails: Fails) -> Self {
            Self {
                fails,
                steps: RefCell::default(),
            }
        }

        /// Logs `step`, an error when it is the one that fails.
        fn step(&self, step: &'static str, fails: Fails) -> Result<(), String> {
            self.steps.borrow_mut().push(step);
            if self.fails == fails {
                return Err(step.to_string());
            }
            Ok(())
        }
    }

    const MB: Memory = Memory {
        working_mb: 1.0,
        commit_mb: 1.0,
    };

    impl Steps for Log {
        type Pad = ();

        fn open(&self) -> Result<(), String> {
            self.step("open", Fails::Open)
        }

        fn reading(&self, (): &()) -> Result<Scrolled, String> {
            self.step("reading", Fails::Reading).map(|()| read_as(MB))
        }

        fn kept(&self, (): &()) -> Result<(), String> {
            self.step("kept hold", Fails::Hold)
        }

        fn fresh(&self) -> Result<(), String> {
            self.step("fresh hold", Fails::Hold)
        }

        fn churn(&self, pad: Option<&()>) -> Result<Churn, String> {
            let step = if pad.is_some() { "churn" } else { "churn bare" };
            self.steps.borrow_mut().push(step);
            Err("no churn here".to_string())
        }

        fn close(&self, (): ()) -> Vec<String> {
            self.steps.borrow_mut().push("close");
            Vec::new()
        }
    }

    /// The steps `s` ran, with "frames" where `between` ran.
    fn ran(s: &Log) -> (Result<Rounds<()>, String>, Vec<&'static str>) {
        let got = run(s, || s.steps.borrow_mut().push("frames"));
        (got, s.steps.borrow().clone())
    }

    #[test]
    fn a_kept_scroller_is_the_hold_and_closes_only_after_the_churn() {
        let (got, steps) = ran(&Log::new(Fails::Nothing));
        assert_eq!(
            steps,
            ["open", "reading", "kept hold", "frames", "churn", "close"]
        );
        assert!(got.expect("ran").scrolled.is_ok());
    }

    #[test]
    fn a_scroller_whose_reading_failed_closes_before_the_fresh_hold() {
        let (got, steps) = ran(&Log::new(Fails::Reading));
        let order = [
            "open",
            "reading",
            "close",
            "fresh hold",
            "frames",
            "churn bare",
        ];
        assert_eq!(steps, order);
        assert!(got.expect("ran").scrolled.is_err());
        let (_, steps) = ran(&Log::new(Fails::Open));
        assert_eq!(steps, ["open", "fresh hold", "frames", "churn bare"]);
    }

    #[test]
    fn a_failed_hold_closes_the_kept_scroller_and_stops_the_part() {
        let (got, steps) = ran(&Log::new(Fails::Hold));
        assert_eq!(steps, ["open", "reading", "kept hold", "close"]);
        let e = got.err().expect("stopped");
        assert!(e.starts_with("kept hold") && e.contains("left open"), "{e}");
    }
}

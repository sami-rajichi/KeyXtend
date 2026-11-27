//! The capture flow on a test clock (plan task 10): every path through picking, the strip, the countdown, recording,
//! the question and the editor, edges 1, 7, 10, 17 and 22, and the property that no path keeps an unchosen file.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

use super::*;
use kx_module_api::ports::{
    AskKey, BarKey, EditKey, Frame, FreezeError, Keep, Part, Point, RecKey, RecordDone, RecordEnd,
    RecordError, RecordNote, Rect, Shot, SnipAside, SnipKeyboard, SnipLit, SnipNote, Source, Speed,
    StripKey,
};
use kx_platform_fake::{FakeClock, painted};

use crate::COUNTDOWN_S;
use crate::part::Home;

mod aside;
mod bare;
mod canvas;
mod close;
mod editor;
mod ends;
mod frame;
mod notes;
mod picking;
mod play;
mod props;
mod record;
mod reel;
mod spots;
mod strip;

/// The left test screen, at 100 %.
const LEFT: Rect = Rect {
    x: 0,
    y: 0,
    w: 40,
    h: 30,
};
/// The right test screen, at 150 %.
const RIGHT: Rect = Rect {
    x: 40,
    y: 0,
    w: 30,
    h: 30,
};
/// Both screens together, which a picture region's frame keeps within.
const DESKTOP: Rect = Rect {
    x: 0,
    y: 0,
    w: 70,
    h: 30,
};
/// The part the tests take, on the left screen.
const PART: Rect = Rect {
    x: 4,
    y: 6,
    w: 10,
    h: 8,
};
/// The part's top-left pixel, which Region clicks first.
const NEAR: Point = Point {
    x: PART.x,
    y: PART.y,
};
/// The part's bottom-right pixel, the opposite corner.
const FAR: Point = Point {
    x: PART.x + PART.w - 1,
    y: PART.y + PART.h - 1,
};
/// A pixel inside the part, which is also the window.
const INSIDE: Point = Point {
    x: PART.x + 1,
    y: PART.y + 1,
};
/// A point on no screen.
const AWAY: Point = Point { x: 500, y: 500 };
/// A recorded video's length in the tests, in ms.
const LENGTH_MS: u32 = 4_200;
/// One ms, the least the clock moves.
const MS: Duration = Duration::from_millis(1);

/// Both screens as frozen, with one window and the pointer on the left.
fn frozen() -> Frozen {
    Frozen {
        screens: vec![painted(LEFT, 96).unwrap(), painted(RIGHT, 144).unwrap()],
        windows: vec![PART],
        pointer: Point { x: 5, y: 5 },
    }
}

fn part() -> Part {
    Part::new(PART).unwrap()
}

/// The screen holding the part.
fn home() -> Home {
    Home {
        rect: LEFT,
        dpi: 96,
    }
}

/// A picture region's frame as taken: both screens together, the default smallest side, no handle held.
fn region_frame() -> Frame {
    Frame {
        within: DESKTOP,
        min: SnipSettings::from_defaults().unwrap().frame_min_px,
        grip: None,
    }
}

/// The part's pixels as frozen.
fn shot() -> Shot {
    part().cut(&frozen().screens).unwrap()
}

/// Another picture, as the editor flattens it.
fn flat() -> Shot {
    Shot::new(2, 1, vec![9; 8]).unwrap()
}

fn speed(times: u8) -> Speed {
    Speed::new(times).unwrap()
}

/// What the strip of recorded `video` at `speed` holds, `LENGTH_MS` long.
fn reel(video: Video, speed: Speed) -> Held {
    let clip = Recorded {
        video,
        length_ms: LENGTH_MS,
    };
    Held::Video(Reel { clip, speed })
}

/// A run's end after `LENGTH_MS`, for `end`.
fn done(end: RecordEnd) -> RecordNote {
    RecordNote::Done(RecordDone {
        length_ms: LENGTH_MS,
        end,
    })
}

/// The video `Out::Record` named, if any.
fn recorded(outs: &[Out]) -> Option<Video> {
    outs.iter().find_map(|o| match o {
        Out::Record { video, .. } => Some(*video),
        _ => None,
    })
}

/// Where `step` is in `outs`.
fn at(outs: &[Out], step: &Out) -> Option<usize> {
    outs.iter().position(|o| o == step)
}

/// The snip ended with the keyboard back and the key unlit.
fn ended(outs: &[Out]) -> bool {
    let has = |step: Out| outs.contains(&step);
    has(Out::End) && has(Out::Aside(SnipAside::Back)) && has(Out::Lit(SnipLit::Off))
}

/// Something was copied, saved or recorded.
fn kept_any(outs: &[Out]) -> bool {
    outs.iter()
        .any(|o| matches!(o, Out::Keep { .. } | Out::Record { .. }))
}

/// A flow on a clock the test moves; it never sleeps.
struct Rig {
    flow: Flow,
    clock: FakeClock,
    /// The settings the flow started with.
    settings: SnipSettings,
}

impl Rig {
    fn new() -> Self {
        Self::with(|_| {})
    }

    /// A rig whose defaults `set` changes first.
    fn with(set: impl FnOnce(&mut SnipSettings)) -> Self {
        let mut settings = SnipSettings::from_defaults().unwrap();
        set(&mut settings);
        let clock = FakeClock::new(Mono::default());
        let flow = Flow::new(Arc::new(clock.clone()), settings.clone());
        Self {
            flow,
            clock,
            settings,
        }
    }

    /// The whole countdown of the settings the flow started with.
    fn countdown(&self) -> Duration {
        SECOND * u32::from(self.settings.countdown_s)
    }

    /// How long a tick shows, as the settings the flow started with say.
    fn shown(&self) -> Duration {
        self.settings.shown()
    }

    fn on(&mut self, input: In) -> Vec<Out> {
        self.flow.on(input)
    }

    /// Moves the time on by `by`, then ticks.
    fn wait(&mut self, by: Duration) -> Vec<Out> {
        self.clock.advance(by);
        self.on(In::Tick)
    }

    fn now(&self) -> Mono {
        self.clock.now()
    }

    fn stage(&self) -> &Stage {
        self.flow.stage()
    }

    /// The Snip key, the ui's answer that the keyboard stepped aside, then the freeze.
    fn picking(&mut self) {
        self.on(In::Key);
        self.on(In::Aside);
        self.on(In::Frozen(Ok(frozen())));
    }

    /// A click on the frozen screen at (`x`, `y`).
    fn spot(&mut self, x: i32, y: i32) -> Vec<Out> {
        self.on(In::Spot(Point { x, y }))
    }

    /// The part's two corners clicked, as Region and Video region take it.
    fn take(&mut self) {
        self.on(In::Spot(NEAR));
        self.on(In::Spot(FAR));
    }

    /// A picture taken, with its strip.
    fn picture(&mut self) {
        self.picking();
        self.take();
    }

    /// A video's part taken, before it records.
    fn ready(&mut self) {
        self.picking();
        self.on(In::Bar(BarKey::VideoRegion));
        self.take();
    }

    /// Record, the countdown, and the run's start.
    fn recording(&mut self) -> Video {
        self.ready();
        let mut outs = self.on(In::Strip(StripKey::Record));
        for _ in COUNTDOWN_S {
            if recorded(&outs).is_some() {
                break;
            }
            outs = self.wait(SECOND);
        }
        let video = recorded(&outs).unwrap();
        self.on(In::Note {
            video,
            note: RecordNote::Time { ms: 0 },
        });
        video
    }

    /// A recording stopped, with its strip.
    fn stopped(&mut self) -> Video {
        let video = self.recording();
        self.on(In::Record(RecKey::Stop));
        self.on(In::Note {
            video,
            note: done(RecordEnd::Stopped),
        });
        video
    }

    /// The editor on a picture.
    fn editor(&mut self) {
        self.picture();
        self.on(In::Strip(StripKey::Edit));
    }

    /// A Copy or Save of the editor's picture, answered done.
    fn flat_kept(&mut self, keep: Keep) {
        self.on(In::Flat { keep, shot: flat() });
        self.on(In::Kept(Outcome::Done));
    }
}

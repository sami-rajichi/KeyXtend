//! The frame log (`[ui] frame_log`): each press's and each frame's steps on Qt's clock, in µs, joined into one line per
//! press and per frame to measure the key light (G8). It holds times and window handles only, never a key or text.

/// A frame's steps on Qt's clock, in µs; 0 for a step that did not happen.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    /// The update request reached the window.
    pub update: i64,
    /// The frame began (`beforeFrameBegin`).
    pub begin: i64,
    /// The scene began to sync (`beforeSynchronizing`).
    pub sync: i64,
    /// The scene was drawn (`afterRendering`).
    pub rendered: i64,
    /// The frame was handed to the screen (`frameSwapped`).
    pub swap: i64,
    /// The frame ended (`afterFrameEnd`).
    pub done: i64,
}

impl Frame {
    /// Its first step, or 0 for none.
    fn first(&self) -> i64 {
        let steps = [self.update, self.begin, self.sync, self.rendered, self.swap];
        steps.into_iter().find(|&t| t != 0).unwrap_or(self.done)
    }
}

/// A step of a press's action that the keyboard window marks.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mark {
    /// The key's action starts.
    Start,
    /// The keyboard sent the key, injection included.
    Sent,
    /// The action and its redraw ended.
    End,
}

/// A frame line, in µs: when it started, then each step's share of its CPU time; `None` where a step was missing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameTimes {
    /// Its first step on Qt's clock.
    pub at_us: i64,
    /// Update request to begin: Qt's events, polish and animations.
    pub polish_us: Option<i64>,
    /// Begin to sync: Direct3D's frame start, with any wait for a frame slot.
    pub wait_us: Option<i64>,
    /// Sync to drawn.
    pub render_us: Option<i64>,
    /// Drawn to handed to the screen.
    pub present_us: Option<i64>,
    /// The whole frame.
    pub busy_us: Option<i64>,
}

/// A press line: when Qt got the press's message (or, unseen, the press), then each step from it in µs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PressTimes {
    /// Qt got the message, or the press when the message went unseen, on Qt's clock.
    pub at_us: i64,
    /// The press reached the window.
    pub qt_us: Option<i64>,
    /// The key's action started.
    pub start_us: Option<i64>,
    /// The keyboard sent the key.
    pub sent_us: Option<i64>,
    /// The action and its redraw ended.
    pub end_us: Option<i64>,
    /// The first presented frame after the press: its update request.
    pub update_us: Option<i64>,
    /// Its begin.
    pub begin_us: Option<i64>,
    /// Its sync.
    pub sync_us: Option<i64>,
    /// Its scene drawn.
    pub rendered_us: Option<i64>,
    /// Its hand-over to the screen.
    pub swap_us: Option<i64>,
    /// Its end.
    pub done_us: Option<i64>,
}

/// One press on a window, from Qt's message to its first presented frame.
#[derive(Debug, Default)]
struct Press {
    window: usize,
    msg: i64,
    qt: i64,
    start: i64,
    sent: i64,
    end: i64,
    frame: Option<Frame>,
}

/// `b - a` when both steps happened.
fn gap(a: i64, b: i64) -> Option<i64> {
    (a != 0 && b != 0).then_some(b - a)
}

impl Press {
    fn times(&self) -> PressTimes {
        let at = if self.msg != 0 { self.msg } else { self.qt };
        let from = |t: i64| (t != 0).then_some(t - at);
        let f = self.frame.unwrap_or_default();
        PressTimes {
            at_us: at,
            qt_us: gap(self.msg, self.qt),
            start_us: from(self.start),
            sent_us: from(self.sent),
            end_us: from(self.end),
            update_us: from(f.update),
            begin_us: from(f.begin),
            sync_us: from(f.sync),
            rendered_us: from(f.rendered),
            swap_us: from(f.swap),
            done_us: from(f.done),
        }
    }
}

/// Joins the steps Qt and the keyboard window report into lines.
#[derive(Debug, Default)]
pub struct Log {
    press: Option<Press>,
}

impl Log {
    /// A log with no press open.
    #[must_use]
    pub const fn new() -> Self {
        Self { press: None }
    }

    /// Qt got a press on `window`: its message left the queue at `msg` (0 unseen) and reached the window at `qt`. The
    /// last press's line comes out if it had not.
    pub fn pressed(&mut self, window: usize, msg: i64, qt: i64) -> Option<PressTimes> {
        let last = self.press.take().map(|p| p.times());
        self.press = Some(Press {
            window,
            msg,
            qt,
            ..Press::default()
        });
        last
    }

    /// The keyboard window marks `mark` at `at` in the open press, once; the press's line once it is whole.
    pub fn mark(&mut self, mark: Mark, at: i64) -> Option<PressTimes> {
        let p = self.press.as_mut()?;
        let step = match mark {
            Mark::Start => &mut p.start,
            Mark::Sent => &mut p.sent,
            Mark::End => &mut p.end,
        };
        if *step == 0 {
            *step = at;
        }
        self.whole()
    }

    /// Window `window` drew frame `f`: its line, and the press's once this was its first presented frame after it
    /// and the action ended.
    pub fn framed(&mut self, window: usize, f: Frame) -> (FrameTimes, Option<PressTimes>) {
        if let Some(p) = self.press.as_mut() {
            let shows = p.window == window && f.swap != 0 && f.first() >= p.qt;
            if p.frame.is_none() && shows {
                p.frame = Some(f);
            }
        }
        let first = f.first();
        let line = FrameTimes {
            at_us: first,
            polish_us: gap(f.update, f.begin),
            wait_us: gap(f.begin, f.sync),
            render_us: gap(f.sync, f.rendered),
            present_us: gap(f.rendered, f.swap),
            busy_us: gap(first, f.done),
        };
        (line, self.whole())
    }

    /// The open press's line, which closes it, once its action ended and its frame is out.
    fn whole(&mut self) -> Option<PressTimes> {
        let whole = |p: &Press| p.end != 0 && p.frame.is_some();
        let p = self.press.take_if(|p| whole(p))?;
        Some(p.times())
    }
}

#[cfg(test)]
mod tests;

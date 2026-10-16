//! G6's scene: a window laid under the test build's keyboard, the pad on it, and the simulated user's glide, taps and
//! click. Each look reads the window's picture above the keyboard and UI Automation's percents at the glide's start.
#![cfg(windows)]

use std::cell::Cell;
use std::time::Instant;

use kx_module_api::ports::Dir;
use kx_platform_windows::screen;
use spike_core::uia::{self, Uia};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::UI::Accessibility::IUIAutomationScrollPattern;

use crate::aim;
use crate::apps::{Ctx, Opened};
use crate::g6judge::{Look, Pair};
use crate::keymap::Reader;
use crate::pin::{Pinned, pt};
use crate::product::Keyboard;
use crate::scroll::{self, along};
use crate::simin::{self, Act};
use crate::simuser::{self, guard};
use crate::win::{self, sleep_ms};

/// One speed's reads: the look over it, and its percents towards its way in ms since the first read.
pub struct Phase {
    /// The look from its start to its end.
    pub look: Look,
    /// (ms, percent) while it scrolled.
    pub reads: Vec<(u64, f64)>,
}

/// A window under the keyboard and the pad that scrolls it.
pub struct Scene<'a> {
    /// The gate's context.
    pub ctx: &'a Ctx<'a>,
    /// The window it scrolls.
    pub app: &'a Opened,
    kb: Keyboard,
    reader: Reader,
    pin: Pinned,
    band: RECT,
    sp: Option<IUIAutomationScrollPattern>,
    clicks: Cell<u64>,
}

impl<'a> Scene<'a> {
    /// Lays `app` under the keyboard, then finds the pad, the glide's start and what UI Automation scrolls there.
    pub fn new(ctx: &'a Ctx<'a>, uia: &Uia, app: &'a Opened) -> Result<Self, String> {
        let (kb, reader) = (Keyboard::find()?, Reader::new()?);
        let pin = Pinned::new(ctx, kb.hwnd, app)?;
        let band = aim::band(pin.win, pin.kb, ctx.cfg.g6.inset_px);
        let band = band.ok_or("nothing shows above the keyboard")?;
        let sp = uia.scroller(pt(pin.start));
        Ok(Self {
            ctx,
            app,
            kb,
            reader,
            pin,
            band,
            sp,
            clicks: Cell::new(0),
        })
    }

    /// The percents UI Automation reads at the glide's start; none without a scroll pattern there.
    pub fn pair(&self) -> Pair {
        self.sp.as_ref().and_then(uia::percent)
    }

    /// The scroll pattern at the glide's start, for G6's own step.
    pub fn scroller(&self) -> Option<&IUIAutomationScrollPattern> {
        self.sp.as_ref()
    }

    /// The window's picture above the keyboard.
    fn picture(&self) -> Result<Vec<u8>, String> {
        scroll::picture(self.band, self.ctx.spike.tools.shot_cap())
    }

    /// A look over `ms`.
    pub fn look(&self, ms: u64) -> Result<Look, String> {
        Ok(self.phase(Dir::Down, ms)?.look)
    }

    /// Reads the percents towards `dir` every `sample_ms` for `ms`, between two pictures.
    pub fn phase(&self, dir: Dir, ms: u64) -> Result<Phase, String> {
        let (pic, before) = (self.picture()?, self.pair());
        let began = Instant::now();
        let mut reads = Vec::new();
        while win::ms_since(began) < ms {
            sleep_ms(self.ctx.cfg.g6.sample_ms.min(ms));
            if let Some(p) = self.pair().and_then(|p| along(dir, p)) {
                reads.push((win::ms_since(began), p));
            }
        }
        let (pic2, after) = (self.picture()?, self.pair());
        let share = scroll::changed_share(&pic, &pic2);
        let look = Look {
            share,
            before,
            after,
        };
        Ok(Phase { look, reads })
    }

    /// Glides from the start onto the pad's hub as the simulated user; the glide's last point outside the keyboard.
    pub fn glide_in(&self) -> Result<POINT, String> {
        guard(self.app, &[pt(self.pin.start)])?;
        let mover = |p: POINT| simin::as_user(&[Act::Move(pt(p))]);
        self.pin.glide_in(self.ctx, mover)
    }

    /// A tap at `p` once the keyboard is the window there, then the pad's settle.
    fn tap(&self, p: POINT) -> Result<(), String> {
        let top = win::root_at(p);
        if top != self.kb.hwnd {
            return Err(format!("{} covers the pad", win::describe(top)));
        }
        simuser::press(self.ctx, pt(p))?;
        sleep_ms(self.ctx.cfg.pad.settle_ms);
        Ok(())
    }

    /// Taps the arrow towards `dir`.
    pub fn arrow(&self, dir: Dir) -> Result<(), String> {
        self.tap(self.pin.pad.arrow(dir))
    }

    /// Taps the hub, where Stop shows while it scrolls.
    pub fn stop(&self) -> Result<(), String> {
        self.tap(self.pin.pad.centre())
    }

    /// Clicks the window at the glide's start: a press outside our windows.
    pub fn click_out(&self) -> Result<(), String> {
        self.clicks.set(self.clicks.get() + 1);
        simuser::click(self.ctx, self.app, pt(self.pin.start))
    }

    /// How many times it clicked outside our windows.
    pub fn clicks(&self) -> u64 {
        self.clicks.get()
    }

    /// True when `p` is on the window and the pointer is on the keyboard now.
    pub fn came_from(&self, p: POINT) -> bool {
        let now = screen::cursor().map(|c| POINT { x: c.x, y: c.y });
        win::root_at(p) == self.app.hwnd && now.is_ok_and(|c| win::root_at(c) == self.kb.hwnd)
    }

    /// True when the pad stands still: Stop, which reads its speed while it scrolls, reads none.
    pub fn idle(&self) -> bool {
        let stop = self.ctx.cfg.pad.stop_id();
        self.reader
            .described(self.kb.hwnd, &stop)
            .is_ok_and(|d| d.is_empty())
    }
}

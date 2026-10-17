//! G6's scene: a window laid under the test build's keyboard, the pad on it, and the simulated user's glide, taps and
//! click, which it counts. Each look reads the window's picture above the keyboard and UI Automation's percents at the
//! glide's start.
#![cfg(windows)]

use std::cell::Cell;
use std::time::Instant;

use kx_module_api::ports::Dir;
use kx_platform_windows::screen;
use spike_core::uia::{self, Uia};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::UI::Accessibility::IUIAutomationScrollPattern;
use windows::core::BOOL;

use crate::apps::{AppKind, Ctx, Opened};
use crate::g6judge::{self, Look, Pair, Stops};
use crate::g6spot::{Spot, Tried};
use crate::keymap::Reader;
use crate::padread::PadRead;
use crate::pin::{Pinned, pt};
use crate::product::Keyboard;
use crate::scroll::{self, along};
use crate::simin::{self, Act};
use crate::simuser;
use crate::win::{self, sleep_ms};
use crate::{aim, g6spot, g6uia, mouseguard};

/// One speed's reads: the look over it, and its percents towards its way in ms since the first read.
pub struct Phase {
    /// The look from its start to its end.
    pub look: Look,
    /// (ms, percent) while it scrolled.
    pub reads: Vec<(u64, f64)>,
}

/// One look's half: the window's picture above the keyboard and its percents.
pub struct Snap {
    pic: Vec<u8>,
    /// The percents.
    pub pair: Pair,
}

/// The look from snap `a` to snap `b`.
pub fn between(a: &Snap, b: &Snap) -> Look {
    Look {
        share: scroll::changed_share(&a.pic, &b.pic),
        before: a.pair,
        after: b.pair,
    }
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
    made: Cell<Stops>,
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
            made: Cell::new(Stops::default()),
        })
    }

    /// The percents UI Automation reads at the glide's start; none without a scroll pattern there.
    pub fn pair(&self) -> Pair {
        self.sp.as_ref().and_then(uia::percent)
    }

    /// The share of its height the document at the glide's start shows, in percent, which shrinks as it grows; none
    /// without a scroll pattern there.
    #[allow(unsafe_code, reason = "A plain property read on a live pattern.")]
    pub fn view(&self) -> Option<f64> {
        let sp = self.sp.as_ref()?;
        // SAFETY: a plain property read on a live pattern; a failure reads as none.
        unsafe { sp.CurrentVerticalViewSize() }.ok()
    }

    /// True when the scroll pattern at the glide's start says it scrolls sideways; none without a pattern there.
    #[allow(unsafe_code, reason = "A plain property read on a live pattern.")]
    pub fn sideways(&self) -> Option<bool> {
        let sp = self.sp.as_ref()?;
        // SAFETY: a plain property read on a live pattern; a failure reads as none.
        unsafe { sp.CurrentHorizontallyScrollable() }
            .ok()
            .map(BOOL::as_bool)
    }

    /// The scroll pattern at the glide's start, for G6's own step.
    pub fn scroller(&self) -> Option<&IUIAutomationScrollPattern> {
        self.sp.as_ref()
    }

    /// The window's picture above the keyboard, and its percents.
    pub fn snap(&self) -> Result<Snap, String> {
        let pic = scroll::picture(self.band, self.ctx.spike.tools.shot_cap())?;
        Ok(Snap {
            pic,
            pair: self.pair(),
        })
    }

    /// A look over `ms`.
    pub fn look(&self, ms: u64) -> Result<Look, String> {
        Ok(self.phase(Dir::Down, ms)?.look)
    }

    /// The look that shows a stop: once the percents read the same twice `sample_ms` apart, waiting at most `still_ms`
    /// for a late provider, a look over `still_ms`.
    pub fn still_look(&self) -> Result<Look, String> {
        let g = &self.ctx.cfg.g6;
        g6judge::steady(|| self.pair(), sleep_ms, (g.sample_ms, g.still_ms));
        self.look(g.still_ms)
    }

    /// Reads the percents towards `dir` every `sample_ms` for `ms`, between two pictures.
    pub fn phase(&self, dir: Dir, ms: u64) -> Result<Phase, String> {
        let first = self.snap()?;
        let began = Instant::now();
        let mut reads = Vec::new();
        while win::ms_since(began) < ms {
            sleep_ms(self.ctx.cfg.g6.sample_ms.min(ms));
            if let Some(p) = self.pair().and_then(|p| along(dir, p)) {
                reads.push((win::ms_since(began), p));
            }
        }
        let look = between(&first, &self.snap()?);
        Ok(Phase { look, reads })
    }

    /// Glides from the start onto the pad's hub as the simulated user; the glide's last point outside the keyboard.
    pub fn glide_in(&self) -> Result<POINT, String> {
        let mover = |p: POINT| simin::as_user(&[Act::Move(pt(p))]);
        self.pin.glide_in(self.ctx, self.app, mover)
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

    /// Taps the arrow towards `dir` a third time, which stops the pad, and counts that stop.
    pub fn third(&self, dir: Dir) -> Result<(), String> {
        self.count(|m| m.taps += 1);
        self.arrow(dir)
    }

    /// Taps down for a run to the document's end, which should stop there, and counts that run.
    pub fn to_end(&self) -> Result<(), String> {
        self.count(|m| m.ends += 1);
        self.arrow(Dir::Down)
    }

    /// Taps the hub, where Stop shows while it scrolls, unless the pad stopped itself; true, and counted, when tapped.
    pub fn stop(&self) -> Result<bool, String> {
        let tapped = self.read().stop(|_| self.tap(self.pin.pad.centre()))?;
        if tapped {
            self.count(|m| m.taps += 1);
        }
        Ok(tapped)
    }

    /// Where a press outside our windows lands: the glide's start, but on Explorer only a spot UI Automation shows is
    /// no file or column (`g6spot`); else what each spot tried showed.
    pub fn spot(&self) -> Result<Spot, Vec<Tried>> {
        if self.app.kind != AppKind::Explorer {
            let at = self.pin.start;
            return Ok(Spot { at, part: None });
        }
        g6spot::find(&self.reader, self.app.hwnd, self.ctx.cfg.g6.spot_px)
    }

    /// Clicks `spot`, from `spot()`, once it still shows safe there: a press outside our windows, counted; what UI
    /// Automation showed when it did not click.
    pub fn click_out(&self, spot: Spot) -> Result<Option<Tried>, String> {
        if let Some(refused) = g6spot::check(&self.reader, self.app.hwnd, spot) {
            return Ok(Some(refused));
        }
        self.count(|m| m.clicks += 1);
        simuser::click(self.ctx, self.app, pt(spot.at))?;
        Ok(None)
    }

    /// Puts the keyboard focus on Explorer's list by UI Automation, as its column header may hold it after a press;
    /// true when the focus then lies in the list, and always elsewhere, whose keys reach the document as they are.
    pub fn focus_list(&self) -> Result<bool, String> {
        if self.app.kind != AppKind::Explorer {
            return Ok(true);
        }
        mouseguard::allowed()?;
        Ok(g6uia::focus_at(self.reader.auto(), self.pin.start))
    }

    /// What it did that stops a scroll: its clicks outside our windows, its Stop and third taps and its runs to the end.
    pub fn made(&self) -> Stops {
        self.made.get()
    }

    /// Counts one more stop as `add` says.
    fn count(&self, add: impl FnOnce(&mut Stops)) {
        let mut m = self.made.get();
        add(&mut m);
        self.made.set(m);
    }

    /// True when `p` is on the window and the pointer is on the keyboard now.
    pub fn came_from(&self, p: POINT) -> bool {
        let now = screen::cursor().map(|c| POINT { x: c.x, y: c.y });
        win::root_at(p) == self.app.hwnd && now.is_ok_and(|c| win::root_at(c) == self.kb.hwnd)
    }

    /// The test build's pad as UI Automation reads it.
    fn read(&self) -> PadRead<'_> {
        PadRead {
            reader: &self.reader,
            kb: self.kb.hwnd,
            pad: &self.ctx.cfg.pad,
        }
    }

    /// True when the pad stands still: no arrow reads a speed.
    pub fn idle(&self) -> bool {
        self.read().idle().unwrap_or(false)
    }
}

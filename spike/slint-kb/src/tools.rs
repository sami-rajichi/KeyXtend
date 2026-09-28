//! The Slint face's tools: the tools row, the selection pill, quick-fill and the snip overlay.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use slint::{
    CloseRequestResponse, ComponentHandle, Image, ModelRc, PhysicalPosition, PhysicalSize,
    Rgba8Pixel, SharedPixelBuffer, Timer, TimerMode, VecModel,
};
use spike_core::config::{SpikeConfig, ToolButton};
use spike_core::fill;
use spike_core::hold::Pt;
use spike_core::note::{self, Note};
use spike_core::selwatch::{self, PillPx, PillStep, Watch};
use spike_core::snip::Snip;
use spike_core::{inject, place, window};

use crate::{FACE, Keyboard, Line, Overlay, Pill, Tool, voice};

/// Shown when a window we showed never became visible.
const NOT_SHOWN: &str = "this window did not show: ";

/// What the tools share between callbacks.
pub(crate) struct State {
    /// The spike settings.
    pub(crate) cfg: SpikeConfig,
    /// The keyboard's status line.
    pub(crate) line: Line,
    pill: Pill,
    overlay: Overlay,
    /// Where the pill is now, or `None` while it is hidden.
    shown: Cell<Option<Pt>>,
    note: Note,
    /// The snip in progress and the window that was in front when it began.
    snip: RefCell<Option<(Snip, isize)>>,
    /// The Mic button and the caption bar.
    pub(crate) voice: voice::Parts,
}

/// Adds the tools row to `ui` and starts the pill; a watcher that cannot start is reported, not fatal.
/// Returns the timer that runs them; drop it to stop them.
pub fn start(ui: &Keyboard, cfg: &SpikeConfig, line: Line) -> Result<Timer, String> {
    let s = Rc::new(State {
        pill: pill(cfg)?,
        overlay: overlay(cfg)?,
        cfg: cfg.clone(),
        line,
        shown: Cell::new(None),
        note: Arc::default(),
        snip: RefCell::new(None),
        voice: voice::parts(cfg)?,
    });
    voice::attach(&s);
    ui.set_tools(ModelRc::new(VecModel::from(buttons(cfg))));
    wire(ui, &s);
    on_close(ui, &s);
    let t = &s.cfg.tools;
    let watch = Watch::start(t.selection_poll_ms, PillPx(t.pill_px))
        .inspect_err(|e| s.line.show(e))
        .ok();
    let timer = Timer::default();
    let every = Duration::from_millis(t.selection_poll_ms);
    let s2 = Rc::clone(&s);
    timer.start(TimerMode::Repeated, every, move || {
        tick(&s2, watch.as_ref())
    });
    Ok(timer)
}

/// The tool buttons with their labels and boxes.
fn buttons(cfg: &SpikeConfig) -> Vec<Tool> {
    place::tool_row(cfg)
        .into_iter()
        .map(|(label, p)| Tool {
            label: label.into(),
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
        })
        .collect()
}

/// The pill window, hidden until a selection shows up.
fn pill(cfg: &SpikeConfig) -> Result<Pill, String> {
    let p = Pill::new().map_err(|e| e.to_string())?;
    let t = &cfg.tools;
    p.set_caption(t.pill_title.as_str().into());
    p.set_label(t.labels.copy.as_str().into());
    p.set_pill_width(t.pill_px[0]);
    p.set_pill_height(t.pill_px[1]);
    p.set_font_px(cfg.keyboard.font_px);
    Ok(p)
}

/// The snip overlay, hidden until Snip is clicked.
fn overlay(cfg: &SpikeConfig) -> Result<Overlay, String> {
    let o = Overlay::new().map_err(|e| e.to_string())?;
    let t = &cfg.tools;
    o.set_caption(t.overlay_title.as_str().into());
    o.set_edge(t.snip_edge_px);
    o.set_hint(t.snip_hint.as_str().into());
    Ok(o)
}

/// Connects the tool buttons, the pill's Copy and the overlay's clicks.
/// The state owns the pill and the overlay, so their callbacks hold it weakly.
fn wire(ui: &Keyboard, s: &Rc<State>) {
    let st = Rc::clone(s);
    ui.on_tool(move |i| {
        let t = &st.cfg.tools;
        match usize::try_from(i).ok().and_then(ToolButton::at) {
            Some(ToolButton::FillUser) => fill_with(&st, t.test_user.clone()),
            Some(ToolButton::FillPassword) => fill_with(&st, t.test_password.clone()),
            Some(ToolButton::Snip) => begin_snip(&st),
            Some(ToolButton::Mic) => voice::click(&st),
            None => {}
        }
    });
    let st = Rc::downgrade(s);
    s.pill.on_copy(move || {
        if let Some(st) = st.upgrade()
            && let Err(e) = inject::combo(&st.cfg.tools.copy_keys)
        {
            st.line.show(&e);
        }
    });
    let st = Rc::downgrade(s);
    s.overlay.on_pick(move || {
        if let Some(st) = st.upgrade() {
            pick(&st);
        }
    });
}

/// Closing the keyboard stops voice and ends the app, even while a tool window is up.
fn on_close(ui: &Keyboard, s: &Rc<State>) {
    let st = Rc::clone(s);
    ui.window().on_close_requested(move || {
        voice::stop(&st);
        if let Err(e) = slint::quit_event_loop() {
            st.line.show(&e.to_string());
        }
        CloseRequestResponse::HideWindow
    });
}

/// Moves, shows or hides the pill, and shows the latest fill note.
fn tick(s: &Rc<State>, watch: Option<&Watch>) {
    let now = watch.and_then(Watch::spot);
    match selwatch::step(s.shown.get(), now) {
        PillStep::Show(p) => {
            move_pill(s, p);
            if done(&s.line, s.pill.show()) {
                s.shown.set(Some(p));
                raise(s, Shown::Pill, s.cfg.keyboard.guard_tries);
            }
        }
        PillStep::Move(p) => {
            move_pill(s, p);
            s.shown.set(Some(p));
        }
        PillStep::Hide if done(&s.line, s.pill.hide()) => s.shown.set(None),
        PillStep::Hide | PillStep::Stay => {}
    }
    if let Some(n) = note::take(&s.note) {
        s.line.show(&n);
    }
}

/// Puts the pill's top-left at `p`, in screen pixels.
fn move_pill(s: &State, p: Pt) {
    s.pill
        .window()
        .set_position(PhysicalPosition::new(p.x, p.y));
}

/// True when a show or hide worked; a failure goes to the status line.
pub(crate) fn done(line: &Line, r: Result<(), slint::PlatformError>) -> bool {
    r.map_err(|e| line.show(&e.to_string())).is_ok()
}

/// A window the tools show and must lift above the keyboard.
#[derive(Clone, Copy)]
pub(crate) enum Shown {
    /// The selection pill.
    Pill,
    /// The snip overlay.
    Overlay,
    /// The voice caption bar.
    Caption,
}

impl Shown {
    /// True for a window that lets clicks through to the app below.
    fn clicks_through(self) -> bool {
        matches!(self, Shown::Caption)
    }

    fn title(self, s: &State) -> &str {
        match self {
            Shown::Pill => &s.cfg.tools.pill_title,
            Shown::Overlay => &s.cfg.tools.overlay_title,
            Shown::Caption => &s.cfg.voice.caption.title,
        }
    }

    /// True while the tools still want it shown.
    fn wanted(self, s: &State) -> bool {
        match self {
            Shown::Pill => s.shown.get().is_some(),
            Shown::Overlay => s.snip.borrow().is_some(),
            Shown::Caption => s.voice.up.get(),
        }
    }

    /// Gives up on a window that never showed: it is hidden, and a snip ends so its frozen screen goes.
    fn give_up(self, s: &State) {
        s.line.show(&format!("{NOT_SHOWN}{}", self.title(s)));
        match self {
            Shown::Pill => {
                done(&s.line, s.pill.hide());
                s.shown.set(None);
            }
            Shown::Overlay => {
                end_snip(s);
                done(&s.line, s.overlay.hide());
            }
            Shown::Caption => voice::hide(s),
        }
    }
}

/// Guards our windows with the newly shown `w` above the others; none takes focus.
/// winit shows a new window a moment after `show()`, so this retries up to `tries` times while `w` is still wanted.
pub(crate) fn raise(s: &Rc<State>, w: Shown, tries: u32) {
    if !w.wanted(s) {
        return;
    }
    match window::guard_with_top(w.title(s)) {
        Ok(true) if w.clicks_through() => {
            if let Err(e) = window::click_through(w.title(s)) {
                s.line.show(&e);
            }
        }
        Ok(true) => {}
        Ok(false) if tries > 1 => {
            let st = Rc::clone(s);
            let wait = Duration::from_millis(s.cfg.keyboard.guard_delay_ms);
            Timer::single_shot(wait, move || raise(&st, w, tries - 1));
        }
        Ok(false) => w.give_up(s),
        Err(e) => s.line.show(&e),
    }
}

/// Asks Windows Hello on its own thread, then types `value` into the app in front.
fn fill_with(s: &State, value: String) {
    s.line.show(fill::start(&s.cfg, FACE, value, &s.note));
}

/// Freezes the screen and shows it full-screen over everything.
fn begin_snip(s: &Rc<State>) {
    let prev = window::foreground().0 as isize;
    let snip = match Snip::start(s.cfg.tools.shot_cap()) {
        Ok(x) => x,
        Err(e) => return s.line.show(&e),
    };
    let sh = snip.shot();
    let (w, h) = (sh.width.unsigned_abs(), sh.height.unsigned_abs());
    let buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&sh.rgba(), w, h);
    s.overlay.set_frozen(Image::from_rgba8(buf));
    s.overlay.set_picking(false);
    let win = s.overlay.window();
    win.set_position(PhysicalPosition::new(sh.left, sh.top));
    win.set_size(PhysicalSize::new(w, h));
    *s.snip.borrow_mut() = Some((snip, prev));
    if done(&s.line, s.overlay.show()) {
        raise(s, Shown::Overlay, s.cfg.keyboard.guard_tries);
    } else {
        end_snip(s);
    }
}

/// One click on the overlay; the second saves the region, closes the overlay and gives the focus back.
fn pick(s: &State) {
    let dir = s.cfg.tools.snip_dir();
    let (note, prev) = {
        let mut slot = s.snip.borrow_mut();
        let Some((snip, prev)) = slot.as_mut() else {
            return;
        };
        let Some(note) = snip.pick(&dir) else {
            return;
        };
        (note, window::from_raw(*prev))
    };
    end_snip(s);
    done(&s.line, s.overlay.hide());
    window::bring_back(prev, 0);
    s.line.show(&note);
}

/// Drops the snip and its frozen screen, which is private.
fn end_snip(s: &State) {
    *s.snip.borrow_mut() = None;
    s.overlay.set_frozen(Image::default());
}

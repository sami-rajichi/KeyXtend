//! The Slint face's Mic button and caption bar; the worker's events come back on the UI thread.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;

use slint::{ComponentHandle, PhysicalPosition, PhysicalSize, Timer, TimerMode};
use spike_core::config::SpikeConfig;
use spike_core::voice::{self, Caption, Event, Session, Update};

use crate::CaptionBar;
use crate::tools::{Shown, State, done, raise};

thread_local! {
    /// The tools, for events that arrive from the worker's reader thread.
    static TOOLS: RefCell<Weak<State>> = const { RefCell::new(Weak::new()) };
}

/// The voice parts of the tools state.
pub(crate) struct Parts {
    bar: CaptionBar,
    session: RefCell<Session>,
    /// The bar is shown.
    pub(crate) up: Cell<bool>,
    hide: Timer,
}

/// The caption bar, and the worker started now so the model is loaded before the first click.
pub(crate) fn parts(cfg: &SpikeConfig) -> Result<Parts, String> {
    let bar = CaptionBar::new().map_err(|e| e.to_string())?;
    bar.set_caption(cfg.voice.caption.title.as_str().into());
    bar.set_font_px(cfg.keyboard.font_px);
    bar.set_pad(cfg.keyboard.gap_px);
    let back = |e: Event| drop(slint::invoke_from_event_loop(move || on_event(e)));
    let args: Vec<String> = std::env::args().skip(1).collect();
    Ok(Parts {
        bar,
        session: RefCell::new(Session::start(cfg, &args, back)),
        up: Cell::new(false),
        hide: Timer::default(),
    })
}

/// Lets the worker's events reach `s`.
pub(crate) fn attach(s: &Rc<State>) {
    TOOLS.with(|t| *t.borrow_mut() = Rc::downgrade(s));
}

/// The Mic button: starts or stops recording in the language of the app in front.
pub(crate) fn click(s: &Rc<State>) {
    let u = s.voice.session.borrow_mut().click();
    apply(s, u);
}

/// Ends the worker, which closes its local server with it.
pub(crate) fn stop(s: &State) {
    s.voice.session.borrow_mut().stop();
}

/// Hides the caption bar.
pub(crate) fn hide(s: &State) {
    if done(&s.line, s.voice.bar.hide()) {
        s.voice.up.set(false);
    }
}

fn on_event(e: Event) {
    let Some(s) = TOOLS.with(|t| t.borrow().upgrade()) else {
        return;
    };
    let u = s.voice.session.borrow_mut().on_event(&e);
    apply(&s, u);
}

/// Shows a typing problem on the status line and the caption in the bar.
fn apply(s: &Rc<State>, u: Update) {
    if let Some(n) = u.note {
        s.line.show(&n);
    }
    if let Some(c) = u.caption {
        show(s, &c);
    }
}

/// Shows `c` at the bottom centre of the screen in front, above every window, without focus.
fn show(s: &Rc<State>, c: &Caption) {
    let v = &s.voice;
    v.bar.set_text(c.text.as_str().into());
    match voice::bar_place(s.cfg.voice.caption.px) {
        Ok((p, (w, h))) => {
            let win = v.bar.window();
            win.set_size(PhysicalSize::new(w.unsigned_abs(), h.unsigned_abs()));
            win.set_position(PhysicalPosition::new(p.x, p.y));
        }
        Err(e) => s.line.show(&e),
    }
    if !v.up.get() && done(&s.line, v.bar.show()) {
        v.up.set(true);
        raise(s, Shown::Caption, s.cfg.keyboard.guard_tries);
    }
    match c.hide_ms {
        Some(ms) => {
            let st = Rc::downgrade(s);
            let later = move || {
                if let Some(s) = st.upgrade() {
                    hide(&s);
                }
            };
            v.hide
                .start(TimerMode::SingleShot, Duration::from_millis(ms), later);
        }
        None => v.hide.stop(),
    }
}

//! A fresh target-window box typed in through the product, as G27, G21's product half and G8's typing churn need: it is
//! started, focused and typed in, then its first layout comes back, held input is let go and the window closes.
#![cfg(windows)]

use spike_core::geom;

use crate::apps::{self, AppKind, Ctx, Opened};
use crate::g27type::Typist;
use crate::{launch, probe, simuser, win};

/// What one fresh box gave: the drive's result, whether the app's first layout came back, and what was left open.
pub struct Boxed<T> {
    /// What the drive gave.
    pub got: Result<T, String>,
    /// The app's first layout came back.
    pub back: bool,
    /// What clean-up let go or left open.
    pub left: Vec<String>,
}

impl<T> Boxed<T> {
    /// A box that never opened, failing with `e`.
    pub fn failed(e: String) -> Self {
        Self {
            got: Err(e),
            back: true,
            left: Vec::new(),
        }
    }
}

/// Clicks the middle of target-window's box, as G21 does, and checks the box took the keyboard.
fn focus_target(ctx: &Ctx, app: &Opened) -> Result<(), String> {
    simuser::click(ctx, app, geom::centre(&win::rect(app.hwnd)?))?;
    let target = ctx.cfg.app(AppKind::Target.name())?;
    let edit = target.focus_class.clone().unwrap_or_default();
    if win::class(win::focus(app.hwnd)) == edit {
        Ok(())
    } else {
        Err(format!("the {edit} box did not take the focus"))
    }
}

/// A typist on `app`, `drive` run with it, then the app's first layout put back: what `drive` gave, and whether that
/// layout came back.
pub fn typed_in<T>(
    ctx: &Ctx,
    app: &Opened,
    drive: impl FnOnce(&Typist) -> Result<T, String>,
) -> (Result<T, String>, bool) {
    match Typist::new(ctx, app) {
        Ok(t) => (drive(&t), t.put_back()),
        Err(e) => (Err(e), true),
    }
}

/// A fresh target-window started with `args`, in front with its box focused, and `drive` run on it, as the module says.
pub fn in_target<T>(
    ctx: &Ctx,
    args: &[String],
    drive: impl FnOnce(&Typist) -> Result<T, String>,
) -> Boxed<T> {
    let started = launch::start_target_with(ctx, args).and_then(|app| launch::to_front(ctx, app));
    let app = match started {
        Ok(app) => app,
        Err(e) => return Boxed::failed(e),
    };
    let (got, back) = match focus_target(ctx, &app) {
        Ok(()) => typed_in(ctx, &app, drive),
        Err(e) => (Err(e), true),
    };
    let mut left = probe::released();
    left.extend(apps::clean_up(ctx, app));
    Boxed { got, back, left }
}

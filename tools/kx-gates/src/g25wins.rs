//! G25's system windows: the clipboard panel from the product's Win and V keys, the task switcher and the power dialog.
//! Each is watched for (`g25watch`), and Esc goes out only while it, or our own Notepad tab where allowed, is in front.
#![cfg(windows)]

use kx_mod_keyboard::{Latch, Role};
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;

use crate::apps::{Ctx, Opened};
use crate::g25watch::{self, Watch};
use crate::keymap::Reader;
use crate::modekey::ModeKey;
use crate::ownkeys::{self, Desk, Real};
use crate::product::Keyboard;
use crate::simuser::keys_ours;
use crate::{board, g10keys, g25, win};

/// The result's note when a window of the power dialog's class stays open, so the owner closes it.
const LEFT_OPEN: &str = "a window of the power dialog's class is still open: the owner closes it";
/// The result's note when no watched window showed in time, as one of a class `[g25]` lacks, or a late one, may show.
const UNSEEN: &str =
    "no watched window showed in time: one that shows later is left for the owner to close";

/// True when Esc may go out: the window in `front` is one of `shown`, or `app` when it may take Esc.
pub fn esc_ok<W: PartialEq>(front: &W, shown: &[W], app: Option<&W>) -> bool {
    shown.contains(front) || app == Some(front)
}

/// Which window in front may take G25's Esc, besides a watched one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscTo {
    /// A watched window only: the power dialog.
    Watched,
    /// A watched window or Notepad: the clipboard panel, which never takes the front.
    WatchedOrApp,
    /// A watched window, and Notepad only once the wait for one in front ran out: the switcher, which takes it late.
    WatchedFirst,
}

impl EscTo {
    /// Whether Notepad may take Esc, given whether the wait for a watched window in front `ran_out`.
    pub fn app_takes(self, ran_out: bool) -> bool {
        match self {
            Self::Watched => false,
            Self::WatchedOrApp => true,
            Self::WatchedFirst => ran_out,
        }
    }

    /// Whether Notepad must come back to the front by itself once the watched windows closed: after the panel only.
    pub fn app_back(self) -> bool {
        self == Self::WatchedOrApp
    }

    /// Whether G25 brings Notepad back itself once the watched windows closed: after the switcher, whose hidden window
    /// Windows leaves in front.
    pub fn fetches_app(self) -> bool {
        self == Self::WatchedFirst
    }
}

/// A window's class and program file name, for the result.
fn about(w: HWND) -> Value {
    json!({ "class": win::class(w), "program": win::program_name(w) })
}

/// Brings Notepad back with G25's front step, recorded as `back` and any error as `back_error`; true once it is back.
fn fetch(ctx: &Ctx, app: &Opened, result: &mut Value) -> bool {
    let back = g25::front(ctx, app);
    result["back"] = json!(back.is_ok());
    if let Err(e) = &back {
        result["back_error"] = json!(e);
    }
    back.is_ok()
}

/// Sends `esc` on `desk` while a window of `shown` is in front, or `app` when it `takes` Esc and shows our tab: true
/// when it went out, an error when Notepad shows another tab.
pub(crate) fn send_esc(
    desk: &dyn Desk,
    app: &Opened,
    shown: &[HWND],
    takes: bool,
    esc: &[u16],
) -> Result<bool, String> {
    let front = desk.front();
    if shown.contains(&front) {
        desk.combo(esc)?;
        return Ok(true);
    }
    if takes && front == app.hwnd {
        ownkeys::send(desk, app, esc)?;
        return Ok(true);
    }
    Ok(false)
}

/// Waits for `watch`'s windows, sends Esc only while one of them, or Notepad as `to` allows, is in front, then waits
/// for them to close; Notepad must then come back by itself, or G25 brings it back, as `to` says.
pub fn closes(ctx: &Ctx, app: &Opened, watch: &Watch, to: EscTo) -> Result<Value, String> {
    let opened = g25::until(ctx, || !watch.shown().is_empty());
    let found: Vec<Value> = watch.shown().into_iter().map(about).collect();
    let first = to == EscTo::WatchedFirst;
    let own = opened && first && g25::until(ctx, || esc_ok(&foreground(), &watch.shown(), None));
    let takes = to.app_takes(!own).then_some(app.hwnd);
    let safe = opened
        && g25::until(ctx, || {
            esc_ok(&foreground(), &watch.shown(), takes.as_ref())
        });
    let at_esc = about(foreground());
    let esc = &ctx.cfg.g25.esc;
    let sent = safe && send_esc(&Real, app, &watch.shown(), takes.is_some(), esc)?;
    let back = || !to.app_back() || foreground() == app.hwnd;
    let closed = sent && g25::until(ctx, || watch.shown().is_empty() && back());
    let mut result = json!({
        "opened": opened, "windows": found, "esc_sent": sent, "front_at_esc": at_esc, "closed": closed,
        "ok": opened && closed,
    });
    if first {
        result["came_to_front"] = json!(own);
    }
    if closed && to.fetches_app() {
        let fetched = fetch(ctx, app, &mut result);
        result["ok"] = json!(opened && closed && fetched);
    }
    if !opened {
        result["note"] = json!(UNSEEN);
    }
    Ok(result)
}

/// `none_open` on the windows shown now, before G25 starts, before each Alt tap that brings Notepad back and before
/// its Power click.
pub fn no_dialog(ctx: &Ctx) -> Result<(), String> {
    g25watch::none_open(
        &g25watch::seen(),
        std::slice::from_ref(&ctx.cfg.g25.power_class),
    )
}

/// True when both latches `after` the undo, Win's and its spare Shift's, read as let go.
pub fn both_let_go(after: (Option<bool>, Option<bool>)) -> bool {
    after == (Some(false), Some(false))
}

/// Lets the Win latch go without a lone press, as G10 does: Shift latches over it first. Win's and Shift's states
/// after, each if read.
pub(crate) fn let_go(win_id: &str) -> Result<(Option<bool>, Option<bool>), String> {
    let (spare, reader, kb) = (
        board::latch_id(Latch::Shift)?,
        Reader::new()?,
        Keyboard::find()?,
    );
    let read = |id: &str| reader.toggled(kb.hwnd, id).ok().flatten();
    let press = |id: &str| reader.restore(kb.hwnd, id).is_ok();
    Ok(g10keys::undo((win_id, &spare), read, press))
}

/// The product's Win and V keys show the clipboard panel, which Esc closes; Win and its spare Shift are let go after,
/// whatever happened, and the part passes only once both read as let go.
pub fn clip_window(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let g = &ctx.cfg.g25;
    let (win_id, v) = (board::latch_id(Latch::Win)?, board::char_key(g.v_scan)?);
    let win_key = ModeKey::by_id(win_id.clone())?;
    let latched = keys_ours(app).and_then(|()| win_key.set(true, g25::wait(ctx)));
    let watch = Watch::start(&g.panel_classes, None);
    let typed = latched
        .and_then(|()| keys_ours(app))
        .and_then(|()| win_key.keyboard.click(&v));
    let shown = typed.and_then(|()| closes(ctx, app, &watch, EscTo::WatchedOrApp));
    let after = let_go(&win_id).unwrap_or((None, None));
    let mut result = shown?;
    result["win_let_go"] = json!(after.0 == Some(false));
    result["shift_let_go"] = json!(after.1 == Some(false));
    result["ok"] = json!(result["ok"] == true && both_let_go(after));
    Ok(result)
}

/// True when G25 may tap Alt to bring Notepad back after Power: no window of the dialog's class shows, neither the
/// shell's `dialogs` nor `any` other, as the tap would reach it.
pub fn alt_ok<W>(dialogs: &[W], any: &[W]) -> bool {
    dialogs.is_empty() && any.is_empty()
}

/// The Power part with the shortcuts layer hidden, as it covers the Power key; the layer is then put back as found.
pub fn power(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let (layer, wait) = (ModeKey::of(Role::Layer)?, g25::wait(ctx));
    let was = layer.on()?;
    let mut result = g25::part(layer.set(false, wait).and_then(|()| dialog(ctx, app)));
    let back = layer.put_back(was, wait);
    result["layer_as_found"] = json!(back.is_ok());
    result["ok"] = json!(result["ok"] == true && back.is_ok());
    if let Err(e) = back {
        result["layer_error"] = json!(e);
    }
    Ok(result)
}

/// The product's Power key, never clicked while a window of the dialog's class shows, opens the power dialog, which
/// Esc cancels only while it is in front; nothing confirms it, and one that stays open is left for the owner. Once no
/// window of its class shows, Notepad comes back.
fn dialog(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    no_dialog(ctx)?;
    let g = &ctx.cfg.g25;
    let (kb, id) = (Keyboard::find()?, board::id_of(Role::Power)?);
    let class = std::slice::from_ref(&g.power_class);
    // The class alone too, as a dialog whose program does not read is never picked as the shell's.
    let any = Watch::start(class, None);
    let watch = Watch::start(class, Some(&g.power_program));
    keys_ours(app)?;
    kb.click(&id)?;
    let mut result = g25::part(closes(ctx, app, &watch, EscTo::Watched));
    if alt_ok(&watch.shown(), &any.shown()) {
        fetch(ctx, app, &mut result);
    } else {
        result["left_open"] = json!(LEFT_OPEN);
    }
    Ok(result)
}

#[cfg(test)]
mod tests;

//! G25's system windows: the clipboard panel from the product's Win and V keys, the task switcher and the power dialog.
//! Each is watched for (`g25watch`), and Esc goes out only while it, or Notepad where allowed, is in front.
#![cfg(windows)]

use kx_mod_keyboard::{Latch, Role};
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;

use crate::apps::{Ctx, Opened};
use crate::g25watch::{self, Facts, Watch, picked};
use crate::keymap::Reader;
use crate::modekey::ModeKey;
use crate::product::Keyboard;
use crate::simuser::keys_ours;
use crate::{board, g10keys, g25, keys, probe, win};

/// Why G25 refuses to start, or to click Power, while a window of the power dialog's class already shows.
const ALREADY_OPEN: &str = "a power dialog is already open: the owner closes it first";
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

    /// Whether Notepad must be back in front once the watched windows closed.
    pub fn app_back(self) -> bool {
        self != Self::Watched
    }
}

/// A window's class and program file name, for the result.
fn about(w: HWND) -> Value {
    json!({ "class": win::class(w), "program": win::program_name(w) })
}

/// Waits for `watch`'s windows, sends Esc only while one of them, or Notepad as `to` allows, is in front, then waits
/// for them to close, with Notepad back in front when `to` needs it.
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
    if safe {
        keys::combo(&ctx.cfg.g25.esc)?;
    }
    let back = || !to.app_back() || foreground() == app.hwnd;
    let closed = safe && g25::until(ctx, || watch.shown().is_empty() && back());
    let mut result = json!({
        "opened": opened, "windows": found, "esc_sent": safe, "front_at_esc": at_esc, "closed": closed,
        "ok": opened && closed,
    });
    if first {
        result["came_to_front"] = json!(own);
    }
    if !opened {
        result["note"] = json!(UNSEEN);
    }
    Ok(result)
}

/// Ok unless one of the windows `seen` has one of the dialog's `classes`, whatever its program, old or new: G25's Alt
/// taps would reach it, so G25 refuses with why.
pub fn none_open(seen: &[Facts], classes: &[String]) -> Result<(), String> {
    if seen.iter().any(|f| picked(f, classes, None)) {
        return Err(ALREADY_OPEN.to_owned());
    }
    Ok(())
}

/// `none_open` on the windows shown now, before G25 starts and before its Power click.
pub fn no_dialog(ctx: &Ctx) -> Result<(), String> {
    none_open(
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
fn let_go(win_id: &str) -> Result<(Option<bool>, Option<bool>), String> {
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
        result["back"] = json!(probe::front(ctx, app).is_ok());
    } else {
        result["left_open"] = json!(LEFT_OPEN);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(class: &str, program: Option<&str>) -> Facts {
        Facts {
            class: class.into(),
            program: program.map(Into::into),
        }
    }

    #[test]
    fn g25_refuses_to_start_while_any_window_of_the_dialogs_class_shows() {
        let class = ["#32770".to_owned()];
        assert_eq!(none_open(&[], &class), Ok(()));
        let notepad = facts("Notepad", Some("notepad.exe"));
        assert_eq!(none_open(std::slice::from_ref(&notepad), &class), Ok(()));
        for open in [
            facts("#32770", Some("explorer.exe")),
            facts("#32770", None),
            facts("#32770", Some("notepad.exe")),
        ] {
            let why = none_open(&[notepad.clone(), open.clone()], &class);
            assert_eq!(why, Err(ALREADY_OPEN.to_owned()), "{open:?}");
        }
    }

    #[test]
    fn the_panel_passes_only_once_win_and_its_spare_shift_both_read_let_go() {
        assert!(both_let_go((Some(false), Some(false))));
        assert!(
            !both_let_go((Some(false), Some(true))),
            "Shift still latched"
        );
        assert!(!both_let_go((Some(true), Some(false))), "Win still latched");
        assert!(!both_let_go((Some(false), None)), "Shift did not read");
        assert!(!both_let_go((None, Some(false))), "Win did not read");
    }

    #[test]
    fn alt_goes_out_after_power_only_while_no_window_of_the_dialogs_class_shows() {
        let (dialog, other) = (1, 2);
        assert!(alt_ok::<i32>(&[], &[]), "all closed");
        assert!(!alt_ok(&[dialog], &[dialog]), "the dialog still shows");
        assert!(
            !alt_ok(&[], &[other]),
            "one whose program did not read may be the dialog"
        );
    }

    #[test]
    fn notepad_takes_esc_for_the_panel_and_for_the_switcher_only_once_its_wait_ran_out() {
        assert!(
            !EscTo::Watched.app_takes(false) && !EscTo::Watched.app_takes(true),
            "the power dialog"
        );
        assert!(
            EscTo::WatchedOrApp.app_takes(false),
            "the panel never takes the front"
        );
        assert!(
            !EscTo::WatchedFirst.app_takes(false),
            "the switcher came to the front"
        );
        assert!(
            EscTo::WatchedFirst.app_takes(true),
            "it never did: today's rule"
        );
        assert!(
            !EscTo::Watched.app_back()
                && EscTo::WatchedFirst.app_back()
                && EscTo::WatchedOrApp.app_back()
        );
    }

    #[test]
    fn esc_goes_out_only_while_a_watched_window_or_an_allowed_app_is_in_front() {
        let (app, dialog, other) = (1, 2, 3);
        assert!(esc_ok(&dialog, &[dialog], None));
        assert!(
            !esc_ok(&app, &[dialog], None),
            "only the power dialog itself takes Esc"
        );
        assert!(
            esc_ok(&app, &[dialog], Some(&app)),
            "Notepad, under the switcher or the panel"
        );
        assert!(
            !esc_ok(&other, &[dialog], Some(&app)),
            "another app never gets Esc"
        );
        assert!(!esc_ok(&other, &[], None), "nothing shown");
    }
}

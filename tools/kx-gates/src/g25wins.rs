//! G25's system windows: the clipboard panel from the product's Win and V keys, the task switcher and the power dialog.
//! Each is picked from plain window facts, and Esc goes out only while it, or Notepad where allowed, is in front.
#![cfg(windows)]

use std::collections::HashSet;

use kx_mod_keyboard::{Latch, Role};
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;

use crate::apps::{self, Ctx, Opened};
use crate::keymap::Reader;
use crate::modekey::ModeKey;
use crate::product::Keyboard;
use crate::simuser::keys_ours;
use crate::{board, g10keys, g25, keys, probe, win};

/// The result's note when the power dialog stays open, so the owner cancels it.
const LEFT_OPEN: &str = "the power dialog is still open: the owner cancels it";
/// The result's note when no watched window showed in time, as one of a class `[g25]` lacks, or a late one, may show.
const UNSEEN: &str =
    "no watched window showed in time: one that shows later is left for the owner to close";

/// What G25 reads of a window: its class and program, never its title, which may be private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// Its class.
    pub class: String,
    /// Its program's file name, when it can be read.
    pub program: Option<String>,
}

impl Facts {
    /// The facts of window `w`.
    fn of(w: HWND) -> Self {
        Self {
            class: win::class(w),
            program: win::program_name(w),
        }
    }
}

/// True when a window with `facts` has one of `classes` and, when `program` names one, belongs to it.
pub fn picked(facts: &Facts, classes: &[String], program: Option<&str>) -> bool {
    let from = |p: &str| {
        facts
            .program
            .as_deref()
            .is_some_and(|f| apps::same_program(f, p))
    };
    classes.contains(&facts.class) && program.is_none_or(from)
}

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

/// The windows of one kind a person sees that they did not see when the watch started.
pub struct Watch<'a> {
    before: HashSet<isize>,
    classes: &'a [String],
    program: Option<&'a str>,
}

impl<'a> Watch<'a> {
    /// Starts watching for windows of `classes`, from `program` when it names one.
    pub fn start(classes: &'a [String], program: Option<&'a str>) -> Self {
        let before = win::seen_windows().into_iter().map(win::key).collect();
        Self {
            before,
            classes,
            program,
        }
    }

    /// The watched windows shown now; a window that was there but cloaked, as the panel's are, counts once shown.
    pub fn shown(&self) -> Vec<HWND> {
        let new = |w: &HWND| !self.before.contains(&win::key(*w));
        let kind = |w: &HWND| picked(&Facts::of(*w), self.classes, self.program);
        win::seen_windows()
            .into_iter()
            .filter(new)
            .filter(kind)
            .collect()
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

/// Lets the Win latch go without a lone press, as G10 does: Shift latches over it first. Win's state after, if read.
fn let_go(win_id: &str) -> Result<Option<bool>, String> {
    let (spare, reader, kb) = (
        board::latch_id(Latch::Shift)?,
        Reader::new()?,
        Keyboard::find()?,
    );
    let read = |id: &str| reader.toggled(kb.hwnd, id).ok().flatten();
    let press = |id: &str| reader.restore(kb.hwnd, id).is_ok();
    Ok(g10keys::undo((win_id, &spare), read, press).0)
}

/// The product's Win and V keys show the clipboard panel, which Esc closes; Win is let go after, whatever happened.
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
    let let_go = let_go(&win_id).is_ok_and(|after| after == Some(false));
    let mut result = shown?;
    result["win_let_go"] = json!(let_go);
    result["ok"] = json!(result["ok"] == true && let_go);
    Ok(result)
}

/// The product's Power key opens the power dialog, which Esc cancels only while it is in front; nothing confirms it,
/// and a dialog that stays open is left for the owner. Once none shows, Notepad comes back, so clean-up closes it.
pub fn power(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let g = &ctx.cfg.g25;
    let (kb, id) = (Keyboard::find()?, board::id_of(Role::Power)?);
    let watch = Watch::start(std::slice::from_ref(&g.power_class), Some(&g.power_program));
    keys_ours(app)?;
    kb.click(&id)?;
    let mut result = g25::part(closes(ctx, app, &watch, EscTo::Watched));
    if watch.shown().is_empty() {
        // Best effort, and never while the dialog shows, as its Alt tap would reach the dialog.
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

    fn list(classes: &[&str]) -> Vec<String> {
        classes.iter().map(|&c| c.to_owned()).collect()
    }

    #[test]
    fn the_switcher_is_picked_by_its_class_alone() {
        let classes = list(&["XamlExplorerHostIslandWindow", "MultitaskingViewFrame"]);
        let w11 = facts("XamlExplorerHostIslandWindow", Some("explorer.exe"));
        assert!(picked(&w11, &classes, None));
        assert!(
            picked(&facts("MultitaskingViewFrame", None), &classes, None),
            "an unread program does not matter"
        );
        assert!(!picked(
            &facts("Notepad", Some("notepad.exe")),
            &classes,
            None
        ));
        assert!(!picked(&w11, &[], None), "no class, no window");
    }

    #[test]
    fn the_power_dialog_needs_its_class_and_the_shells_program() {
        let (class, shell) = (list(&["#32770"]), Some("explorer.exe"));
        assert!(
            picked(&facts("#32770", Some("EXPLORER.EXE")), &class, shell),
            "in any case"
        );
        assert!(
            !picked(&facts("#32770", Some("notepad.exe")), &class, shell),
            "Notepad's own dialog"
        );
        assert!(
            !picked(&facts("#32770", None), &class, shell),
            "an unread program is never the dialog"
        );
        assert!(!picked(
            &facts("Shell_TrayWnd", Some("explorer.exe")),
            &class,
            shell
        ));
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

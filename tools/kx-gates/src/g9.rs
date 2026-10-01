//! G9 window flags survive (roadmap G9, ADR-0013): the guard's flags stay on the keyboard and every guarded window after
//! an outside topmost drop and style strip, a hide and show, and a resize, while target-window keeps the focus.
#![cfg(windows)]

use std::collections::HashMap;

use kx_platform_windows::window::{foreground, kept_ex};
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

use crate::apps::{self, Ctx};
use crate::g9win::{self, ex_of};
use crate::keymap::Reader;
use crate::launch;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};

/// Each flag by name: the guard's three, then the two that make a window click-through.
const FLAGS: [(&str, u32); 5] = [
    ("no_activate", WS_EX_NOACTIVATE.0),
    ("topmost", WS_EX_TOPMOST.0),
    ("tool_window", WS_EX_TOOLWINDOW.0),
    ("layered", WS_EX_LAYERED.0),
    ("transparent", WS_EX_TRANSPARENT.0),
];
/// The steps that hide and show the keyboard, and grow and shrink it; a run that breaks off between them undoes them.
const HIDE: &str = "hide";
const SHOW: &str = "show";
const GROW: &str = "grow";
const SHRINK: &str = "shrink";

/// The flags a window must keep, by the guard's own rule: its three, and click-through when it had both at first.
fn wants(first_ex: u32) -> u32 {
    kept_ex(first_ex, 0)
}

/// The names of the flags in `want` that `ex` lacks.
fn missing(ex: u32, want: u32) -> Vec<&'static str> {
    let lacks = |&&(_, bit): &&(&str, u32)| want & bit != 0 && ex & bit == 0;
    FLAGS.iter().filter(lacks).map(|(name, _)| *name).collect()
}

/// One look at one product window after a step.
#[derive(Debug, Clone, Serialize)]
struct Seen {
    step: &'static str,
    window: String,
    ex: String,
    missing: Vec<&'static str>,
    front_kept: bool,
}

/// True when every step saw at least one window, none lacked a flag, and the target stayed in front.
fn passed(seen: &[Seen], steps: &[&str]) -> bool {
    let looked = |s: &&str| seen.iter().any(|v| v.step == *s);
    steps.iter().all(looked) && seen.iter().all(|v| v.missing.is_empty() && v.front_kept)
}

/// What G9 knows across steps: each window's first style, and every look.
struct Watch {
    first: HashMap<isize, u32>,
    seen: Vec<Seen>,
    keyboard: HWND,
    target: HWND,
}

impl Watch {
    /// The shown product windows G9 checks: the keyboard, and every window that was no-activate when first seen.
    /// A window that may take the focus, such as Settings, is left alone.
    fn ours(&mut self) -> Vec<HWND> {
        let first = &mut self.first;
        let keyboard = self.keyboard;
        let guarded = |w: &HWND| {
            let was = *first.entry(win::key(*w)).or_insert_with(|| ex_of(*w));
            *w == keyboard || was & WS_EX_NOACTIVATE.0 != 0
        };
        product::shown().into_iter().filter(guarded).collect()
    }

    /// Looks at every window G9 checks after `step`.
    fn look(&mut self, step: &'static str) {
        let front_kept = foreground() == self.target;
        for w in self.ours() {
            let ex = ex_of(w);
            self.seen.push(Seen {
                step,
                window: format!("{:#x} {}", win::key(w), win::class(w)),
                ex: format!("{ex:#010x}"),
                missing: missing(ex, wants(self.first[&win::key(w)])),
                front_kept,
            });
        }
    }

    /// True when `step` was looked at.
    fn did(&self, step: &str) -> bool {
        self.seen.iter().any(|v| v.step == step)
    }
}

/// What a step does before its look.
enum Act<'a> {
    /// Nothing: the first look.
    Nothing,
    /// An outside program turns topmost off.
    DropTopmost,
    /// An outside program clears the guard's flags.
    StripFlags,
    /// A top-bar button or the bubble is pressed, by UI Automation id.
    Press(&'a str),
}

/// The steps, in order, by name; each is followed by a look.
fn plan<'a>(ctx: &'a Ctx) -> [(&'static str, Act<'a>); 7] {
    let (g9, g8) = (&ctx.cfg.g9, &ctx.cfg.g8);
    [
        ("start", Act::Nothing),
        ("topmost", Act::DropTopmost),
        ("styles", Act::StripFlags),
        (HIDE, Act::Press(&g9.put_away)),
        (SHOW, Act::Press(&g9.bring_back)),
        (GROW, Act::Press(&g8.grow)),
        (SHRINK, Act::Press(&g8.shrink)),
    ]
}

/// Runs every step on the shown product with target-window in front; a refused write breaks the run off.
fn steps(ctx: &Ctx, w: &mut Watch) -> Result<(), String> {
    let reader = Reader::new()?;
    for (step, act) in plan(ctx) {
        match act {
            Act::Nothing => {}
            Act::DropTopmost => g9win::drop_topmost(&w.ours())?,
            Act::StripFlags => g9win::strip_flags(&w.ours())?,
            Act::Press(id) => product::invoke(&reader, id)?,
        }
        sleep_ms(ctx.cfg.g9.settle_ms);
        w.look(step);
    }
    Ok(())
}

/// After a run that broke off, shows the keyboard again and takes back a grow; best effort.
fn undo(ctx: &Ctx, w: &Watch) {
    let Ok(reader) = Reader::new() else {
        return;
    };
    if w.did(HIDE) && !w.did(SHOW) {
        let _ = product::invoke(&reader, &ctx.cfg.g9.bring_back);
    }
    if w.did(GROW) && !w.did(SHRINK) {
        let _ = product::invoke(&reader, &ctx.cfg.g8.shrink);
    }
}

/// Runs G9 on the product, called `name`; it writes the product windows' styles, so only with the owner's yes.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let keyboard = Keyboard::find()?.hwnd;
    let target = launch::to_front(ctx, launch::start_target(ctx)?)?;
    let (first, seen) = (HashMap::new(), Vec::new());
    let mut w = Watch {
        first,
        seen,
        keyboard,
        target: target.hwnd,
    };
    let ran = steps(ctx, &mut w);
    if ran.is_err() {
        undo(ctx, &w);
    }
    let repaired = g9win::repair(&w.first);
    let left_open = apps::clean_up(ctx, target);
    let names: Vec<&str> = plan(ctx).iter().map(|(step, _)| *step).collect();
    let pass = ran.is_ok() && passed(&w.seen, &names);
    let mut v = json!({
        "gate": "G9", "keyboard": name, "seen": w.seen, "put_back": repaired, "left_open": left_open, "pass": pass,
    });
    // Only a run that broke off carries `error`.
    if let Err(e) = ran {
        v["error"] = json!(e);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use kx_platform_windows::window::{KEPT, THROUGH};

    use super::*;

    /// The keyboard window's style, recorded from the debug app on 2026-10-01: no-activate, layered, tool, topmost.
    const KEYBOARD_EX: u32 = 0x0808_0088;

    fn seen(step: &'static str, ex: u32, first: u32, front_kept: bool) -> Seen {
        Seen {
            step,
            window: "kb".into(),
            ex: format!("{ex:#010x}"),
            missing: missing(ex, wants(first)),
            front_kept,
        }
    }

    #[test]
    fn the_names_cover_the_guards_flags_and_click_through() {
        let kept = FLAGS[..3].iter().fold(0, |all, (_, bit)| all | bit);
        let through = FLAGS[3..].iter().fold(0, |all, (_, bit)| all | bit);
        assert_eq!((kept, through), (KEPT, THROUGH));
    }

    #[test]
    fn layered_alone_is_not_wanted_back_after_a_strip_as_the_guard_keeps_only_both() {
        let stripped = kept_ex(KEYBOARD_EX, KEYBOARD_EX & !(KEPT | THROUGH));
        assert!(
            missing(stripped, wants(KEYBOARD_EX)).is_empty(),
            "{stripped:#x}"
        );
        let glow = KEYBOARD_EX | WS_EX_TRANSPARENT.0;
        let kept = kept_ex(glow, glow & !(KEPT | THROUGH));
        assert!(
            missing(kept, wants(glow)).is_empty(),
            "both click-through flags stay"
        );
    }

    #[test]
    fn the_recorded_keyboard_keeps_every_flag_and_a_lost_one_is_named() {
        assert!(missing(KEYBOARD_EX, wants(KEYBOARD_EX)).is_empty());
        assert_eq!(
            missing(KEYBOARD_EX & !WS_EX_TOPMOST.0, wants(KEYBOARD_EX)),
            ["topmost"]
        );
        let glow = KEYBOARD_EX | WS_EX_TRANSPARENT.0;
        assert_eq!(
            missing(KEYBOARD_EX, wants(glow)),
            ["transparent"],
            "a click-through window stays so"
        );
        assert!(
            missing(glow, wants(KEYBOARD_EX)).is_empty(),
            "extra flags are fine"
        );
    }

    #[test]
    fn g9_passes_only_when_every_step_looked_kept_every_flag_and_the_focus() {
        let steps = ["start", "topmost"];
        let good = [
            seen("start", KEYBOARD_EX, KEYBOARD_EX, true),
            seen("topmost", KEYBOARD_EX, KEYBOARD_EX, true),
        ];
        assert!(passed(&good, &steps));
        assert!(!passed(&good[..1], &steps), "a step saw no window");
        let lost = [
            good[0].clone(),
            seen("topmost", KEYBOARD_EX & !WS_EX_TOPMOST.0, KEYBOARD_EX, true),
        ];
        assert!(!passed(&lost, &steps));
        let focus = [
            good[0].clone(),
            seen("topmost", KEYBOARD_EX, KEYBOARD_EX, false),
        ];
        assert!(!passed(&focus, &steps), "the target lost the focus");
    }
}

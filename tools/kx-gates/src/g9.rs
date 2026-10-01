//! G9 window flags survive (roadmap G9, ADR-0013): the guard's flags stay on the keyboard and every guarded window after
//! an outside topmost drop, style strip and click-through toggle, a hide and show, and a resize, with target-window in
//! front.
#![cfg(windows)]

use std::collections::HashMap;

use kx_platform_windows::window::{THROUGH, foreground, kept_ex};
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

use crate::apps::{self, Ctx};
use crate::g8ui::{self, Size, after_grow};
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
/// The steps that hide and show the keyboard, and size it; a run that breaks off between them undoes them.
const HIDE: &str = "hide";
const SHOW: &str = "show";
const GROW: &str = "grow";
const SHRINK: &str = "shrink";
const GROW_BACK: &str = "grow_back";
/// The steps after a grow, by name, in the order `after_grow` presses.
const BACK: [&str; 2] = [SHRINK, GROW_BACK];

/// The flags a window must keep, by the guard's own rule: its three, and click-through when it had both at first.
fn wants(first_ex: u32) -> u32 {
    kept_ex(first_ex, 0)
}

/// True when style `first_ex` lets clicks through; the guard keeps that for good, so G9 turns it on nowhere else.
fn was_through(first_ex: u32) -> bool {
    first_ex & THROUGH == THROUGH
}

/// The names of the flags in `want` that `ex` lacks.
fn missing(ex: u32, want: u32) -> Vec<&'static str> {
    let lacks = |&&(_, bit): &&(&str, u32)| want & bit != 0 && ex & bit == 0;
    FLAGS.iter().filter(lacks).map(|(name, _)| *name).collect()
}

/// The presses and step names after a grow, as G8 sizes back.
fn back(grew: bool) -> impl Iterator<Item = (Size, &'static str)> {
    after_grow(grew).iter().copied().zip(BACK)
}

/// The button that sizes a keyboard `now` px wide back toward `first` px; none within `slack` px.
fn mend(first: i32, now: i32, slack: i32) -> Option<Size> {
    if (now - first).abs() <= slack {
        None
    } else if now > first {
        Some(Size::Shrink)
    } else {
        Some(Size::Grow)
    }
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

/// What G9 knows across steps: each window's first style, every look, and the keyboard's width before the grow.
struct Watch {
    first: HashMap<isize, u32>,
    seen: Vec<Seen>,
    keyboard: HWND,
    target: HWND,
    width: Option<i32>,
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

    /// The windows G9 checks that let clicks through when first seen.
    fn through(&mut self) -> Vec<HWND> {
        let ours = self.ours();
        let was = |w: &HWND| was_through(self.first[&win::key(*w)]);
        ours.into_iter().filter(was).collect()
    }

    /// Waits `[g9] settle_ms`, then looks at every window G9 checks after `step`.
    fn look(&mut self, ctx: &Ctx, step: &'static str) {
        sleep_ms(ctx.cfg.g9.settle_ms);
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

/// The keyboard window's width, in px.
fn width(hwnd: HWND) -> Result<i32, String> {
    win::rect(hwnd).map(|r| r.right - r.left)
}

/// What a step does before its look.
enum Act<'a> {
    /// Nothing: the first look.
    Nothing,
    /// An outside program turns topmost off.
    DropTopmost,
    /// An outside program clears the guard's flags and click-through.
    StripFlags,
    /// An outside program turns click-through back on, where it was.
    ThroughOn,
    /// A top-bar button or the bubble is pressed, by UI Automation id.
    Press(&'a str),
}

/// The steps before the resize, in order, by name; each is followed by a look.
fn plan<'a>(ctx: &'a Ctx) -> [(&'static str, Act<'a>); 6] {
    let g9 = &ctx.cfg.g9;
    [
        ("start", Act::Nothing),
        ("topmost", Act::DropTopmost),
        ("styles", Act::StripFlags),
        ("click_through", Act::ThroughOn),
        (HIDE, Act::Press(&g9.put_away)),
        (SHOW, Act::Press(&g9.bring_back)),
    ]
}

/// Grows the keyboard and sizes it back as G8 does, with a look after each press.
fn sizes(ctx: &Ctx, w: &mut Watch, reader: &Reader) -> Result<(), String> {
    w.width = Some(width(w.keyboard)?);
    let grew = g8ui::resize(ctx, reader, w.keyboard, Size::Grow)?;
    w.look(ctx, GROW);
    for (size, step) in back(grew.ms.is_some()) {
        g8ui::resize(ctx, reader, w.keyboard, size)?;
        w.look(ctx, step);
    }
    Ok(())
}

/// Runs every step on the shown product with target-window in front; a refused write breaks the run off.
fn steps(ctx: &Ctx, w: &mut Watch) -> Result<(), String> {
    let reader = Reader::new()?;
    for (step, act) in plan(ctx) {
        match act {
            Act::Nothing => {}
            Act::DropTopmost => g9win::drop_topmost(&w.ours())?,
            Act::StripFlags => g9win::strip_flags(&w.ours())?,
            Act::ThroughOn => g9win::through_on(&w.through())?,
            Act::Press(id) => product::invoke(&reader, id)?,
        }
        w.look(ctx, step);
    }
    sizes(ctx, w, &reader)
}

/// After a run that broke off, shows the keyboard again and sizes it back to its first width; best effort.
fn undo(ctx: &Ctx, w: &Watch) {
    let Ok(reader) = Reader::new() else {
        return;
    };
    if w.did(HIDE) && !w.did(SHOW) {
        let _ = product::invoke(&reader, &ctx.cfg.g9.bring_back);
        sleep_ms(ctx.cfg.g9.settle_ms);
    }
    let now = width(w.keyboard).ok();
    let size = w
        .width
        .zip(now)
        .and_then(|(first, now)| mend(first, now, ctx.cfg.g9.slack_px));
    if let Some(size) = size {
        let _ = g8ui::resize(ctx, &reader, w.keyboard, size);
    }
}

/// Runs G9 on the product, called `name`; it writes the product windows' styles, so only with the owner's yes.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let keyboard = Keyboard::find()?.hwnd;
    let target = launch::to_front(ctx, launch::start_target(ctx)?)?;
    let mut w = Watch {
        first: HashMap::new(),
        seen: Vec::new(),
        keyboard,
        target: target.hwnd,
        width: None,
    };
    let ran = steps(ctx, &mut w);
    if ran.is_err() {
        undo(ctx, &w);
    }
    let repaired = g9win::repair(&w.first);
    let left_open = apps::clean_up(ctx, target);
    let mut names: Vec<&str> = plan(ctx).iter().map(|(step, _)| *step).collect();
    names.extend([GROW, SHRINK]);
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
mod tests;

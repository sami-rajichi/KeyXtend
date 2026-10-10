//! G9 window flags survive (roadmap G9, ADR-0013): the guard's flags stay on the keyboard and every guarded window after
//! an outside topmost drop, style strip and click-through toggle, a hide and show, and a resize, with target-window in
//! front.
#![cfg(windows)]

use std::collections::HashMap;

use kx_platform_windows::window::{foreground, is_through, kept_ex};
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

use crate::apps::{self, Ctx};
use crate::g8ui::{self, Resize, Size, after_grow};
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
    kept_ex(0, is_through(first_ex))
}

/// The names of the flags in `want` that `ex` lacks.
fn missing(ex: u32, want: u32) -> Vec<&'static str> {
    let lacks = |&&(_, bit): &&(&str, u32)| want & bit != 0 && ex & bit == 0;
    FLAGS.iter().filter(lacks).map(|(name, _)| *name).collect()
}

/// The names of the flags in `ex` the guard must refuse: click-through on a window that took clicks at first.
fn extra(ex: u32, first_ex: u32) -> Vec<&'static str> {
    let refused = if is_through(first_ex) {
        0
    } else {
        WS_EX_TRANSPARENT.0
    };
    let has = |&&(_, bit): &&(&str, u32)| refused & bit != 0 && ex & bit != 0;
    FLAGS.iter().filter(has).map(|(name, _)| *name).collect()
}

/// True when at least one size press changed the keyboard's box.
fn any_changed(presses: &[Resize]) -> bool {
    presses.iter().any(|r| r.ms.is_some())
}

/// The presses and step names after a grow, as G8 sizes back.
fn back(grew: bool) -> impl Iterator<Item = (Size, &'static str)> {
    after_grow(grew).iter().copied().zip(BACK)
}

/// One look at one product window after a step.
#[derive(Debug, Clone, Serialize)]
struct Seen {
    step: &'static str,
    window: String,
    ex: String,
    missing: Vec<&'static str>,
    extra: Vec<&'static str>,
    front_kept: bool,
    /// The window in front instead of target-window, by class and program; null while the target kept it.
    front: Option<String>,
}

/// The window `front` named by class and program, unless it is the `target`.
fn in_front(front: HWND, target: HWND) -> Option<String> {
    (front != target).then(|| win::describe(front))
}

/// True when every step saw at least one window, none lacked a flag or kept a refused one, and the target stayed in
/// front.
fn passed(seen: &[Seen], steps: &[&str]) -> bool {
    let looked = |s: &&str| seen.iter().any(|v| v.step == *s);
    let kept = |v: &Seen| v.missing.is_empty() && v.extra.is_empty() && v.front_kept;
    steps.iter().all(looked) && seen.iter().all(kept)
}

/// Adds each of `windows` to `written` once, in order.
fn note_written(written: &mut Vec<HWND>, windows: &[HWND]) {
    for &w in windows {
        if !written.contains(&w) {
            written.push(w);
        }
    }
}

/// What G9 knows across steps: each window's first style, the windows it wrote, every look, and the keyboard's width
/// before the grow.
struct Watch {
    first: HashMap<isize, u32>,
    written: Vec<HWND>,
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

    /// The windows G9 checks, kept as written, since the next act writes their styles.
    fn writing(&mut self) -> Vec<HWND> {
        let windows = self.ours();
        note_written(&mut self.written, &windows);
        windows
    }

    /// Waits `[g9] settle_ms`, then looks at every window G9 checks after `step`.
    fn look(&mut self, ctx: &Ctx, step: &'static str) {
        sleep_ms(ctx.cfg.g9.settle_ms);
        let front = foreground();
        let front_kept = front == self.target;
        for w in self.ours() {
            let (ex, first) = (ex_of(w), self.first[&win::key(w)]);
            self.seen.push(Seen {
                step,
                window: format!("{:#x} {}", win::key(w), win::class(w)),
                ex: format!("{ex:#010x}"),
                missing: missing(ex, wants(first)),
                extra: extra(ex, first),
                front_kept,
                front: in_front(front, self.target),
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
    /// An outside program clears the guard's flags and click-through.
    StripFlags,
    /// An outside program turns click-through on everywhere: back on where it was, refused on the rest.
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

/// Grows the keyboard and sizes it back as G8 does, with a look after each press; an error when no press resized it.
fn sizes(ctx: &Ctx, w: &mut Watch, reader: &Reader) -> Result<(), String> {
    w.width = Some(g8ui::width(w.keyboard)?);
    let grew = g8ui::resize(ctx, reader, w.keyboard, Size::Grow)?;
    w.look(ctx, GROW);
    let mut presses = vec![];
    for (size, step) in back(grew.ms.is_some()) {
        presses.push(g8ui::resize(ctx, reader, w.keyboard, size)?);
        w.look(ctx, step);
    }
    presses.push(grew);
    if any_changed(&presses) {
        Ok(())
    } else {
        Err("no size button changed the keyboard".to_string())
    }
}

/// Runs every step on the shown product with target-window in front; a refused write breaks the run off.
fn steps(ctx: &Ctx, w: &mut Watch) -> Result<(), String> {
    let reader = Reader::new()?;
    for (step, act) in plan(ctx) {
        match act {
            Act::Nothing => {}
            Act::DropTopmost => g9win::drop_topmost(&w.writing())?,
            Act::StripFlags => g9win::strip_flags(&w.writing())?,
            Act::ThroughOn => g9win::through_on(&w.writing())?,
            Act::Press(id) => product::invoke(&reader, id)?,
        }
        w.look(ctx, step);
    }
    sizes(ctx, w, &reader)
}

/// After every run, shows the keyboard again if a run broke off while it was hidden, and sizes it back to its first
/// width if a press went missing; what could not be undone.
fn undo(ctx: &Ctx, w: &Watch) -> Vec<String> {
    let reader = match Reader::new() {
        Ok(reader) => reader,
        Err(e) => return vec![format!("nothing undone: {e}")],
    };
    let mut left = Vec::new();
    if w.did(HIDE) && !w.did(SHOW) {
        if let Err(e) = product::invoke(&reader, &ctx.cfg.g9.bring_back) {
            left.push(format!("the keyboard stays hidden: {e}"));
        }
        sleep_ms(ctx.cfg.g9.settle_ms);
    }
    if let Some(first) = w.width
        && let Err(e) = g8ui::size_back(ctx, &reader, w.keyboard, first)
    {
        left.push(format!("the size was not put back: {e}"));
    }
    left
}

/// Runs G9 on the product, called `name`; it writes the product windows' styles, so only with the owner's yes.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let keyboard = Keyboard::find()?.hwnd;
    let target = launch::to_front(ctx, launch::start_target(ctx)?)?;
    let mut w = Watch {
        first: HashMap::new(),
        written: Vec::new(),
        seen: Vec::new(),
        keyboard,
        target: target.hwnd,
        width: None,
    };
    let ran = steps(ctx, &mut w);
    let mut not_put_back = undo(ctx, &w);
    let repaired = g9win::repair(&w.first, &w.written);
    not_put_back.extend(repaired.not_put_back);
    let left_open = apps::clean_up(ctx, target);
    let mut names: Vec<&str> = plan(ctx).iter().map(|(step, _)| *step).collect();
    names.extend([GROW, SHRINK]);
    let pass = ran.is_ok() && passed(&w.seen, &names);
    let mut v = json!({
        "gate": "G9", "keyboard": name, "seen": w.seen, "put_back": repaired.fixed, "not_put_back": not_put_back,
        "left_open": left_open, "pass": pass,
    });
    // Only a run that broke off, or whose size step failed, carries `error`.
    if let Err(e) = ran {
        v["error"] = json!(e);
    }
    Ok(v)
}

#[cfg(test)]
mod tests;

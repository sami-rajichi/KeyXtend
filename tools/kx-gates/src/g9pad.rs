//! G9 on the scroll outline (ADR-0029): no edge window while the pad is idle; while it is hovered and while it scrolls,
//! exactly `[outline] edges` of them, each click-through, no-activate and topmost, titled from kx-ui's
//! `window.outline`, and under the keyboard in the z-order with only our windows between, such as its shadow.
//! Target-window, filled, lies under the keyboard and keeps the front at every look.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use kx_platform_windows::window::{foreground, kept_ex};
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;

use crate::apps::{Ctx, Opened};
use crate::g9win::{ex_of, missing};
use crate::keymap::Reader;
use crate::padread::PadRead;
use crate::pin::Pinned;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{mouse, outline};

/// One look at the outline after a step.
#[derive(Debug, Clone, Serialize)]
struct Seen {
    step: &'static str,
    /// The edge windows shown.
    edges: usize,
    /// Each edge's missing flags, by name.
    missing: Vec<Vec<&'static str>>,
    /// The edges lay under the keyboard with only ours between; true when none showed.
    under: bool,
    /// When they did not: the z-order from the first to the last of the keyboard and the edges.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stack: Vec<String>,
    /// Target-window was still in front.
    front_kept: bool,
    ok: bool,
}

/// True when a look passes: `want` edges each with every flag and just under the keyboard, or none at all when
/// `want` is 0, with the target still in front, `front_kept`.
fn judged(want: usize, missing: &[Vec<&str>], under: bool, front_kept: bool) -> bool {
    missing.len() == want && missing.iter().all(Vec::is_empty) && under && front_kept
}

/// Where G9 looks: its context, the keyboard window, the target-window that should keep the front and the outline's
/// titles.
struct Desk<'a> {
    ctx: &'a Ctx<'a>,
    keyboard: HWND,
    target: HWND,
    titles: Vec<String>,
}

impl Desk<'_> {
    /// Waits the pad's settle, then looks at the outline after `step`, which should show `want` edges.
    fn look(&self, step: &'static str, want: usize) -> Seen {
        sleep_ms(self.ctx.cfg.pad.settle_ms);
        let edges = outline::shown(&self.titles);
        let flags = kept_ex(0, true);
        let missing: Vec<_> = edges.iter().map(|&w| missing(ex_of(w), flags)).collect();
        let (order, ours) = (win::top_windows(), product::shown());
        let under = edges.is_empty()
            || outline::just_under(&order, self.keyboard, &edges, |w| ours.contains(&w));
        let stack = if under {
            Vec::new()
        } else {
            outline::stack(&order, self.keyboard, &edges, win::describe)
        };
        let front_kept = foreground() == self.target;
        let ok = judged(want, &missing, under, front_kept);
        Seen {
            step,
            edges: edges.len(),
            missing,
            under,
            stack,
            front_kept,
            ok,
        }
    }

    /// Presses down, rests the pointer on the rest key off the pad, and looks while it scrolls.
    fn scrolling(&self, reader: &Reader) -> Result<Seen, String> {
        product::invoke(reader, &self.ctx.cfg.pad.arrow(Dir::Down))?;
        let rest = Keyboard::find()?.at(&self.ctx.cfg.outline.rest)?;
        mouse::move_to(rest)?;
        Ok(self.look("scrolling", self.ctx.cfg.outline.edges))
    }
}

/// Runs G9's outline part on the keyboard window `keyboard` with `target` laid under it; Stop is pressed whatever
/// happens unless the pad is idle, and the pointer leaves the keyboard after.
pub fn run(ctx: &Ctx, keyboard: HWND, target: &Opened) -> Result<Value, String> {
    let desk = Desk {
        ctx,
        keyboard,
        target: target.hwnd,
        titles: outline::titles(ctx)?,
    };
    let (reader, edges) = (Reader::new()?, ctx.cfg.outline.edges);
    let pin = Pinned::new(ctx, keyboard, target)?;
    mouse::move_to(pin.start)?;
    let mut seen = vec![desk.look("idle", 0)];
    pin.glide_in(ctx, target, mouse::move_to)?;
    seen.push(desk.look("hovered", edges));
    let scrolled = desk.scrolling(&reader);
    let read = PadRead {
        reader: &reader,
        kb: keyboard,
        pad: &ctx.cfg.pad,
    };
    let stopped = read.stop(|id| product::restore(&reader, id));
    seen.extend(scrolled.as_ref().ok().cloned());
    seen.push(desk.look("stopped", 0));
    mouse::move_to(pin.start)?;
    scrolled?;
    stopped?;
    let ok = seen.iter().all(|s| s.ok);
    Ok(json!({ "ok": ok, "seen": seen }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shown_outline_needs_every_edge_flagged_and_just_under_the_keyboard() {
        let clean = vec![Vec::new(); 4];
        assert!(judged(4, &clean, true, true));
        assert!(!judged(4, &clean[..3], true, true), "an edge is missing");
        assert!(!judged(4, &clean, false, true), "not under the keyboard");
        let mut flagless = clean.clone();
        flagless[2].push("transparent");
        assert!(!judged(4, &flagless, true, true), "an edge takes clicks");
    }

    #[test]
    fn an_idle_pad_shows_no_outline() {
        assert!(judged(0, &[], true, true));
        assert!(!judged(0, &[Vec::new()], true, true), "an edge stayed");
    }

    #[test]
    fn a_look_fails_once_the_target_lost_the_front() {
        assert!(
            !judged(4, &vec![Vec::new(); 4], true, false),
            "the outline took it"
        );
        assert!(!judged(0, &[], true, false), "idle, but the target lost it");
    }
}

//! G9 on the scroll outline (ADR-0029): no edge window while the pad is idle; while it is hovered and while it scrolls,
//! exactly `[outline] edges` of them, each click-through, no-activate and topmost, titled from kx-ui's
//! `window.outline`, and just under the keyboard in the z-order. Target-window, filled, lies under the keyboard.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use kx_platform_windows::window::kept_ex;
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;

use crate::apps::{Ctx, Opened};
use crate::g9::missing;
use crate::g9win::ex_of;
use crate::keymap::Reader;
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
    /// The edges lay just under the keyboard; true when none showed.
    under: bool,
    ok: bool,
}

/// True when a look passes: `want` edges each with every flag and just under the keyboard, or none at all when
/// `want` is 0.
fn judged(want: usize, missing: &[Vec<&str>], under: bool) -> bool {
    missing.len() == want && missing.iter().all(Vec::is_empty) && under
}

/// Where G9 looks: its context, the keyboard window and the outline's titles.
struct Desk<'a> {
    ctx: &'a Ctx<'a>,
    keyboard: HWND,
    titles: Vec<String>,
}

impl Desk<'_> {
    /// Waits the pad's settle, then looks at the outline after `step`, which should show `want` edges.
    fn look(&self, step: &'static str, want: usize) -> Seen {
        sleep_ms(self.ctx.cfg.pad.settle_ms);
        let edges = outline::shown(&self.titles);
        let flags = kept_ex(0, true);
        let missing: Vec<_> = edges.iter().map(|&w| missing(ex_of(w), flags)).collect();
        let under =
            edges.is_empty() || outline::just_under(&win::top_windows(), self.keyboard, &edges);
        let ok = judged(want, &missing, under);
        Seen {
            step,
            edges: edges.len(),
            missing,
            under,
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
/// happens, and the pointer leaves the keyboard after.
pub fn run(ctx: &Ctx, keyboard: HWND, target: &Opened) -> Result<Value, String> {
    let desk = Desk {
        ctx,
        keyboard,
        titles: outline::titles(ctx)?,
    };
    let (reader, edges) = (Reader::new()?, ctx.cfg.outline.edges);
    let pin = Pinned::new(ctx, keyboard, target)?;
    mouse::move_to(pin.start)?;
    let mut seen = vec![desk.look("idle", 0)];
    pin.glide_in(ctx, mouse::move_to)?;
    seen.push(desk.look("hovered", edges));
    let scrolled = desk.scrolling(&reader);
    let stopped = product::restore(&reader, &ctx.cfg.pad.stop_id());
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
        assert!(judged(4, &clean, true));
        assert!(!judged(4, &clean[..3], true), "an edge is missing");
        assert!(!judged(4, &clean, false), "not under the keyboard");
        let mut flagless = clean.clone();
        flagless[2].push("transparent");
        assert!(!judged(4, &flagless, true), "an edge takes clicks");
    }

    #[test]
    fn an_idle_pad_shows_no_outline() {
        assert!(judged(0, &[], true));
        assert!(!judged(0, &[Vec::new()], true), "an edge stayed");
    }
}

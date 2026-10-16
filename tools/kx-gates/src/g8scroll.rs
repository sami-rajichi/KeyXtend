//! G8's scroll steps (spec §5.2, ADR-0029): the pointer glides from a filled target-window onto the pad, whose arrow,
//! pressed by UI Automation, scrolls for `[g8_scroll] scroll_ms` with the outline shown, then Stop. Memory is read
//! once, `[g8] warm_ms` after a scroll, against ADR-0021's idle budget; each churn round scrolls, each way in turn.
#![cfg(windows)]

use std::cell::Cell;

use kx_module_api::ports::Dir;
use serde::Serialize;

use crate::apps::{self, Ctx, Opened};
use crate::budget::Part;
use crate::g8cfg::G8;
use crate::g8host::{self, Host};
use crate::keymap::Reader;
use crate::pin::Pinned;
use crate::proc::{Memory, Process};
use crate::product::{self, Keyboard};
use crate::win::sleep_ms;
use crate::{launch, mouse, outline};

/// The reading's part for the private working set.
pub const MEMORY: &str = "scrolled_memory_mb";
/// Its part for the private commit.
pub const COMMIT: &str = "scrolled_commit_mb";

/// One scroll: whether the pad lit, as Stop read a speed, and whether the outline showed all its edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Scroll {
    /// The pad scrolled.
    pub lit: bool,
    /// The outline showed every edge.
    pub outlined: bool,
}

/// The reading after a scroll.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scrolled {
    /// What the scroll showed.
    pub scroll: Scroll,
    /// Private memory `[g8] warm_ms` after it.
    pub memory: Memory,
    /// The machine's memory at the reading.
    pub host: Host,
}

/// The reading after a scroll against the cold budgets; a scroll that did not light the pad or show the outline reads
/// as not measured, so both parts fail.
pub fn parts(g: &G8, scrolled: Option<&Scrolled>) -> [Part; 2] {
    let shown = scrolled.filter(|s| s.scroll.lit && s.scroll.outlined);
    let read = shown.map(|s| s.memory);
    [
        Part::at_most(MEMORY, read.map(|m| m.working_mb), g.memory_mb),
        Part::at_most(COMMIT, read.map(|m| m.commit_mb), g.commit_mb),
    ]
}

/// A filled target-window under the keyboard and the pad that scrolls it.
pub struct Scroller<'a> {
    ctx: &'a Ctx<'a>,
    target: Opened,
    pin: Pinned,
    keyboard: Keyboard,
    reader: Reader,
    titles: Vec<String>,
    down: Cell<bool>,
}

impl<'a> Scroller<'a> {
    /// Starts a filled target-window and lays it under the keyboard; closed again when the laying fails.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        let filled = [ctx.target.fill_arg.clone()];
        let target = launch::start_target_with(ctx, &filled)?;
        let laid = Keyboard::find().and_then(|kb| Ok((Pinned::new(ctx, kb.hwnd, &target)?, kb)));
        let (pin, keyboard) = match laid {
            Ok(laid) => laid,
            Err(e) => return Err(format!("{e}; left open: {:?}", apps::clean_up(ctx, target))),
        };
        Ok(Self {
            ctx,
            target,
            pin,
            keyboard,
            reader: Reader::new()?,
            titles: outline::titles(ctx)?,
            down: Cell::new(true),
        })
    }

    /// Glides onto the pad, presses an arrow, down and up in turn, and reads the pad and the outline after the pad's
    /// settle; Stop goes out after `scroll_ms`, also after a failed read, then the pointer leaves the keyboard.
    pub fn once(&self) -> Result<Scroll, String> {
        let (ctx, pad) = (self.ctx, &self.ctx.cfg.pad);
        let dir = if self.down.replace(!self.down.get()) {
            Dir::Down
        } else {
            Dir::Up
        };
        self.pin.glide_in(ctx, mouse::move_to)?;
        product::invoke(&self.reader, &pad.arrow(dir))?;
        sleep_ms(pad.settle_ms);
        let lit = self.reader.described(self.keyboard.hwnd, &pad.stop_id());
        let outlined = outline::shown(&self.titles).len() == ctx.cfg.outline.edges;
        sleep_ms(ctx.cfg.g8_scroll.scroll_ms.saturating_sub(pad.settle_ms));
        product::restore(&self.reader, &pad.stop_id())?;
        sleep_ms(pad.settle_ms);
        mouse::move_to(self.pin.start)?;
        let lit = !lit?.is_empty();
        Ok(Scroll { lit, outlined })
    }

    /// One scroll, then `[g8] warm_ms`, then the reading of `p` and of the machine.
    pub fn reading(&self, p: &Process) -> Result<Scrolled, String> {
        let scroll = self.once()?;
        sleep_ms(self.ctx.cfg.g8.warm_ms);
        Ok(Scrolled {
            scroll,
            memory: p.memory()?,
            host: g8host::read(self.ctx),
        })
    }

    /// Closes target-window; what was left open.
    pub fn close(self) -> Vec<String> {
        let Self {
            ctx, target, pin, ..
        } = self;
        drop(pin);
        apps::clean_up(ctx, target)
    }
}

/// The reading after a scroll and the scroller that took it, kept for the churn's scrolls.
pub type Taken<'a> = (Result<Scrolled, String>, Option<Scroller<'a>>);

/// Opens a scroller and reads `p` after one scroll, then runs `hold`, as target-window took the front. A scroller
/// whose reading failed is closed, so the churn runs without scrolls; so is one whose `hold` failed.
pub fn taken<'a>(
    ctx: &'a Ctx<'a>,
    p: &Process,
    hold: &dyn Fn() -> Result<(), String>,
) -> Result<Taken<'a>, String> {
    let (scrolled, pad) = match Scroller::open(ctx) {
        Ok(pad) => match pad.reading(p) {
            Ok(read) => (Ok(read), Some(pad)),
            Err(e) => (Err(format!("{e}; left open: {:?}", pad.close())), None),
        },
        Err(e) => (Err(e), None),
    };
    match hold() {
        Ok(()) => Ok((scrolled, pad)),
        Err(e) => Err(format!("{e}; left open: {:?}", pad.map(Scroller::close))),
    }
}

/// A reading of `memory` after a scroll that lit the pad and showed the outline, for tests.
#[cfg(test)]
pub fn read_as(memory: Memory) -> Scrolled {
    Scrolled {
        scroll: Scroll {
            lit: true,
            outlined: true,
        },
        memory,
        host: Host {
            free_mb: None,
            vm: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget;

    fn g8() -> G8 {
        crate::config::load().expect("loads").g8
    }

    #[test]
    fn the_reading_after_a_scroll_meets_the_cold_budgets() {
        let g = g8();
        let within = Memory {
            working_mb: g.memory_mb,
            commit_mb: g.commit_mb,
        };
        assert!(budget::all_pass(&parts(&g, Some(&read_as(within)))));
        let over = Memory {
            commit_mb: g.commit_mb + 1.0,
            ..within
        };
        let failed: Vec<&str> = parts(&g, Some(&read_as(over)))
            .iter()
            .filter(|p| !p.pass)
            .map(|p| p.name)
            .collect();
        assert_eq!(failed, [COMMIT]);
    }

    #[test]
    fn a_scroll_that_lit_nothing_or_hid_the_outline_is_not_a_reading() {
        let g = g8();
        let low = Memory {
            working_mb: 1.0,
            commit_mb: 1.0,
        };
        let mut dark = read_as(low);
        dark.scroll.lit = false;
        let mut bare = read_as(low);
        bare.scroll.outlined = false;
        for s in [Some(&dark), Some(&bare), None] {
            assert!(parts(&g, s).iter().all(|p| !p.pass));
        }
    }
}

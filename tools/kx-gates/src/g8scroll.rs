//! G8's scroll steps (spec §5.2, ADR-0029): the pointer glides from a filled target-window onto the pad, whose arrow,
//! pressed by UI Automation, scrolls for `[g8_scroll] scroll_ms` with the outline shown, then Stop. Memory is read
//! once, `[g8] warm_ms` after a scroll, against ADR-0021's idle budget; each churn round scrolls, as `way` turns, and
//! `g8rounds` runs them in order.
#![cfg(windows)]

use std::cell::Cell;

use kx_module_api::ports::Dir;
use serde::Serialize;

use crate::apps::{self, Ctx, Opened};
use crate::budget::Part;
use crate::g8cfg::G8;
use crate::g8host::{self, Host};
use crate::keymap::Reader;
use crate::padread::PadRead;
use crate::pin::Pinned;
use crate::proc::{Memory, Process};
use crate::product::{self, Keyboard};
use crate::win::sleep_ms;
use crate::{launch, mouse, outline};

/// The reading's part for the private working set.
pub const MEMORY: &str = "scrolled_memory_mb";
/// Its part for the private commit.
pub const COMMIT: &str = "scrolled_commit_mb";

/// One scroll: whether the pad lit, as its arrow read a speed, and whether the outline showed all its edges.
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
    runs: Cell<u32>,
}

impl<'a> Scroller<'a> {
    /// Starts a filled target-window and lays it under the keyboard; closed again when the laying fails.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        let (reader, titles) = (Reader::new()?, outline::titles(ctx)?);
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
            reader,
            titles,
            runs: Cell::new(0),
        })
    }

    /// Glides onto the pad, presses its arrow the way `way` gives, and reads the arrow and the outline after the
    /// pad's settle; Stop goes out after `scroll_ms` unless the pad stopped itself, also after a failed read, then
    /// the pointer leaves the keyboard.
    pub fn once(&self) -> Result<Scroll, String> {
        let (ctx, pad) = (self.ctx, &self.ctx.cfg.pad);
        let dir = way(self.runs.replace(self.runs.get().saturating_add(1)));
        self.pin.glide_in(ctx, &self.target, mouse::move_to)?;
        product::invoke(&self.reader, &pad.arrow(dir))?;
        sleep_ms(pad.settle_ms);
        let read = PadRead {
            reader: &self.reader,
            kb: self.keyboard.hwnd,
            pad,
        };
        let lit = read.said(dir);
        let outlined = outline::shown(&self.titles).len() == ctx.cfg.outline.edges;
        sleep_ms(ctx.cfg.g8_scroll.scroll_ms.saturating_sub(pad.settle_ms));
        let stopped = read.stop(|id| product::restore(&self.reader, id));
        sleep_ms(pad.settle_ms);
        mouse::move_to(self.pin.start)?;
        let lit = !lit?.is_empty();
        stopped?;
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

    /// The filled target-window it scrolls, which a hold brings back to the front while it is kept.
    pub fn window(&self) -> &Opened {
        &self.target
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

/// Which way scroll number `k` goes: down twice, then up and down in turn, so the window stays one to two runs below
/// its top and no run ends where one began, at the top.
fn way(k: u32) -> Dir {
    if k < 2 || k % 2 == 1 {
        Dir::Down
    } else {
        Dir::Up
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
    fn the_scrolls_go_down_twice_then_turn_so_none_runs_back_to_the_top() {
        let ways: Vec<Dir> = (0..6).map(way).collect();
        let (d, u) = (Dir::Down, Dir::Up);
        assert_eq!(ways, [d, d, u, d, u, d]);
        let mut at = 0;
        for k in 0..60 {
            at += if way(k) == d { 1 } else { -1 };
            assert!((1..=2).contains(&at), "run {k} ends {at} runs down");
        }
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

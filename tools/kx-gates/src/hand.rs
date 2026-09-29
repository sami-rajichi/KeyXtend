//! The owner's hand try: the engine acts on the real mouse for a while, then reports what it saw.
#![cfg(windows)]

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use spike_core::hold::Mode;
use windows::Win32::Foundation::RECT;

use crate::apps::Ctx;
use crate::assist::{Assist, Setup};
use crate::hookio::Source;
use crate::win::{self, sleep_ms};

/// Hand-try mode: Right-click.
pub const RIGHT: &str = "right";
/// Hand-try mode: Grab.
pub const GRAB: &str = "grab";

/// The boxes of every open face window, where presses are never held.
fn face_rects(ctx: &Ctx) -> Vec<RECT> {
    win::top_windows()
        .into_iter()
        .filter(|&w| win::title(w) == ctx.spike.keyboard.title)
        .filter_map(|w| win::rect(w).ok())
        .collect()
}

/// Runs the engine in mode `name` on the real mouse for `secs` seconds.
pub fn run(ctx: &Ctx, name: &str, secs: u64) -> Result<Value, String> {
    let mode = match name {
        RIGHT => Mode::RightClick,
        GRAB => Mode::Grab,
        _ => return Err(format!("the hand try runs {RIGHT} or {GRAB}, not {name}")),
    };
    let (a, h) = (&ctx.cfg.assist, &ctx.spike.hold);
    let end = Instant::now()
        .checked_add(Duration::from_secs(secs))
        .ok_or_else(|| format!("--secs {secs} is too long"))?;
    let assist = Assist::start(Setup {
        hold_ms: h.ms,
        still_px: h.still_px,
        source: Source::Physical,
        own: face_rects(ctx),
        reply_ms: a.reply_ms,
    })?;
    assist.set_mode(mode)?;
    println!("hand try: {name} for {secs} s; hold still {} ms", h.ms);
    while Instant::now() < end {
        sleep_ms(a.rearm_ms);
        if mode == Mode::Grab {
            // Grab turns itself off after a drop, and this try has no keyboard to turn it back on.
            assist.set_mode(mode)?;
        }
    }
    let report = assist.stop()?;
    println!("seen {}; errors {:?}", report.seen, report.errors);
    Ok(json!({
        "gate": "assist", "mode": name, "secs": secs, "seen": report.seen,
        "hook_calls": report.hook_us.len(), "hook_max_us": report.hook_us.iter().max(),
        "max_wait_ms": report.max_wait_ms,
        "errors": report.errors,
    }))
}

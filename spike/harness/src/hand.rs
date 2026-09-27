//! The owner's hand try: the engine acts on the real mouse for a while, then reports what it saw.

use serde_json::{Value, json};
use spike_core::hold::Mode;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

use crate::apps::Ctx;
use crate::assist::{Assist, Setup};
use crate::hookio::Source;
use crate::win::{self, sleep_ms};

/// Hand-try mode: Right-click.
pub const RIGHT: &str = "right";
/// Hand-try mode: Grab.
pub const GRAB: &str = "grab";
/// Milliseconds per second.
const MS_PER_S: u64 = 1000;

/// The boxes of every open face, where presses are never held.
fn face_rects(ctx: &Ctx) -> Vec<RECT> {
    let titles: Vec<&String> = ctx.spike.keyboard.titles.values().collect();
    win::top_windows()
        .into_iter()
        .filter(|&w| titles.contains(&&win::title(w)))
        .filter_map(|w| {
            let mut r = RECT::default();
            // SAFETY: plain query into a local.
            unsafe { GetWindowRect(w, &mut r) }.ok().map(|()| r)
        })
        .collect()
}

/// Runs the engine in mode `name` on the real mouse for `secs` seconds.
pub fn run(ctx: &Ctx, name: &str, secs: u64) -> Result<Value, String> {
    let mode = match name {
        RIGHT => Mode::RightClick,
        GRAB => Mode::Grab,
        _ => return Err(format!("the hand try runs {RIGHT} or {GRAB}, not {name}")),
    };
    let a = &ctx.cfg.assist;
    let assist = Assist::start(Setup {
        hold_ms: a.hold_ms,
        still_px: a.still_px,
        source: Source::Physical,
        own: face_rects(ctx),
        reply_ms: a.reply_ms,
    })?;
    assist.set_mode(mode)?;
    println!("hand try: {name} for {secs} s; hold still {} ms", a.hold_ms);
    sleep_ms(secs.saturating_mul(MS_PER_S));
    let report = assist.stop()?;
    println!("seen {}; errors {:?}", report.seen, report.errors);
    Ok(json!({
        "gate": "assist", "mode": name, "secs": secs, "seen": report.seen,
        "hook_calls": report.hook_us.len(), "hook_max_us": report.hook_us.iter().max(),
        "max_wait_ms": report.max_wait_ms,
        "errors": report.errors,
    }))
}

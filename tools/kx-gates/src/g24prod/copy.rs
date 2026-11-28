//! One copy's run: with the keyboard as a bubble, the full screen first, then every Snip window before idle memory, a
//! snip and a recording after it, and the Save dialog last. Hidden takes the full screen and the sped long recording.
#![cfg(windows)]

use std::path::Path;

use kx_module_api::ports::SnipKeyboard;
use serde_json::{Value, json};

use super::scene::Scene;
use super::shots;
use super::snip::Snip;
use super::{judge, low, video};
use crate::budget;
use crate::g25::part;
use crate::proc::{Memory, Process};
use crate::win::sleep_ms;

/// The parts of a bubble copy, in the result.
pub const BUBBLE_PARTS: [&str; 10] = [
    "apart", "full", "freeze", "region", "edit", "warm", "window", "record", "memory", "low",
];
/// The parts of a hidden copy.
pub const HIDDEN_PARTS: [&str; 3] = ["apart", "full", "speed"];

/// The copy's folders: its data folder, and the folder holding the test build and its worker.
pub struct Dirs<'a> {
    pub data: &'a Path,
    pub workers: &'a Path,
}

/// What `f` gave as a part; after a failure the snip is reset, so the next part starts clean.
fn step(s: &Snip, f: impl FnOnce() -> Result<Value, String>) -> Value {
    let got = f();
    if got.is_err() {
        s.reset();
    }
    part(got)
}

/// The copy's memory after `[g24] idle_ms`, once the snip ended.
fn idle(s: &Snip) -> Result<Memory, String> {
    s.ended()?;
    sleep_ms(s.g.idle_ms);
    Process::to_measure(s.pid)?.memory()
}

/// The growth from `start` to now, each figure against `[g24] back_mb`.
fn memory(s: &Snip, start: Result<Memory, String>) -> Result<Value, String> {
    let (start, now) = (start?, idle(s)?);
    let parts = judge::back(start, now, s.g.back_mb);
    Ok(json!({ "idle": start, "after": now, "parts": parts, "ok": budget::all_pass(&parts) }))
}

/// The bubble copy's parts, by name; the first snip is the judged full screen, and idle comes once each window showed.
pub fn bubble(s: &Snip, sc: &Scene, dirs: &Dirs) -> Value {
    let mut v = json!({ "apart": sc.apart() });
    v["full"] = step(s, || shots::full(s, sc));
    match shots::region(s, sc) {
        Ok([freeze, region]) => {
            v["freeze"] = freeze;
            v["region"] = region;
        }
        Err(e) => {
            s.reset();
            v["freeze"] = part(Err(e));
        }
    }
    v["edit"] = step(s, || shots::edit(s, sc));
    v["warm"] = step(s, || video::warm(s, sc));
    let start = idle(s);
    if start.is_err() {
        s.reset();
    }
    v["window"] = step(s, || shots::window(s, sc));
    v["record"] = step(s, || video::short(s, sc, dirs.data));
    v["memory"] = step(s, || memory(s, start));
    v["low"] = step(s, || low::low(s, sc, dirs.workers));
    v
}

/// The hidden copy's parts, by name.
pub fn hidden(s: &Snip, sc: &Scene, dirs: &Dirs) -> Value {
    let mut v = json!({ "apart": sc.apart() });
    v["full"] = step(s, || shots::full(s, sc));
    v["speed"] = step(s, || video::sped(s, sc, dirs.data, dirs.workers));
    v
}

/// The part names of a copy with the keyboard `way`.
pub fn names(way: SnipKeyboard) -> &'static [&'static str] {
    if way == SnipKeyboard::Bubble {
        &BUBBLE_PARTS
    } else {
        &HIDDEN_PARTS
    }
}

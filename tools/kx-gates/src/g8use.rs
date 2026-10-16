//! G8's reading after use (ADR-0028), right after G2 and before the churn's theme rounds: the mode key turns its mode
//! on with a click, the pointer rests once on every key of the board without clicking, and after the warm-up's wait
//! memory is judged by the cold budgets again. Then the key puts the mode back as it was, even when a step failed.
#![cfg(windows)]

use kx_mod_keyboard::Role;
use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
use serde::Serialize;
use serde_json::Value;
use windows::Win32::Foundation::POINT;

use crate::apps::Ctx;
use crate::budget::Part;
use crate::g8cfg::G8;
use crate::modekey::{ModeKey, Wait};
use crate::proc::{Memory, Process};
use crate::product::Keyboard;
use crate::win::{self, sleep_ms};
use crate::{board, g8ui, mouse};

/// The reading's part for the private working set.
pub const MEMORY: &str = "used_memory_mb";
/// Its part for the private commit.
pub const COMMIT: &str = "used_commit_mb";
/// The reading's field in G8's result line.
pub const FIELD: &str = "use";
/// What a broken step says when the mode went back.
const BACK: &str = "the mode key is back as it was";
/// What it says when the mode did not, which changes the next cold start.
const NOT_BACK: &str =
    "the mode key did not go back, so set it with its key before the next cold start";
/// What a step that held says when only the mode's put-back failed.
const READ_LEFT_OUT: &str = "the reading after use is left out";

/// What the use did, and private memory once it settled.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Used {
    /// Keys the pointer rested on, once each: every key of the board.
    pub hovers: usize,
    /// The mode was on already, so it stays on.
    pub mode_was_on: bool,
    /// The layout of the app in front at the reading, so `g8 arabic` shows its layout held.
    pub layout: Option<String>,
    /// Private memory at the reading.
    pub memory: Memory,
}

/// Refuses a use step that could pass with nothing used: no rest, no wait for the light, or a mode key that is no
/// mouse-assist key of the product's board.
pub fn check(g: &G8) -> Result<(), String> {
    if g.use_hover_ms == 0 || g.use_lit_ms == 0 {
        return Err("g8.use_hover_ms and g8.use_lit_ms must be above 0".to_string());
    }
    let assist =
        |(id, role, _): &(String, Role, _)| *id == g.use_mode && matches!(role, Role::Assist(_));
    if !board::roles()?.iter().any(assist) {
        return Err(format!(
            "g8.use_mode: {} is no mouse-assist key of the product's board",
            g.use_mode
        ));
    }
    Ok(())
}

/// The reading after use against the cold budgets; with no reading, both parts fail as not measured.
pub fn parts(g: &G8, used: Option<&Used>) -> [Part; 2] {
    let read = used.map(|u| u.memory);
    [
        Part::at_most(MEMORY, read.map(|m| m.working_mb), g.memory_mb),
        Part::at_most(COMMIT, read.map(|m| m.commit_mb), g.commit_mb),
    ]
}

/// The layout in front at the reading after use, from G8's result line `v`; none when it was not read.
pub fn layout_in(v: &Value) -> Option<String> {
    v[FIELD]["layout"].as_str().map(String::from)
}

/// The screen centres of the keys `ids` names, in that order, as `at` finds them; refused when any has none, as every
/// board key has a box (G10), so a part sweep would read too low.
pub fn sweep(ids: &[String], at: impl Fn(&str) -> Option<POINT>) -> Result<Vec<POINT>, String> {
    let points: Vec<POINT> = ids.iter().filter_map(|id| at(id)).collect();
    match ids.len() - points.len() {
        0 => Ok(points),
        missed => Err(format!(
            "{missed} of {} board keys have no box on screen, so a part sweep would read too low",
            ids.len()
        )),
    }
}

/// The use's result once the mode was put `back`: an error says whether the mode went back, and a failed put-back
/// fails a step that held, as the next cold start would differ.
fn joined<T>(used: Result<T, String>, back: Result<(), String>) -> Result<T, String> {
    match (used, back) {
        (used, Ok(())) => used.map_err(|e| format!("{e}; {BACK}")),
        (used, Err(b)) => {
            let first = used.err().unwrap_or_else(|| READ_LEFT_OUT.to_string());
            Err(format!("{first}; {NOT_BACK} ({b})"))
        }
    }
}

/// Rests the pointer once on each board key, in board order, then moves it off the keyboard, so no hover stays; the
/// keys hovered. Refused before any move when a key has no box.
fn hover(g: &G8, kb: &Keyboard) -> Result<usize, String> {
    let points = sweep(&board::ids()?, |id| kb.at(id).ok())?;
    let first = *points
        .first()
        .ok_or("no key of the board has a box on screen")?;
    for &at in &points {
        mouse::move_to(at)?;
        sleep_ms(g.use_hover_ms);
    }
    let away = g8ui::away_from(win::rect(kb.hwnd)?, first, win::desktop().1);
    mouse::move_to(away)?;
    Ok(points.len())
}

/// The sweep, the warm-up's wait, then the reading of `p`, with the mode on since `was` or since the step began.
fn read_after(g: &G8, kb: &Keyboard, p: &Process, was: bool) -> Result<Used, String> {
    let hovers = hover(g, kb)?;
    sleep_ms(g.warm_ms);
    Ok(Used {
        hovers,
        mode_was_on: was,
        layout: WinLayouts.foreground().map(|l| l.to_string()),
        memory: p.memory()?,
    })
}

/// The use step on the started product `p`: its mode on, the sweep, the wait and the reading, then the mode back as it
/// was, also after a failed step.
pub fn run(ctx: &Ctx, p: &Process) -> Result<Used, String> {
    let g = &ctx.cfg.g8;
    let key = ModeKey::by_id(g.use_mode.clone())?;
    let wait = Wait {
        lit_ms: g.use_lit_ms,
        poll_ms: ctx.cfg.timing.poll_ms,
    };
    let was = key.on()?;
    let used = key
        .set(true, wait)
        .and_then(|()| read_after(g, &key.keyboard, p, was));
    joined(used, key.put_back(was, wait))
}

/// A reading of `memory` after a whole sweep with the mode turned on, for tests.
#[cfg(test)]
pub fn read_as(memory: Memory) -> Used {
    Used {
        hovers: 1,
        mode_was_on: false,
        layout: None,
        memory,
    }
}

#[cfg(test)]
mod tests;

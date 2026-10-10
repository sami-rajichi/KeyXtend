//! G10's key plan (roadmap G10): a step for every key that types no character at once, pressed as a screen reader
//! presses, with what target-window must log. Each step leaves the keyboard, the window and the layout as it found
//! them.
#![cfg(windows)]

use kx_mod_keyboard::{LangKey, Latch, Role};
use kx_module_api::ports::Scan;
use kx_platform_windows::keys;

/// What one step must leave in target-window's key log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    /// These scan codes go down in this order; Windows may add others between, such as the Ctrl before `AltGr`.
    Keys(Vec<u16>),
    /// No key goes down.
    Nothing,
}

impl Expect {
    /// True when the scan codes that went down, `downs`, meet this; each wanted code is matched as Windows reports it.
    pub fn met(&self, downs: &[u16]) -> bool {
        match self {
            Self::Nothing => downs.is_empty(),
            Self::Keys(want) => {
                let mut left = downs.iter();
                want.iter().all(|&w| left.any(|&d| d == keys::reported(w)))
            }
        }
    }
}

/// One step: the board keys pressed in turn, and what must come of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The board key the step checks.
    pub key: String,
    /// The board keys pressed, in order.
    pub presses: Vec<String>,
    /// What target-window must log.
    pub expect: Expect,
    /// True when the key must read as latched after its first press, and as let go after the last.
    pub toggles: bool,
}

/// The scan code of send key `probe`, which goes out after each modifier latches.
pub fn probe_scan(roles: &[(String, Role, Option<Scan>)], probe: &str) -> Result<Scan, String> {
    let found = roles
        .iter()
        .find(|(id, role, _)| id == probe && *role == Role::Send);
    found
        .and_then(|(.., sc)| *sc)
        .ok_or_else(|| format!("{probe} is no send key of the product's board"))
}

/// A step for key `id`, which presses `presses`.
fn step(id: &str, presses: &[&str], expect: Expect, toggles: bool) -> Step {
    Step {
        key: id.to_string(),
        presses: presses.iter().map(ToString::to_string).collect(),
        expect,
        toggles,
    }
}

/// The step for key `id` with `role` and scan code `sc`; none for a character key or the language key.
fn step_for(
    id: &str,
    role: Role,
    sc: Option<Scan>,
    probe: (&str, Scan),
    stay: bool,
) -> Option<Step> {
    let code = sc.map(Scan::code);
    match (role, code) {
        (Role::Send, Some(c)) => Some(step(id, &[id], Expect::Keys(vec![c]), false)),
        (Role::Caps, Some(c)) => Some(step(id, &[id, id], Expect::Keys(vec![c, c]), false)),
        // Win's shortcuts reach Windows itself, so it is latched and let go with nothing between.
        (Role::Latch(Latch::Win), Some(_)) => Some(step(id, &[id, id], Expect::Nothing, true)),
        (Role::Latch(_), Some(c)) => {
            let keys = Expect::Keys(vec![c, probe.1.code()]);
            let presses: &[&str] = if stay {
                &[id, probe.0, id]
            } else {
                &[id, probe.0]
            };
            Some(step(id, presses, keys, true))
        }
        (Role::Later, _) => Some(step(id, &[id], Expect::Nothing, false)),
        _ => None,
    }
}

/// The steps for the board's `roles`, in board order: send keys, Caps Lock, the modifiers with `probe` after each,
/// and the keys of later versions. `stay` is true when modifiers stay latched after a key.
pub fn plan(
    roles: &[(String, Role, Option<Scan>)],
    probe: &str,
    stay: bool,
) -> Result<Vec<Step>, String> {
    let probe = (probe, probe_scan(roles, probe)?);
    let steps = roles
        .iter()
        .filter_map(|(id, role, sc)| step_for(id, *role, *sc, probe, stay));
    Ok(steps.collect())
}

/// What a press of the language key must do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LangWant {
    /// One layout or none: nothing.
    Nothing,
    /// Carousel and Cycle: the app in front steps to the next layout.
    Steps,
    /// List: the list opens, and the panel's Esc key closes it.
    Opens,
}

/// What the language key of `design` does with `layouts` installed.
pub fn lang_want(design: LangKey, layouts: usize) -> LangWant {
    match (layouts > 1, design) {
        (false, _) => LangWant::Nothing,
        (true, LangKey::List) => LangWant::Opens,
        (true, _) => LangWant::Steps,
    }
}

#[cfg(test)]
mod tests;

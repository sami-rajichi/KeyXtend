//! G10's key plan (roadmap G10): a step for every key but the character keys and the language key, pressed as a
//! screen reader presses, with what target-window must log. Each step leaves the keyboard, the window and the layout
//! as it found them.
#![cfg(windows)]

use kx_mod_keyboard::{LangKey, Latch, Role};
use kx_module_api::ports::Scan;
use kx_platform_windows::keys;

/// Why a key is left to the owner's Narrator and NVDA check.
const NO_TOGGLE: &str = "its latch shows no toggle state through UI Automation";
/// Why a failed step's key is left to the owner: its latch read on, or nothing, after.
const MAY_BE_ON: &str = "the step failed and its latch may still be on";

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
    /// True when the last press puts the key's mode back, so it goes out even after the mouse guard stopped the gate.
    pub puts_back: bool,
}

impl Step {
    /// True when this step's latch reads as still on, `on`, after the step failed, so one more press lets it go.
    pub fn left_latched(&self, on: Option<bool>) -> bool {
        self.toggles && on == Some(true)
    }

    /// Why the owner checks this step's key by hand, if at all: its latch never read (`unread`), or the step `failed`
    /// and its latch did not read as let go `after`.
    pub fn owner_why(
        &self,
        unread: bool,
        failed: bool,
        after: Option<bool>,
    ) -> Option<&'static str> {
        if !self.toggles {
            None
        } else if unread {
            Some(NO_TOGGLE)
        } else if failed && after != Some(false) {
            Some(MAY_BE_ON)
        } else {
            None
        }
    }
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
        puts_back: false,
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
        // A mouse-assist key flips its mode and back; its light follows the mouse module's answer, so it is not read.
        (Role::Assist(_), _) => Some(Step {
            puts_back: true,
            ..step(id, &[id, id], Expect::Nothing, false)
        }),
        (Role::Later, _) => Some(step(id, &[id], Expect::Nothing, false)),
        _ => None,
    }
}

/// The steps for the board's `roles`, in board order: send keys, Caps Lock, the modifiers with `probe` after each,
/// the mouse-assist keys twice, and the keys of later versions. `stay` is true when modifiers stay latched after a key.
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

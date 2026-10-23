//! G10's key plan (roadmap G10): a step for every key but the character keys, the language key and the Power key,
//! pressed as a screen reader presses, with what target-window must log. Each step leaves the keyboard, the window
//! and the layout as it found them.
#![cfg(windows)]

use kx_mod_keyboard::{LangKey, Latch, Role};
use kx_module_api::ports::Scan;
use kx_platform_windows::keys;

/// Why a key is left to the owner's Narrator and NVDA check.
const NO_TOGGLE: &str = "its latch shows no toggle state through UI Automation";
/// Why a failed step's key is left to the owner: its latch read on, or nothing, after.
const MAY_BE_ON: &str = "the step failed and its latch may still be on";
/// The most presses Win's undo needs: let go a spare that may predate Win, latch it, let Win go, let the spare go.
const UNDO_PRESSES: usize = 4;

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
    /// The latch pressed between the key's presses, so the key lets go used and never goes out alone: Win's.
    pub spare: Option<String>,
}

impl Step {
    /// True when this step's latch reads as still on, `on`, after the step failed, so one more press lets it go; a
    /// step with a spare lets go by `undo` instead.
    pub fn left_latched(&self, on: Option<bool>) -> bool {
        self.toggles && self.spare.is_none() && on == Some(true)
    }

    /// Refuses this step while its spare reads `on`, or cannot be read: the key pressed after it would go out alone.
    pub fn ready(&self, on: Option<bool>) -> Result<(), String> {
        match &self.spare {
            Some(id) if on != Some(false) => Err(format!(
                "{id} did not read as let go, so {} could go out alone",
                self.key
            )),
            _ => Ok(()),
        }
    }

    /// The spare and why the owner checks it by hand, if at all: the step `failed` and it did not read as let go
    /// `after`.
    pub fn spare_why(&self, failed: bool, after: Option<bool>) -> Option<(&str, &'static str)> {
        let id = self.spare.as_deref()?;
        (failed && after != Some(false)).then_some((id, MAY_BE_ON))
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

/// The first latch key but Win, which G10 latches between Win's presses so a lone Win never opens Start.
fn spare(roles: &[(String, Role, Option<Scan>)]) -> Result<&str, String> {
    let other = |role: &Role| matches!(role, Role::Latch(r) if r.latch != Latch::Win);
    let found = roles
        .iter()
        .find(|(_, role, sc)| other(role) && sc.is_some());
    found
        .map(|(id, ..)| id.as_str())
        .ok_or_else(|| "the product's board has no latch key but Win".to_string())
}

/// A step for key `id`, which presses `presses`.
fn step(id: &str, presses: &[&str], expect: Expect, toggles: bool) -> Step {
    Step {
        key: id.to_string(),
        presses: presses.iter().map(ToString::to_string).collect(),
        expect,
        toggles,
        puts_back: false,
        spare: None,
    }
}

/// The step for key `id` with `role` and scan code `sc`; none for a character key, the language key or the Power key.
/// Win's step latches `spare` between its presses.
fn step_for(
    id: &str,
    role: Role,
    sc: Option<Scan>,
    probe: (&str, Scan),
    spare: &str,
    stay: bool,
) -> Option<Step> {
    let code = sc.map(Scan::code);
    match (role, code) {
        (Role::Send, Some(c)) => Some(step(id, &[id], Expect::Keys(vec![c]), false)),
        (Role::Caps, Some(c)) => Some(step(id, &[id, id], Expect::Keys(vec![c, c]), false)),
        // Win's shortcuts reach Windows, and Win let go unused opens Start: `spare` uses it, so nothing goes down.
        (Role::Latch(r), Some(_)) if r.latch == Latch::Win => Some(Step {
            spare: Some(spare.to_string()),
            ..step(id, &[id, spare, id, spare], Expect::Nothing, true)
        }),
        (Role::Latch(_), Some(c)) => {
            let keys = Expect::Keys(vec![c, probe.1.code()]);
            let presses: &[&str] = if stay {
                &[id, probe.0, id]
            } else {
                &[id, probe.0]
            };
            Some(step(id, presses, keys, true))
        }
        // A mouse-assist key flips its mode and back, its light following the mouse module's answer, so it is not
        // read; the Shortcuts key shows the layer and hides it again. Nothing goes out.
        (Role::Assist(_) | Role::Layer, _) => Some(Step {
            puts_back: true,
            ..step(id, &[id, id], Expect::Nothing, false)
        }),
        (Role::Later, _) => Some(step(id, &[id], Expect::Nothing, false)),
        // The Power key is never pressed: with the power module running, it opens the system's shut-down dialog.
        _ => None,
    }
}

/// The steps for the board's `roles`, in board order: send keys, Caps Lock, the modifiers with `probe` after each
/// (Win with another latch), the mouse-assist keys and the Shortcuts key twice, and the keys of later versions.
/// `stay` is true when modifiers stay latched after a key.
pub fn plan(
    roles: &[(String, Role, Option<Scan>)],
    probe: &str,
    stay: bool,
) -> Result<Vec<Step>, String> {
    let probe = (probe, probe_scan(roles, probe)?);
    let spare = spare(roles)?;
    let steps = roles
        .iter()
        .filter_map(|(id, role, sc)| step_for(id, *role, *sc, probe, spare, stay));
    Ok(steps.collect())
}

/// The next press of a failed Win step's undo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Undo {
    /// Press the spare to let it go; it sends nothing.
    Free,
    /// Press the spare to latch it, which uses Win.
    Use,
    /// Press Win, right after the spare latched over it, so it lets go used and sends nothing.
    Key,
}

/// The next undo press, from what Win (`key`) and the spare read now and whether the last press latched the spare
/// over Win (`used`); none when both read as let go or one cannot be read.
fn next(key: Option<bool>, spare: Option<bool>, used: bool) -> Option<Undo> {
    match (key, spare) {
        (Some(true), Some(true)) if used => Some(Undo::Key),
        // A spare already on may have latched before Win, so it would not use it: it lets go first.
        (_, Some(true)) => Some(Undo::Free),
        (Some(true), Some(false)) => Some(Undo::Use),
        _ => None,
    }
}

/// Lets a failed Win step's latches go, `keys` being Win's and its spare's ids, read by `read` and pressed by `press`
/// (false when it did not go): Win only right after the spare latched over it, and a press whose key does not read
/// flipped after ends it. Both latches as they read after.
pub fn undo(
    keys: (&str, &str),
    mut read: impl FnMut(&str) -> Option<bool>,
    mut press: impl FnMut(&str) -> bool,
) -> (Option<bool>, Option<bool>) {
    let mut used = false;
    for _ in 0..UNDO_PRESSES {
        let (key, spare) = (read(keys.0), read(keys.1));
        let Some(go) = next(key, spare, used) else {
            break;
        };
        let (id, was) = if go == Undo::Key {
            (keys.0, key)
        } else {
            (keys.1, spare)
        };
        if !press(id) || read(id) != was.map(|on| !on) {
            break;
        }
        used = go == Undo::Use;
    }
    (read(keys.0), read(keys.1))
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

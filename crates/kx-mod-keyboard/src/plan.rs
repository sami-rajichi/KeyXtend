//! What a click on a key sends, chosen from the layout shown (ADR-0009): characters as text, and dead keys,
//! the key after one and shortcuts as scan codes, so the app's own layout adds accents and runs shortcuts.

use crate::latch::Latches;
use crate::legend::{LegendMods, slot};
use crate::spec::{Latch, Role};
use kx_module_api::ports::{KeyChars, Scan, Slot, TextRoute};
use kx_module_api::{REDACTED_MARKER, Redacted};
use std::collections::BTreeMap;
use std::fmt;

/// How one click reaches the app in front.
#[derive(Clone, PartialEq, Eq)]
pub enum Route {
    /// A key alone.
    Tap(Scan),
    /// Keys held in order around a key.
    Chord(Vec<Scan>, Scan),
    /// Text, as Unicode or as key presses in the app's own layout.
    Text(Redacted<String>, TextRoute),
}

/// Shows the kind of route only, never a key or the text, since a run of routes would show what was typed.
impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tap(_) => write!(f, "Tap({REDACTED_MARKER})"),
            Self::Chord(held, _) => write!(f, "Chord({} held, {REDACTED_MARKER})", held.len()),
            Self::Text(_, by) => write!(f, "Text({REDACTED_MARKER}, {by:?})"),
        }
    }
}

/// What a sent key does to a dead key's wait.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Wait {
    /// A dead key went out: the next character goes as a key press, so the app adds the accent.
    Start,
    /// A key that types went out, so nothing waits.
    End,
    /// A key went out while a dead key waits: the wait ends if the layout says the key types anything.
    Ask,
}

/// What a click on a key does.
#[derive(Clone, PartialEq, Eq)]
pub enum Plan {
    /// Sends `route`; then the latches let go as their mode says, and a dead key's wait follows `wait`.
    Send {
        /// How it reaches the app.
        route: Route,
        /// What it does to a dead key's wait.
        wait: Wait,
    },
    /// Taps Caps Lock; the latches and a dead key's wait stay.
    Caps(Scan),
    /// Latches or lets go of a modifier, from the key that sends the scan code.
    Latch(Latch, Scan),
    /// The language key: the face steps or opens the list.
    Lang,
    /// A key from a later version, or one without the code its role needs: nothing.
    Later,
    /// A character key that types nothing with the latched Shift or `AltGr`: nothing, and the latches and wait stay.
    Empty,
}

/// Shows no scan code, not even of Caps Lock or a modifier, as the rule for logs asks.
impl fmt::Debug for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send { route, wait } => f
                .debug_struct("Send")
                .field("route", route)
                .field("wait", wait)
                .finish(),
            Self::Caps(_) => f.write_str("Caps"),
            Self::Latch(l, _) => write!(f, "Latch({l:?})"),
            Self::Lang => f.write_str("Lang"),
            Self::Later => f.write_str("Later"),
            Self::Empty => f.write_str("Empty"),
        }
    }
}

/// What a plan is made from.
#[derive(Copy, Clone)]
pub struct Now<'a> {
    /// What each key types in the layout shown, by scan code.
    pub keys: &'a BTreeMap<Scan, KeyChars>,
    /// The latched modifiers.
    pub latches: &'a Latches,
    /// Caps Lock is on.
    pub caps: bool,
    /// A dead key waits for the next key.
    pub dead: bool,
    /// The `key_apps` setting.
    pub apps: &'a [String],
    /// The app in front, asked only for a click that types text while `apps` names any app.
    pub app: &'a dyn Fn() -> Option<String>,
}

/// What a click on a key with `role` and scan code `sc` does now.
#[must_use]
pub fn plan(role: Role, sc: Option<Scan>, now: &Now<'_>) -> Plan {
    match (role, sc) {
        (Role::Char, Some(sc)) => char_plan(sc, now),
        (Role::Send, Some(sc)) => shortcut(sc, now),
        (Role::Caps, Some(sc)) => Plan::Caps(sc),
        (Role::Latch(l), Some(sc)) => Plan::Latch(l, sc),
        (Role::Lang, _) => Plan::Lang,
        _ => Plan::Later,
    }
}

/// Text goes as key presses to an app in `apps`, matched by file name without case; else as Unicode.
/// The app in front is asked only when `apps` names any app.
#[must_use]
pub fn text_route(apps: &[String], app: impl FnOnce() -> Option<String>) -> TextRoute {
    if apps.is_empty() {
        return TextRoute::Unicode;
    }
    let app = app().map(|a| a.to_lowercase());
    let listed = app.is_some_and(|a| apps.iter().any(|k| k.trim().to_lowercase() == a));
    if listed {
        TextRoute::Keys
    } else {
        TextRoute::Unicode
    }
}

/// A character key: a shortcut with Ctrl, Alt or Win or for a key missing from the table, nothing for an empty
/// key, a key press for a dead key, for the key after one and for a base that types nothing, else its text.
fn char_plan(sc: Scan, now: &Now<'_>) -> Plan {
    let m = LegendMods::of(now.latches, now.caps, now.keys);
    let Some(k) = now.keys.get(&sc).filter(|_| !m.shortcut) else {
        return shortcut(sc, now);
    };
    // No slot is `is_empty`'s rule: the key shows no letter, so a click sends nothing.
    let Some(s) = slot(k, m) else {
        return Plan::Empty;
    };
    let press = |wait| Plan::Send {
        route: keys(now.latches.held(&unheld(s, m)), sc),
        wait,
    };
    let text = k.get(s);
    if k.is_dead(s) {
        press(Wait::Start)
    } else if text.is_empty() {
        press(asked(now))
    } else if now.dead {
        press(Wait::End)
    } else {
        let by = text_route(now.apps, now.app);
        let route = Route::Text(Redacted::new(text.to_owned()), by);
        Plan::Send {
            route,
            wait: Wait::End,
        }
    }
}

/// Key `sc` with every latched modifier held around it, for the app's own layout to read.
fn shortcut(sc: Scan, now: &Now<'_>) -> Plan {
    Plan::Send {
        route: keys(now.latches.held(&[]), sc),
        wait: asked(now),
    }
}

/// A key that may type nothing: while a dead key waits, the layout must say.
fn asked(now: &Now<'_>) -> Wait {
    if now.dead { Wait::Ask } else { Wait::End }
}

/// `key` alone, or with `held` around it.
fn keys(held: Vec<Scan>, key: Scan) -> Route {
    if held.is_empty() {
        Route::Tap(key)
    } else {
        Route::Chord(held, key)
    }
}

/// The latches slot `s` ignores, so a key press types what its legend shows: Shift, under `AltGr`.
fn unheld(s: Slot, m: LegendMods) -> Vec<Latch> {
    let shifted = matches!(s, Slot::Shift | Slot::CapsShift);
    if m.shift && !shifted {
        vec![Latch::Shift]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests;

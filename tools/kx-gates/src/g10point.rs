//! G10 by point: each key G10 finds by id is hit-tested at the centre of its box, as Narrator's mouse mode reads what
//! is under the pointer. `ElementFromPoint` takes a point, so the pointer never moves.
#![cfg(windows)]

use serde_json::{Value, json};
use spike_core::uia;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Accessibility::IUIAutomationElement;

use crate::keymap::{KeyMap, KeyRead, Reader};
use crate::win;

/// Why a key was not hit-tested.
const NO_BOX: &str = "no box on screen";

/// What UI Automation found at a key's centre.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    /// Its UI Automation id; a key's board id.
    id: String,
    /// What a screen reader says; kept only for the keyboard's own elements.
    name: String,
    /// Its class and control type.
    kind: String,
    /// The process that shows it.
    pid: u32,
    /// Its own window handle; 0 for an element drawn inside a window.
    hwnd: isize,
}

/// The window `WindowFromPoint` gives at a key's centre: its handle, process, and class with program.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct Under {
    hwnd: isize,
    pid: u32,
    window: String,
}

impl Under {
    /// The window under screen point `p`, child windows included.
    fn at(p: POINT) -> Self {
        let w = win::under(p);
        Self {
            hwnd: win::key(w),
            pid: win::owner(w).0,
            window: win::describe(w),
        }
    }
}

/// One key hit-tested: where, what UI Automation found or why it found nothing, and the window there.
struct Probe {
    key: String,
    at: Option<POINT>,
    hit: Result<Hit, String>,
    under: Option<Under>,
}

/// How one key fared when hit-tested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// The key itself, in the keyboard's process.
    Matched,
    /// Another element of the keyboard's process, such as its window or another key.
    WrongElement,
    /// An element of another process, such as the window behind.
    WrongWindow,
    /// No box on screen, or UI Automation found nothing there.
    Unread,
}

impl Outcome {
    /// Every outcome, in the order the result block counts them.
    const ALL: [Self; 4] = [
        Self::Matched,
        Self::WrongElement,
        Self::WrongWindow,
        Self::Unread,
    ];

    /// The word the result block uses.
    fn word(self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::WrongElement => "wrong_element",
            Self::WrongWindow => "wrong_window",
            Self::Unread => "unread",
        }
    }
}

/// Each board key in `ids` that the walk `read` found, with the centre of its box; none when it has no area.
fn aims(read: &[KeyRead], ids: &[String]) -> Vec<(String, Option<POINT>)> {
    let map = KeyMap::new(read.iter().map(|k| (k.id.clone(), k.rect)).collect());
    let found = |id: &&String| read.iter().any(|k| &k.id == *id);
    ids.iter()
        .filter(found)
        .map(|id| (id.clone(), map.at(id)))
        .collect()
}

/// How key `key` fared: it matches when `hit` is that key, shown by keyboard process `kb`.
fn judge(key: &str, kb: u32, hit: Option<&Hit>) -> Outcome {
    match hit {
        None => Outcome::Unread,
        Some(h) if h.pid != kb => Outcome::WrongWindow,
        Some(h) if h.id != key => Outcome::WrongElement,
        Some(_) => Outcome::Matched,
    }
}

impl Probe {
    /// How this key fared with keyboard process `kb`.
    fn outcome(&self, kb: u32) -> Outcome {
        judge(&self.key, kb, self.hit.as_ref().ok())
    }

    /// This key's result line; another app's element keeps no name, as it may hold the owner's text.
    fn line(&self, kb: u32) -> Value {
        let hit = self.hit.as_ref().ok().map(|h| {
            json!({ "id": h.id, "name": (h.pid == kb).then_some(&h.name), "kind": h.kind, "pid": h.pid,
                "hwnd": h.hwnd })
        });
        json!({
            "key": self.key, "at": self.at.map(|p| [p.x, p.y]), "outcome": self.outcome(kb).word(), "hit": hit,
            "error": self.hit.as_ref().err(), "under": self.under,
        })
    }
}

/// The `by_point` block over `probes` with keyboard process `kb`: a count per outcome, the first `shown` failures in
/// full, and `ok` when every key matched.
fn tally(probes: &[Probe], kb: u32, shown: usize) -> Value {
    let count = |o: Outcome| probes.iter().filter(|p| p.outcome(kb) == o).count();
    let failed = probes.iter().filter(|p| p.outcome(kb) != Outcome::Matched);
    let failures: Vec<Value> = failed.take(shown).map(|p| p.line(kb)).collect();
    let mut v = json!({ "keys": probes.len(), "failures": failures });
    for o in Outcome::ALL {
        v[o.word()] = json!(count(o));
    }
    v["ok"] = json!(!probes.is_empty() && count(Outcome::Matched) == probes.len());
    v
}

/// What `el` says about itself; its id and process must read, the rest may be blank.
#[allow(
    unsafe_code,
    reason = "UI Automation property reads on a live element."
)]
fn read_hit(el: &IUIAutomationElement) -> Result<Hit, String> {
    // SAFETY: plain property reads on a live element; a failed read gives an error or a blank.
    let (id, pid, hwnd) = unsafe {
        let id = el
            .CurrentAutomationId()
            .map_err(|e| format!("UIA id: {e}"))?;
        let pid = el
            .CurrentProcessId()
            .map_err(|e| format!("UIA process: {e}"))?;
        (id, pid, el.CurrentNativeWindowHandle().unwrap_or_default())
    };
    Ok(Hit {
        id: id.to_string(),
        name: uia::name(el),
        kind: uia::kind(el),
        pid: u32::try_from(pid).unwrap_or_default(),
        hwnd: win::key(hwnd),
    })
}

/// Hit-tests key `key` at `at` with `reader`; a key with no box is not hit-tested.
fn probe(reader: &Reader, key: String, at: Option<POINT>) -> Probe {
    let el = at
        .ok_or_else(|| NO_BOX.to_string())
        .and_then(|p| reader.at(p));
    Probe {
        key,
        at,
        hit: el.and_then(|el| read_hit(&el)),
        under: at.map(Under::at),
    }
}

/// Hit-tests each board key in `ids` that the walk `read` found on keyboard window `kb`: the `by_point` block, with
/// the first `shown` failures in full.
pub fn run(reader: &Reader, kb: HWND, read: &[KeyRead], ids: &[String], shown: usize) -> Value {
    let pid = win::owner(kb).0;
    let probes: Vec<Probe> = aims(read, ids)
        .into_iter()
        .map(|(key, at)| probe(reader, key, at))
        .collect();
    tally(&probes, pid, shown)
}

#[cfg(test)]
mod tests;

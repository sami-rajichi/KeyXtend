//! G10 by point: each key G10 finds by id is hit-tested at the centre of its box, as Narrator's mouse mode reads what
//! is under the pointer. `ElementFromPoint` takes a point, so the pointer never moves.
#![cfg(windows)]

use kx_platform_windows::uia;
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Accessibility::IUIAutomationElement;

use crate::keymap::{KeyMap, KeyRead, Reader};
use crate::win;

/// Why a key was not hit-tested.
const NO_BOX: &str = "no box on screen";

/// A keyboard element's UI Automation id, a key's board id, and what a screen reader says.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Label {
    id: String,
    name: String,
}

/// What UI Automation found at a key's centre.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    /// Its label, read only for the keyboard's own elements, as another app's may hold the owner's text.
    label: Option<Label>,
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
        Some(h) if h.label.as_ref().is_none_or(|l| l.id != key) => Outcome::WrongElement,
        Some(_) => Outcome::Matched,
    }
}

impl Probe {
    /// How this key fared with keyboard process `kb`.
    fn outcome(&self, kb: u32) -> Outcome {
        judge(&self.key, kb, self.hit.as_ref().ok())
    }

    /// This key's result line; another app's element has no id or name in it.
    fn line(&self, kb: u32) -> Value {
        let hit = self.hit.as_ref().ok().map(|h| {
            let label = h.label.as_ref();
            json!({ "id": label.map(|l| &l.id), "name": label.map(|l| &l.name), "kind": h.kind, "pid": h.pid,
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

/// What `read` gives, read only when keyboard process `kb` shows the element, process `pid`; a gone keyboard is 0.
fn own<T>(
    pid: u32,
    kb: u32,
    read: impl FnOnce() -> Result<T, String>,
) -> Result<Option<T>, String> {
    (pid == kb && kb != 0).then(read).transpose()
}

/// The id and name of `el`, a keyboard element; its id must read, its name may be blank.
#[allow(
    unsafe_code,
    reason = "a UI Automation property read on a live element."
)]
fn read_label(el: &IUIAutomationElement) -> Result<Label, String> {
    // SAFETY: a plain property read on a live element; a failed read gives an error.
    let id = unsafe { el.CurrentAutomationId() }.map_err(|e| format!("UIA id: {e}"))?;
    Ok(Label {
        id: id.to_string(),
        name: uia::name(el),
    })
}

/// What `el` says about itself, its label only when keyboard process `kb` shows it; its process must read.
#[allow(
    unsafe_code,
    reason = "UI Automation property reads on a live element."
)]
fn read_hit(el: &IUIAutomationElement, kb: u32) -> Result<Hit, String> {
    // SAFETY: plain property reads on a live element; a failed read gives an error or a blank.
    let (pid, hwnd) = unsafe {
        let pid = el
            .CurrentProcessId()
            .map_err(|e| format!("UIA process: {e}"))?;
        (pid, el.CurrentNativeWindowHandle().unwrap_or_default())
    };
    let pid = u32::try_from(pid).unwrap_or_default();
    Ok(Hit {
        label: own(pid, kb, || read_label(el))?,
        kind: uia::kind(el),
        pid,
        hwnd: win::key(hwnd),
    })
}

/// Hit-tests key `key` at `at` with `reader` for keyboard process `kb`; a key with no box is not hit-tested.
fn probe(reader: &Reader, kb: u32, key: String, at: Option<POINT>) -> Probe {
    let el = at
        .ok_or_else(|| NO_BOX.to_string())
        .and_then(|p| reader.at(p));
    Probe {
        key,
        at,
        hit: el.and_then(|el| read_hit(&el, kb)),
        under: at.map(Under::at),
    }
}

/// Hit-tests each board key in `ids` that the walk `read` found on keyboard window `kb`: the `by_point` block, with
/// the first `shown` failures in full.
pub fn run(reader: &Reader, kb: HWND, read: &[KeyRead], ids: &[String], shown: usize) -> Value {
    let pid = win::owner(kb).0;
    let probes: Vec<Probe> = aims(read, ids)
        .into_iter()
        .map(|(key, at)| probe(reader, pid, key, at))
        .collect();
    tally(&probes, pid, shown)
}

#[cfg(test)]
mod tests;

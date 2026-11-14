//! The ids a stream watches: each must hear a note within its wait, once or again after each note until a last one.

use kx_module_api::Mono;
use std::collections::BTreeMap;
use std::time::Duration;

/// How long a stream waits on an ask's id before it ends a silent worker.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Wait {
    /// No wait: the id may stay silent, and an earlier wait on it ends.
    Off,
    /// The next note on the id must come within this time; then it may stay silent.
    Once(Duration),
    /// A note on the id must come within this time, and again after each note, until a last note.
    Each(Duration),
}

/// One watched id.
#[derive(Copy, Clone, Debug)]
struct Watch {
    /// How long the id may stay silent.
    wait: Duration,
    /// True when each note starts the wait again, false when the first note ends it.
    each: bool,
    /// When it is late.
    due: Mono,
}

/// Every watched id, by id.
#[derive(Debug, Default)]
pub(super) struct Watches(BTreeMap<u32, Watch>);

impl Watches {
    /// Watches `id` as `wait` says from `now`.
    pub(super) fn set(&mut self, id: u32, wait: Wait, now: Mono) {
        let (wait, each) = match wait {
            Wait::Off => {
                self.0.remove(&id);
                return;
            }
            Wait::Once(wait) => (wait, false),
            Wait::Each(wait) => (wait, true),
        };
        let due = now.saturating_add(wait);
        self.0.insert(id, Watch { wait, each, due });
    }

    /// Takes a note on `id` at `now`: its wait starts again, or ends after one note or at a `last` one. False when
    /// `id` is not watched.
    pub(super) fn heard(&mut self, id: u32, now: Mono, last: bool) -> bool {
        let Some(watch) = self.0.get_mut(&id) else {
            return false;
        };
        if watch.each && !last {
            watch.due = now.saturating_add(watch.wait);
        } else {
            self.0.remove(&id);
        }
        true
    }

    /// When the first watched id is late, if any is watched.
    pub(super) fn due(&self) -> Option<Mono> {
        self.0.values().map(|watch| watch.due).min()
    }

    /// True once a watched id is late at `now`.
    pub(super) fn late(&self, now: Mono) -> bool {
        self.due().is_some_and(|due| due <= now)
    }

    /// True when no id is watched.
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Watches nothing more.
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
}

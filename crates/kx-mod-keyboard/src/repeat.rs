//! Key repeat while a key is held (spec §4.6): the press types at once, as the face does; a hold types again at the
//! delay, then once a beat, until the pointer lets go. It makes no OS call: each input brings the face clock's time.

use kx_module_api::Mono;
use std::fmt;
use std::time::Duration;

/// Where one hold is.
#[derive(Clone)]
enum Phase<K> {
    /// No key is held.
    Idle,
    /// `key` is held and types again at `due`.
    Held { key: K, due: Mono },
}

/// The hold of one key at a time, which says when it types again by the settings' delay and pace.
#[derive(Clone)]
pub struct Repeat<K> {
    delay: Duration,
    every: Duration,
    phase: Phase<K>,
}

/// Shows the phase only, never the key, since a run of keys would show what was typed.
impl<K> fmt::Debug for Repeat<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let phase = match self.phase {
            Phase::Idle => "idle",
            Phase::Held { .. } => "held",
        };
        f.debug_struct("Repeat")
            .field("phase", &phase)
            .finish_non_exhaustive()
    }
}

impl<K: Clone> Repeat<K> {
    /// Nothing held; a held key first repeats after `delay`, then once every `every`.
    #[must_use]
    pub fn new(delay: Duration, every: Duration) -> Self {
        Self {
            delay,
            every,
            phase: Phase::Idle,
        }
    }

    /// `key` went down at `now` and typed: it types again after the delay. A key still held is dropped.
    pub fn press(&mut self, key: K, now: Mono) {
        self.phase = Phase::Held {
            key,
            due: now.saturating_add(self.delay),
        };
    }

    /// The hold timer at `now`: the key to type when a repeat is due. A late tick types once and skips the beats it
    /// missed, never a catch-up burst.
    pub fn tick(&mut self, now: Mono) -> Option<K> {
        let Phase::Held { key, due } = &mut self.phase else {
            return None;
        };
        if now < *due {
            return None;
        }
        *due = next(*due, now, self.every);
        Some(key.clone())
    }

    /// The pointer let go, Qt took the press away, the keyboard hid, or the face ended the burst: nothing more types.
    pub fn leave(&mut self) {
        self.phase = Phase::Idle;
    }

    /// True while a key is held.
    #[must_use]
    pub fn held(&self) -> bool {
        matches!(self.phase, Phase::Held { .. })
    }

    /// The key held, or none.
    #[must_use]
    pub fn key(&self) -> Option<&K> {
        match &self.phase {
            Phase::Held { key, .. } => Some(key),
            Phase::Idle => None,
        }
    }
}

/// The first beat after `now` on the pace that starts at `due`.
fn next(due: Mono, now: Mono, every: Duration) -> Mono {
    let step = every.as_micros().max(1);
    let beats = now.saturating_since(due).as_micros() / step + 1;
    let ahead = u64::try_from(beats.saturating_mul(step)).unwrap_or(u64::MAX);
    due.saturating_add(Duration::from_micros(ahead))
}

#[cfg(test)]
mod tests;

//! Key repeat while a key is held (spec §4.6): a short click types once, on let-go; a hold types at the delay, then
//! once a beat, until the pointer lets go or leaves. It makes no OS call: each input brings the face clock's time.

use kx_module_api::Mono;
use std::borrow::Borrow;
use std::fmt;
use std::time::Duration;

/// Where one hold is.
#[derive(Clone, PartialEq, Eq)]
enum Phase<K> {
    /// No key is held.
    Idle,
    /// `key` went down at `since`; it types at the delay when it `repeats`, else only on let-go.
    Held { key: K, since: Mono, repeats: bool },
    /// `key` has typed and types again at `due`.
    Repeating { key: K, due: Mono },
}

/// The hold of one key at a time, which says when it types by the settings' delay and pace.
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
            Phase::Repeating { .. } => "repeating",
        };
        f.debug_struct("Repeat")
            .field("phase", &phase)
            .finish_non_exhaustive()
    }
}

impl<K: Clone + PartialEq> Repeat<K> {
    /// Nothing held; a held key first repeats after `delay`, then once every `every`.
    #[must_use]
    pub fn new(delay: Duration, every: Duration) -> Self {
        Self {
            delay,
            every,
            phase: Phase::Idle,
        }
    }

    /// `key` went down at `now`: one that `repeats` types at the delay, any other only on let-go. A key still held
    /// is dropped, typing nothing.
    pub fn press(&mut self, key: K, repeats: bool, now: Mono) {
        self.phase = Phase::Held {
            key,
            since: now,
            repeats,
        };
    }

    /// The hold timer at `now`: the key to type when a repeat is due. A late tick types once and skips the beats it
    /// missed, never a catch-up burst.
    pub fn tick(&mut self, now: Mono) -> Option<K> {
        let (key, due) = match &self.phase {
            Phase::Held {
                key,
                since,
                repeats: true,
            } => (key, since.saturating_add(self.delay)),
            Phase::Repeating { key, due } => (key, *due),
            _ => return None,
        };
        if now < due {
            return None;
        }
        let key = key.clone();
        let due = next(due, now, self.every);
        self.phase = Phase::Repeating {
            key: key.clone(),
            due,
        };
        Some(key)
    }

    /// The pointer let go of `key`: the key to type when it has not typed yet, as a click; a burst ends with nothing.
    pub fn let_go<Q>(&mut self, key: &Q) -> Option<K>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        match std::mem::replace(&mut self.phase, Phase::Idle) {
            Phase::Held { key: held, .. } if held.borrow() == key => Some(held),
            _ => None,
        }
    }

    /// The pointer left the key, the keyboard hid, or the face ended the burst: nothing more types.
    pub fn leave(&mut self) {
        self.phase = Phase::Idle;
    }

    /// True once the held key has typed and repeats on; only the tests ask.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn bursting(&self) -> bool {
        matches!(self.phase, Phase::Repeating { .. })
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

//! A held key's typing state (spec §4.6): which keys may repeat now, and the burst that keeps the latches down
//! until it ends.

use super::{Io, Typing};
use crate::legend::LegendMods;
use crate::plan::{Now, Plan, Wait, plan};

impl Typing {
    /// True when key `id` may repeat while held now: the board lets it, no Ctrl, Alt or Win acts on it, and it is
    /// not a dead key, so neither a shortcut nor an accent fires many times.
    #[must_use]
    pub fn repeats(&self, io: &Io<'_>, id: &str) -> bool {
        let known = self.keys.get(id).filter(|_| self.repeating.contains(id));
        let Some(&(role, sc)) = known else {
            return false;
        };
        let caps = io.source.caps_on();
        if LegendMods::of(&self.latches, caps, &io.table.keys).shortcut {
            return false;
        }
        let now = Now {
            keys: &io.table.keys,
            latches: &self.latches,
            caps,
            dead: self.dead,
            apps: &[],
            app: &|| None,
        };
        matches!(plan(role, sc, &now), Plan::Send { wait, .. } if wait != Wait::Start)
    }

    /// A burst starts: the latches stay down until `release`.
    pub fn hold(&mut self) {
        self.burst.get_or_insert(false);
    }

    /// The burst ends: if a key went out, the latches let go as their mode says. True when one went out.
    pub fn release(&mut self) -> bool {
        let went = self.burst.take() == Some(true);
        if went {
            self.latches.used();
        }
        went
    }
}

//! `Debug` for the state, which never shows the keys' legends or lights, as they tell the latches and Caps Lock.

use super::{KeyState, StateView};
use std::fmt;

/// Shows no field: its legends and lights tell the latches and Caps Lock, as `Latches` hides them.
impl fmt::Debug for KeyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyState").finish_non_exhaustive()
    }
}

/// Shows the language key, the list and the place, never the keys' states.
impl fmt::Debug for StateView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateView")
            .field("lang", &self.lang)
            .field("list", &self.list)
            .field("at", &self.at)
            .finish_non_exhaustive()
    }
}

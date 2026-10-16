//! The pad's tap machine of spec §5.2: a tap goes slow, the same tap again goes fast, a third stops, and another
//! way goes slow there. Pure: the actor passes each tap and publishes what changed.

use kx_module_api::ports::{Dir, ScrollChanged};

/// Where the pad stands.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Pad {
    /// Still.
    #[default]
    Idle,
    /// Scrolling one way.
    Scrolling {
        /// The way it scrolls.
        dir: Dir,
        /// It scrolls at the fast speed.
        fast: bool,
    },
}

/// What a tap changed; the actor publishes `ScrollChanged` after each.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Change {
    /// Scrolling began, slow, from still.
    Began,
    /// Scrolling turned another way, slow.
    Turned,
    /// The same way again went fast.
    Faster,
    /// Scrolling stopped.
    Stopped,
}

impl Change {
    /// The tap set a new way, so the tap's checks run (`Halt::unaimed`, `Halt::blocked`).
    #[must_use]
    pub const fn new_way(self) -> bool {
        matches!(self, Self::Began | Self::Turned)
    }
}

impl Pad {
    /// Takes a tap on the `dir` arrow and says what changed.
    pub fn tap(&mut self, dir: Dir) -> Change {
        let (slow, fast) = (
            Self::Scrolling { dir, fast: false },
            Self::Scrolling { dir, fast: true },
        );
        let (next, change) = match *self {
            Self::Idle => (slow, Change::Began),
            Self::Scrolling { dir: way, .. } if way != dir => (slow, Change::Turned),
            Self::Scrolling { fast: false, .. } => (fast, Change::Faster),
            Self::Scrolling { fast: true, .. } => (Self::Idle, Change::Stopped),
        };
        *self = next;
        change
    }

    /// Stops, for the Stop button or a stop rule; true when it was scrolling, so the change is published.
    pub fn stop(&mut self) -> bool {
        let was = self.scrolling();
        *self = Self::Idle;
        was
    }

    /// The way it scrolls; none while still.
    #[must_use]
    pub const fn dir(self) -> Option<Dir> {
        match self {
            Self::Idle => None,
            Self::Scrolling { dir, .. } => Some(dir),
        }
    }

    /// It scrolls at the fast speed.
    #[must_use]
    pub const fn fast(self) -> bool {
        matches!(self, Self::Scrolling { fast: true, .. })
    }

    /// It scrolls.
    #[must_use]
    pub const fn scrolling(self) -> bool {
        matches!(self, Self::Scrolling { .. })
    }

    /// The state the actor publishes while the module runs.
    #[must_use]
    pub const fn changed(self) -> ScrollChanged {
        ScrollChanged {
            live: true,
            dir: self.dir(),
            fast: self.fast(),
        }
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;

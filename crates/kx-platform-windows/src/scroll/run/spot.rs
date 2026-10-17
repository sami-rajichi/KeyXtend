//! Whose window a point lands on: the keyboard, our others that page or not, or another app's, from our marks.

/// Whose window a point lands on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Whose {
    /// The keyboard, whose entries the guard watches.
    Keyboard,
    /// Another of our windows that does not page, such as the banner.
    Ours,
    /// One of our windows that pages, such as an open list: a tap there turns its page.
    Pages,
    /// Another app's.
    Theirs,
}

impl Whose {
    /// Whose a root window is: ours and watched is the keyboard, ours and marked as paging pages, so another app's
    /// marks count for nothing.
    #[must_use]
    pub const fn of(ours: bool, watched: bool, pages: bool) -> Self {
        match (ours, watched, pages) {
            (false, ..) => Self::Theirs,
            (true, true, _) => Self::Keyboard,
            (true, false, true) => Self::Pages,
            (true, false, false) => Self::Ours,
        }
    }

    /// True for our windows the target passes over: every one that does not page.
    #[must_use]
    pub const fn passed(self) -> bool {
        matches!(self, Self::Keyboard | Self::Ours)
    }
}

/// The window at a point: its raw id and whose it is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spot {
    /// The window's raw id.
    pub window: u64,
    /// Whose it is.
    pub whose: Whose,
}

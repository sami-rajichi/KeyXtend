//! The window guard port: our windows never take the focus, take clicks only where asked, stack as asked and stay out
//! of captures while shielded; and the focus token that gives the focus back.

use super::UiError;
use kx_module_api::ports::Rect;

/// What a request to watch a window's entries found.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Watched {
    /// The guard watches the window's entries now.
    Held,
    /// The window shows, but the guard does not hold it yet.
    Unguarded,
}

/// Which window had the focus before ours, so it can get it back.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FocusToken(u64);

impl FocusToken {
    /// Wraps the adapter's raw window handle.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw window handle.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Keeps our windows from taking the focus, and shapes where they take clicks.
pub trait WindowGuard: Send + Sync {
    /// Guards our visible windows but those titled one of `spare` with their shadows and tips, which may leave the band
    /// as Snip's do for a system dialog (ADR-0059), and says how many. The one titled `top` goes last so it lands in
    /// front, as a hidden window shows again at its old place; call it on the UI thread, which owns the windows.
    ///
    /// # Errors
    /// `Refused` when the OS will not guard a window, as from another thread.
    fn guard_own(&self, spare: &[&str], top: Option<&str>) -> Result<usize, UiError>;

    /// Lets clicks outside `rects` pass through the window titled `title`, a call for the UI thread.
    /// The rects are physical pixels relative to the window's top-left corner.
    ///
    /// # Errors
    /// `NoWindow` for an unknown title, `Refused` when the OS says no.
    fn shape(&self, title: &str, rects: &[Rect]) -> Result<(), UiError>;

    /// Lifts our shown windows titled `title` alone in front of our others, leaving their owners where they are, as a
    /// tip does when it shows; a call for the UI thread.
    ///
    /// # Errors
    /// `NoWindow` while no such window shows, `Refused` when the OS says no.
    fn lift_alone(&self, title: &str) -> Result<(), UiError>;

    /// Until `unwatch_entries`, each time the pointer enters our guarded window titled `title`, keeps the last point it
    /// was at outside that window, for the scroll target (ADR-0029); a call for the UI thread. `Unguarded` while the
    /// window shows but the guard does not hold it yet; the caller asks again after a later guard try.
    ///
    /// # Errors
    /// `NoWindow` while no such window shows, `Refused` when the OS says no.
    fn watch_entries(&self, title: &str) -> Result<Watched, UiError>;

    /// Stops the entry watch on our windows titled `title`, shown or not, and forgets the point it kept, as the scroll
    /// module stops; a call for the UI thread.
    fn unwatch_entries(&self, title: &str);

    /// Puts our shown windows titled `title` just under our window titled `under`, never activating them, as the
    /// outline goes just under the keyboard; a call for the UI thread.
    ///
    /// # Errors
    /// `NoWindow` while either is not shown, `Refused` when the OS says no.
    fn lay_under(&self, title: &str, under: &str) -> Result<(), UiError>;

    /// Puts our windows titled `title`, shown or not, under every other window of ours that stays on top, never
    /// activating any, as Snip's frozen screens go under its bar and the keyboard (ADR-0059); a call for the UI thread.
    /// A window shows where it lies, so asking as one is about to show is enough.
    ///
    /// # Errors
    /// `NoWindow` without such a window, `Refused` when the OS says no.
    fn lay_low(&self, title: &str) -> Result<(), UiError>;

    /// Marks our shown window titled `title` as one that pages, as an open list does (ADR-0029). While a marked window
    /// shows, every platform's scroll target aims at it whatever point it kept, so a pad tap turns its page, and it
    /// passes over our other windows. The mark goes with the window; a call for the UI thread, as the window opens.
    ///
    /// # Errors
    /// `NoWindow` while no such window shows, `Refused` when the OS says no.
    fn mark_pages(&self, title: &str) -> Result<(), UiError>;

    /// Keeps our windows titled `title`, shown or not, out of every screen capture while `on`, or lets them back in; a
    /// call for the UI thread. A window shielded before it first shows is in no capture (ADR-0056 decision 8).
    ///
    /// # Errors
    /// `NoWindow` without such a window, `Refused` when the system cannot keep it out; every window of that title is
    /// then let back in, all or none, and the caller steps it aside.
    fn shield(&self, title: &str, on: bool) -> Result<(), UiError>;

    /// Notes the window that has the focus now.
    fn remember(&self) -> FocusToken;

    /// Gives the focus back to the noted window, if it still exists.
    fn give_back(&self, token: FocusToken);
}

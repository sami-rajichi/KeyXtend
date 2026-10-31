//! UI ports: the window guard, screens, the system look, backdrops, the one running copy, the tray icon, the heaps'
//! free memory, the speaker and the file manager's reveal. The data types are OS-neutral; adapters keep their own
//! handles behind them.

mod memory;
mod reveal;
mod sound;
mod tray;

pub use memory::Memory;
pub use reveal::{Reveal, RevealError};
pub use sound::{Clip, ClipId, Speaker, SpeakerConfig, level};
pub use tray::{BadImage, OnTray, Tray, TrayEvent, TrayIcon, TrayImage};

use kx_module_api::ports::{Point, Rect, Screen};
use std::fmt;
use thiserror::Error;

/// A colour as red, green, blue and alpha.
pub type Rgba = [u8; 4];

/// Why the OS could not do a UI request.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum UiError {
    /// No window of ours, or of the running copy, has that title.
    #[error("no such window")]
    NoWindow,
    /// The OS refused the call, with its error code.
    #[error("the system refused the call (code {0})")]
    Refused(i32),
}

/// What a request to watch a window's entries found.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Watched {
    /// The guard watches the window's entries now.
    Held,
    /// The window shows, but the guard does not hold it yet.
    Unguarded,
}

/// Another copy of the app already runs in this user session.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("another copy is running")]
pub struct Running;

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
    /// Guards our visible windows except the one titled `spare`, with the one titled `top` last so it lands in front (a
    /// hidden window shows again at its old place), and says how many. Call it on the UI thread, which owns the windows.
    ///
    /// # Errors
    /// `Refused` when the OS will not guard a window, as from another thread.
    fn guard_own(&self, spare: Option<&str>, top: Option<&str>) -> Result<usize, UiError>;

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

    /// Marks our shown window titled `title` as one that pages, as an open list does (ADR-0029). While a marked window
    /// shows, every platform's scroll target aims at it whatever point it kept, so a pad tap turns its page, and it
    /// passes over our other windows. The mark goes with the window; a call for the UI thread, as the window opens.
    ///
    /// # Errors
    /// `NoWindow` while no such window shows, `Refused` when the OS says no.
    fn mark_pages(&self, title: &str) -> Result<(), UiError>;

    /// Notes the window that has the focus now.
    fn remember(&self) -> FocusToken;

    /// Gives the focus back to the noted window, if it still exists.
    fn give_back(&self, token: FocusToken);
}

/// The screens and the pointer.
pub trait Screens: Send + Sync {
    /// Every screen's work area, with its whole DPI as the OS gives it.
    fn work_areas(&self) -> Vec<Screen>;

    /// Where the pointer is, in physical pixels.
    ///
    /// # Errors
    /// `Refused` when the OS cannot say.
    fn cursor(&self) -> Result<Point, UiError>;
}

/// The accent fills: a darker one on light, a lighter one on dark.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Accent {
    /// For light mode.
    pub on_light: Rgba,
    /// For dark mode.
    pub on_dark: Rgba,
}

/// The seven colours high contrast asks every app to use.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SysColors {
    /// Window background.
    pub window: Rgba,
    /// Text on the window background.
    pub window_text: Rgba,
    /// Selected item background.
    pub highlight: Rgba,
    /// Text on a selected item.
    pub highlight_text: Rgba,
    /// Button face.
    pub button: Rgba,
    /// Text on a button.
    pub button_text: Rgba,
    /// Disabled text.
    pub gray_text: Rgba,
}

/// The system look right now.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SysLook {
    /// Apps use the dark mode.
    pub dark: bool,
    /// The accent, when it could be read.
    pub accent: Option<Accent>,
    /// The system colours, while high contrast is on.
    pub contrast: Option<SysColors>,
    /// The system's animation effects are off.
    pub anim_off: bool,
    /// The pointer's size in physical pixels at 100 % scale.
    pub pointer_px: u32,
}

impl SysLook {
    /// The pointer's size when the system cannot say: Windows' default, its smallest.
    pub const POINTER_PX: u32 = 32;
}

/// Light, with no accent or high contrast, animations on and the usual pointer.
impl Default for SysLook {
    fn default() -> Self {
        Self {
            dark: false,
            accent: None,
            contrast: None,
            anim_off: false,
            pointer_px: Self::POINTER_PX,
        }
    }
}

/// Keeps a watch running; dropping it stops the watch once a running callback returns. A `Send + Sync` service may hold it.
#[must_use = "dropping a WatchGuard stops the watch"]
pub struct WatchGuard(Option<Box<dyn FnOnce() + Send + Sync>>);

impl WatchGuard {
    /// A guard that runs `stop` when it drops.
    pub fn new(stop: impl FnOnce() + Send + Sync + 'static) -> Self {
        Self(Some(Box::new(stop)))
    }
}

impl Drop for WatchGuard {
    fn drop(&mut self) {
        if let Some(stop) = self.0.take() {
            stop();
        }
    }
}

impl fmt::Debug for WatchGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WatchGuard").finish_non_exhaustive()
    }
}

/// The system look and the system's display language.
pub trait SystemLook: Send + Sync {
    /// The look now.
    fn read(&self) -> SysLook;

    /// Calls `on_change` with each new look until the guard drops.
    /// It may run on another thread than the caller's, so it hands work to the UI thread and never blocks on the thread that drops the guard.
    ///
    /// # Errors
    /// `Refused` when the OS will not report changes.
    fn watch(&self, on_change: Box<dyn Fn(SysLook) + Send>) -> Result<WatchGuard, UiError>;

    /// The system's display language, which only the tray speaks, as a BCP-47 tag such as `fr-FR`; empty when unknown.
    fn ui_language(&self) -> String;
}

/// The material behind a window.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum BackdropKind {
    /// A solid plate drawn by the UI.
    #[default]
    None,
    /// The system's Mica material.
    Mica,
    /// The system's frosted glass.
    Acrylic,
}

/// Sets the material behind a window.
pub trait Backdrop: Send + Sync {
    /// Puts `kind` behind the window titled `title`, in dark or light; call it on the UI thread.
    ///
    /// # Errors
    /// `NoWindow` for an unknown title, `Refused` when the OS says no; the UI then draws a solid plate.
    fn apply(&self, title: &str, kind: BackdropKind, dark: bool) -> Result<(), UiError>;
}

/// A request from a later copy to the running one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Show the keyboard.
    Show,
    /// End the app.
    Quit,
}

/// Being the one running copy; dropping it waits for a running `on`, unless its quit could not be posted, then lets
/// another copy claim.
pub trait Claim: Send {
    /// Calls `on` with each command a later copy sends; it may run on another thread and never blocks on the thread
    /// that drops the claim. A later call replaces `on`, even one made while `on` runs, from any thread.
    ///
    /// # Errors
    /// `Refused` when the OS will not set up the listener; an earlier `on` then stays.
    fn listen(&mut self, on: Box<dyn Fn(Command) + Send>) -> Result<(), UiError>;
}

/// Keeps one running copy per user session.
pub trait Instance: Send + Sync {
    /// Claims the session for this copy.
    /// When the OS cannot check, the adapter returns a claim, so the app runs as the only copy.
    ///
    /// # Errors
    /// `Running` only when another copy holds the claim.
    fn claim(&self) -> Result<Box<dyn Claim>, Running>;

    /// Sends `command` to the running copy.
    ///
    /// # Errors
    /// `NoWindow` when no copy is running or it does not listen yet, `Refused` when the OS says no.
    fn send(&self, command: Command) -> Result<(), UiError>;
}

#[cfg(test)]
mod tests;

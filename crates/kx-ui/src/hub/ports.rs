//! The UI ports the app hands the hub: the guard, the screens, the look, the backdrop, the tray, memory, the speaker,
//! the folder reveal and the Esc watch.

use kx_platform::ui::{
    Backdrop, EscWatch, Memory, Reveal, Screens, Speaker, SystemLook, Tray, WindowGuard,
};
use std::sync::Arc;

/// The UI ports the app hands over; they run on the UI thread unless their docs say otherwise.
pub struct UiPorts {
    /// Keeps our windows from taking the focus and shapes where they take clicks.
    pub guard: Arc<dyn WindowGuard>,
    /// The screens and the pointer.
    pub screens: Arc<dyn Screens>,
    /// The system look and the system's display language.
    pub look: Arc<dyn SystemLook>,
    /// The material behind a window.
    pub backdrop: Arc<dyn Backdrop>,
    /// The tray icon, whose events come on the adapter's thread.
    pub tray: Arc<dyn Tray>,
    /// Gives the heaps' free pages back after a look change (ADR-0023).
    pub memory: Arc<dyn Memory>,
    /// Plays the sounds; any thread may call it, since it never blocks.
    pub speaker: Arc<dyn Speaker>,
    /// Shows the clipboard history's folder in the file manager; it may wait, so the hub calls it off the UI thread.
    pub reveal: Arc<dyn Reveal>,
    /// Hears the hand's Esc while Snip's frozen screens show (ADR-0059); it calls back on its own thread.
    pub esc: Arc<dyn EscWatch>,
}

//! What a tap did, for the face to follow up, and why it sent nothing or not all of it.

use kx_module_api::ports::{InjectError, ShortcutError, Toggle};
use thiserror::Error;

/// Why a tap sent nothing, or not all of it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum TapError {
    /// The board has no such key, and the layer draws none now.
    #[error("the board has no such key")]
    UnknownKey,
    /// The keys or text did not all reach the app.
    #[error(transparent)]
    Inject(#[from] InjectError),
    /// The shortcut did not all reach the app.
    #[error(transparent)]
    Shortcut(#[from] ShortcutError),
}

/// What a tap did, for the face to follow up.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tapped {
    /// Keys or text went out, Caps Lock was tapped or a latch changed: the state needs a redraw.
    Done,
    /// The language key was clicked; the face steps or opens the list.
    Lang {
        /// The click landed on the key's back part.
        back: bool,
    },
    /// A mouse-assist side key: nothing was sent and the latches stay; the face asks the mouse module to flip this.
    Assist(Toggle),
    /// The Shortcuts key: nothing was sent and the latches stay; the face shows or hides the layer.
    Layer,
    /// The Power key: nothing was sent and the latches stay; the face asks the power module for the dialog.
    Power,
    /// The clipboard key, or the history chord while it is wanted, which used the latches: nothing was sent; the face
    /// asks the clipboard module to open or close its panel.
    Clip,
    /// The layer's Next key: nothing was sent and the latches stay; the face turns the page.
    Next,
    /// A key from a later version: nothing happened.
    Later,
    /// A character key that types nothing with the latched Shift or `AltGr`: nothing happened.
    Empty,
    /// The app in front has another layout than the one shown, and this key's result depends on it: nothing
    /// was sent. The face refreshes the layouts, which read the same layout source, and redraws.
    Stale,
}

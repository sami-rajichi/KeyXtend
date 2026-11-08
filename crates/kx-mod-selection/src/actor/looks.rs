//! What the actor hears from the watch, the mouse and the keyboard: each look feeds the strip, a press outside hides
//! it, a release looks again, and a typed key ends the caret's Paste.

use kx_module_api::ports::{SelKind, SelSpot};

use super::Actor;
use crate::strip::Input;

impl Actor {
    /// Looks at the focused box and feeds what it shows, `click` when a press and release that began in another app
    /// just came; a failed look, as from a hung app, hides the strip.
    pub(super) fn look(&mut self, click: bool) {
        let Some(run) = self.ports.run.as_mut() else {
            return;
        };
        let input = match run.look() {
            Ok(spot) => {
                // Only a click at a bare caret asks the clipboard, and only for its formats.
                let clip = click && bare(&spot) && run.clip_holds();
                Input::Look { spot, click, clip }
            }
            Err(_) => Input::Lost,
        };
        self.step(input);
    }

    /// A press on our windows (true) or in another app, which hides the strip; any press ends the edit's swallow.
    pub(super) fn pressed(&mut self, ours: bool) {
        self.press = Some(ours);
        self.swallow = false;
        if !ours {
            self.step(Input::Press);
        }
    }

    /// A release on our windows (true) or in another app: it looks, unless the press began on our windows or Grab
    /// carries it, so a drag select that ends over the keyboard still looks.
    pub(super) fn released(&mut self, ours: bool) {
        let began_out = self.press.take().map_or(!ours, |p| !p);
        if self.carrying || !began_out {
            return;
        }
        self.look(true);
    }

    /// A piece the keyboard typed is a key; a break, such as our Esc, also hides a selection until the next look, as
    /// the app may tell nothing. The one break right after our own edit is neither, so the act stays put away.
    pub(super) fn typed(&mut self, broke: bool) {
        if std::mem::take(&mut self.swallow) && broke {
            return;
        }
        self.step(Input::Key);
        if broke {
            self.step(Input::Lost);
        }
    }
}

/// True for a bare caret in a box that takes a paste.
fn bare(spot: &SelSpot) -> bool {
    spot.kind == SelKind::Editable && !spot.selected && spot.caret.is_some()
}

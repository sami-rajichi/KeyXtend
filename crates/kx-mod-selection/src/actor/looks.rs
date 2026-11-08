//! What the actor hears from the watch, the mouse and the keyboard: each look feeds the strip, a press outside hides
//! it, a release looks again, and a typed key ends the caret's Paste.

use kx_module_api::ports::{Piece, SelKind, SelSpot};

use super::Actor;
use crate::strip::Input;

impl Actor {
    /// Looks at the focused box and feeds what it shows; a failed look, as from a hung app, hides the strip.
    pub(super) fn look(&mut self) {
        let click = std::mem::take(&mut self.click);
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

    /// A press on our windows (true) or in another app, which hides the strip.
    pub(super) fn pressed(&mut self, ours: bool) {
        self.press = Some(ours);
        if !ours {
            self.swallow = false;
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
        self.click = true;
        self.look();
    }

    /// A piece the keyboard typed; the one break right after our own edit is no key, so the act stays put away.
    pub(super) fn typed(&mut self, piece: &Piece) {
        let own = std::mem::take(&mut self.swallow) && matches!(piece, Piece::Break);
        if !own {
            self.step(Input::Key);
        }
    }
}

/// True for a bare caret in a box that takes a paste.
fn bare(spot: &SelSpot) -> bool {
    spot.kind == SelKind::Editable && !spot.selected && spot.caret.is_some()
}

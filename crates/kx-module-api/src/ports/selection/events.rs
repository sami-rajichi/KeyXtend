//! The selection chips' bus events (ADR-0041): the strip the ui draws, its clicks, and the edit keys the keyboard sends,
//! as the keyboard is the only typist. Every one needs `Selection`, and no `Debug` prints the strip's boxes.

use crate::ports::{Rect, Screen};
use crate::{Capability, Event, Redacted};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The gate of every event here.
const GATE: Option<Capability> = Some(Capability::Selection);

listed! {
    /// A keycap on the strip, in the order it shows in every language (ADR-0020).
    pub enum SelChip ("Every chip, in the strip's order.") {
        /// Copy the selection.
        Copy,
        /// Cut the selection.
        Cut,
        /// Paste what the clipboard holds.
        Paste,
        /// Open a translation of the selection in the browser.
        Translate,
    }
}

listed! {
    /// What each keycap shows, so it never shows nothing; the selection module's `look` setting names one.
    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Legend ("Every legend.") {
        /// Its icon and its word; a view takes this until its settings say.
        #[default]
        Both,
        /// Its icon alone.
        Icons,
        /// Its word alone.
        Words,
    }
}

impl Legend {
    /// The keycap shows its icon.
    #[must_use]
    pub fn icons(self) -> bool {
        self != Self::Words
    }

    /// The keycap shows its word.
    #[must_use]
    pub fn words(self) -> bool {
        self != Self::Icons
    }
}

listed! {
    /// An edit key the keyboard sends for a chip.
    pub enum Edit ("Every edit.") {
        /// Copy, as Ctrl+C or the terminal's copy key.
        Copy,
        /// Cut, as Ctrl+X; never in a terminal.
        Cut,
        /// Paste, as Ctrl+V or the terminal's paste key.
        Paste,
    }
}

/// Where the strip goes: the first and last line boxes, or the caret's box twice, and the screen that holds them; its
/// `Debug` hides the boxes.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct StripSpot {
    /// The first line box, in physical pixels.
    pub first: Rect,
    /// The last line box.
    pub last: Rect,
    /// The screen that holds them: the strip stays on its work area, sized for its DPI.
    pub screen: Screen,
}

/// Shows the screen, and the boxes as redacted.
impl fmt::Debug for StripSpot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StripSpot")
            .field("first", &Redacted::new(self.first))
            .field("last", &Redacted::new(self.last))
            .field("screen", &self.screen)
            .finish()
    }
}

/// From the selection module: the strip to draw, or none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChipsView {
    /// Show `chips` by `spot`.
    Shown {
        /// Counts each new strip, so a click names the strip it saw.
        round: u32,
        /// The chips, in `SelChip::ALL`'s order.
        chips: Vec<SelChip>,
        /// Where the strip goes.
        spot: StripSpot,
        /// The chip just clicked, which shows its tick before the strip fades.
        acted: Option<SelChip>,
        /// What each keycap shows, from the module's settings.
        legend: Legend,
    },
    /// No strip, as before the module speaks and after it stops.
    Hidden,
}

impl Event for ChipsView {
    const NAME: &'static str = "chips-view";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the ui: `chip` on the strip of `round` was clicked.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChipAct {
    /// The round of the strip clicked.
    pub round: u32,
    /// The chip clicked.
    pub chip: SelChip,
}

impl Event for ChipAct {
    const NAME: &'static str = "chip-act";
    const CAPABILITY: Option<Capability> = GATE;
}

/// To the keyboard: send `edit` exactly, adding no latch and leaving every latch as it was (ADR-0041 decision 4).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EditAsk {
    /// Names this request, so its answer names it too.
    pub round: u32,
    /// The edit to send.
    pub edit: Edit,
    /// A terminal has focus, so the keyboard sends the table's terminal keys, which never stop a program.
    pub terminal: bool,
}

impl Event for EditAsk {
    const NAME: &'static str = "edit-ask";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: the edit an `EditAsk` asked for went out, or not.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EditDone {
    /// The round of the request.
    pub round: u32,
    /// The edit went out.
    pub ok: bool,
}

impl Event for EditDone {
    const NAME: &'static str = "edit-done";
    const CAPABILITY: Option<Capability> = GATE;
}

#[cfg(test)]
mod tests;

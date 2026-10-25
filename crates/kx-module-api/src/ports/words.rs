//! Word prediction's bus events (ADR-0033): typed text travels only to modules holding `ReadFocusedText`, and every
//! text here prints redacted.

use crate::{Capability, Event, Redacted};

/// The gate of every event here but `PressesWanted`.
const GATE: Option<Capability> = Some(Capability::ReadFocusedText);

/// One piece of what the keyboard typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    /// Characters, a Space key's space included.
    Text(Redacted<String>),
    /// One Backspace.
    Erase,
    /// Enter or Tab: the word ends whole, as at a space, and a new line or field starts.
    Advance,
    /// An arrow, Delete, Home, End, Esc, a shortcut or chord, or a language switch: the word may be half edited.
    Break,
}

/// From the keyboard: a piece that reached the app; pieces the app refused are never told.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typed {
    /// What was typed.
    pub piece: Piece,
}

impl Event for Typed {
    const NAME: &'static str = "typed";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: the latches that change what the next key types; the default has no latch.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TypingMode {
    /// Shift is latched.
    pub shift: bool,
    /// Caps Lock is on.
    pub caps: bool,
    /// Ctrl, Alt or the system key (Win, Cmd or Super) is latched.
    pub shortcut: bool,
}

impl Event for TypingMode {
    const NAME: &'static str = "typing-mode";
    const CAPABILITY: Option<Capability> = GATE;
}

/// To the keyboard: erase `erase` characters the keyboard typed, then type `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeWord {
    /// The round of the lane the word came from.
    pub round: u32,
    /// How many characters (Unicode scalars) to erase first; each platform's injector turns them into Backspace
    /// presses, one per character, as one press removes a whole surrogate pair.
    pub erase: u16,
    /// What to type then; the keyboard types nothing when it holds a control character.
    pub text: Redacted<String>,
}

impl TypeWord {
    /// How many characters `typed` holds, as `erase` counts them; `None` past `u16::MAX`.
    #[must_use]
    pub fn erase_of(typed: &str) -> Option<u16> {
        u16::try_from(typed.chars().count()).ok()
    }
}

impl Event for TypeWord {
    const NAME: &'static str = "type-word";
    const CAPABILITY: Option<Capability> = GATE;
}

/// What a chip offers.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChipKind {
    /// A word.
    Word,
    /// An email domain, after an `@`.
    Domain,
    /// A known email.
    Email,
    /// Opens the email list.
    More,
    /// Brings back the word just forgotten.
    Undo,
}

/// One chip in the lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    /// What the chip shows and types.
    pub text: Redacted<String>,
    /// What it offers.
    pub kind: ChipKind,
    /// The text is written right to left.
    pub rtl: bool,
}

/// What the chip lane shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lane {
    /// These chips, the best first.
    Chips(Vec<Chip>),
    /// A password box has focus: a lock and a short note.
    Locked,
    /// Prediction is off.
    Off,
    /// Nothing to offer.
    Empty,
}

/// The email list's shown page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListPage {
    /// The emails on this page.
    pub items: Vec<Redacted<String>>,
    /// This page's index.
    pub page: u16,
    /// How many pages the list has.
    pub pages: u16,
}

/// From the prediction module: the lane to draw and the menu's switches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggested {
    /// Counts each new lane, so a pick names the lane it saw.
    pub round: u32,
    /// What the lane shows.
    pub lane: Lane,
    /// The next chip tapped is forgotten.
    pub forgetting: bool,
    /// Learning is on, not paused.
    pub learn: bool,
    /// Prediction is on.
    pub on: bool,
    /// The email list, while it is open.
    pub list: Option<ListPage>,
}

impl Event for Suggested {
    const NAME: &'static str = "suggested";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the ui: the chip at `index` of lane `round` was tapped.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChipPicked {
    /// The round of the lane tapped.
    pub round: u32,
    /// The chip's place in the lane.
    pub index: u8,
}

impl Event for ChipPicked {
    const NAME: &'static str = "chip-picked";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the ui: a tap on the menu, the list or a control chip.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WordsAsk {
    /// Forget the next chip tapped.
    Forget,
    /// Drop the forget, or close the list.
    Cancel,
    /// Pause learning, or resume it on false.
    Pause(bool),
    /// Turn prediction on, or off on false.
    Power(bool),
    /// Bring back the word just forgotten.
    Undo,
    /// Open the email list, `rows` to a page (`[ui] page_rows`); the ui sends it for a tap on the More chip too.
    More {
        /// Rows per page.
        rows: u8,
    },
    /// Turn the list by `page` pages, `rows` to a page.
    Page {
        /// How many pages to turn, back when below 0.
        page: i8,
        /// Rows per page.
        rows: u8,
    },
    /// Pick the row at this index of the shown page.
    ListPick(u8),
    /// Send the lane again.
    Again,
}

impl Event for WordsAsk {
    const NAME: &'static str = "words-ask";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the prediction module: the mouse module tells each press (`PointerPressed`) while `on`; it carries nothing, so
/// it needs no capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PressesWanted {
    /// Tell each press.
    pub on: bool,
}

impl Event for PressesWanted {
    const NAME: &'static str = "presses-wanted";
}

#[cfg(test)]
mod tests;

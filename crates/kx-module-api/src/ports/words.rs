//! Word prediction's bus events (ADR-0033): typed text travels only to modules holding `ReadFocusedText`, and every
//! text here prints redacted.

use super::PanelLine;
use crate::{Capability, Event, Redacted};

/// The gate of every event here but `PressesWanted`, `PanelPressesWanted` and `PressesAsk`.
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

/// From the keyboard: the latches and wait that change what the next key types; the default has none.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact the keyboard tells at once, read on its own by the prediction module."
)]
pub struct TypingMode {
    /// Shift is latched.
    pub shift: bool,
    /// Caps Lock is on.
    pub caps: bool,
    /// Ctrl, Alt or the system key (Win, Cmd or Super) is latched.
    pub shortcut: bool,
    /// A dead key waits, so the app composes the next key with it and a picked word would type nothing.
    pub dead: bool,
    /// The most characters a picked word may erase or type (`[keyboard] text_cap`); `None` until the keyboard tells.
    pub cap: Option<u16>,
}

impl Event for TypingMode {
    const NAME: &'static str = "typing-mode";
    const CAPABILITY: Option<Capability> = GATE;
}

/// To the keyboard: send `TypingMode` and `AimedLine` again, for a prediction module that starts after the keyboard.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TypingModeAsk;

impl Event for TypingModeAsk {
    const NAME: &'static str = "typing-mode-ask";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: the line prediction follows, the open panel's while it is aimed at, else `PanelLine::None`. It
/// goes out in order with `Typed`, and is the one event that brings prediction a panel's text (ADR-0040).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AimedLine {
    /// The line, with a search's text.
    pub line: PanelLine,
}

impl Event for AimedLine {
    const NAME: &'static str = "aimed-line";
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
    /// What to type then; the keyboard types nothing when it holds a hidden mark (`is_hidden`).
    pub text: Redacted<String>,
    /// It was offered for a panel's search line: it goes there or nowhere, never to the app (ADR-0040).
    pub line: bool,
}

impl TypeWord {
    /// How many characters `typed` holds, as `erase` counts them; `None` past `u16::MAX`.
    #[must_use]
    pub fn erase_of(typed: &str) -> Option<u16> {
        u16::try_from(typed.chars().count()).ok()
    }

    /// True when `erase` and the characters of `text` each stay within `cap`, as the keyboard's `text_cap` allows.
    #[must_use]
    pub fn fits(erase: u16, text: &str, cap: u16) -> bool {
        erase <= cap && text.chars().count() <= usize::from(cap)
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
    /// An email saved in Quick-fill's notebook, which prediction never stores or learns.
    Saved,
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
    /// The module is not running, as before its first lane and after it stops or fails: no chips and no menu.
    Stopped,
}

/// The email list's shown page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListPage {
    /// The emails on this page, each a known or saved email chip.
    pub items: Vec<Chip>,
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

listed! {
    /// Who asks the mouse module to tell presses (ADR-0041 decision 6).
    pub enum PressAsker ("Every asker.") {
        /// Word prediction, which lets a word go at a click.
        Predict,
        /// The selection chips, which look again when a button comes up.
        Selection,
    }
}

/// From an asker: the mouse module tells each press (`PointerPressed`) and release (`PointerReleased`) while any asker
/// is `on`; it carries no text or point, so it needs no capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PressesWanted {
    /// Who asks.
    pub asker: PressAsker,
    /// Tell each press.
    pub on: bool,
}

impl Event for PressesWanted {
    const NAME: &'static str = "presses-wanted";
}

/// From the keyboard, while a panel is open: the mouse module tells each press, as one outside our windows takes the
/// aim away (ADR-0040); it carries nothing, so it needs no capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PanelPressesWanted {
    /// Tell each press.
    pub on: bool,
}

impl Event for PanelPressesWanted {
    const NAME: &'static str = "panel-presses-wanted";
}

/// Asks the prediction module and the keyboard to send their wish for presses again, for a mouse module that starts
/// later; it carries nothing, so it needs no capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PressesAsk;

impl Event for PressesAsk {
    const NAME: &'static str = "presses-ask";
}

#[cfg(test)]
mod tests;

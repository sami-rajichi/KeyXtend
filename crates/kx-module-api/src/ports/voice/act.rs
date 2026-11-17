//! The clicks the bar and the Voice panel send: a bar button, the pointer over the bar, a page, a row, a download's
//! button, an option, a panel button, Compare's start with its set text, and an answer.

use super::panel::VoiceTab;
use crate::Redacted;

listed! {
    /// A button on the bar; the gear sends `VoiceOpen` instead.
    pub enum BarButton ("Every bar button.") {
        /// Stop listening or writing, typing nothing.
        Cancel,
        /// Stop listening and type the text.
        Stop,
        /// Copy the text.
        Copy,
        /// Erase the text typed.
        Undo,
        /// Write the recording a Cancel left.
        Anyway,
        /// Listen again after Nothing heard.
        Listen,
        /// Download the language's model.
        Download,
        /// Close the bar, wiping the recording.
        Close,
    }
}

listed! {
    /// A button outside the panel's rows: the header's, the footer's, a question's and the folder line's.
    pub enum PanelButton ("Every panel button.") {
        /// Back to the home list, or up one folder.
        Back,
        /// Close the panel.
        Close,
        /// Use the ticked model for the language, or the ticked device.
        Use,
        /// Download the ticked model.
        Download,
        /// Delete the ticked model's file, or the ticked texts.
        Delete,
        /// Check the ticked card for the models in use.
        Check,
        /// Try again: write the held recording again with the choices here.
        Again,
        /// Compare again: back to the set text, to record it once more.
        Rerun,
        /// Insert the ticked texts, in tick order.
        Insert,
        /// Copy the ticked texts.
        Copy,
        /// Pin the ticked texts, or unpin them when all are pinned.
        Pin,
        /// Ask to delete every unpinned text.
        DeleteAll,
        /// Stop Compare models.
        Cancel,
        /// Change the models folder: open the folder list.
        Change,
        /// Show the models folder in the file manager.
        Open,
        /// Use the shown folder as the models folder.
        Here,
    }
}

listed! {
    /// A button on a model's row while its download runs or waits.
    pub enum FetchButton ("Every download button.") {
        /// Pause it, keeping its part.
        Pause,
        /// Go on from its part.
        Resume,
        /// Stop it and delete its part.
        Cancel,
    }
}

/// What a click on the bar or the panel names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoiceClick {
    /// A bar button.
    Bar(BarButton),
    /// The pointer came over the bar, or left it; while over it, the bar stays.
    Over(bool),
    /// A home row: open its page.
    Open(VoiceTab),
    /// A row of the shown page: tick or untick it, or open a folder in the folder list.
    Row(u8),
    /// Turn a page, from the panel's arrows or a scroll pad page tap, which the ui turns into this.
    Page {
        /// Turn back.
        back: bool,
    },
    /// A download's button on a model's row.
    Fetch {
        /// The row's place on the shown page.
        row: u8,
        /// The button.
        button: FetchButton,
    },
    /// An option's switch.
    Flip(u8),
    /// An option's minus or plus, or a choice's left or right chevron.
    Step {
        /// The option's place on the page.
        row: u8,
        /// Plus, or the next choice.
        up: bool,
    },
    /// A panel button.
    Button(PanelButton),
    /// Start Compare models with the set text the ui shows, from its translations; empty when the language has none.
    Compare(Redacted<String>),
    /// Answer the shown question: yes on true.
    Confirm(bool),
    /// The ui's rows per page (`page_rows`); the module pages by the last one heard.
    Rows(u8),
}

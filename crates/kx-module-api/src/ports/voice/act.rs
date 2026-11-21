//! The clicks the bar and the Voice panel send: a bar button, the pointer over the bar, a page, a row, a download's
//! button, an option, a panel button, Compare's start with its set text, an answer, and the cloud pages' switches,
//! buttons and typed lines, whose text stays redacted.

use super::panel::{CloudLine, ModelSide, VoiceTab};
use crate::{Redacted, Secret};

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
        /// Try again: send the held recording to the cloud again, after a cloud note.
        Again,
        /// Listen again after Nothing heard.
        Listen,
        /// Download the language's model.
        Download,
        /// Open service: a cloud note's service opens on its card, or on its key page for the key note.
        Service,
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

listed! {
    /// A button of the Cloud page, a service's card or its key page (ADR-0047).
    pub enum CloudButton ("Every cloud button.") {
        /// Switch the ticked service on: its card shows first, to confirm.
        On,
        /// Switch the ticked or shown service off.
        Off,
        /// Show the ticked or shown service's card.
        Facts,
        /// Open the ticked service's key page.
        Lines,
        /// Test the key.
        Test,
        /// Remove the ticked or shown service's sealed key, also while it is off.
        Remove,
        /// Open the shown page's link in the browser: the facts' page, or where keys are made.
        Link,
    }
}

/// What a click on the bar or the panel names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoiceClick {
    /// A bar button.
    Bar(BarButton),
    /// The pointer came over the bar, or left it; while over it, the bar stays.
    Over(bool),
    /// A home row: open its page; the Models page's Cloud band and slow line open the Cloud page too.
    Open(VoiceTab),
    /// Show the Models page's Local or Cloud side.
    Side(ModelSide),
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
    /// Answer the shown question, a card's switch-on among them: yes on true.
    Confirm(bool),
    /// The ui's rows per page (`page_rows`); the module pages by the last one heard.
    Rows(u8),
    /// Turn cloud services on, or off on false (`cloud`).
    Cloud(bool),
    /// A button of the cloud pages.
    Service(CloudButton),
    /// The key line's text, set as the shown service's key.
    SetKey(Secret),
    /// The address line's text, set as the shown service's address.
    SetAddress(Redacted<String>),
    /// The Other model line's text, set as the shown service's model id.
    SetModel(Redacted<String>),
    /// Save a typed line's text as the shown service's key, address or model id, as Enter on it does (ADR-0049).
    Save(CloudLine),
    /// Put the clipboard history's text on a typed line: the key as dots, an address or model id as text to save.
    Paste(CloudLine),
    /// A click on a typed line: it takes the letters, with its caret `at` into what it draws.
    Caret {
        /// The line.
        line: CloudLine,
        /// UTF-16 units of its text, or dots on the key line; it may tell a secret's length.
        at: Redacted<u32>,
    },
    /// Ask only for hosts that keep nothing, or not on false: `OpenRouter`'s switch.
    KeepNothing(bool),
    /// Use the local model: write the held recording with the language's local model, after a cloud fault.
    Local,
}

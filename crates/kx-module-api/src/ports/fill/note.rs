//! The panel's short notes: the result of a fill, an import, a check or a pick, and why something was refused. A
//! note carries the numbers its text names, so a new number never touches a page.

listed! {
    /// A short note the panel shows, such as after a fill.
    pub enum FillNote ("Every note.") {
        /// The fill was typed.
        Typed,
        /// The fill did not finish: part of it, such as the user and a Tab, may have been typed.
        NotTyped,
        /// Paste found no text: the clipboard is empty or holds an image.
        NoText,
        /// Paste heard no answer: the clipboard history is not on.
        ClipOff,
        /// Paste found the clipboard busy, or the system refused it; try again.
        ClipBusy,
        /// The password does not open it.
        WrongPassword,
        /// The box or file is damaged or foreign.
        Damaged,
        /// The file is of a kind this version does not read.
        Unsupported,
        /// A new backup password is shorter than `box_min_chars`.
        TooShort,
        /// A value over `field_chars`, or holding a hidden mark, was refused.
        Refused,
        /// The notebook holds `max_entries` entries or the entry `max_fields` fields, or a box's entries would pass
        /// such a limit.
        Full,
        /// A `KeePass` import or a Bring back finished: its counts say how many entries came in and what was left out.
        Imported,
        /// The memory check passed: the backup password is still known.
        Remembered,
        /// The memory check's last try was wrong, so a new backup password must be made.
        TriesUsed,
        /// The backup to bring back could not be read, as its place is gone or failed.
        Unread,
        /// The password does not open a `KeePass` file, which may also need a key file, as the worker cannot tell.
        FileLocked,
        /// A `KeePass` file passes `MAX_KEEPASS_BYTES`, so it was not opened.
        FileTooLarge,
        /// A `KeePass` file did not read: it is gone or failed, or the worker failed or took too long.
        FileUnread,
        /// The folder holds or lies in the data folder, which a format wipes, so it was refused.
        DataFolder,
        /// The place cannot be written.
        ReadOnly,
        /// The backup box passes `box_mb`, so it was not written or opened.
        TooLarge,
        /// The vault worker failed or took too long, so the backup waits.
        BoxFailed,
        /// The notebook did not read, so no box is written until Bring back, or Back up now and its Yes.
        Paused,
        /// The notebook or the backup's state did not save, so a change could be lost until a save works.
        SaveFailed,
    }
}

/// The numbers a note's text names; each is 0 for a note that names none of them.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct NoteCounts {
    /// Entries brought in.
    pub added: u32,
    /// What an import left out: entries skipped and fields dropped, its worker's too.
    pub skipped: u32,
}

/// A note as a page shows it, with the numbers its text names.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PageNote {
    /// Which note.
    pub note: FillNote,
    /// The numbers its text names.
    pub counts: NoteCounts,
}

impl PageNote {
    /// Note `note`, which names no number.
    #[must_use]
    pub fn plain(note: FillNote) -> Self {
        Self {
            note,
            counts: NoteCounts::default(),
        }
    }
}

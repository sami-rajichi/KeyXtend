//! The page the clipboard panel draws (spec §7 "Panel"): its rows, filter, search, preview, question, storage line
//! and note. Every text, path and app name in it stays redacted.

use crate::Redacted;
use crate::ports::{Date, Pixels, Room};
use std::path::PathBuf;

/// Declares a plain enum with `ALL`, every variant in declaration order, so a variant is listed by being declared.
macro_rules! listed {
    ($(#[$meta:meta])* $vis:vis enum $name:ident ($all:literal) { $($(#[$doc:meta])* $variant:ident,)+ }) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        $vis enum $name {
            $($(#[$doc])* $variant,)+
        }

        impl $name {
            #[doc = $all]
            pub const ALL: [Self; [$(stringify!($variant)),+].len()] = [$(Self::$variant),+];
        }
    };
}

listed! {
    /// What an entry holds, for its icon.
    pub enum ClipKind ("Every kind.") {
        /// Text.
        Text,
        /// A link.
        Link,
        /// An image.
        Image,
        /// Files or folders.
        File,
        /// Video or audio files.
        Media,
    }
}

/// Which entries the list shows.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Filter {
    /// Every entry.
    All,
    /// Text and links.
    Text,
    /// Images.
    Images,
    /// Files.
    Files,
    /// Video and audio.
    Media,
}

impl Filter {
    /// Every filter, in the order the chips show and the arrow keys step through.
    pub const ORDER: [Self; 5] = [
        Self::All,
        Self::Text,
        Self::Images,
        Self::Files,
        Self::Media,
    ];
}

/// When an entry was copied, in local time.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct When {
    /// Calendar days before today: 0 today, 1 yesterday.
    pub days_ago: u32,
    /// The local date, which also gives the weekday.
    pub date: Date,
    /// The local hour, 0 to 23.
    pub hour: u8,
    /// The local minute, 0 to 59.
    pub minute: u8,
}

/// One row of the shown page.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact the ui draws on its own: direction, font, pin and expiry."
)]
pub struct ClipRow {
    /// The entry's id, so the ui keeps a row's place as the list moves.
    pub id: u64,
    /// What the entry holds.
    pub kind: ClipKind,
    /// The start of its text, or its file names.
    /// An empty title means the ui shows the kind's name.
    pub title: Redacted<String>,
    /// The title reads right to left.
    pub rtl: bool,
    /// The title needs the Arabic font.
    pub arabic: bool,
    /// The name of the app it came from, when known.
    pub app: Option<Redacted<String>>,
    /// When it was copied.
    pub when: When,
    /// Its size in bytes: what is kept, or the files' size for a file row; `None` when a path was left unchecked.
    pub bytes: Option<u64>,
    /// Its place in the tick order, from 1, while ticked.
    pub tick: Option<u16>,
    /// It is pinned.
    pub pinned: bool,
    /// It is media that leaves at the end of today.
    pub expires: bool,
    /// Its thumbnail, once the worker drew it.
    pub thumb: Option<Pixels>,
}

/// The preview page's content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Preview {
    /// The entry's text, which the ui pages.
    Text {
        /// The text, cut to the preview's length.
        text: Redacted<String>,
        /// How many characters the whole entry holds.
        chars: u32,
    },
    /// The image, large, once the worker drew it.
    Image(Option<Pixels>),
    /// The files' paths.
    Files(Redacted<Vec<PathBuf>>),
}

listed! {
    /// A note on why a copy was not kept, or why an insert stopped; a damaged store is a notice, not a note.
    pub enum PanelNote ("Every note.") {
        /// An entry over the size limit was not kept.
        TooBig,
        /// The drive is under the free-space floor, so nothing new is kept.
        Disk,
        /// Pinned entries alone fill a limit, so nothing new is kept.
        PinnedFull,
        /// Setting the clipboard failed, so Insert stopped.
        Busy,
        /// The keyboard did not paste, so Insert stopped.
        PasteFailed,
    }
}

/// Where the shown page is in the list.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Pages {
    /// The shown page, from 0.
    pub at: u16,
    /// How many pages the list has.
    pub count: u16,
}

/// Everything the panel draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipPage {
    /// Counts each new view, so a tick names the page it saw.
    pub round: u32,
    /// The shown page's rows.
    pub rows: Vec<ClipRow>,
    /// The highlighted row's place on the page.
    pub highlight: Option<u8>,
    /// The filter shown.
    pub filter: Filter,
    /// The search line.
    pub search: Redacted<String>,
    /// Where the page is.
    pub pages: Pages,
    /// How many entries are ticked, in every view.
    pub ticked: u16,
    /// The pin action reads Unpin: every ticked entry, or the previewed one, is pinned.
    pub unpin: bool,
    /// The preview page, while it shows.
    pub preview: Option<Preview>,
    /// The Delete-all question while it shows, with how many entries it deletes.
    pub confirm: Option<u16>,
    /// The number of unpinned entries, so the ui disables Delete all at 0.
    pub unpinned: u16,
    /// What the history takes and its drive has free, once read; `None` until then, so no storage line shows.
    pub room: Option<Room>,
    /// The history's size limit, in bytes.
    pub cap: u64,
    /// A note to show, if any.
    pub note: Option<PanelNote>,
    /// How many calendar days entries are kept.
    pub kept_days: u8,
    /// The history holds no entry, so the panel asks for a copy.
    pub empty: bool,
}

//! Ports: the service traits and plain data types that modules, adapters and the UI talk through.

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

mod assist;
mod boxes;
mod calendar;
mod clip;
mod clips;
mod code;
mod face;
mod field;
mod fill;
mod geom;
mod input;
mod layout;
mod layouts;
mod link;
mod packs;
mod pictures;
mod places;
mod pointer;
mod power;
mod random;
mod scroll;
mod sealed;
mod selection;
mod shelf;
mod shortcut;
mod words;

pub use assist::{
    AssistAsk, AssistChanged, AssistNote, AssistToggle, GrabState, LatchesAsk, LatchesChanged,
    LatchesUsed, PointerMoved, PointerPressed, PointerReleased, Toggle,
};
pub use boxes::{
    BOX_EXT, BOX_MARK, BOX_NAME, BOX_PARTS, BOX_VERSION, BoxConfig, BoxError, BoxPart, BoxRun,
    Boxes, BoxesKey, Dropped, KeepassEntry, KeepassField, KeepassImport, MAX_BOX_BYTES,
    MAX_BOX_WORK, MAX_FIELD_CHARS, MAX_KEEPASS_BYTES, MAX_PASSWORD_BYTES, MAX_VAULT_ENTRIES,
    MAX_VAULT_FIELDS, MIN_BOX_WORK, NOTEBOOK_PART, Opened, Padlock, VAULT_MEMORY_MB, box_file,
};
pub use calendar::{
    Calendar, CalendarKey, Date, HOURS_PER_DAY, LocalTime, MIN_PER_HOUR, S_PER_MIN, Weekday,
};
pub use clip::{
    AppRef, ClipConfig, ClipError, ClipNote, ClipPart, ClipRun, ClipWait, Clipboard, ClipboardKey,
    ClipboardWriter, ClipboardWriterKey, FileFact, ImageCodec, probe_ok,
};
pub use clips::{
    ClipAct, ClipKey, ClipKind, ClipLine, ClipOpen, ClipPage, ClipPanel, ClipRow, ClipTake,
    ClipTaken, ClipText, ClipView, Filter, HistoryAsk, HistoryWanted, MB, Pages, PanelKey,
    PanelLine, PanelNote, PasteAsk, Pasted, Preview, When,
};
pub use code::CodeError;
pub use face::{
    FaceAction, FaceError, FaceReply, KeyId, KeyboardFace, KeyboardFaceKey, MAX_PRESETS,
};
pub use field::{
    FieldChange, FieldConfig, FieldEnds, FieldError, FieldKind, FieldRead, FieldRun, FieldWatch,
    FieldWatchKey, after_first, last_words, read_end,
};
pub use fill::{
    BackupAsk, BackupSheet, BoxState, EditSheet, EntryKind, EntryRow, EntrySheet, FieldButton,
    FieldLine, FieldValue, FillAct, FillAsk, FillButton, FillClick, FillClose, FillFilter, FillKey,
    FillLine, FillLive, FillNames, FillNote, FillOpen, FillPage, FillPanel, FillPart, FillText,
    FillType, FillTyped, FillView, FolderSheet, FoundRow, Hint, Label, LabelCount, LabelValue,
    NoteCounts, PageNote, PasswordAsk, PasswordSheet, Pick, PlaceRow, RowButton, Sheet, Spot,
    Typing, VaultEmails,
};
pub use geom::{BASE_DPI, Corner, Point, Rect, Screen, Strip, anchor, physical};
pub use input::{InjectError, InputInjector, InputInjectorKey, Scan, TextRoute};
pub use layout::{KeyChars, LangFacts, LayoutId, LayoutSource, LayoutSourceKey, Slot, SwitchError};
pub use layouts::{LangLabel, LayoutAsk, LayoutChanged, LayoutTable, Layouts, LayoutsKey, step_to};
pub use link::{LINK_SCHEME, Link, LinkError, MAX_LINK_BYTES, WebLink, WebLinkKey};
pub use packs::{
    IndexError, MAX_INDEX_BYTES, MAX_ISO_LEN, MAX_PACK_BYTES, MIN_ISO_LEN, PACK_DIR, PACK_EXT,
    PACK_INDEX, PackError, PackIndex, PackView, WordPacks, WordPacksKey, fits_iso, pack_file,
};
pub use pictures::{
    DibAsk, PictureAsk, PictureConfig, PictureError, PictureRun, Pictures, PicturesKey, Pixels,
    RGBA_BYTES, WORKER_DECODE_BYTES, WORKER_READ_TIMES,
};
pub use places::{
    BOX_HINT, Found, Place, PlaceError, PlaceKind, PlaceState, Places, PlacesKey, is_box_head,
    is_box_name,
};
pub use pointer::{
    Button, HookConfig, HookError, HookRun, HookStats, MAX_HELD_KEYS, PointerAct, PointerEngine,
    PointerEvent, PointerHook, PointerHookKey,
};
pub use power::{PowerAsk, PowerAsked, PowerChanged, PowerDialog, PowerDialogKey, PowerError};
pub use random::{Random, RandomError, RandomKey};
pub use scroll::{
    Aim, Dir, Reach, ScrollAimed, ScrollAsk, ScrollChanged, ScrollConfig, ScrollError, ScrollPage,
    ScrollPeek, ScrollRun, ScrollStop, ScrollTap, ScrollTarget, ScrollTargetKey, TargetId,
};
pub use sealed::{
    BlobName, MAX_BLOB_BYTES, MAX_BLOB_LEN, MIN_BLOB_LEN, RESERVED_NAMES, SealedError, SealedStore,
    SealedStoreKey, VaultStoreKey,
};
pub use selection::{
    Board, BoardPlace, BoardPlaceAsk, ChipAct, ChipsView, Edit, EditAsk, EditDone, Legend, SelChip,
    SelConfig, SelError, SelKind, SelRun, SelSpot, SelectionWatch, SelectionWatchKey, StripSpot,
    same_box,
};
pub use shelf::{Room, SealedShelf, SealedShelfKey};
pub use shortcut::{
    COPY_ID, COPY_TERM_ID, CUT_ID, KeyName, MAX_ID_LEN, PASTE_ID, PASTE_TERM_ID, ShortcutError,
    ShortcutId, ShortcutInfo, Shortcuts, ShortcutsKey, Step,
};
pub use words::{
    AimedLine, Chip, ChipKind, ChipPicked, Lane, ListPage, PanelPressesWanted, Piece, PressAsker,
    PressesAsk, PressesWanted, Suggested, TypeWord, Typed, TypingMode, TypingModeAsk, WordsAsk,
};

use crate::Redacted;
use std::fmt;

/// Writes `name` with its point `at` hidden, as a click's point can show what was clicked.
fn hide_at(f: &mut fmt::Formatter<'_>, name: &str, at: Point) -> fmt::Result {
    f.debug_struct(name)
        .field("at", &Redacted::new(at))
        .finish()
}

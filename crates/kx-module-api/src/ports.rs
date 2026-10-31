//! Ports: the service traits and plain data types that modules, adapters and the UI talk through.

mod assist;
mod calendar;
mod clip;
mod clips;
mod code;
mod face;
mod field;
mod geom;
mod input;
mod layout;
mod layouts;
mod packs;
mod pictures;
mod pointer;
mod power;
mod scroll;
mod sealed;
mod shelf;
mod shortcut;
mod words;

pub use assist::{
    AssistAsk, AssistChanged, AssistNote, AssistToggle, GrabState, LatchesAsk, LatchesChanged,
    LatchesUsed, PointerMoved, PointerPressed, Toggle,
};
pub use calendar::{
    Calendar, CalendarKey, Date, HOURS_PER_DAY, LocalTime, MIN_PER_HOUR, S_PER_MIN, Weekday,
};
pub use clip::{
    AppRef, ClipConfig, ClipError, ClipNote, ClipPart, ClipRun, ClipWait, Clipboard, ClipboardKey,
    ClipboardWriter, ClipboardWriterKey, FileFact, ImageCodec, probe_ok,
};
pub use clips::{
    ClipAct, ClipKey, ClipKind, ClipOpen, ClipPage, ClipPanel, ClipRow, ClipView, Filter,
    HistoryAsk, HistoryWanted, MB, Pages, PanelKey, PanelNote, PasteAsk, Pasted, Preview, When,
};
pub use code::CodeError;
pub use face::{
    FaceAction, FaceError, FaceReply, KeyId, KeyboardFace, KeyboardFaceKey, MAX_PRESETS,
};
pub use field::{
    FieldChange, FieldConfig, FieldEnds, FieldError, FieldKind, FieldRead, FieldRun, FieldWatch,
    FieldWatchKey, last_words,
};
pub use geom::{BASE_DPI, Corner, Point, Rect, Screen, physical};
pub use input::{InjectError, InputInjector, InputInjectorKey, Scan, TextRoute};
pub use layout::{KeyChars, LangFacts, LayoutId, LayoutSource, LayoutSourceKey, Slot, SwitchError};
pub use layouts::{LangLabel, LayoutAsk, LayoutChanged, LayoutTable, Layouts, LayoutsKey, step_to};
pub use packs::{
    IndexError, MAX_INDEX_BYTES, MAX_ISO_LEN, MAX_PACK_BYTES, MIN_ISO_LEN, PACK_DIR, PACK_EXT,
    PACK_INDEX, PackError, PackIndex, PackView, WordPacks, WordPacksKey, fits_iso, pack_file,
};
pub use pictures::{
    DibAsk, PictureAsk, PictureConfig, PictureError, PictureRun, Pictures, PicturesKey, Pixels,
    RGBA_BYTES, WORKER_DECODE_BYTES, WORKER_READ_TIMES,
};
pub use pointer::{
    Button, HookConfig, HookError, HookRun, HookStats, MAX_HELD_KEYS, PointerAct, PointerEngine,
    PointerEvent, PointerHook, PointerHookKey,
};
pub use power::{PowerAsk, PowerAsked, PowerChanged, PowerDialog, PowerDialogKey, PowerError};
pub use scroll::{
    Aim, Dir, Reach, ScrollAimed, ScrollAsk, ScrollChanged, ScrollConfig, ScrollError, ScrollPage,
    ScrollPeek, ScrollRun, ScrollStop, ScrollTap, ScrollTarget, ScrollTargetKey, TargetId,
};
pub use sealed::{
    BlobName, MAX_BLOB_BYTES, MAX_BLOB_LEN, MIN_BLOB_LEN, RESERVED_NAMES, SealedError, SealedStore,
    SealedStoreKey,
};
pub use shelf::{Room, SealedShelf, SealedShelfKey};
pub use shortcut::{
    KeyName, MAX_ID_LEN, PASTE_ID, ShortcutError, ShortcutId, ShortcutInfo, Shortcuts,
    ShortcutsKey, Step,
};
pub use words::{
    Chip, ChipKind, ChipPicked, Lane, ListPage, Piece, PressesAsk, PressesWanted, Suggested,
    TypeWord, Typed, TypingMode, TypingModeAsk, WordsAsk,
};

use crate::Redacted;
use std::fmt;

/// Writes `name` with its point `at` hidden, as a click's point can show what was clicked.
fn hide_at(f: &mut fmt::Formatter<'_>, name: &str, at: Point) -> fmt::Result {
    f.debug_struct(name)
        .field("at", &Redacted::new(at))
        .finish()
}

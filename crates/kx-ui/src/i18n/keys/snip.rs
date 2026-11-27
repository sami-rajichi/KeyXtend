//! Snip's ids (P12 design "What the user sees"): each key, way, ink, reason, note and handle of Snip's windows, which
//! QML picks its icon and text by and sends back on a click. Each match makes a new one name its id.

use kx_module_api::ports::{
    AskKey, BarKey, CropKey, Dim, EditKey, FolderKey, Handle, Ink, Keep, RecKey, Shape,
    SnipKeyboard, SnipNote, SnipSpot, SnipTab, StripKey, Tool, VideoRow,
};

/// The Snip panel's window title, which the capture's shields find it by.
pub const WINDOW_SNIP_PANEL: &str = "window.snip-panel";
/// The title of each frozen screen's window, which no capture sees: captures read the frozen pixels.
pub const WINDOW_SNIP_SCREEN: &str = "window.snip-screen";
/// The capture bar's title, which its shadow and tip share, so its shield keeps all three out.
pub const WINDOW_SNIP_BAR: &str = "window.snip-bar";
/// The hint's title.
pub const WINDOW_SNIP_HINT: &str = "window.snip-hint";
/// The strip's title, shared with its shadow and tip.
pub const WINDOW_SNIP_STRIP: &str = "window.snip-strip";
/// The countdown's title.
pub const WINDOW_SNIP_COUNT: &str = "window.snip-count";
/// The record bar's title, shared with its shadow and tip.
pub const WINDOW_SNIP_RECORD: &str = "window.snip-record";
/// The title of the red edge's and the countdown outline's windows.
pub const WINDOW_SNIP_EDGE: &str = "window.snip-edge";
/// The editor's title.
pub const WINDOW_SNIP_EDITOR: &str = "window.snip-editor";
/// Snip's own windows, which stay off `WINDOWS`: each capture window shields itself through `Snip.shield` and the
/// editor shows only once the capture ended. The close question shows over the editor, or over the strip while the
/// keyboard is aside, where the picture is already taken and a recording hides the question before it starts. Guard
/// runs spare them and the panel, which leave the top while a system dialog shows (ADR-0059).
pub const SNIP_WINDOWS: [&str; 9] = [
    WINDOW_SNIP_SCREEN,
    WINDOW_SNIP_BAR,
    WINDOW_SNIP_HINT,
    WINDOW_SNIP_STRIP,
    WINDOW_SNIP_COUNT,
    WINDOW_SNIP_RECORD,
    WINDOW_SNIP_EDGE,
    WINDOW_SNIP_EDITOR,
    super::WINDOW_SNIP_ASK,
];
/// The tray menu's Snip row, which opens the Snip panel.
pub const TRAY_SNIP: &str = "tray.snip";
/// A capture bar key's word, by `snip_bar_id`.
pub const SNIP_WAY: &str = "snip.way-";
/// The hint under the bar for a way, by `snip_bar_id`.
pub const SNIP_HINT: &str = "snip.hint-";
/// The hint once a region's first corner is clicked.
pub const SNIP_HINT_CORNER: &str = "snip.hint-corner";
/// A strip key's word, by `snip_strip_id`.
pub const SNIP_STRIP: &str = "snip.strip-";
/// A recorded video's speed on its Speed key: `%1` times as fast.
pub const SNIP_SPEED_TIMES: &str = "snip.speed-times";
/// What readers hear after the strip's Play: it opens the video in the user's own player.
pub const SNIP_STRIP_PLAY_SAY: &str = "snip.strip-play-say";
/// What a strip key says, to readers and in its tip, while it waits for the video's copy at its speed.
pub const SNIP_STRIP_WAIT: &str = "snip.strip-wait";
/// A strip key's word once it acted, by `snip_keep_id`.
pub const SNIP_DONE: &str = "snip.done-";
/// What readers call a recorded video's length.
pub const SNIP_LENGTH: &str = "snip.length";
/// The countdown's words under its number.
pub const SNIP_COUNT: &str = "snip.count";
/// What readers call the record bar's time.
pub const SNIP_LIVE: &str = "snip.live";
/// What joins the length's, the countdown's and the time's name to its number for readers.
pub const SNIP_JOIN: &str = "snip.join";
/// A record bar key's word, by `snip_rec_id`.
pub const SNIP_REC: &str = "snip.rec-";
/// A time: `%1` minutes and `%2` seconds, two digits each.
pub const SNIP_TIME: &str = "snip.time";
/// A region's size: `%1` by `%2` pixels.
pub const SNIP_SIZE: &str = "snip.size";
/// Why a key stays dimmed, by `snip_dim_id`.
pub const SNIP_DIM: &str = "snip.dim-";
/// A note, by `snip_note_id`.
pub const SNIP_NOTE: &str = "snip.note-";
/// What readers call a frame's handle, by `snip_handle_id`.
pub const SNIP_HANDLE: &str = "snip.handle-";

/// Writes each id function as one exhaustive match, so a new member cannot go without its id.
macro_rules! ids {
    ($($(#[$doc:meta])* $name:ident($ty:ident) { $($variant:ident => $id:literal,)+ })+) => {
        $(
            $(#[$doc])*
            #[must_use]
            pub const fn $name(v: $ty) -> &'static str {
                match v {
                    $($ty::$variant => $id,)+
                }
            }
        )+
    };
}

ids! {
    /// A capture bar key's id.
    snip_bar_id(BarKey) {
        Region => "region",
        Window => "window",
        Full => "full",
        VideoRegion => "video-region",
        VideoFull => "video-full",
        Settings => "settings",
        Close => "close",
    }
    /// A strip key's id.
    snip_strip_id(StripKey) {
        Copy => "copy",
        Edit => "edit",
        Save => "save",
        Close => "close",
        Record => "record",
        Mic => "mic",
        Sound => "sound",
        Play => "play",
        Speed => "speed",
    }
    /// A record bar key's id.
    snip_rec_id(RecKey) {
        Mic => "mic",
        Sound => "sound",
        Stop => "stop",
        Cancel => "cancel",
    }
    /// An answer to the close question.
    snip_ask_id(AskKey) {
        Save => "save",
        Drop => "drop",
        Back => "back",
    }
    /// How a snip was kept.
    snip_keep_id(Keep) {
        Copy => "copy",
        Save => "save",
    }
    /// Where the strip or the record bar goes.
    snip_spot_id(SnipSpot) {
        Under => "under",
        Above => "above",
        Bottom => "bottom",
        Top => "top",
    }
    /// Why a key stays dimmed, which names its tip.
    snip_dim_id(Dim) {
        Hidden => "hidden",
        Bare => "bare",
        NoVideo => "no-video",
        NoMic => "no-mic",
        MicRefused => "mic-refused",
        NoSound => "no-sound",
        DiskLow => "disk-low",
    }
    /// A note's id.
    snip_note_id(SnipNote) {
        TooBig => "too-big",
        FreezeFailed => "freeze-failed",
        DiskLow => "disk-low",
        Longest => "longest",
        NoVideo => "no-video",
        RecordFailed => "record-failed",
        SaveFailed => "save-failed",
        ClipBusy => "clip-busy",
        CopyFailed => "copy-failed",
        SpeedFailed => "speed-failed",
        Bare => "bare",
        PlaceRefused => "place-refused",
        PlayFailed => "play-failed",
        FolderFailed => "folder-failed",
        MicFailed => "mic-failed",
        SoundFailed => "sound-failed",
    }
    /// An editor tool's id.
    snip_tool_id(Tool) {
        Crop => "crop",
        Pen => "pen",
        Highlighter => "highlighter",
        Box => "box",
        Circle => "circle",
        Arrow => "arrow",
        Line => "line",
        Text => "text",
        Cover => "cover",
        Steps => "steps",
        Eraser => "eraser",
    }
    /// An ink's id.
    snip_ink_id(Ink) {
        Red => "red",
        Yellow => "yellow",
        Green => "green",
        Blue => "blue",
        Black => "black",
        White => "white",
    }
    /// A drawn shape's id.
    snip_shape_id(Shape) {
        Box => "box",
        Circle => "circle",
        Arrow => "arrow",
        Line => "line",
        Cover => "cover",
    }
    /// An editor key's id.
    snip_edit_id(EditKey) {
        Undo => "undo",
        Redo => "redo",
        Close => "close",
    }
    /// A key of Crop's frame.
    snip_crop_id(CropKey) {
        Ok => "ok",
        Cancel => "cancel",
    }
    /// What the keyboard does during a capture.
    snip_keyboard_id(SnipKeyboard) {
        Bubble => "bubble",
        Hidden => "hidden",
        Shown => "shown",
    }
    /// A Snip panel page's id.
    snip_tab_id(SnipTab) {
        Capture => "capture",
        Video => "video",
    }
    /// A Video row's id.
    snip_row_id(VideoRow) {
        Countdown => "countdown",
        Fps => "fps",
        Longest => "longest",
        Mic => "mic",
        Sound => "sound",
    }
    /// A folder key's id.
    snip_folder_id(FolderKey) {
        Change => "change",
        Open => "open",
    }
    /// A frame handle's id.
    snip_handle_id(Handle) {
        TopLeft => "top-left",
        Top => "top",
        TopRight => "top-right",
        Right => "right",
        BottomRight => "bottom-right",
        Bottom => "bottom",
        BottomLeft => "bottom-left",
        Left => "left",
    }
}

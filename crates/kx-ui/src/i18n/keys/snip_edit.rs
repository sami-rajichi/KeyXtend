//! The text keys of Snip's editor, close question and panel (P12 design "The editor", "Close", "The Snip panel"): each
//! by the hub's id where one names it, else by the view's own ids below, which `SnipNames.qml` holds too.

/// The close question's window title.
pub const WINDOW_SNIP_ASK: &str = "window.snip-ask";
/// The editor's title.
pub const SNIP_EDITOR: &str = "snip.editor";
/// A tool's name, by `snip_tool_id`.
pub const SNIP_TOOL: &str = "snip.tool-";
/// An ink's name, by `snip_ink_id`.
pub const SNIP_INK: &str = "snip.ink-";
/// An editor key's name, by `snip_edit_id` or `SNIP_EDIT_OWN`.
pub const SNIP_EDIT: &str = "snip.edit-";
/// A key of Crop's frame, by `snip_crop_id`.
pub const SNIP_CROP: &str = "snip.crop-";
/// The editor's own keys beside the hub's: Maximise, Restore, the grip that moves it and the corner that resizes it.
pub const SNIP_EDIT_OWN: [&str; 4] = ["full", "back", "move", "size"];
/// The options row's labels: the ink, a line's thickness and a text's size.
pub const SNIP_COLOUR: &str = "snip.colour";
/// The thickness label.
pub const SNIP_THICK: &str = "snip.thick";
/// The text size label.
pub const SNIP_TEXT_SIZE: &str = "snip.text-size";
/// A stepper's minus, by `SNIP_STEPPED`.
pub const SNIP_LESS: &str = "snip.less-";
/// A stepper's plus, by `SNIP_STEPPED`.
pub const SNIP_MORE: &str = "snip.more-";
/// The steppers: the editor's thickness and text size, and the panel's countdown and longest video.
pub const SNIP_STEPPED: [&str; 4] = ["thick", "text", "countdown", "longest"];
/// Why an editor or panel key stays dimmed, by `SNIP_WHYS`.
pub const SNIP_WHY: &str = "snip.why-";
/// Nothing to undo, nothing to redo, a stepper at its least or at its most.
pub const SNIP_WHYS: [&str; 4] = ["undo", "redo", "least", "most"];
/// The close question.
pub const SNIP_ASK: &str = "snip.ask";
/// An answer to it, by `snip_ask_id`.
pub const SNIP_ANSWER: &str = "snip.ask-";
/// A panel page's name, by `snip_tab_id`; the panel's texts are all `snip.panel-*`, as the plan names them.
pub const SNIP_PANEL_TAB: &str = "snip.panel-tab-";
/// A panel row's label, by `snip_row_id` or `SNIP_ROW_OWN`.
pub const SNIP_PANEL_ROW: &str = "snip.panel-row-";
/// The Capture page's rows.
pub const SNIP_ROW_OWN: [&str; 2] = ["keyboard", "folder"];
/// A keyboard choice's name, by `snip_keyboard_id`.
pub const SNIP_PANEL_KEYBOARD: &str = "snip.panel-keyboard-";
/// What a keyboard choice does, by `snip_keyboard_id`.
pub const SNIP_PANEL_SAY: &str = "snip.panel-say-";
/// A folder key's name, by `snip_folder_id`.
pub const SNIP_PANEL_FOLDER: &str = "snip.panel-folder-";
/// The folder row while no folder can be used.
pub const SNIP_PANEL_FOLDER_NONE: &str = "snip.panel-folder-none";
/// A switch row's first choice.
pub const SNIP_PANEL_OFF: &str = "snip.panel-off";
/// A switch row's second choice.
pub const SNIP_PANEL_ON: &str = "snip.panel-on";

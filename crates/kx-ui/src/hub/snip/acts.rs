//! The kinds of click QML names to `Hub::snip_act`, and what each one sends: a key goes by the id the view gives it, a
//! minus or plus by the argument's sign, a caret by the argument itself.

use crate::i18n::keys::{
    snip_ask_id, snip_bar_id, snip_crop_id, snip_edit_id, snip_folder_id, snip_handle_id,
    snip_ink_id, snip_keep_id, snip_keyboard_id, snip_rec_id, snip_row_id, snip_strip_id,
    snip_tab_id, snip_tool_id,
};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskKey, BarKey, CropKey, EditKey, FolderKey, Handle, Ink, Keep, RecKey, SnipClick, SnipFit,
    SnipKeyboard, SnipTab, StripKey, TextSize, Thickness, Tool, VideoRow,
};
use serde::Deserialize;
use std::cmp::Ordering;

/// A capture bar key `id`.
pub const BAR: &str = "bar";
/// A strip key `id`.
pub const STRIP: &str = "strip";
/// A record bar key `id`.
pub const REC: &str = "rec";
/// An editor tool `id`.
pub const TOOL: &str = "tool";
/// An ink `id`.
pub const INK: &str = "ink";
/// The thickness's or text size's plus when the argument is above 0, minus below.
pub const SIZE: &str = "size";
/// An editor key `id`.
pub const EDIT: &str = "edit";
/// An answer `id` to the close question.
pub const ASK: &str = "ask";
/// Show the panel page `id`.
pub const TAB: &str = "tab";
/// The keyboard during a capture, `id`.
pub const KEYBOARD: &str = "keyboard";
/// The Video row `id`'s plus or second choice when the argument is above 0, minus or first below.
pub const STEP: &str = "step";
/// A folder key `id`.
pub const FOLDER: &str = "folder";
/// Close the panel.
pub const CLOSE: &str = "close";
/// A click on the text being typed: its caret goes the argument's UTF-16 units in; it sounds nothing.
pub const CARET: &str = "caret";
/// A reader's press on a frame handle `id`, which picks it up.
pub const HANDLE: &str = "handle";
/// Crop's ✓ or ✗ by `id`, while its frame waits.
pub const CROP: &str = "crop";

/// Snip's capture windows, each kept out of every capture: the bar, the strip, the hint, the edge, the countdown
/// and the record bar.
pub const CAPTURE: [&str; 6] = ["bar", "strip", "hint", "edge", "count", "record"];

/// A click QML should never send: an unknown kind or id, or an argument or round out of range.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Bad;

/// True for a click that plays no key sound: a caret, as the press before it sounded.
pub(super) fn silent(kind: &str) -> bool {
    kind == CARET
}

/// The round QML names, as the module counts it.
pub(super) fn round_of(round: i64) -> Result<u32, Bad> {
    u32::try_from(round).map_err(|_| Bad)
}

/// The click `kind` names with `id` and `arg`.
pub(super) fn click_of(kind: &str, id: &str, arg: i32) -> Result<SnipClick, Bad> {
    Ok(match kind {
        BAR => SnipClick::Bar(by_id(BarKey::ALL, snip_bar_id, id)?),
        STRIP => SnipClick::Strip(by_id(StripKey::ALL, snip_strip_id, id)?),
        REC => SnipClick::Record(by_id(RecKey::ALL, snip_rec_id, id)?),
        TOOL => SnipClick::Tool(by_id(Tool::ALL, snip_tool_id, id)?),
        INK => SnipClick::Ink(by_id(Ink::ALL, snip_ink_id, id)?),
        SIZE => SnipClick::Size { up: up(arg)? },
        EDIT => SnipClick::Edit(by_id(EditKey::ALL, snip_edit_id, id)?),
        ASK => SnipClick::Ask(by_id(AskKey::ALL, snip_ask_id, id)?),
        TAB => SnipClick::Tab(by_id(SnipTab::ALL, snip_tab_id, id)?),
        KEYBOARD => SnipClick::Keyboard(by_id(SnipKeyboard::ALL, snip_keyboard_id, id)?),
        STEP => SnipClick::Step {
            row: by_id(VideoRow::ALL, snip_row_id, id)?,
            up: up(arg)?,
        },
        FOLDER => SnipClick::Folder(by_id(FolderKey::ALL, snip_folder_id, id)?),
        CLOSE => SnipClick::ClosePanel,
        CARET => SnipClick::Caret(Redacted::new(u32::try_from(arg).map_err(|_| Bad)?)),
        HANDLE => SnipClick::Handle(by_id(Handle::ALL, snip_handle_id, id)?),
        CROP => SnipClick::Crop(by_id(CropKey::ALL, snip_crop_id, id)?),
        _ => return Err(Bad),
    })
}

/// Copy or Save, by id, as the editor's flattened picture names it.
pub(super) fn keep_of(id: &str) -> Result<Keep, Bad> {
    by_id(Keep::ALL, snip_keep_id, id)
}

/// The member of `all` whose id, by `id_of`, is `id`.
fn by_id<T: Copy, const N: usize>(
    all: [T; N],
    id_of: fn(T) -> &'static str,
    id: &str,
) -> Result<T, Bad> {
    all.into_iter().find(|&t| id_of(t) == id).ok_or(Bad)
}

/// Plus above 0, minus below.
fn up(arg: i32) -> Result<bool, Bad> {
    match arg.cmp(&0) {
        Ordering::Greater => Ok(true),
        Ordering::Less => Ok(false),
        Ordering::Equal => Err(Bad),
    }
}

/// The theme's sizes as QML sends them; each list holds at least one step.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fit {
    strip: u16,
    bar: u16,
    gap: u16,
    thick: Vec<Thick>,
    text: Vec<Text>,
    slop: u16,
    handle: u16,
}

/// One thickness step.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Thick {
    pen: u16,
    marker: u16,
    disc: u16,
    head: u16,
}

/// One text size.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Text {
    height: u16,
    letter: u16,
}

/// The fit `json` gives, `{strip, bar, gap, thick: [{pen, marker, disc, head}], text: [{height, letter}], slop,
/// handle}`.
pub(super) fn fit_of(json: &str) -> Result<SnipFit, Bad> {
    let fit: Fit = serde_json::from_str(json).map_err(|_| Bad)?;
    if fit.thick.is_empty() || fit.text.is_empty() {
        return Err(Bad);
    }
    let thick = |t: Thick| Thickness {
        pen: t.pen,
        marker: t.marker,
        disc: t.disc,
        head: t.head,
    };
    let text = |t: Text| TextSize {
        height: t.height,
        letter: t.letter,
    };
    Ok(SnipFit {
        strip: fit.strip,
        bar: fit.bar,
        gap: fit.gap,
        thick: fit.thick.into_iter().map(thick).collect(),
        text: fit.text.into_iter().map(text).collect(),
        slop: fit.slop,
        handle: fit.handle,
    })
}

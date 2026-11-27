//! Snip's editor, question and panel views the Qt Quick tests draw (`qmltest/fixtures/snip.js`): a picture with every
//! kind of drawing, Arabic text being typed, a pen's first click, a crop, both ends of a step, the question over the
//! editor and over the strip, and both panel pages; and the hub's ids, click kinds and capture windows `SnipNames.qml`
//! must hold.

use crate::snipview::{Page, live, taken};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskKey, BGRX_BYTES, CaptureSheet, CloseAsk, Crop, CropKey, Dim, Drawing, EditKey, Editor, Ends,
    FolderKey, Handle, Ink, Keep, Mark, PictureEdit, Point, Rect, Shape, Shot, SnipKeyboard,
    SnipSheet, SnipStage, SnipTab, TextEdit, Tool, VideoRow, VideoSheet,
};
use kx_ui::i18n::keys::{
    SNIP_EDIT_OWN, SNIP_ROW_OWN, SNIP_STEPPED, SNIP_WHYS, snip_ask_id, snip_crop_id, snip_edit_id,
    snip_folder_id, snip_handle_id, snip_ink_id, snip_keep_id, snip_keyboard_id, snip_row_id,
    snip_shape_id, snip_tab_id, snip_tool_id,
};
use kx_ui::snip_acts as acts;
use serde_json::{Value, json};

/// The whole picture, as large as the part every strip sits by.
const WHOLE: Rect = Rect {
    x: 0,
    y: 0,
    w: 300,
    h: 200,
};
/// The part Crop keeps on the cropped pages.
pub(crate) const KEPT: Rect = Rect {
    x: 50,
    y: 40,
    w: 200,
    h: 120,
};

const fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// A side of the whole picture, in pixels.
const fn side(len: i32) -> u32 {
    len.unsigned_abs()
}

/// `text` as typed text.
fn typed(text: &str) -> Redacted<String> {
    Redacted::new(text.to_owned())
}

/// One of each drawing, the text both ways, each with its ink and size.
fn drawings() -> Vec<Drawing> {
    use Ink::{Black, Blue, Green, Red, Yellow};
    let shape = |shape, from, to| Mark::Shape { shape, from, to };
    let text = |at, words: &str, rtl| Mark::Text {
        at,
        text: typed(words),
        rtl,
    };
    let step = |at, n| Mark::Step { at, n };
    let pen = vec![at(10, 10), at(40, 30), at(70, 12)];
    let marks = [
        (Mark::Pen(pen), Red, 0),
        (Mark::Highlight(vec![at(10, 60), at(120, 60)]), Yellow, 1),
        (shape(Shape::Box, at(140, 20), at(200, 70)), Red, 1),
        (shape(Shape::Circle, at(210, 20), at(290, 60)), Green, 1),
        (shape(Shape::Arrow, at(280, 120), at(200, 90)), Red, 2),
        (shape(Shape::Line, at(20, 180), at(120, 180)), Blue, 0),
        (shape(Shape::Cover, at(150, 170), at(240, 190)), Black, 0),
        (text(at(20, 90), "Look here", false), Red, 1),
        (text(at(290, 140), "انظر هنا", true), Blue, 9),
        (step(at(60, 130), 1), Blue, 0),
        (step(at(120, 130), 2), Yellow, 9),
    ];
    let drawing = |(mark, ink, size)| Drawing { mark, ink, size };
    marks.into_iter().map(drawing).collect()
}

/// A picture with `tool`, its crop `kept`, its pen's first click, the text being typed and its size step.
pub(crate) fn picture(
    tool: Tool,
    kept: Rect,
    pending: Option<Point>,
    typing: Option<TextEdit>,
    size: u8,
) -> PictureEdit {
    PictureEdit {
        crop: Crop::new(kept, side(WHOLE.w), side(WHOLE.h)).expect("a crop"),
        drawings: drawings(),
        tool,
        ink: Ink::Red,
        size,
        pending,
        frame: None,
        typing,
        undo: true,
        redo: false,
    }
}

/// The editor holding `body`, with nothing kept yet.
pub(crate) fn editor(body: PictureEdit) -> SnipStage {
    let (w, h) = (side(WHOLE.w), side(WHOLE.h));
    let bytes = usize::try_from(w * h).expect("a size") * BGRX_BYTES;
    let picture = Shot::new(w, h, vec![0x80; bytes]).expect("a shot");
    SnipStage::Edit(Box::new(Editor {
        picture,
        body,
        save_dim: None,
        done: None,
        place: None,
        full: None,
    }))
}

/// Both ends of the steps: the last size, nothing to undo or redo, Copy done and Save dimmed.
fn ends() -> SnipStage {
    let mut ends = picture(Tool::Pen, WHOLE, None, None, 3);
    (ends.undo, ends.redo) = (false, false);
    let SnipStage::Edit(mut kept) = editor(ends) else {
        unreachable!("an editor")
    };
    (kept.done, kept.save_dim) = (Some(Keep::Copy), Some(Dim::Hidden));
    SnipStage::Edit(kept)
}

/// The pictures: every drawing with Pen picked, Arabic being typed, a pen's first click, a crop with Box picked and
/// with Crop picked, and both ends of the steps with Copy done and Save dimmed.
fn pictures() -> Vec<Page> {
    let shown =
        |tool, kept, pending, typing, size| editor(picture(tool, kept, pending, typing, size));
    let typing = TextEdit {
        at: at(280, 40),
        text: typed("نص عربي"),
        caret: Redacted::new(2),
        rtl: true,
    };
    vec![
        live("edit", shown(Tool::Pen, WHOLE, None, None, 1), None),
        live(
            "typing",
            shown(Tool::Text, WHOLE, None, Some(typing), 0),
            None,
        ),
        live(
            "pending",
            shown(Tool::Pen, WHOLE, Some(at(50, 50)), None, 1),
            None,
        ),
        live(
            "cropped",
            shown(Tool::Box, KEPT, Some(at(60, 50)), None, 1),
            None,
        ),
        live("crop", shown(Tool::Crop, KEPT, None, None, 1), None),
        live("ends", ends(), None),
        Page {
            dialog: true,
            ..live("edit-low", shown(Tool::Pen, WHOLE, None, None, 1), None)
        },
    ]
}

/// The question over the editor and over the strip, and the panel's two pages.
fn others() -> Vec<Page> {
    let asked = |save_dim| Some(CloseAsk { save_dim });
    let ends = |low, high| Ends { low, high };
    let capture = SnipSheet::Capture(CaptureSheet {
        keyboard: SnipKeyboard::Bubble,
        folder: None,
    });
    let sheet = SnipSheet::Video(VideoSheet {
        countdown_s: 3,
        countdown: ends(true, false),
        fps: 30,
        max_min: 30,
        longest: ends(false, true),
        mic_on: false,
        sound_on: true,
    });
    let edit = editor(picture(Tool::Pen, KEPT, None, None, 1));
    vec![
        Page {
            ask: asked(None),
            ..live("ask-edit", edit, None)
        },
        Page {
            ask: asked(Some(Dim::Hidden)),
            ..live("ask-strip", taken(), None)
        },
        Page {
            panel: Some(capture),
            ..live("panel-capture", SnipStage::Idle, None)
        },
        Page {
            panel: Some(sheet),
            ..live("panel-video", SnipStage::Idle, None)
        },
    ]
}

/// Every editor, question and panel page.
pub fn pages() -> Vec<Page> {
    pictures().into_iter().chain(others()).collect()
}

/// The hub's ids of the editor's, question's and panel's keys in their rows' order, and the views' own.
pub fn ids() -> Value {
    json!({
        "tools": Tool::ALL.map(snip_tool_id), "inks": Ink::ALL.map(snip_ink_id),
        "edits": EditKey::ALL.map(snip_edit_id), "keeps": Keep::ALL.map(snip_keep_id),
        "answers": AskKey::ALL.map(snip_ask_id), "tabs": SnipTab::ALL.map(snip_tab_id),
        "rows": VideoRow::ALL.map(snip_row_id), "keyboards": SnipKeyboard::ALL.map(snip_keyboard_id),
        "folders": FolderKey::ALL.map(snip_folder_id), "own": SNIP_EDIT_OWN, "capture": SNIP_ROW_OWN,
        "stepped": SNIP_STEPPED, "whys": SNIP_WHYS, "shapes": Shape::ALL.map(snip_shape_id),
        "handles": Handle::ALL.map(snip_handle_id), "crops": CropKey::ALL.map(snip_crop_id),
    })
}

/// The hub's click kinds by name, as `SnipNames.acts` holds them.
pub fn kinds() -> Value {
    json!({
        "bar": acts::BAR, "strip": acts::STRIP, "rec": acts::REC, "tool": acts::TOOL, "ink": acts::INK,
        "size": acts::SIZE, "edit": acts::EDIT, "ask": acts::ASK,
        "tab": acts::TAB, "keyboard": acts::KEYBOARD, "step": acts::STEP, "folder": acts::FOLDER,
        "close": acts::CLOSE, "caret": acts::CARET, "handle": acts::HANDLE, "crop": acts::CROP,
    })
}

/// The capture windows each shielded on its own, by id, as `SnipNames.capture` holds them.
pub fn windows() -> Value {
    let each = acts::CAPTURE
        .iter()
        .map(|w| ((*w).to_owned(), Value::from(*w)));
    Value::Object(each.collect())
}

/// The hub's click kinds, each a `pub const` text there.
const ACTS: &str = include_str!("../../src/hub/snip/acts.rs");

#[test]
fn every_snip_click_kind_has_a_name_for_qml() {
    let named = kinds();
    let named: Vec<&Value> = named.as_object().expect("names").values().collect();
    let all = ACTS.lines().filter_map(|l| {
        let (_, text) = l.strip_prefix("pub const ")?.split_once(": &str = \"")?;
        text.strip_suffix("\";")
    });
    let missing: Vec<&str> = all.filter(|k| !named.contains(&&Value::from(*k))).collect();
    assert!(missing.is_empty(), "{missing:?}");
}

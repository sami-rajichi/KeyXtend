//! The editor as QML reads it: its picture by URL and size, with its tool, ink, crop, Crop's waiting frame and
//! drawings. Typed text goes as it is, so QML shapes Arabic itself; the view is wiped, never logged.

use super::view::{Frame, frame};
use crate::i18n::keys::{snip_dim_id, snip_ink_id, snip_keep_id, snip_shape_id, snip_tool_id};
use kx_module_api::ports::{
    CropFrame, Drawing, Editor, Mark as Made, PictureEdit, Point, Rect, TextEdit,
};
use serde::Serialize;

/// The editor window's state.
#[derive(Serialize)]
pub(super) struct Edit<'a> {
    src: String,
    width: u32,
    height: u32,
    picture: Picture<'a>,
    save_dim: Option<&'static str>,
    done: Option<&'static str>,
}

/// A picture's tools and drawings, in the picture's own pixels.
#[derive(Serialize)]
struct Picture<'a> {
    crop: Rect,
    drawings: Vec<Drawn<'a>>,
    tool: &'static str,
    ink: &'static str,
    size: u8,
    pending: Option<Point>,
    frame: Option<Framed>,
    typing: Option<Typing<'a>>,
    undo: bool,
    redo: bool,
}

/// Crop's frame waiting: the part it would keep, then its frame as a region's.
#[derive(Serialize)]
struct Framed {
    part: Rect,
    #[serde(flatten)]
    frame: Frame,
}

/// One drawing.
#[derive(Serialize)]
struct Drawn<'a> {
    mark: Mark<'a>,
    ink: &'static str,
    size: u8,
}

/// What a drawing is, by `kind`.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Mark<'a> {
    Pen {
        points: &'a [Point],
    },
    Highlight {
        points: &'a [Point],
    },
    Shape {
        shape: &'static str,
        from: Point,
        to: Point,
    },
    Text {
        at: Point,
        text: &'a str,
        rtl: bool,
    },
    Step {
        at: Point,
        n: u16,
    },
}

/// The text being typed, with its caret in UTF-16 units.
#[derive(Serialize)]
struct Typing<'a> {
    at: Point,
    text: &'a str,
    caret: u32,
    rtl: bool,
}

/// `editor` as QML reads it, its picture at `src`.
pub(super) fn of(editor: &Editor, src: String) -> Edit<'_> {
    Edit {
        src,
        width: editor.picture.width(),
        height: editor.picture.height(),
        picture: picture(&editor.body),
        save_dim: editor.save_dim.map(snip_dim_id),
        done: editor.done.map(snip_keep_id),
    }
}

fn picture(p: &PictureEdit) -> Picture<'_> {
    Picture {
        crop: p.crop.rect(),
        drawings: p.drawings.iter().map(drawn).collect(),
        tool: snip_tool_id(p.tool),
        ink: snip_ink_id(p.ink),
        size: p.size,
        pending: p.pending,
        frame: p.frame.map(framed),
        typing: p.typing.as_ref().map(typing),
        undo: p.undo,
        redo: p.redo,
    }
}

fn drawn(d: &Drawing) -> Drawn<'_> {
    let mark = match &d.mark {
        Made::Pen(points) => Mark::Pen { points },
        Made::Highlight(points) => Mark::Highlight { points },
        Made::Shape { shape, from, to } => Mark::Shape {
            shape: snip_shape_id(*shape),
            from: *from,
            to: *to,
        },
        Made::Text { at, text, rtl } => Mark::Text {
            at: *at,
            text: text.expose(),
            rtl: *rtl,
        },
        Made::Step { at, n } => Mark::Step { at: *at, n: *n },
    };
    Drawn {
        mark,
        ink: snip_ink_id(d.ink),
        size: d.size,
    }
}

fn framed(f: CropFrame) -> Framed {
    Framed {
        part: f.part,
        frame: frame(f.frame),
    }
}

fn typing(t: &TextEdit) -> Typing<'_> {
    Typing {
        at: t.at,
        text: t.text.expose(),
        caret: *t.caret.expose(),
        rtl: t.rtl,
    }
}

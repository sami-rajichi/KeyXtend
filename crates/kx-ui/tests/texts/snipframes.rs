//! The frame pages the Qt Quick tests draw (plan 2026-11-26 tasks 4 and 5): the picture's strip with its frame on the
//! one screen, and with its right handle held, picked up just off its spot, the strip hidden; and Crop's frame waiting
//! inside the part kept, with and without its right handle held on its spot. Last, a strip whose Play waits.

use crate::snipedits::{KEPT, editor, picture};
use crate::snipview::{Page, live, recorded, still, taken};
use kx_module_api::ports::{
    CropFrame, Frame, Grip, Handle, Point, Rect, SnipStage, StripKey, Taken, Tool,
};

/// A frame's least side on the pages.
const MIN: u16 = 8;

/// The picture's strip under the part.
fn strip() -> Taken {
    match taken() {
        SnipStage::Strip(t) => t,
        _ => unreachable!("a strip"),
    }
}

/// The picture's strip with its frame on its screen, `grip` held, when the strip has no keys and hides.
fn framed(grip: Option<Grip>) -> SnipStage {
    let t = strip();
    let frame = Frame {
        within: t.screen,
        min: MIN,
        grip,
    };
    let keys = if grip.is_some() {
        Vec::new()
    } else {
        t.keys.clone()
    };
    SnipStage::Strip(Taken {
        frame: Some(frame),
        keys,
        ..t
    })
}

/// The part Crop's frame waits round on its pages, inside `KEPT`.
const FRAMED: Rect = Rect {
    x: 80,
    y: 60,
    w: 120,
    h: 70,
};

/// Crop picked on `KEPT`, its frame round `FRAMED` waiting, its right handle held on its spot when `held`.
fn cropping(held: bool) -> SnipStage {
    let mut body = picture(Tool::Crop, KEPT, None, None, 1);
    let at = Point {
        x: FRAMED.x + FRAMED.w,
        y: FRAMED.y + FRAMED.h / 2,
    };
    let grip = held.then_some(Grip {
        handle: Handle::Right,
        at,
    });
    let frame = Frame {
        within: KEPT,
        min: MIN,
        grip,
    };
    body.frame = Some(CropFrame {
        part: FRAMED,
        frame,
    });
    editor(body)
}

/// A recorded video's strip at 2× while Play waits for its copy at that speed.
fn waiting() -> SnipStage {
    let SnipStage::Strip(sped) = recorded(2, 37_500) else {
        unreachable!("a strip")
    };
    let waits = Some(StripKey::Play);
    SnipStage::Strip(Taken { waits, ..sped })
}

/// The frame's pages: none held, and the right handle held three pixels right of and one below its spot; then Crop's,
/// and last a strip waiting for its copy, so no other page's round moves.
pub fn pages() -> Vec<Page> {
    let r = strip().part.rect();
    let at = Point {
        x: r.x + r.w + 3,
        y: r.y + r.h / 2 + 1,
    };
    let grip = Grip {
        handle: Handle::Right,
        at,
    };
    vec![
        still("framed", framed(None)),
        still("held", framed(Some(grip))),
        live("crop-framed", cropping(false), None),
        live("crop-held", cropping(true), None),
        live("waiting", waiting(), None),
    ]
}

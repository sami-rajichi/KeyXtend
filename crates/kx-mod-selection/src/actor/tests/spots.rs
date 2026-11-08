//! The boxes the actor's tests focus: a selection on one line of any kind, and a bare caret in an editable box.

use kx_module_api::ports::{Rect, Screen, SelKind, SelSpot};

/// The screen every box is on.
fn screen() -> Screen {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    Screen {
        rect,
        dpi: 96,
        primary: true,
    }
}

/// A line box at row `y`.
pub(super) fn line(y: i32) -> Rect {
    Rect {
        x: 100,
        y,
        w: 200,
        h: 20,
    }
}

/// A box of `kind` with a selection on the line at `y`; the fake gives it its id.
pub(super) fn sel(kind: SelKind, y: i32) -> SelSpot {
    SelSpot {
        box_id: 0,
        kind,
        selected: true,
        first: Some(line(y)),
        last: Some(line(y)),
        caret: None,
        screen: screen(),
    }
}

/// An editable box with a bare caret at `x`.
pub(super) fn caret(x: i32) -> SelSpot {
    let at = Rect {
        x,
        y: 300,
        w: 1,
        h: 20,
    };
    SelSpot {
        selected: false,
        first: None,
        last: None,
        caret: Some(at),
        ..sel(SelKind::Editable, 0)
    }
}

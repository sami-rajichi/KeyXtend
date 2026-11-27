//! The editor: a crop that is never empty and lies wholly in its picture, a scale that is never 0, and the place it
//! opens at next time, as the keyboard's (`FaceAction::Placed`).

use super::view::editor;
use crate::ports::Rect;
use crate::ports::snip::{Crop, Scale};

/// A rectangle at (`x`, `y`) of `w` by `h`.
const fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

#[test]
fn a_crop_is_never_empty_and_lies_wholly_in_its_picture() {
    let (w, h) = (4, 3);
    let whole = rect(0, 0, 4, 3);
    let kept = Crop::new(whole, w, h).map(Crop::rect);
    assert_eq!(kept, Some(whole), "the whole picture, before any crop");
    assert!(
        Crop::new(rect(3, 2, 1, 1), w, h).is_some(),
        "the last pixel"
    );
    let bad = [
        rect(0, 0, 0, 3),
        rect(0, 0, 4, 0),
        rect(-1, 0, 2, 2),
        rect(0, -1, 2, 2),
        rect(1, 0, 4, 3),
        rect(0, 1, 4, 3),
        rect(i32::MAX, 0, 1, 1),
    ];
    for r in bad {
        assert_eq!(Crop::new(r, w, h), None, "{r:?}");
    }
    let far = Crop::new(rect(i32::MAX - 1, 0, 1, 1), u32::MAX, 1);
    assert!(far.is_some(), "far edges add without overflow");
}

#[test]
fn the_editor_opens_where_it_was_last_placed_or_at_its_default() {
    let mut open = editor();
    assert_eq!(open.place, None, "its default the first time");
    open.place = Some(rect(10, 20, 900, 600));
    let shown = format!("{open:?}");
    assert!(shown.contains("place: Some(Rect { x: 10, y: 20"), "{shown}");
}

#[test]
fn a_scale_is_never_0_and_gives_back_what_it_shows() {
    assert_eq!(Scale::new(0), None);
    assert_eq!(Scale::new(Scale::PER).map(Scale::shown), Some(Scale::PER));
}

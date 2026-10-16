//! Our open language list paged by the scroll pad: a page turn moves it as its page buttons do, and nothing moves
//! with no list open or before the UI gave its page size.

use super::Pad;
use crate::rig::STATE;
use kx_module_api::ports::{FaceReply, ScrollPage};
use serde_json::Value;

/// The module with the List design and its list open.
fn listing() -> Pad {
    let p = Pad::quiet(Some(("lang_key", "list".into())));
    assert_eq!(p.b.rig.tap("lang"), Ok(STATE));
    p
}

/// The open list's highlighted row.
fn hi(p: &Pad) -> Value {
    p.b.rig.state()["list"]["hi"].clone()
}

/// Plan edge case 11: our list turns one page per tap, as its page buttons do.
#[test]
fn a_page_turn_moves_the_open_list_as_its_page_buttons_do() {
    let p = listing();
    assert_eq!(p.b.rig.panel("down", 2), Ok(STATE));
    assert_eq!(p.relabel(), FaceReply::default(), "nothing to redraw yet");
    p.b.send(&ScrollPage { back: false });
    assert_eq!(p.relabel(), STATE, "the list redraws");
    assert_eq!(hi(&p), 2, "two-row pages: page 2 starts at row 3");
    p.b.send(&ScrollPage { back: false });
    assert_eq!(hi(&p), 2, "the last page, whose button is dimmed");
    p.b.send(&ScrollPage { back: true });
    assert_eq!(hi(&p), 0);
    p.b.send(&ScrollPage { back: true });
    assert_eq!(hi(&p), 0, "the first page");
    assert_eq!(p.relabel(), FaceReply::default(), "nothing moved");
    assert!(p.b.rig.fakes.injector.strokes().is_empty());
}

#[test]
fn a_page_turn_waits_for_the_page_size_the_ui_gives_with_a_list_key() {
    let p = listing();
    p.b.send(&ScrollPage { back: false });
    assert_eq!(hi(&p), 0);
    assert_eq!(p.relabel(), FaceReply::default());
}

#[test]
fn a_page_turn_with_no_list_open_does_nothing() {
    let p = listing();
    assert_eq!(p.b.rig.panel("esc", 2), Ok(STATE));
    assert_eq!(p.relabel(), FaceReply::default(), "nothing to redraw yet");
    p.b.send(&ScrollPage { back: false });
    assert!(p.b.rig.state()["list"].is_null());
    assert_eq!(p.relabel(), FaceReply::default());
    assert!(p.b.rig.fakes.injector.strokes().is_empty());
}

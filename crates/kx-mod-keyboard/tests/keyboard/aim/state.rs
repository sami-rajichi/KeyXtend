//! The aim in the state QML draws from: only a panel open and aimed says so, and a change made off the UI's thread shows
//! at the next relabel, which asks one redraw.

use super::{ELSEWHERE, Quick};
use crate::rig::STATE;
use kx_module_api::ports::{FaceAction, FaceReply};

#[test]
fn the_state_is_aimed_only_while_a_panel_is_open_and_aimed() {
    let q = Quick::quiet(None);
    assert!(!q.aimed());
    q.open(true);
    assert!(q.aimed());
    q.away();
    assert!(!q.aimed(), "a state read settles the count");
    q.aim();
    assert!(q.aimed());
    q.open(false);
    assert!(!q.aimed());
}

#[test]
fn an_outside_click_shows_at_the_next_relabel_and_asks_a_redraw_once() {
    let q = Quick::opened(None);
    let relabel = || q.b.rig.act(FaceAction::Relabel);
    q.b.rig.state();
    assert_eq!(relabel(), Ok(FaceReply::default()), "nothing to show yet");
    q.away();
    assert_eq!(relabel(), Ok(STATE));
    assert!(!q.aimed());
    assert_eq!(relabel(), Ok(FaceReply::default()), "once");
}

#[test]
fn the_front_window_changing_shows_at_the_next_relabel_too() {
    let q = Quick::opened(None);
    let relabel = || q.b.rig.act(FaceAction::Relabel);
    q.b.rig.state();
    q.window(Some(ELSEWHERE));
    assert_eq!(relabel(), Ok(STATE));
    assert!(!q.aimed());
    assert_eq!(relabel(), Ok(FaceReply::default()));
}

#[test]
fn a_form_and_its_end_show_at_the_next_relabel() {
    use kx_module_api::ports::{FillLine, PanelLine};
    let q = Quick::opened(None);
    let relabel = || q.b.rig.act(FaceAction::Relabel);
    q.away();
    q.b.rig.state();
    q.b.send(&FillLine(PanelLine::Private));
    assert_eq!(relabel(), Ok(STATE));
    assert!(q.aimed());
    q.b.send(&FillLine(PanelLine::None));
    assert_eq!(relabel(), Ok(STATE));
    assert!(!q.aimed());
}

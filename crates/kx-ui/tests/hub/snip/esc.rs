//! The hand's Esc (ADR-0059): the hub hears the physical Esc only while frozen screens show with no system dialog in
//! front, each press then tells Snip `Esc` with the newest round, and the watch stops as the frozen screens go.

use super::{editing, heard, idle, picking, screens, strip};
use crate::Rig;
use crate::logs::warnings_of;
use kx_module_api::ports::{SnipAct, SnipClick, SnipPage, SnipView};
use kx_platform::ui::UiError;

/// Round `round`'s strip, with a system dialog in front.
fn dialog(round: u32) -> SnipPage {
    SnipPage {
        dialog: true,
        ..strip(round)
    }
}

#[test]
fn the_hand_is_heard_while_frozen_screens_show_and_not_once_they_go() {
    let rig = Rig::new();
    let heard = heard(&rig);
    assert!(!rig.esc.watching(), "before any view");
    heard.shown(&rig, picking(1, screens()));
    assert!(rig.esc.watching(), "over the frozen screens");
    heard.shown(&rig, strip(2));
    assert_eq!(rig.esc.starts(), 1, "one watch while they show");
    for gone in [idle(3), editing(4, None), dialog(5)] {
        heard.shown(&rig, gone);
        assert!(!rig.esc.watching());
    }
    heard.shown(&rig, strip(6));
    assert!(rig.esc.watching(), "the dialog went");
}

#[test]
fn a_press_over_frozen_screens_tells_snip_esc_with_the_newest_round() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, picking(1, screens()));
    assert!(rig.esc.press());
    let esc = SnipAct {
        round: 1,
        click: SnipClick::Esc,
    };
    assert_eq!(heard.sent.acts(), [esc]);
}

#[test]
fn a_press_after_the_frozen_screens_went_but_before_the_qt_thread_saw_it_tells_nothing() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, picking(1, screens()));
    heard.bus.publish(&SnipView::new(idle(2))).unwrap();
    assert!(rig.esc.press(), "the watch still runs");
    assert!(heard.sent.clicks().is_empty());
}

#[test]
fn a_refused_watch_warns_once_for_its_frozen_screens_and_is_asked_again_for_the_next() {
    let rig = Rig::new();
    let heard = heard(&rig);
    rig.esc.set_error(Some(UiError::Refused(5)));
    let warned = warnings_of(|| {
        heard.shown(&rig, picking(1, screens()));
        heard.shown(&rig, strip(2));
    });
    assert_eq!(warned.len(), 1, "{warned:?}");
    rig.esc.set_error(None);
    heard.shown(&rig, idle(3));
    heard.shown(&rig, picking(4, screens()));
    assert!(rig.esc.watching());
}

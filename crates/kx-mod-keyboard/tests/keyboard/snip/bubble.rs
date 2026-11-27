//! The frozen screens after a tap on the keyboard's bubble (ADR-0059): the bubble is ours, so after the Snip key our
//! Esc still goes to Snip and ends the snip.

use super::SNIP;
use super::lines::Lines;
use kx_module_api::ports::{PanelLine, PointerPressed};

#[test]
fn a_tap_on_the_bubble_after_the_snip_key_keeps_esc_for_the_frozen_screens() {
    let l = Lines::new();
    l.b.rig.tap(SNIP).unwrap();
    l.line(Some(PanelLine::None));
    l.b.send(&PointerPressed { ours: true });
    assert!(l.esc_to_snip(), "the bubble is ours, so the aim holds");
}

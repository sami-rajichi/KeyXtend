//! The pad under the shortcuts layer: showing the layer hides the pad, so a pointer on it leaves it, and the scroll
//! module hears so once, as when the pointer moves off.

use super::{Heard, Pad};
use crate::layer::KEY;
use kx_module_api::ports::FaceReply;

#[test]
fn showing_the_layer_under_the_pointer_ends_the_pads_hover_once() {
    let p = Pad::quiet(None);
    p.scroll(true, None, false);
    p.hover(true);
    assert_eq!(p.heard(), [Heard::Peek(true)]);
    p.b.rig.tap(KEY).unwrap();
    assert_eq!(p.heard(), [Heard::Peek(false)], "the outline goes");
    assert_eq!(p.hover(true), FaceReply::default());
    assert!(p.heard().is_empty(), "a stale hover on the hidden pad");
    p.b.rig.tap(KEY).unwrap();
    assert!(p.heard().is_empty(), "the pointer is on the Shortcuts key");
    p.b.rig.tap(KEY).unwrap();
    assert!(p.heard().is_empty(), "off the pad, showing says nothing");
}

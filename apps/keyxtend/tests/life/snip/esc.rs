//! Esc over the frozen screens through the whole app (ADR-0059): the hand's Esc ends a pick and brings the keyboard
//! back as it was, and so does our keyboard's Esc after the Snip key and a tap on the bubble.

use super::{snip, until, view};
use crate::rig::{Rig, WAIT, hear};
use keyxtend::boot::Boot;
use kx_module_api::Subscription;
use kx_module_api::ports::{FaceAction, KeyId, SnipAside, SnipLine, SnipStage, SnipView};
use kx_ui::Mode;
use serde_json::Value;
use std::sync::mpsc::Receiver;

/// The keyboard's Esc key.
const ESC_KEY: &str = "esc";

/// The keyboard's mode as QML reads it.
fn mode(boot: &Boot) -> Value {
    serde_json::from_str(&boot.hub.mode_json()).unwrap()
}

/// A started app in a snip at its Pick view, its keyboard turned into the bubble as QML does, with its mode before the
/// snip, the views and the steps aside heard from then on, and the subscriptions that keep them.
struct Picking {
    boot: Boot,
    before: Value,
    views: Receiver<SnipView>,
    asides: Receiver<SnipAside>,
    _ears: [Subscription; 2],
}

impl Picking {
    fn new(rig: &Rig) -> Self {
        let mut boot = rig.load();
        let (line_ear, lines) = hear::<SnipLine>(&boot.kernel.bus());
        boot.start().unwrap();
        // The snip module's first light and line, told on its own thread, would drop a Snip key click heard before.
        assert_eq!(lines.recv_timeout(WAIT).unwrap().0, None, "it starts idle");
        let before = mode(&boot);
        let (view_ear, views) = snip(&boot);
        // The keyboard heard the capture's line before this ear, so its keys may go there.
        while lines.recv_timeout(WAIT).unwrap().0.is_none() {}
        drop((line_ear, lines));
        boot.hub.keep_mode(Mode::Minimised);
        let (aside_ear, asides) = hear::<SnipAside>(&boot.kernel.bus());
        Self {
            boot,
            before,
            views,
            asides,
            _ears: [view_ear, aside_ear],
        }
    }

    /// The Esc ended the snip: the frozen screens went and the keyboard came back as it was.
    fn ended(&self) {
        until(&self.views, |s| matches!(s, SnipStage::Idle));
        let frozen = view(&self.boot)["frozen"].as_array().map(Vec::len);
        assert_eq!(frozen, Some(0), "the frozen screens went");
        assert_eq!(self.asides.recv_timeout(WAIT).unwrap(), SnipAside::Back);
        assert_eq!(
            mode(&self.boot),
            self.before,
            "the keyboard is back as it was"
        );
    }
}

#[test]
fn the_hands_esc_over_frozen_screens_ends_the_snip_and_brings_the_keyboard_back_as_it_was() {
    let rig = Rig::new();
    let mut p = Picking::new(&rig);
    p.boot.hub.snip_woken();
    assert!(
        rig.snip.esc.press(),
        "the hand's Esc is heard over the frozen screens"
    );
    p.ended();
    p.boot.hub.snip_woken();
    assert!(!rig.snip.esc.watching(), "and not once they went");
    p.boot.kernel.stop_all();
}

#[test]
fn our_keyboards_esc_after_the_snip_key_and_a_tap_on_the_bubble_ends_the_snip() {
    let rig = Rig::new();
    let mut p = Picking::new(&rig);
    p.boot.hub.keep_mode(Mode::Shown);
    let esc = FaceAction::Tap {
        id: KeyId::new(ESC_KEY),
        back: false,
    };
    let before = rig.injector.strokes().len();
    let _ = p.boot.hub.act(esc);
    p.ended();
    assert_eq!(rig.injector.strokes().len(), before, "the app heard no Esc");
    p.boot.kernel.stop_all();
}

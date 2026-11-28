//! The keyboard's step aside: the view goes first, so the ui hides Snip's windows before it answers; only an answer to
//! that view or a newer one freezes, only the newest freeze's screens count, and the key that started it is timed.

use kx_module_api::Clock;
use kx_module_api::ports::{
    AskKey, FrozenScreen, Rect, SnipAct, SnipAside, SnipClick, SnipStage, StripKey,
};
use toml::Value;

use super::super::Msg;
use super::fakes::SCREEN;
use super::rig::{self, Heard, Rig};
use crate::settings::keys;

/// A rig whose keyboard does `keyboard` during a capture.
fn rig_with(keyboard: &str) -> Rig {
    Rig::build(|_, host| host.put(keys::KEYBOARD, Value::String(keyboard.into())))
}

/// The last step aside, and the windows of the view told just before it.
fn before_aside(rig: &Rig) -> (SnipStage, SnipAside) {
    let heard = rig.heard();
    let at = heard.iter().rposition(|h| matches!(h, Heard::Aside(_)));
    let (before, aside) = heard.split_at(at.unwrap());
    let stage = before.iter().rev().find_map(|h| match h {
        Heard::View(page) => Some(page.stage.clone()),
        _ => None,
    });
    let Heard::Aside(aside) = aside[0] else {
        unreachable!("found as an aside")
    };
    (stage.unwrap(), aside)
}

/// Ends held jobs, oldest first, until the fake froze the screens once more.
fn freeze_one(rig: &Rig) {
    let froze = rig.fake.freeze.caps().len();
    while rig.fake.freeze.caps().len() == froze {
        assert!(rig::release_one(), "a freeze is held");
    }
}

#[test]
fn a_snip_from_the_editor_hides_it_first_and_freezes_on_the_uis_answer_in_every_mode() {
    for (keyboard, way) in [
        ("bubble", SnipAside::Bubble),
        ("hidden", SnipAside::Hidden),
        ("shown", SnipAside::Stay),
    ] {
        let mut rig = rig_with(keyboard);
        rig.snap();
        rig.act(SnipClick::Strip(StripKey::Edit));
        rig.send(Msg::Key);
        let freezes = rig.fake.freeze.caps().len();
        rig.act(SnipClick::Ask(AskKey::Drop));
        assert_eq!(before_aside(&rig), (SnipStage::Idle, way), "{keyboard}");
        assert_eq!(rig.fake.freeze.caps().len(), freezes, "{keyboard}: waits");
        rig.act(SnipClick::Aside);
        assert_eq!(rig.fake.freeze.caps().len(), freezes + 1, "{keyboard}");
        assert!(matches!(rig.stage(), SnipStage::Pick(_)), "{keyboard}");
    }
}

#[test]
fn an_answer_to_a_view_before_the_step_aside_is_dropped() {
    let mut rig = Rig::new();
    let old = rig.page().round;
    rig.send(Msg::Key);
    rig.send(Msg::Act(SnipAct {
        round: old,
        click: SnipClick::Aside,
    }));
    assert!(rig.fake.freeze.caps().is_empty(), "an older snip's answer");
    rig.act(SnipClick::Aside);
    assert_eq!(rig.fake.freeze.caps().len(), 1);
}

#[test]
fn only_the_newest_freezes_screens_count() {
    let mut rig = rig_with("shown");
    rig.actor.ports.spawn = rig::held;
    for _ in 0..3 {
        rig.send(Msg::Key);
    }
    let old = Rect {
        w: SCREEN.w / 2,
        ..SCREEN
    };
    rig.fake.freeze.set_screens(vec![(old, 96)]);
    freeze_one(&rig);
    rig.fake.freeze.set_screens(vec![(SCREEN, 96)]);
    freeze_one(&rig);
    rig.pump();
    assert!(matches!(rig.stage(), SnipStage::Pick(_)));
    let frozen: Vec<Rect> = rig.page().frozen.iter().map(FrozenScreen::rect).collect();
    assert_eq!(frozen, [SCREEN], "the first snip's screens are dropped");
}

#[test]
fn the_snip_key_that_starts_a_snip_is_timed_until_its_screens_freeze() {
    let mut rig = Rig::new();
    rig.actor.ports.spawn = rig::held;
    rig.key();
    assert_eq!(
        rig.actor.keyed,
        Some(rig.fake.clock.now()),
        "the key started a snip"
    );
    rig.drain();
    assert!(matches!(rig.stage(), SnipStage::Pick(_)));
    assert_eq!(rig.actor.keyed, None, "timed once, as the screens froze");
    rig.send(Msg::Key);
    assert_eq!(rig.actor.keyed, None, "a key during a snip starts none");
}

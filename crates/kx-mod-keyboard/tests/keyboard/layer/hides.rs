//! While the layer shows, the side keys it hides and the pad are no keys: a stale click where one was, such as Power
//! under the Next cell, does nothing and answers as an unknown key does.

use super::{KEY, side};
use crate::assist::{Bench, ear};
use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::ports::{
    Dir, FaceAction, FaceError, FaceReply, GrabState, PowerAsked, PowerChanged, ScrollChanged,
    ScrollTap,
};
use std::sync::{Arc, Mutex};

/// What a hidden key or the pad would ask another module.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Asked {
    /// The Power key asked for the dialog.
    Power,
    /// An arrow of the pad asked to scroll.
    Pad,
}

/// An arrow of the pad.
const DOWN: FaceAction = FaceAction::Pad {
    dir: Some(Dir::Down),
};

/// The keyboard with the power, mouse and scroll modules live, so every side key would act, and what it asked them.
fn live() -> (Bench, Arc<Mutex<Vec<Asked>>>) {
    let asked = Arc::new(Mutex::new(Vec::new()));
    let heard = Arc::clone(&asked);
    let b = Bench::after(None, move |bus| {
        vec![
            ear(bus, &heard, |_: &PowerAsked| Asked::Power),
            ear(bus, &heard, |_: &ScrollTap| Asked::Pad),
        ]
    });
    b.send(&PowerChanged { live: true });
    b.mouse(true, false, GrabState::Off);
    b.send(&ScrollChanged {
        live: true,
        dir: None,
        fast: false,
    });
    (b, asked)
}

/// What the keyboard asked since the last look.
fn taken(asked: &Mutex<Vec<Asked>>) -> Vec<Asked> {
    std::mem::take(&mut *asked.lock().unwrap())
}

#[test]
fn a_tap_press_or_hold_on_a_side_key_the_layer_hides_or_the_pad_is_no_key() {
    let (b, asked) = live();
    let hidden: Vec<String> = side(&b.rig).into_iter().filter(|id| id != KEY).collect();
    assert!(hidden.iter().any(|id| id == "power"), "{hidden:?}");
    assert!(hidden.iter().any(|id| id == "rclick"), "{hidden:?}");
    let hold = KeyboardSettings::from_defaults().unwrap().repeat_delay_ms;
    b.rig.tap(KEY).unwrap();
    b.said();
    for id in &hidden {
        assert_eq!(b.rig.tap(id), Err(FaceError::UnknownKey), "tap {id}");
        assert_eq!(b.rig.press(id), Err(FaceError::UnknownKey), "press {id}");
        assert_eq!(b.rig.tick_in(hold), FaceReply::default(), "hold {id}");
        assert_eq!(b.rig.let_go(id), Err(FaceError::UnknownKey), "let go {id}");
    }
    assert_eq!(b.rig.act(DOWN), Err(FaceError::UnknownKey), "the pad hides");
    assert!(taken(&asked).is_empty(), "no power dialog, no scroll");
    assert!(b.said().is_empty(), "no mouse mode flips");
    assert!(b.rig.fakes.injector.strokes().is_empty());
    b.rig.tap(KEY).unwrap();
    b.rig.tap("power").unwrap();
    b.rig.act(DOWN).unwrap();
    assert_eq!(taken(&asked), [Asked::Power, Asked::Pad], "back again");
}

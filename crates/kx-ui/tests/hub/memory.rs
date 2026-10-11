//! Free memory goes back to the system after a look change (ADR-0023): each give-back asks the port once, and a refusal
//! warns only the first time and stops nothing.

use crate::Rig;
use crate::logs::warnings_of;
use kx_platform::ui::UiError;

#[test]
fn nothing_is_given_back_until_asked_then_once_per_ask() {
    let (rig, _face, _host) = Rig::with_face();
    assert!(rig.hub.step_theme() && rig.hub.flip_mode());
    assert_eq!(rig.memory.calls(), 0, "the look change alone asks nothing");
    rig.hub.give_back();
    rig.hub.give_back();
    assert_eq!(rig.memory.calls(), 2);
}

#[test]
fn a_refused_give_back_warns_once_and_the_next_ones_still_ask() {
    let rig = Rig::new();
    rig.memory.set_error(Some(UiError::Refused(5)));
    let warned = warnings_of(|| {
        for _ in 0..3 {
            rig.hub.give_back();
        }
    });
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert_eq!(rig.memory.calls(), 3);
    rig.memory.set_error(None);
    assert!(warnings_of(|| rig.hub.give_back()).is_empty());
    assert_eq!(rig.memory.calls(), 4);
}

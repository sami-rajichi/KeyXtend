//! The keyboard's entry watch through the hub: a keyboard the guard does not hold yet is no refusal, and only the watch
//! that follows a guard run's last try, with the scroll module live, warns that it never held, once.

use super::STILL;
use crate::assist::heard;
use crate::logs::warnings_of;
use kx_module_api::ports::ScrollChanged;
use kx_platform::ui::UiError;

/// The keyboard's title, as QML names it.
const KB: &str = "kb";

/// Before the guard's first try, as the module starts, and between its tries, the watch waits quietly.
#[test]
fn a_watch_before_the_guard_holds_the_keyboard_is_quiet() {
    let (rig, bus, _subs) = heard();
    bus.publish(&STILL).unwrap();
    rig.guard.set_unguarded(true);
    let quiet = warnings_of(|| {
        assert!(!rig.hub.watch_entries(KB), "before any guard run");
        assert!(rig.hub.guard(1, None), "no window yet, so it tries again");
        assert!(!rig.hub.watch_entries(KB), "between tries");
    });
    assert!(quiet.is_empty(), "{quiet:?}");
    rig.guard.set_unguarded(false);
    rig.guard.set_count(1);
    assert!(!rig.hub.guard(2, None));
    assert!(rig.hub.watch_entries(KB), "the last try holds it");
}

#[test]
fn a_guard_run_that_ends_with_the_watch_never_held_warns_once() {
    let (rig, bus, _subs) = heard();
    bus.publish(&STILL).unwrap();
    rig.guard.set_unguarded(true);
    rig.guard.set_count(1);
    let warned = warnings_of(|| {
        assert!(!rig.hub.guard(1, None), "the run ends");
        assert!(!rig.hub.watch_entries(KB));
        assert!(!rig.hub.watch_entries(KB), "once");
    });
    assert_eq!(warned.len(), 1, "{warned:?}");
}

/// A run that ended with the module off or before it stopped, or a keyboard not shown, never warns; a refusal still
/// does at once.
#[test]
fn only_a_live_modules_run_warns_and_a_refusal_warns_at_once() {
    let (rig, bus, _subs) = heard();
    rig.guard.set_count(1);
    let quiet = warnings_of(|| {
        assert!(!rig.hub.guard(1, None), "the module is off");
        bus.publish(&STILL).unwrap();
        rig.guard.set_unguarded(true);
        assert!(!rig.hub.watch_entries(KB));
        assert!(!rig.hub.guard(1, None));
        rig.hub.unwatch_entries(KB);
        assert!(!rig.hub.watch_entries(KB), "the module stopped since");
        assert!(!rig.hub.guard(1, None));
        rig.guard.set_error(Some(UiError::NoWindow));
        assert!(!rig.hub.watch_entries(KB), "not shown");
        bus.publish(&ScrollChanged::OFF).unwrap();
    });
    assert!(quiet.is_empty(), "{quiet:?}");
    bus.publish(&STILL).unwrap();
    rig.guard.set_error(Some(UiError::Refused(2)));
    let refused = warnings_of(|| assert!(!rig.hub.watch_entries(KB)));
    assert_eq!(refused.len(), 1, "{refused:?}");
}

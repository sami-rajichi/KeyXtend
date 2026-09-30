//! Step and pick only ask the app in front; the shown layout changes when a refresh sees the switch.

use crate::common::{ids, service, source};
use kx_module_api::ports::{LayoutId, Layouts, SwitchError};

/// A layout id no fake installs.
const UNKNOWN: LayoutId = LayoutId::new(0x0419_0419);

#[test]
fn a_step_only_asks_until_the_app_switches() {
    let [en, fr, _] = ids();
    let fake = source();
    let (layouts, heard) = service(&fake);
    fake.set_follow(false);
    assert_eq!(layouts.step(1), Ok(()));
    assert_eq!(fake.requests(), [fr]);
    assert!(!layouts.refresh(), "the app kept its layout");
    assert_eq!(layouts.current().label.id, en);
    assert!(heard.ids().is_empty());
    fake.set_foreground(Some(fr));
    assert!(layouts.refresh());
    assert_eq!(layouts.current().label.id, fr);
    assert_eq!(heard.ids(), [fr]);
    assert!(!layouts.refresh(), "nothing new");
}

#[test]
fn steps_follow_the_list_and_wrap_both_ways() {
    let [en, fr, ar] = ids();
    let fake = source();
    let (layouts, heard) = service(&fake);
    let steps = [
        (1, fr, true),
        (1, ar, true),
        (1, en, true),
        (-1, ar, true),
        (-2, en, true),
        (3, en, false),
    ];
    for (by, want, changed) in steps {
        assert_eq!(layouts.step(by), Ok(()));
        assert_eq!(layouts.refresh(), changed, "step {by}");
        assert_eq!(shown_now(&layouts), want, "step {by}");
    }
    let asked: Vec<LayoutId> = steps.iter().map(|s| s.1).collect();
    assert_eq!(fake.requests(), asked);
    assert_eq!(heard.ids(), [fr, ar, en, ar, en]);
}

/// The layout the service shows now.
fn shown_now(layouts: &impl Layouts) -> LayoutId {
    layouts.current().label.id
}

#[test]
fn pick_asks_for_an_installed_layout_only() {
    let [_, _, ar] = ids();
    let fake = source();
    let (layouts, _) = service(&fake);
    assert_eq!(layouts.pick(UNKNOWN), Err(SwitchError::NotInstalled));
    assert!(
        fake.requests().is_empty(),
        "an unknown id is never asked for"
    );
    assert_eq!(layouts.pick(ar), Ok(()));
    assert_eq!(fake.requests(), [ar]);
    assert!(layouts.refresh());
    assert_eq!(shown_now(&layouts), ar);
}

#[test]
fn with_no_app_in_front_requests_fail_and_the_pick_stays() {
    let [en, fr, _] = ids();
    let fake = source();
    let (layouts, heard) = service(&fake);
    fake.set_foreground(None);
    assert_eq!(layouts.step(1), Err(SwitchError::NoTarget));
    assert_eq!(layouts.pick(fr), Err(SwitchError::NoTarget));
    assert!(!layouts.refresh());
    assert_eq!(shown_now(&layouts), en);
    assert!(heard.ids().is_empty());
}

#[test]
fn a_refused_request_leaves_the_pick() {
    let [en, fr, _] = ids();
    let fake = source();
    let (layouts, _) = service(&fake);
    fake.set_error(Some(SwitchError::Refused));
    assert_eq!(layouts.step(-1), Err(SwitchError::Refused));
    assert_eq!(layouts.pick(fr), Err(SwitchError::Refused));
    assert!(!layouts.refresh());
    assert_eq!(shown_now(&layouts), en);
}

#[test]
fn a_layout_in_front_that_is_not_installed_keeps_the_pick() {
    let [en, ..] = ids();
    let fake = source();
    let (layouts, heard) = service(&fake);
    fake.set_foreground(Some(UNKNOWN));
    assert!(!layouts.refresh());
    assert_eq!(shown_now(&layouts), en);
    assert!(heard.ids().is_empty());
}

//! The actor driven directly: it says it is live, opens the dialog once per click with nothing held between clicks,
//! tells the user when the dialog does not open, and says it is off when it quits.

use crate::rig::{Rig, SAYS_LIVE, SAYS_OFF, refuse, tells_failed};
use kx_mod_power::Msg;
use kx_module_api::ports::PowerError;
use proptest::collection::vec;
use proptest::prelude::*;

/// The most messages one property case sends.
const MAX_STEPS: usize = 32;

#[test]
fn it_begins_live() {
    let rig = Rig::new();
    rig.begin();
    assert_eq!(rig.take(), [SAYS_LIVE]);
}

#[test]
fn a_click_opens_the_dialog_once_and_tells_nothing() {
    let rig = Rig::new();
    rig.begin();
    rig.take();
    assert!(rig.send(Msg::Asked));
    assert_eq!(rig.dialog.opened(), 1);
    assert_eq!(rig.take(), []);
}

#[test]
fn two_clicks_open_it_twice() {
    let rig = Rig::new();
    rig.send(Msg::Asked);
    rig.send(Msg::Asked);
    assert_eq!(rig.dialog.opened(), 2, "nothing held between clicks");
}

#[test]
fn a_dialog_that_does_not_open_tells_the_user_once_and_stays_live() {
    for error in [PowerError::Unavailable, PowerError::Failed] {
        let rig = Rig::new();
        rig.dialog.set_error(Some(error));
        assert!(rig.send(Msg::Asked), "{error:?}");
        assert_eq!(rig.take(), [tells_failed()], "never retried: {error:?}");
        assert!(rig.send(Msg::Ask));
        assert_eq!(rig.take(), [SAYS_LIVE], "still live: {error:?}");
        rig.dialog.set_error(None);
        rig.send(Msg::Asked);
        assert_eq!(rig.dialog.opened(), 1, "the next click opens: {error:?}");
    }
}

#[test]
fn an_open_with_no_thread_tells_the_user() {
    let rig = Rig::with(refuse);
    assert!(rig.send(Msg::Asked));
    assert_eq!(rig.take(), [tells_failed()]);
    assert_eq!(rig.dialog.opened(), 0);
}

#[test]
fn a_state_ask_tells_live_again() {
    let rig = Rig::new();
    rig.begin();
    assert!(rig.send(Msg::Ask));
    assert_eq!(rig.take(), [SAYS_LIVE, SAYS_LIVE]);
}

#[test]
fn a_quit_tells_off_and_ends_the_actor() {
    let rig = Rig::new();
    rig.begin();
    rig.take();
    assert!(!rig.send(Msg::Quit));
    assert_eq!(rig.take(), [SAYS_OFF]);
    assert_eq!(rig.dialog.opened(), 0);
}

proptest! {
    /// Any mix of clicks, state asks and refused opens: each click opens once or tells the notice once, each state ask
    /// tells live once, and only the quit says off.
    #[test]
    fn each_message_does_its_own_part_and_nothing_more(
        steps in vec((any::<bool>(), any::<bool>()), 0..MAX_STEPS),
    ) {
        let rig = Rig::new();
        rig.begin();
        prop_assert_eq!(rig.take(), vec![SAYS_LIVE]);
        let mut opened = 0;
        for (click, refused) in steps {
            rig.dialog.set_error(refused.then_some(PowerError::Failed));
            let msg = if click { Msg::Asked } else { Msg::Ask };
            prop_assert!(rig.send(msg));
            let want = match (click, refused) {
                (false, _) => vec![SAYS_LIVE],
                (true, true) => vec![tells_failed()],
                (true, false) => {
                    opened += 1;
                    Vec::new()
                }
            };
            prop_assert_eq!(rig.take(), want);
            prop_assert_eq!(rig.dialog.opened(), opened);
        }
        prop_assert!(!rig.send(Msg::Quit));
        prop_assert_eq!(rig.take(), vec![SAYS_OFF]);
    }
}

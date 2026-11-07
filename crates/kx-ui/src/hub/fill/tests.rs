//! The handle the hub keeps for the panel holds only the `ui` module's grants: it sends `FillAct` and `FillNames` but
//! can never paste, and a click names only a round the vault can count.

use super::super::words::tests::hub;
use super::acts::{self, Bad, act_of};
use kx_module_api::ports::{FillAct, FillButton, FillClick, FillNames, PasteAsk};
use kx_module_api::{Bus, Capability, Event, EventError};
use kx_test_support::host::FakeCore;
use std::sync::Arc;

#[test]
fn the_kept_handle_sends_clicks_and_names_and_can_never_paste() {
    let hub = hub();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.hear_fill(&bus).expect("the panel is heard");
    let kept = hub.fill().bus.clone().expect("the handle is kept");
    assert_eq!(kept.check::<FillAct>(), Ok(()));
    assert_eq!(kept.check::<FillNames>(), Ok(()));
    let refused = EventError::NotGranted(PasteAsk::NAME, Capability::WriteClipboard);
    assert_eq!(
        kept.check::<PasteAsk>(),
        Err(refused),
        "only the keyboard pastes"
    );
}

#[test]
fn a_round_the_vault_cannot_count_is_bad() {
    let click = FillClick::Button(FillButton::Close);
    let top = i64::from(u32::MAX);
    for (round, want) in [(0, 0), (3, 3), (top, u32::MAX)] {
        let sent = act_of(acts::BUTTON, "close", 0, round);
        assert_eq!(sent, Ok(FillAct { round: want, click }), "{round}");
    }
    for round in [-1, top + 1, i64::MIN] {
        assert_eq!(act_of(acts::BUTTON, "close", 0, round), Err(Bad), "{round}");
    }
}

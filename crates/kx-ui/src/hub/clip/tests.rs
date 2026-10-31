//! The handle the hub keeps for the panel's clicks holds only the `ui` module's grants: it sends `ClipAct` but can
//! never paste, and only a tick needs the kept view, of the round QML drew.

use super::super::words::tests::hub;
use super::acts::{self, Dropped, act_of};
use crate::i18n::keys::CLIP_FILTERS;
use kx_module_api::ports::{ClipAct, PasteAsk};
use kx_module_api::{Bus, Capability, Event, EventError};
use kx_test_support::host::FakeCore;
use std::sync::Arc;

#[test]
fn the_kept_handle_sends_clicks_and_can_never_paste() {
    let hub = hub();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.hear_clip(&bus).expect("the panel is heard");
    let kept = hub.clip().bus.clone().expect("the handle is kept");
    assert_eq!(kept.check::<ClipAct>(), Ok(()));
    let refused = EventError::NotGranted(PasteAsk::NAME, Capability::WriteClipboard);
    assert_eq!(
        kept.check::<PasteAsk>(),
        Err(refused),
        "ADR-0034: only the keyboard pastes"
    );
}

#[test]
fn only_a_tick_needs_the_kept_view_and_its_round() {
    assert_eq!(act_of(acts::TICK, 0, 4, None), Err(Dropped::Bad));
    assert_eq!(act_of(acts::CLOSE, 0, 1, None), Ok(ClipAct::Close));
    let tick = ClipAct::Tick { round: 4, row: 2 };
    assert_eq!(act_of(acts::TICK, 2, 4, Some((4, 3))), Ok(tick));
    let past = act_of(acts::TICK, 3, 4, Some((4, 3)));
    assert_eq!(past, Err(Dropped::Bad), "past the page");
    for round in [3, 5, -4, i64::from(u32::MAX) + 4] {
        let late = act_of(acts::TICK, 2, round, Some((4, 3)));
        assert_eq!(late, Err(Dropped::Stale), "drawn from round {round}");
    }
}

#[test]
fn a_filter_click_names_the_filter_at_its_place_in_the_json_list() {
    for (at, &(filter, _)) in (0..).zip(&CLIP_FILTERS) {
        let want = Ok(ClipAct::Filter(filter));
        assert_eq!(act_of(acts::FILTER, at, 0, None), want);
    }
    let past = i32::try_from(CLIP_FILTERS.len()).expect("a few filters");
    assert_eq!(act_of(acts::FILTER, past, 0, None), Err(Dropped::Bad));
}

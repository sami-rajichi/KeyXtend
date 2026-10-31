//! The handle the hub keeps for the panel's clicks holds only the `ui` module's grants: it sends `ClipAct` but can
//! never paste, and an act needs the kept view only for a tick.

use super::super::words::tests::hub;
use super::{act_of, acts};
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
fn only_a_tick_needs_the_kept_view() {
    assert_eq!(act_of(acts::TICK, 0, None), None);
    assert_eq!(act_of(acts::CLOSE, 0, None), Some(ClipAct::Close));
    let tick = ClipAct::Tick { round: 4, row: 2 };
    assert_eq!(act_of(acts::TICK, 2, Some((4, 3))), Some(tick));
    assert_eq!(act_of(acts::TICK, 3, Some((4, 3))), None, "past the page");
}

#[test]
fn a_filter_click_names_the_filter_at_its_place_in_the_json_list() {
    for (at, &(filter, _)) in (0..).zip(&CLIP_FILTERS) {
        let want = Some(ClipAct::Filter(filter));
        assert_eq!(act_of(acts::FILTER, at, None), want);
    }
    let past = i32::try_from(CLIP_FILTERS.len()).expect("a few filters");
    assert_eq!(act_of(acts::FILTER, past, None), None);
}

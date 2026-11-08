//! An act in flight has a deadline on the injected clock: an edit the keyboard never answers, or an open the
//! shell holds, hides its strip past `edit_ms` and tells the user, and the loop wakes for it.

use std::time::Duration;

use kx_module_api::keys;
use kx_module_api::ports::{ChipsView, EditDone, SelChip, SelKind};
use toml::Value;

use super::super::Msg;
use super::rig::Rig;
use super::spots::sel;
use crate::settings::TRANSLATE_OK_KEY;

/// The `edit_ms` these tests set.
const EDIT_MS: u32 = 2_000;
/// The smallest step of the clock.
const TICK: Duration = Duration::from_micros(1);

/// A rig whose edits and opens may go unanswered for `EDIT_MS`, with `setup` after.
fn rig(setup: impl FnOnce(&mut kx_test_support::host::FakeHost)) -> Rig {
    Rig::build(|h| {
        h.put("edit_ms", Value::Integer(EDIT_MS.into()));
        setup(h);
    })
}

/// The deadline.
fn edit() -> Duration {
    Duration::from_millis(EDIT_MS.into())
}

#[test]
fn an_edit_the_keyboard_never_answers_hides_its_strip_and_says_so_past_edit_ms() {
    let mut rig = rig(|_| {});
    rig.focus(sel(SelKind::Editable, 40));
    rig.act(SelChip::Copy);
    assert_eq!(rig.wait(), Some(edit()), "the loop wakes for it");
    rig.advance(edit().saturating_sub(TICK));
    assert!(rig.notices().is_empty());
    rig.shown();
    rig.advance(TICK);
    assert_eq!(rig.view(), ChipsView::Hidden);
    assert_eq!(rig.keys(), [keys::KEYS_NOT_SENT]);
    assert_eq!(rig.wait(), None, "nothing more to wake for");
    let told = rig.count();
    rig.answer(true);
    assert_eq!(rig.count(), told, "a late yes ticks nothing");
}

#[test]
fn an_open_the_shell_holds_hides_its_strip_and_says_the_page_did_not_open() {
    let mut rig = rig(|h| h.put(TRANSLATE_OK_KEY, Value::Boolean(true)));
    rig.hold_opens();
    rig.watch.set_text("hello");
    rig.focus(sel(SelKind::ReadOnly, 40));
    rig.act(SelChip::Translate);
    assert_eq!(rig.wait(), Some(edit()));
    rig.advance(edit());
    assert_eq!(rig.view(), ChipsView::Hidden);
    assert_eq!(rig.keys(), [keys::TRANSLATE_FAILED]);
    let told = rig.count();
    rig.release_opens();
    assert_eq!(rig.count(), told, "the late open ticks nothing");
}

#[test]
fn an_answer_in_time_ends_the_deadline() {
    let mut rig = rig(|_| {});
    rig.focus(sel(SelKind::Editable, 40));
    rig.act(SelChip::Copy);
    rig.answer(true);
    assert_eq!(rig.wait(), None);
    rig.advance(edit());
    assert!(rig.notices().is_empty());
}

#[test]
fn a_deadline_past_hides_only_its_own_strip() {
    let mut rig = rig(|_| {});
    rig.focus(sel(SelKind::Editable, 40));
    rig.act(SelChip::Copy);
    rig.focus(sel(SelKind::Editable, 60));
    let (round, _) = rig.shown();
    rig.advance(edit());
    assert_eq!(rig.keys(), [keys::KEYS_NOT_SENT]);
    assert_eq!(rig.shown().0, round, "the new strip stays");
}

/// An older edit's yes, while a newer round's edit flies, still swallows its own break, and the newer stays in flight.
#[test]
fn an_older_edits_yes_swallows_its_break_while_a_newer_edit_flies() {
    let mut rig = rig(|_| {});
    rig.focus(sel(SelKind::Editable, 40));
    let old = rig.round();
    rig.act(SelChip::Copy);
    rig.advance(edit() / 2);
    rig.focus(sel(SelKind::Editable, 60));
    let new = rig.round();
    rig.act(SelChip::Copy);
    rig.send(Msg::Done(EditDone {
        round: old,
        ok: true,
    }));
    rig.broke();
    let shows = matches!(rig.view(), ChipsView::Shown { round, acted: None, .. } if round == new);
    assert!(shows, "the old edit's break is no key: {:?}", rig.heard());
    rig.advance(edit() / 2);
    assert!(rig.notices().is_empty(), "the old edit is done");
    assert_eq!(rig.wait(), Some(edit() / 2), "the loop wakes for the newer");
    rig.answer(true);
    let ticked = matches!(rig.view(), ChipsView::Shown { round, acted: Some(SelChip::Copy), .. } if round == new);
    assert!(ticked, "{:?}", rig.heard());
}

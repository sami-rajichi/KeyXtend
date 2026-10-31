//! The fake shortcuts: sends recorded in order with their held keys, unknown ids and set errors refused, and a
//! `Debug` that shows counts only.

use super::*;
use kx_module_api::ports::{KeyName, Step};
use std::sync::Arc;

/// Left Ctrl and left Shift.
const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);

fn id(text: &str) -> ShortcutId {
    ShortcutId::new(text).unwrap()
}

/// `id` as one tap of key `key`.
fn info(text: &str, key: &str) -> ShortcutInfo {
    let step = Step {
        held: Vec::new(),
        key: KeyName::new(key).unwrap(),
    };
    ShortcutInfo {
        id: id(text),
        steps: vec![step],
    }
}

/// A fake that knows `copy` and `f5`.
fn fake() -> FakeShortcuts {
    let fake = FakeShortcuts::new();
    fake.set_known(vec![info("copy", "c"), info("f5", "f5")]);
    fake
}

#[test]
fn known_is_what_the_test_set() {
    assert!(FakeShortcuts::new().known().is_empty());
    assert_eq!(fake().known(), [info("copy", "c"), info("f5", "f5")]);
}

#[test]
fn sends_are_recorded_in_order_with_their_held_keys() {
    let fake = fake();
    assert_eq!(fake.send(&id("copy"), &[CTRL, SHIFT]), Ok(()));
    assert_eq!(fake.send(&id("f5"), &[]), Ok(()));
    let want = [(id("copy"), vec![CTRL, SHIFT]), (id("f5"), Vec::new())];
    assert_eq!(fake.sent(), want);
}

#[test]
fn an_unknown_id_is_refused_and_records_nothing() {
    let fake = fake();
    assert_eq!(
        fake.send(&id("paste"), &[CTRL]),
        Err(ShortcutError::Unknown)
    );
    let empty = FakeShortcuts::new();
    assert_eq!(empty.send(&id("copy"), &[]), Err(ShortcutError::Unknown));
    assert!(fake.sent().is_empty() && empty.sent().is_empty());
}

#[test]
fn a_set_error_refuses_every_later_send_until_cleared() {
    let fake = fake();
    let refused = ShortcutError::Refused { sent: 1, total: 4 };
    fake.set_error(Some(refused));
    assert_eq!(fake.send(&id("copy"), &[]), Err(refused));
    assert_eq!(fake.send(&id("f5"), &[]), Err(refused));
    assert_eq!(fake.send(&id("paste"), &[]), Err(ShortcutError::Unknown));
    assert!(fake.sent().is_empty());
    fake.set_error(None);
    assert_eq!(fake.send(&id("copy"), &[]), Ok(()));
    assert_eq!(fake.sent().len(), 1);
}

#[test]
fn clear_forgets_the_sends_but_not_the_ids() {
    let fake = fake();
    fake.send(&id("copy"), &[]).unwrap();
    fake.clear();
    assert!(fake.sent().is_empty());
    assert_eq!(fake.known().len(), 2);
}

#[test]
fn clones_share_one_record_and_serve_as_the_port_from_another_thread() {
    let fake = fake();
    let port: Arc<dyn Shortcuts> = Arc::new(fake.clone());
    std::thread::spawn(move || port.send(&id("f5"), &[CTRL]))
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(fake.sent(), [(id("f5"), vec![CTRL])]);
}

#[test]
fn debug_shows_counts_and_no_id_or_key() {
    let fake = fake();
    fake.send(&id("copy"), &[CTRL]).unwrap();
    let shown = format!("{fake:?}");
    assert_eq!(shown, "FakeShortcuts { known: 2, sent: 1, .. }");
}

#[test]
fn the_history_chord_is_what_the_test_scripted_in_any_order() {
    let fake = fake();
    let (win, v) = (Scan::new(0xE05B), Scan::new(0x2F));
    assert!(!fake.is_history(&[win], v), "none until scripted");
    fake.set_history(Some((vec![win, CTRL], v)));
    assert!(fake.is_history(&[win, CTRL], v));
    assert!(fake.is_history(&[CTRL, win], v), "in any order");
    assert!(!fake.is_history(&[win], v), "a held key missing");
    assert!(!fake.is_history(&[win, CTRL, SHIFT], v), "one more held");
    assert!(!fake.is_history(&[win, CTRL], SHIFT), "another key");
    fake.set_history(None);
    assert!(!fake.is_history(&[win, CTRL], v));
    assert!(fake.sent().is_empty(), "an ask sends nothing");
}

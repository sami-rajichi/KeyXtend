//! Paste at once from the first dictation (ADR-0048 decision 5): the clipboard is checked for a private marker at the
//! read, so a plain text there before the start is pasted over and comes back. Our own bar Copy's mark is not the
//! history's echo, so after it the text is typed until the next copy or clear.

use kx_module_api::ports::ClipNote;

use super::super::rig::Rig;
use super::{lend, lent, round_trip};
use crate::fixture::text;

/// The text on the clipboard before the start.
const BEFORE: &str = "there before the start";

#[test]
fn at_the_start_a_plain_text_is_pasted_over_and_comes_back() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some(BEFORE));
    assert_eq!(round_trip(&mut rig), [text(BEFORE)]);
    assert_eq!(lent(&rig), [true]);
}

#[test]
fn at_the_start_a_marked_text_is_typed_and_the_clipboard_left_untouched() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_marked(Some("hunter2"));
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
    assert_eq!(rig.fakes.clip.takes(), 0, "never read");
    assert!(rig.pastes().is_empty());
}

#[test]
fn after_our_own_bar_copy_the_text_is_typed_until_the_next_copy() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some("copied on the bar"));
    rig.push(ClipNote::Marked { app: None });
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
    assert_eq!(rig.fakes.clip.takes(), 0, "never read");
    rig.fakes.clip.set_text(Some("copied"));
    rig.text("copied");
    assert_eq!(round_trip(&mut rig), [text("copied")]);
}

//! When a lent paste leaves the clipboard alone: one with a private marker on it is never read or written back, even
//! when a stale echo kept the board, a kept entry that cannot come back whole is typed, a text past `back_chars` is
//! never read whole, and a copy made during the gap is never overwritten.

use kx_module_api::ports::{ClipNote, ClipPart, SealedError};
use kx_platform_fake::ShelfCall;
use toml::Value;

use super::super::rig::Rig;
use super::{SAID, holding, html, lend, lent, round_trip};
use crate::fixture::{app, text};

/// The most characters the rigs here read back.
const CHARS: usize = 1000;

/// A rig that reads at most `CHARS` characters back.
fn capped() -> Rig {
    let chars = i64::try_from(CHARS).unwrap();
    Rig::build(|_, host| host.put("back_chars", Value::Integer(chars)))
}

#[test]
fn a_copy_another_app_marked_private_is_never_read_or_written_back() {
    let mut rig = holding(vec![text("kept")]);
    rig.push(ClipNote::Marked {
        app: Some(app("vault")),
    });
    rig.fakes.clip.set_marked(Some("hunter2"));
    let takes = rig.fakes.clip.takes();
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
    assert_eq!(rig.fakes.clip.takes(), takes, "never read");
    assert!(rig.pastes().is_empty());
    rig.fakes.clip.set_text(Some("after"));
    rig.text("after");
    lend(&mut rig);
    assert_eq!(rig.pastes().len(), 1, "a new copy is no longer private");
}

#[test]
fn a_text_past_back_chars_is_never_read_whole_and_the_text_is_typed() {
    let mut rig = capped();
    let long = "a".repeat(CHARS + 1);
    rig.fakes.clip.set_text(Some(&long));
    rig.text(&long);
    lend(&mut rig);
    assert_eq!(rig.fakes.clip.take_cap(), Some(CHARS));
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
    let fits = "b".repeat(CHARS);
    rig.fakes.clip.set_text(Some(&fits));
    rig.text(&fits);
    lend(&mut rig);
    assert_eq!(rig.pastes().len(), 1, "a text within the cap comes back");
}

#[test]
fn a_copy_made_during_the_gap_is_never_overwritten_but_our_echo_is() {
    let mut rig = holding(vec![text("kept")]);
    lend(&mut rig);
    rig.answer(true);
    rig.push(ClipNote::Marked { app: None });
    rig.advance(rig.settings.paste_gap());
    let back = rig.fakes.writer.writes();
    assert_eq!(
        back.last(),
        Some(&vec![text("kept")]),
        "our echo changes nothing"
    );
    lend(&mut rig);
    rig.answer(true);
    rig.fakes.clip.set_text(Some("new"));
    rig.text("new");
    rig.advance(rig.settings.paste_gap());
    let writes: Vec<Vec<ClipPart>> = rig.fakes.writer.writes();
    assert_eq!(writes.last(), Some(&vec![text(SAID)]), "the new copy stays");
    assert_eq!(writes.len(), back.len() + 1);
}

#[test]
fn a_stale_echo_never_puts_back_a_marked_text() {
    let mut rig = Rig::new();
    rig.push(ClipNote::TooBig {
        app: None,
        bytes: u64::MAX,
    });
    rig.fakes.clip.set_text(Some("long"));
    assert_eq!(round_trip(&mut rig), [text("long")], "two echoes still due");
    rig.push(ClipNote::Marked {
        app: Some(app("vault")),
    });
    rig.fakes.clip.set_marked(Some("hunter2"));
    let (takes, tries) = (rig.fakes.clip.takes(), rig.fakes.writer.tries());
    lend(&mut rig);
    assert_eq!(lent(&rig).last(), Some(&false));
    let after = (rig.fakes.clip.takes(), rig.fakes.writer.tries());
    assert_eq!(after, (takes, tries), "never read or written");
}

#[test]
fn a_kept_entry_on_the_board_is_refused_once_its_twin_on_the_clipboard_is_marked() {
    let mut rig = holding(vec![text("kept")]);
    assert_eq!(round_trip(&mut rig), [text("kept")], "two echoes still due");
    rig.push(ClipNote::Marked {
        app: Some(app("vault")),
    });
    rig.fakes.clip.set_marked(Some("kept"));
    let tries = rig.fakes.writer.tries();
    lend(&mut rig);
    let refused = (lent(&rig).last().copied(), rig.fakes.writer.tries());
    assert_eq!(
        refused,
        (Some(false), tries),
        "the whole entry never goes back"
    );
}

#[test]
fn a_kept_entry_that_cannot_come_back_whole_is_typed_and_the_clipboard_untouched() {
    let mut rig = holding(vec![text("rich"), html("<b>rich</b>")]);
    rig.fakes.shelf.fail(ShelfCall::Load, SealedError::Damaged);
    lend(&mut rig);
    let refused = (lent(&rig), rig.fakes.writer.tries());
    assert_eq!(
        refused,
        (vec![false], 0),
        "its text alone would drop the HTML"
    );
}

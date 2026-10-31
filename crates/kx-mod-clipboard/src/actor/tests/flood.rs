//! A flood of copies: each waits in a queue of `queue_copies` until the actor takes it, a copy past it is dropped with
//! one warning per flood that names no content, and every other note still goes.

use std::sync::mpsc;

use kx_module_api::ports::ClipNote;
use toml::Value;

use super::super::Msg;
use super::super::watched::Watched;
use super::logs::lines_of;
use super::rig::Rig;
use crate::fixture::text;

/// Words of the warning, which carries no content.
const FELL_BEHIND: &str = "fell behind";
/// What every copied text in these tests starts with, so a log line holding one is found.
const SECRET: &str = "zebra-";

/// A copy of the text `SECRET` and `n`.
fn copy(n: usize) -> ClipNote {
    ClipNote::Copied {
        parts: vec![text(&format!("{SECRET}{n}"))],
        app: None,
    }
}

/// How many of `lines` are the flood's warning.
fn warned(lines: &[String]) -> usize {
    lines.iter().filter(|l| l.contains(FELL_BEHIND)).count()
}

#[test]
fn copies_past_the_queue_are_dropped_with_one_warning_per_flood() {
    let (tx, inbox) = mpsc::channel();
    let (watched, copies) = Watched::new(&tx, 2);
    let lines = lines_of(|| (0..5).for_each(|n| watched.tell(copy(n))));
    assert_eq!(inbox.try_iter().count(), 2, "one message per copy kept");
    let kept: Vec<ClipNote> = copies.try_iter().collect();
    assert_eq!(kept, [copy(0), copy(1)], "the first copies wait");
    assert_eq!(warned(&lines), 1, "{lines:?}");
    assert!(lines.iter().all(|l| !l.contains(SECRET)), "{lines:?}");
    let again = lines_of(|| (5..9).for_each(|n| watched.tell(copy(n))));
    assert_eq!(
        copies.try_iter().count(),
        2,
        "copies taken free their places"
    );
    assert_eq!(warned(&again), 1, "a new flood warns again");
}

#[test]
fn every_other_note_goes_while_the_queue_is_full() {
    let (tx, inbox) = mpsc::channel();
    let (watched, _copies) = Watched::new(&tx, 1);
    for note in [
        copy(0),
        copy(1),
        ClipNote::Busy,
        ClipNote::Cleared { app: None },
    ] {
        watched.tell(note);
    }
    let msgs: Vec<Msg> = inbox.try_iter().collect();
    let notes = [ClipNote::Busy, ClipNote::Cleared { app: None }];
    match msgs.as_slice() {
        [Msg::Copied, Msg::Note(a), Msg::Note(b)] => assert_eq!([a, b], [&notes[0], &notes[1]]),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_flood_before_the_actor_wakes_keeps_only_the_queued_copies() {
    let mut rig = Rig::build(|_, host| host.put("queue_copies", Value::Integer(3)));
    for n in 0..10 {
        rig.fakes.clip.push(copy(n));
    }
    rig.open();
    let want: Vec<String> = (0..3).rev().map(|n| format!("{SECRET}{n}")).collect();
    assert_eq!(rig.titles(), want, "the newest copies were dropped");
}

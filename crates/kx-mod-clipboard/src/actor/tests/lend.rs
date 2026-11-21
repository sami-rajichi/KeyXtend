//! Paste at once for another module (owner, 2026-11-17): the clipboard is kept, the text written as ours and pasted
//! once, the answer sent right after the keyboard's, and the clipboard put back after the gap. One with a private
//! marker on it, a skipped app's copy, one that cannot be put back whole, a busy read, a late ask or an insert under
//! way answers no and writes nothing.

use std::time::Duration;

use kx_module_api::ports::{
    ClipAct, ClipError, ClipNote, ClipPart, FileFact, ImageCodec, PasteText, TextPasted,
};
use kx_module_api::{Clock, Mono, Redacted, Zeroizing};

use super::super::Msg;
use super::ears::Heard;
use super::rig::Rig;
use crate::fixture::{files, image, text};

mod back;
mod gone;
mod props;
mod skip;
mod start;

/// The asker's round.
const ASKER: u32 = 7;
/// How long the asker waits, as the voice module's default.
const WAIT_MS: u32 = 2000;
/// The text the asker hands over.
const SAID: &str = "hello ";
/// A copied file.
const PATH: &str = "C:\\a.txt";

/// The ask for `SAID`.
fn said() -> PasteText {
    let text = Redacted::new(Zeroizing::new(SAID.to_owned()));
    PasteText {
        round: ASKER,
        text,
        wait_ms: WAIT_MS,
    }
}

/// The ask for `SAID`, made at `at`.
fn ask(rig: &mut Rig, at: Mono) {
    rig.send(Msg::Lend(said(), at));
}

/// The ask for `SAID`, made now.
fn lend(rig: &mut Rig) {
    let now = rig.fakes.clock.now();
    ask(rig, now);
}

/// Every answer sent, as its flag.
fn lent(rig: &Rig) -> Vec<bool> {
    let pick = |h| match h {
        Heard::Lent(TextPasted { round, ok }) if round == ASKER => Some(ok),
        _ => None,
    };
    rig.all().into_iter().filter_map(pick).collect()
}

/// Lends, the keyboard pastes, and the gap passes; gives the parts put back.
fn round_trip(rig: &mut Rig) -> Vec<ClipPart> {
    lend(rig);
    rig.answer(true);
    rig.advance(rig.settings.paste_gap());
    rig.fakes.writer.writes().pop().unwrap()
}

/// A rig whose newest copy is `parts`, the clipboard's text set to match.
fn holding(parts: Vec<ClipPart>) -> Rig {
    let mut rig = Rig::new();
    let said = parts.iter().find_map(|p| match p {
        ClipPart::Text(t) => Some(t.expose().clone()),
        _ => None,
    });
    rig.fakes.clip.set_text(said.as_deref());
    rig.copy(parts, None);
    rig
}

/// An HTML part.
fn html(s: &str) -> ClipPart {
    ClipPart::Html(Redacted::new(s.as_bytes().to_vec()))
}

#[test]
fn the_text_is_pasted_once_answered_then_the_copy_comes_back() {
    let mut rig = holding(vec![text("kept")]);
    lend(&mut rig);
    assert_eq!(rig.fakes.writer.writes(), [vec![text(SAID)]]);
    assert_eq!((rig.pastes().len(), lent(&rig)), (1, vec![]));
    rig.answer(true);
    assert_eq!(lent(&rig), [true], "answered right after the keyboard");
    assert_eq!(rig.fakes.writer.writes().len(), 1, "the app reads it first");
    rig.advance(rig.settings.paste_gap());
    assert_eq!(
        rig.fakes.writer.writes(),
        [vec![text(SAID)], vec![text("kept")]]
    );
    rig.open();
    assert_eq!(rig.titles(), ["kept"], "the history keeps neither write");
}

#[test]
fn the_echoes_of_our_writes_keep_the_entry_whole_and_another_mark_holds_the_paste() {
    let mut rig = holding(vec![text("kept"), html("<b>kept</b>")]);
    let whole = round_trip(&mut rig);
    assert_eq!(whole, [text("kept"), html("<b>kept</b>")]);
    let marked = || ClipNote::Marked { app: None };
    rig.push(marked());
    rig.push(marked());
    assert_eq!(round_trip(&mut rig), whole, "our two echoes");
    for _ in 0..3 {
        rig.push(marked());
    }
    let tries = rig.fakes.writer.tries();
    lend(&mut rig);
    assert_eq!(lent(&rig).last(), Some(&false), "a mark not ours");
    assert_eq!(rig.fakes.writer.tries(), tries, "nothing written");
}

#[test]
fn pictures_and_files_come_back_whole_and_a_png_alone_is_refused() {
    let dib = ClipPart::Image {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(vec![4]),
    };
    let mut rig = holding(vec![image(), dib.clone()]);
    assert_eq!(round_trip(&mut rig), [image(), dib]);
    let mut rig = Rig::build(|f, _| f.clip.set_file(PATH, FileFact::Size(1)));
    rig.copy(vec![files(&[PATH])], None);
    assert_eq!(round_trip(&mut rig), [files(&[PATH])]);
    let mut rig = holding(vec![image()]);
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
}

#[test]
fn a_text_the_history_never_kept_comes_back_as_text_and_an_empty_clipboard_empty() {
    let mut rig = Rig::new();
    rig.push(ClipNote::TooBig {
        app: None,
        bytes: u64::MAX,
    });
    rig.fakes.clip.set_text(Some("long"));
    assert_eq!(round_trip(&mut rig), [text("long")]);
    rig.push(ClipNote::Cleared { app: None });
    rig.fakes.clip.set_text(None);
    assert_eq!(round_trip(&mut rig), []);
}

#[test]
fn a_clipboard_it_cannot_put_back_or_read_answers_no_and_writes_nothing() {
    let mut rig = Rig::new();
    lend(&mut rig);
    let none = (lent(&rig), rig.fakes.writer.tries());
    assert_eq!(none, (vec![false], 0), "no text may still be a picture");
    let mut rig = holding(vec![text("kept")]);
    rig.fakes.clip.fail_text(ClipError::Busy);
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
    assert!(rig.pastes().is_empty());
}

#[test]
fn a_refused_write_answers_no_and_puts_back_unless_the_clipboard_is_held() {
    let mut rig = holding(vec![text("kept")]);
    rig.fakes.writer.queue(Err(ClipError::Busy));
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 1));
    rig.fakes.writer.queue(Err(ClipError::Io));
    lend(&mut rig);
    assert_eq!(lent(&rig), [false, false]);
    assert_eq!(rig.fakes.writer.writes(), [vec![text("kept")]], "put back");
    assert!(rig.pastes().is_empty());
}

#[test]
fn a_paste_not_all_sent_puts_back_after_the_gap_and_one_never_answered_at_once() {
    let mut rig = holding(vec![text("kept")]);
    lend(&mut rig);
    rig.answer(false);
    assert_eq!(lent(&rig), [false]);
    assert_eq!(
        rig.fakes.writer.writes().len(),
        1,
        "a part may still be read"
    );
    rig.advance(rig.settings.paste_gap());
    assert_eq!(rig.fakes.writer.writes().len(), 2, "back after the gap");
    rig.advance(rig.settings.save());
    lend(&mut rig);
    assert_eq!(
        rig.wait(),
        rig.settings.paste_wait(),
        "the loop wakes for it"
    );
    rig.advance(rig.settings.paste_wait());
    assert_eq!(lent(&rig), [false, false]);
    assert_eq!(rig.fakes.writer.writes().last(), Some(&vec![text("kept")]));
    rig.answer(true);
    assert_eq!(lent(&rig).len(), 2, "a late answer is dropped");
}

#[test]
fn an_ask_past_its_start_answers_no_and_leaves_the_clipboard() {
    let mut rig = holding(vec![text("kept")]);
    let at = rig.fakes.clock.now();
    let start = u64::from(said().start_ms());
    rig.advance(Duration::from_millis(start + 1));
    ask(&mut rig, at);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 0));
}

#[test]
fn an_insert_and_a_lend_never_run_together() {
    let mut rig = holding(vec![text("kept")]);
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Insert));
    lend(&mut rig);
    assert_eq!((lent(&rig), rig.fakes.writer.tries()), (vec![false], 1));
    rig.answer(true);
    rig.advance(rig.settings.paste_gap());
    rig.open();
    rig.tick_row(0);
    lend(&mut rig);
    rig.send(Msg::Act(ClipAct::Insert));
    assert_eq!(
        rig.pastes().len(),
        2,
        "an insert asked meanwhile is dropped"
    );
    rig.answer(true);
    assert_eq!(lent(&rig), [false, true]);
}

#[test]
fn an_inserted_entry_comes_back_whole_after_its_echo() {
    let mut rig = holding(vec![text("rich"), html("<i>rich</i>")]);
    rig.text("plain");
    rig.open();
    rig.tick_row(1);
    rig.send(Msg::Act(ClipAct::Insert));
    rig.answer(true);
    rig.advance(rig.settings.paste_gap());
    rig.push(ClipNote::Marked { app: None });
    rig.fakes.clip.set_text(Some("rich"));
    assert_eq!(round_trip(&mut rig), [text("rich"), html("<i>rich</i>")]);
}

#[test]
fn a_stop_puts_the_clipboard_back() {
    let mut rig = holding(vec![text("kept")]);
    lend(&mut rig);
    rig.answer(true);
    rig.stop();
    assert_eq!(rig.fakes.writer.writes().last(), Some(&vec![text("kept")]));
}

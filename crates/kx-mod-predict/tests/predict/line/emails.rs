//! The menu and emails on a search line: `@@` lists known and saved emails, a pick from the chips or the list goes to the
//! line, a saved email marks nothing, and Forget with its Undo works as in a box.

use super::{aimed, search};
use crate::rig::{ROWS, Rig};
use kx_mod_predict::Msg;
use kx_mod_predict::email::Emails;
use kx_mod_predict::score::Learned;
use kx_module_api::ports::{ChipKind, FieldKind, Lane, PanelLine, WordsAsk};
use kx_module_api::{Redacted, Zeroizing};

/// A known email and a saved one.
const KNOWN: &str = "nour@y.com";
const SAVED: &str = "alex@notebook.org";

/// A rig in English knowing both emails, aimed at a search holding `text`.
fn knowing(text: &str) -> Rig {
    let bytes = Emails::new(&[KNOWN.to_owned()], usize::MAX).to_bytes();
    let name = kx_mod_predict::emails_blob().unwrap();
    let mut rig = Rig::build(|f, _| f.store.plant(&name, bytes));
    rig.lang("en");
    rig.focus(FieldKind::Password);
    let saved = Redacted::new(Zeroizing::new(SAVED.to_owned()));
    rig.send(Msg::Saved(vec![saved]));
    rig.line(search(text));
    rig
}

#[test]
fn an_at_at_on_a_search_lists_known_and_saved_emails() {
    let rig = knowing("@@");
    let want = [
        (KNOWN.to_owned(), ChipKind::Email),
        (SAVED.to_owned(), ChipKind::Saved),
    ];
    assert_eq!(rig.chips(), want);
}

#[test]
fn a_chip_after_at_at_replaces_it_on_the_line() {
    let mut rig = knowing("find @@");
    rig.pick(1);
    assert_eq!(rig.typed_words(), [(2, SAVED.to_owned())]);
    assert_eq!(rig.word_targets(), [true]);
}

#[test]
fn the_list_opens_on_a_search_and_its_row_goes_to_the_line() {
    let mut rig = knowing("@@");
    rig.ask(WordsAsk::More { rows: ROWS });
    assert!(rig.lane().list.is_some());
    rig.ask(WordsAsk::ListPick(0));
    assert_eq!(rig.typed_words(), [(2, KNOWN.to_owned())]);
    assert_eq!(rig.word_targets(), [true]);
    assert!(rig.lane().list.is_none());
}

#[test]
fn a_new_line_closes_the_list() {
    let mut rig = knowing("@@");
    rig.ask(WordsAsk::More { rows: ROWS });
    rig.line(search("@@ "));
    assert!(rig.lane().list.is_none());
}

#[test]
fn a_saved_pick_on_a_search_leaves_no_wait_so_typing_it_by_hand_later_is_learned() {
    let mut rig = knowing("al");
    rig.focus(FieldKind::Text);
    assert_eq!(rig.chips(), [(SAVED.to_owned(), ChipKind::Saved)]);
    rig.pick(0);
    assert_eq!(rig.word_targets(), [true]);
    rig.line(search(SAVED));
    rig.line(PanelLine::None);
    let (start, rest) = SAVED.split_at(2);
    rig.typed(start);
    rig.typed(rest);
    rig.typed(" ");
    rig.stop();
    let name = kx_mod_predict::emails_blob().unwrap();
    let bytes = rig.fakes.store.blob(&name).unwrap();
    let list = Emails::from_bytes(&bytes, usize::MAX).unwrap();
    assert!(list.list().contains(&SAVED.to_owned()));
}

#[test]
fn a_saved_pick_on_a_search_is_never_stored() {
    let mut rig = knowing("al");
    rig.pick(0);
    rig.line(search(SAVED));
    rig.stop();
    let name = kx_mod_predict::emails_blob().unwrap();
    let bytes = rig.fakes.store.blob(&name).unwrap();
    let list = Emails::from_bytes(&bytes, usize::MAX).unwrap();
    assert!(!list.list().contains(&SAVED.to_owned()));
}

#[test]
fn a_domain_chip_goes_to_the_line() {
    let mut rig = aimed("alex@");
    assert!(rig.chips().iter().all(|c| c.1 == ChipKind::Domain));
    rig.pick(0);
    assert_eq!(rig.word_targets(), [true]);
    assert_eq!(rig.typed_words()[0].0, 0, "nothing erased");
}

#[test]
fn forget_then_undo_work_on_a_search() {
    let mut rig = aimed("hel");
    rig.ask(WordsAsk::Forget);
    assert!(rig.lane().forgetting);
    rig.pick(0);
    assert_eq!(rig.words(), ["help"], "hello is gone");
    assert!(rig.typed_words().is_empty(), "forgetting types nothing");
    rig.pick(0);
    assert_eq!(rig.words(), ["hello", "help"], "the Undo chip");
}

#[test]
fn a_word_forgotten_on_a_search_stays_forgotten_after_the_stop() {
    let mut rig = aimed("hel");
    rig.ask(WordsAsk::Forget);
    rig.pick(0);
    rig.stop();
    let learner = crate::rig::saved(&rig.fakes, "en").unwrap();
    assert_eq!(learner.view(None).count("hello"), 0);
    assert_eq!(rig.lane().lane, Lane::Stopped);
}

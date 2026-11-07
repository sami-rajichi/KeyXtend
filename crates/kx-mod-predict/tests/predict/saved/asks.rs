//! Asking for the notebook's emails: prediction asks at start and at turn-on, holds none while off, and an
//! open email list follows a set that shrinks or empties.

use super::{SAVED, knowing, notebook};
use crate::rig::{Heard, ROWS, Rig};
use kx_mod_predict::ON_KEY;
use kx_module_api::ports::{FieldKind, Lane, WordsAsk};
use toml::Value;

/// How many times `rig` asked Quick-fill to say its state again.
fn asks(rig: &Rig) -> usize {
    rig.all().iter().filter(|h| **h == Heard::FillAsk).count()
}

/// `n` saved emails.
fn users(n: usize) -> Vec<String> {
    (0..n).map(|n| format!("user{n}@notebook.org")).collect()
}

#[test]
fn prediction_asks_for_the_notebooks_emails_at_start_and_at_turn_on() {
    let mut rig = Rig::new();
    assert_eq!(asks(&rig), 1, "at start");
    rig.ask(WordsAsk::Power(false));
    assert_eq!(asks(&rig), 1, "never when turned off");
    rig.ask(WordsAsk::Power(true));
    assert_eq!(asks(&rig), 2, "again at turn-on");
    let off = Rig::build(|_, host| host.put(ON_KEY, Value::Boolean(false)));
    assert_eq!(asks(&off), 0, "a start while off asks nothing");
}

#[test]
fn turning_prediction_off_drops_the_notebooks_emails_and_keeps_none_while_off() {
    let mut rig = knowing(&[], &SAVED);
    rig.ask(WordsAsk::Power(false));
    rig.send(notebook(&SAVED));
    rig.ask(WordsAsk::Power(true));
    rig.focus(FieldKind::Text);
    rig.typed("@@");
    assert_eq!(
        rig.lane().lane,
        Lane::Empty,
        "none kept until the vault answers"
    );
    rig.send(notebook(&SAVED));
    assert_eq!(rig.chips().len(), SAVED.len(), "its answer shows");
}

#[test]
fn a_set_that_shrinks_moves_the_open_list_to_a_page_it_has() {
    let many = users(7);
    let all: Vec<&str> = many.iter().map(String::as_str).collect();
    let mut rig = knowing(&[], &all);
    rig.typed("@@");
    rig.ask(WordsAsk::More { rows: ROWS });
    rig.ask(WordsAsk::Page {
        page: 1,
        rows: ROWS,
    });
    assert_eq!(rig.lane().list.map(|l| l.page), Some(1));
    rig.send(notebook(&all[..3]));
    let list = rig.lane().list.unwrap();
    assert_eq!((list.page, list.pages, list.items.len()), (0, 1, 3));
    rig.ask(WordsAsk::ListPick(2));
    assert_eq!(
        rig.typed_words(),
        [(2, many[2].clone())],
        "its rows still type"
    );
}

#[test]
fn a_set_that_empties_closes_the_open_list() {
    let many = users(7);
    let all: Vec<&str> = many.iter().map(String::as_str).collect();
    let mut rig = knowing(&[], &all);
    rig.typed("@@");
    rig.ask(WordsAsk::More { rows: ROWS });
    assert!(rig.lane().list.is_some());
    rig.send(notebook(&[]));
    assert_eq!(rig.lane().list, None);
}

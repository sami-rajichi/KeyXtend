//! Filling: Both types the user, a Tab and the password with the Tab wait and never Enter; one button types one value;
//! a value `FillText` refuses sends nothing; the answer of the last request alone counts, and a fill waits for it.

use std::time::Duration;

use kx_module_api::ports::{EntryKind, FillNote, FillPart, FillType, FillTyped};

use super::*;
use crate::book::Notebook;
use crate::entry::{Draft, Entry, Stamp};
use crate::fixture::{LIMITS, draft, field};
use crate::panel::{Fill, Part};
use kx_platform_fake::FakeRandom;

/// The default Tab wait.
const GAP: Duration = Duration::from_millis(150);
/// The login's user and password.
const USER: &str = "alex@example.com";
const PASSWORD: &str = "zebra-quartz";

/// A notebook holding `drafts`, and their ids in order.
fn book(drafts: Vec<Draft>) -> (Notebook, Vec<u64>) {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(3));
    let ids = drafts.into_iter().map(|d| {
        let now = Stamp::from_unix_ms(1);
        book.add(d, now, &random, &LIMITS).unwrap()
    });
    let ids = ids.collect();
    (book, ids)
}

/// A login with its user and password.
fn login() -> Draft {
    let fields = vec![
        field("user", USER, false),
        field("password", PASSWORD, true),
    ];
    draft("Bank", EntryKind::Login, fields)
}

/// The texts and Tabs `ask` types.
fn typed(ask: &FillType) -> Vec<String> {
    let part = |p: &FillPart| match p {
        FillPart::Text(t) => t.text().expose().to_string(),
        FillPart::Tab => "<tab>".to_owned(),
    };
    ask.parts.iter().map(part).collect()
}

/// The fill of entry `id` with `parts`.
fn fill(id: u64, parts: Vec<Part>) -> Fill {
    Fill { id, parts }
}

#[test]
fn both_types_the_user_a_tab_and_the_password_with_the_tab_wait_and_never_enter() {
    let (book, ids) = book(vec![login()]);
    let both = fill(ids[0], vec![Part::Value(0), Part::Tab, Part::Value(1)]);
    let ask = Filling::default().ask(&both, &book, GAP).unwrap();
    assert_eq!(typed(&ask), [USER, "<tab>", PASSWORD]);
    assert_eq!(ask.tab_gap, GAP);
    let enter = |t: &String| t.contains(['\r', '\n']);
    assert!(!typed(&ask).iter().any(enter), "no Enter");
}

#[test]
fn one_button_types_its_one_value_each_in_a_new_round() {
    let (book, ids) = book(vec![login()]);
    let mut filling = Filling::default();
    let user = filling
        .ask(&fill(ids[0], vec![Part::Value(0)]), &book, GAP)
        .unwrap();
    assert_eq!(typed(&user), [USER]);
    let answered = filling.answer(FillTyped {
        round: user.round,
        ok: true,
    });
    assert_eq!(answered, Some(Ok(ids[0])));
    let password = filling
        .ask(&fill(ids[0], vec![Part::Value(1)]), &book, GAP)
        .unwrap();
    assert_eq!(typed(&password), [PASSWORD]);
    assert_ne!(user.round, password.round);
}

#[test]
fn a_value_fill_text_refuses_sends_nothing_with_the_refused_note() {
    let marked = draft("Odd", EntryKind::Other, vec![field("text", "a\tb", false)]);
    let entry = Entry::restored(9, marked, Stamp::from_unix_ms(1), None);
    let book = Notebook::new(vec![entry]);
    let mut filling = Filling::default();
    let got = filling.ask(&fill(9, vec![Part::Value(0)]), &book, GAP);
    assert_eq!(got, Err(FillNote::Refused));
    let (book, ids) = self::book(vec![login()]);
    assert!(
        filling
            .ask(&fill(ids[0], vec![Part::Value(0)]), &book, GAP)
            .is_ok(),
        "nothing waits"
    );
}

#[test]
fn a_fill_while_one_waits_is_refused_with_the_not_finished_note_never_queued() {
    let (book, ids) = book(vec![login()]);
    let mut filling = Filling::default();
    let first = filling
        .ask(&fill(ids[0], vec![Part::Value(0)]), &book, GAP)
        .unwrap();
    let again = filling.ask(&fill(ids[0], vec![Part::Value(1)]), &book, GAP);
    assert_eq!(again, Err(FillNote::NotTyped));
    assert_eq!(
        filling.answer(FillTyped {
            round: first.round,
            ok: false
        }),
        Some(Err(FillNote::NotTyped))
    );
    assert!(
        filling
            .ask(&fill(ids[0], vec![Part::Value(1)]), &book, GAP)
            .is_ok()
    );
}

#[test]
fn only_the_last_requests_answer_counts_and_only_once() {
    let (book, ids) = book(vec![login()]);
    let mut filling = Filling::default();
    let first = filling
        .ask(&fill(ids[0], vec![Part::Value(0)]), &book, GAP)
        .unwrap();
    let older = FillTyped {
        round: first.round.wrapping_sub(1),
        ok: true,
    };
    assert_eq!(filling.answer(older), None);
    let ok = FillTyped {
        round: first.round,
        ok: true,
    };
    assert_eq!(filling.answer(ok), Some(Ok(ids[0])));
    assert_eq!(filling.answer(ok), None, "answered already");
}

#[test]
fn a_release_lets_the_next_fill_go_and_drops_the_old_answer_once_it_is_sent() {
    let (book, ids) = book(vec![login()]);
    let mut filling = Filling::default();
    let first = filling
        .ask(&fill(ids[0], vec![Part::Value(0)]), &book, GAP)
        .unwrap();
    filling.release();
    assert_eq!(
        filling.answer(FillTyped {
            round: first.round,
            ok: true
        }),
        Some(Ok(ids[0]))
    );
    let second = filling
        .ask(&fill(ids[0], vec![Part::Value(1)]), &book, GAP)
        .unwrap();
    filling.release();
    let third = filling
        .ask(&fill(ids[0], vec![Part::Value(0)]), &book, GAP)
        .unwrap();
    assert_eq!(
        filling.answer(FillTyped {
            round: second.round,
            ok: true
        }),
        None
    );
    assert_eq!(
        filling.answer(FillTyped {
            round: third.round,
            ok: true
        }),
        Some(Ok(ids[0]))
    );
}

#[test]
fn a_request_never_prints_a_value() {
    let (book, ids) = book(vec![login()]);
    let both = fill(ids[0], vec![Part::Value(0), Part::Tab, Part::Value(1)]);
    let ask = Filling::default().ask(&both, &book, GAP).unwrap();
    let shown = format!("{ask:?}");
    assert!(
        !shown.contains(USER) && !shown.contains(PASSWORD),
        "{shown}"
    );
}

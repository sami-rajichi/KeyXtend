//! Emails: one `@` lists domains, a typed `@@` lists known emails with More past the lane, the list pages by the rows
//! the UI tells, a list pick sends the email, and typed emails are learned in text boxes only.

use crate::rig::Rig;
use kx_mod_predict::email::Emails;
use kx_module_api::ports::{ChipKind, FieldKind, WordsAsk};

/// Rows per page the tests' UI shows.
const ROWS: u8 = 5;

/// Seven known emails, the most recent first.
fn seven() -> Vec<String> {
    (0..7).map(|n| format!("user{n}@mail.org")).collect()
}

/// A rig in English, in a text box, knowing the emails `known`.
fn knowing(known: &[String]) -> Rig {
    let bytes = Emails::new(known, usize::MAX).to_bytes();
    let mut rig = Rig::build(|f, _| {
        let name = kx_mod_predict::emails_blob().unwrap();
        f.store.plant(&name, bytes);
    });
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

#[test]
fn one_at_lists_the_domains_in_order() {
    let mut rig = knowing(&[]);
    rig.typed("sami@");
    let domains: Vec<String> = rig.chips().into_iter().map(|c| c.0).collect();
    assert_eq!(domains, rig.settings.domains);
    rig.typed("g");
    assert_eq!(rig.chips(), [("gmail.com".to_owned(), ChipKind::Domain)]);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(1, "gmail.com".to_owned())], "no space");
}

#[test]
fn a_typed_at_at_lists_known_emails_with_more_and_the_list_pages() {
    let mut rig = knowing(&seven());
    rig.typed("@@");
    let kinds: Vec<ChipKind> = rig.chips().into_iter().map(|c| c.1).collect();
    let lane = usize::from(rig.settings.chips);
    assert_eq!(kinds.len(), lane);
    assert_eq!(kinds.last(), Some(&ChipKind::More));
    rig.ask(WordsAsk::More { rows: ROWS });
    let list = rig.lane().list.unwrap();
    assert_eq!((list.items.len(), list.page, list.pages), (5, 0, 2));
    rig.ask(WordsAsk::Page {
        page: 1,
        rows: ROWS,
    });
    let list = rig.lane().list.unwrap();
    assert_eq!((list.items.len(), list.page), (2, 1));
    rig.ask(WordsAsk::Page {
        page: 3,
        rows: ROWS,
    });
    assert_eq!(rig.lane().list.unwrap().page, 1, "past the end stays");
    rig.ask(WordsAsk::ListPick(1));
    assert_eq!(rig.typed_words(), [(2, "user6@mail.org".to_owned())]);
    assert!(rig.lane().list.is_none(), "a pick closes the list");
}

#[test]
fn a_list_row_past_the_page_types_nothing_and_keeps_the_list() {
    let mut rig = knowing(&seven());
    rig.typed("@@");
    rig.ask(WordsAsk::More { rows: ROWS });
    rig.ask(WordsAsk::ListPick(ROWS));
    assert!(
        rig.typed_words().is_empty(),
        "the next page's first email is not typed"
    );
    assert_eq!(rig.lane().list.map(|l| l.page), Some(0));
}

#[test]
fn a_tap_on_the_more_chip_waits_for_the_uis_more_and_cancel_closes_the_list() {
    let mut rig = knowing(&seven());
    rig.typed("@@");
    let more = u8::try_from(rig.chips().len() - 1).unwrap();
    rig.pick(more);
    assert!(rig.lane().list.is_none() && rig.typed_words().is_empty());
    rig.ask(WordsAsk::More { rows: ROWS });
    assert!(rig.lane().list.is_some());
    rig.ask(WordsAsk::Cancel);
    assert!(rig.lane().list.is_none());
}

#[test]
fn a_known_email_shows_from_its_start_and_a_typed_one_is_learned() {
    let mut rig = knowing(&["sami@x.org".to_owned()]);
    rig.typed("sa");
    assert!(
        rig.chips()
            .contains(&("sami@x.org".to_owned(), ChipKind::Email))
    );
    rig.typed(" ");
    rig.typed("nour@y.com ");
    rig.stop();
    let bytes = rig
        .fakes
        .store
        .blob(&kx_mod_predict::emails_blob().unwrap())
        .unwrap();
    let saved = Emails::from_bytes(&bytes, usize::MAX).unwrap();
    assert_eq!(saved.list(), ["nour@y.com", "sami@x.org"]);
}

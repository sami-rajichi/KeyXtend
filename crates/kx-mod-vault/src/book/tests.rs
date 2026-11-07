//! The notebook: random ids drawn again when taken, the entry and byte limits with their note, edits that keep the
//! id and use, fills, removals, the `@@` emails, and stamps that never step back.

use super::*;
use crate::fixture::{LIMITS, draft, field, own};
use kx_module_api::ports::{EntryKind, FillNote, Random, RandomError};
use kx_platform_fake::FakeRandom;

/// The time of the first save, and of a later one.
const MONDAY: Stamp = Stamp::from_unix_ms(1_700_000_000_000);
const FRIDAY: Stamp = Stamp::from_unix_ms(1_700_345_600_000);
/// A time between them, and a day in ms.
const TUESDAY: Stamp = Stamp::from_unix_ms(1_700_086_400_000);
const DAY_MS: i64 = 86_400_000;
/// The seed of the tests' random source.
const SEED: u64 = 41;

/// The first `n` ids the source seeded `seed` gives.
fn ids(seed: u64, n: usize) -> Vec<u64> {
    let source = FakeRandom::new(seed);
    let mut out = Vec::new();
    for _ in 0..n {
        let mut bytes = [0; 8];
        source.fill(&mut bytes).unwrap();
        out.push(u64::from_le_bytes(bytes));
    }
    out
}

/// A login named `name` for user `user`.
fn login(name: &str, user: &str) -> Draft {
    draft(
        name,
        EntryKind::Login,
        vec![
            field("user", user, false),
            field("password", "s3cret", true),
        ],
    )
}

/// A source that gives the same bytes every time.
struct Same;

impl Random for Same {
    fn fill(&self, buf: &mut [u8]) -> Result<(), RandomError> {
        buf.fill(7);
        Ok(())
    }
}

#[test]
fn a_new_entry_takes_a_random_id_the_change_time_and_no_use() {
    let mut book = Notebook::default();
    let id = book
        .add(
            login("Bank", "alex"),
            MONDAY,
            &FakeRandom::new(SEED),
            &LIMITS,
        )
        .unwrap();
    assert_eq!(id, ids(SEED, 1)[0]);
    let entry = book.get(id).unwrap();
    assert_eq!(
        (entry.changed(), entry.used(), book.len()),
        (MONDAY, None, 1)
    );
}

#[test]
fn an_id_already_taken_is_drawn_again() {
    let mut book = Notebook::default();
    let random = FakeRandom::new(SEED);
    let first = book
        .add(login("Bank", "alex"), MONDAY, &random, &LIMITS)
        .unwrap();
    let again = book
        .add(
            login("Mail", "sam"),
            MONDAY,
            &FakeRandom::new(SEED),
            &LIMITS,
        )
        .unwrap();
    assert_eq!((first, again), (ids(SEED, 2)[0], ids(SEED, 2)[1]));
}

#[test]
fn no_random_bytes_or_a_stuck_source_adds_nothing() {
    let mut book = Notebook::default();
    let random = FakeRandom::new(SEED);
    random.fail_next();
    assert_eq!(
        book.add(login("Bank", "alex"), MONDAY, &random, &LIMITS),
        Err(EntryError::NoRandom)
    );
    book.add(login("Bank", "alex"), MONDAY, &Same, &LIMITS)
        .unwrap();
    assert_eq!(
        book.add(login("Mail", "sam"), MONDAY, &Same, &LIMITS),
        Err(EntryError::NoRandom)
    );
    assert_eq!(book.len(), 1);
}

#[test]
fn the_notebook_holds_at_most_max_entries() {
    let limits = Limits {
        max_entries: 2,
        ..LIMITS
    };
    let (mut book, random) = (Notebook::default(), FakeRandom::new(SEED));
    for name in ["a", "b"] {
        book.add(login(name, "alex"), MONDAY, &random, &limits)
            .unwrap();
    }
    let full = book.add(login("c", "alex"), MONDAY, &random, &limits);
    assert_eq!(full.err().and_then(EntryError::note), Some(FillNote::Full));
    assert_eq!(book.len(), 2);
}

#[test]
fn the_saved_notebook_fits_its_blob() {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(SEED));
    let id = book
        .add(login("Bank", "alex"), MONDAY, &random, &LIMITS)
        .unwrap();
    let limits = Limits {
        max_bytes: book.bytes(),
        ..LIMITS
    };
    assert_eq!(
        book.add(login("B", "x"), MONDAY, &random, &limits),
        Err(EntryError::Full)
    );
    let longer = login("Bank", "alexandra");
    assert_eq!(
        book.edit(id, longer, FRIDAY, &limits),
        Err(EntryError::Full)
    );
    let user = &book.get(id).unwrap().fields()[0].value;
    assert_eq!(
        user.expose().as_str(),
        "alex",
        "the refused edit changed nothing"
    );
    assert_eq!(
        book.edit(id, login("Bank", "sam"), FRIDAY, &limits),
        Ok(true)
    );
}

#[test]
fn a_refused_draft_adds_nothing() {
    let mut book = Notebook::default();
    let bad = draft("Bank", EntryKind::Login, vec![field("user", "a\nb", false)]);
    assert_eq!(
        book.add(bad, MONDAY, &FakeRandom::new(SEED), &LIMITS),
        Err(EntryError::Refused)
    );
    assert!(book.is_empty());
}

#[test]
fn an_edit_keeps_the_id_and_last_use_and_takes_the_new_time() {
    let mut book = Notebook::default();
    let id = book
        .add(
            login("Bank", "alex"),
            MONDAY,
            &FakeRandom::new(SEED),
            &LIMITS,
        )
        .unwrap();
    assert!(book.use_at(id, TUESDAY));
    assert_eq!(
        book.edit(id, login("Banque", "sam"), FRIDAY, &LIMITS),
        Ok(true)
    );
    let entry = book.get(id).unwrap();
    assert_eq!(
        (
            entry.name().expose().as_str(),
            entry.changed(),
            entry.used()
        ),
        ("Banque", FRIDAY, Some(TUESDAY))
    );
    assert_eq!(
        book.edit(id ^ 1, login("X", "y"), FRIDAY, &LIMITS),
        Ok(false)
    );
    let bad = draft(
        "Bank",
        EntryKind::Login,
        vec![field("pin", "\u{200e}1", true)],
    );
    assert_eq!(
        book.edit(id, bad, FRIDAY, &LIMITS),
        Err(EntryError::Refused)
    );
    assert_eq!(book.get(id).unwrap().changed(), FRIDAY);
}

#[test]
fn a_fill_notes_its_time_and_a_removal_takes_the_entry_out() {
    let mut book = Notebook::default();
    let id = book
        .add(
            login("Bank", "alex"),
            MONDAY,
            &FakeRandom::new(SEED),
            &LIMITS,
        )
        .unwrap();
    assert!(book.use_at(id, FRIDAY));
    assert_eq!(book.get(id).map(Entry::used), Some(Some(FRIDAY)));
    assert!(!book.use_at(id ^ 1, FRIDAY));
    assert_eq!(book.remove(id).map(|e| e.id()), Some(id));
    assert!(book.remove(id).is_none() && book.is_empty());
}

#[test]
fn the_emails_are_the_shown_values_under_an_email_label() {
    let mut book = Notebook::default();
    let random = FakeRandom::new(SEED);
    let fields = vec![
        field("email", "alex@example.com", false),
        field("email", "secret@example.com", true),
        field("user", "alex@work.example", false),
        own("Email", "own@example.com", false),
    ];
    book.add(
        draft("Alex", EntryKind::Person, fields),
        MONDAY,
        &random,
        &LIMITS,
    )
    .unwrap();
    let mail = vec![field("email", "أليكس@مثال.تونس", false)];
    book.add(
        draft("أليكس", EntryKind::Person, mail),
        MONDAY,
        &random,
        &LIMITS,
    )
    .unwrap();
    let emails: Vec<String> = book
        .emails()
        .iter()
        .map(|e| e.expose().to_string())
        .collect();
    assert_eq!(emails, ["alex@example.com", "أليكس@مثال.تونس"]);
}

#[test]
fn a_clock_stepped_back_still_stamps_each_change_after_the_newest() {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(SEED));
    let past = |n: i64| Stamp::from_unix_ms(FRIDAY.unix_ms() + n);
    let a = book
        .add(login("Bank", "alex"), FRIDAY, &random, &LIMITS)
        .unwrap();
    let b = book
        .add(login("Mail", "sam"), MONDAY, &random, &LIMITS)
        .unwrap();
    assert_eq!(book.get(b).unwrap().changed(), past(1));
    assert!(book.use_at(a, MONDAY));
    assert_eq!(book.get(a).unwrap().used(), Some(past(2)));
    assert_eq!(
        book.edit(b, login("Mail", "kim"), MONDAY, &LIMITS),
        Ok(true)
    );
    assert_eq!(book.get(b).unwrap().changed(), past(3));
    let later = Stamp::from_unix_ms(FRIDAY.unix_ms() + DAY_MS);
    assert!(book.use_at(b, later));
    assert_eq!(
        book.get(b).unwrap().used(),
        Some(later),
        "a clock ahead keeps its time"
    );
}

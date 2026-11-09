//! The notebook's emails (Quick-fill's `VaultEmails`): saved chips after `@@` and the start of one, a new set in place
//! of the old while the same set again keeps the shown lane, one both typed and saved shown once, and a saved email
//! picked as a chip or from the list never learned or stored, while one typed by hand is.

mod asks;
mod marks;

use crate::rig::{ROWS, Rig, blob, plant};
use kx_mod_predict::Msg;
use kx_mod_predict::email::Emails;
use kx_module_api::ports::{ChipKind, ChipPicked, FieldKind, Lane, Piece, WordsAsk};
use kx_module_api::{Redacted, Zeroizing};

/// Emails saved in the notebook.
const SAVED: [&str; 2] = ["alex@notebook.org", "lea@carnet.fr"];

/// The notebook's `texts`, as the vault tells them.
fn notebook(texts: &[&str]) -> Msg {
    let secret = |t: &&str| Redacted::new(Zeroizing::new((*t).to_owned()));
    Msg::Saved(texts.iter().map(secret).collect())
}

/// A rig in English, in a text box, knowing the typed emails `typed` and the notebook's `saved`.
fn knowing(typed: &[&str], saved: &[&str]) -> Rig {
    let list: Vec<String> = typed.iter().map(|e| (*e).to_owned()).collect();
    let bytes = Emails::new(&list, usize::MAX).to_bytes();
    let name = kx_mod_predict::emails_blob().unwrap();
    let mut rig = Rig::build(|f, _| f.store.plant(&name, bytes));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.send(notebook(saved));
    rig
}

/// The keyboard tells what the last pick typed: one erase per character, then its text.
fn landed(rig: &mut Rig) {
    let (erase, text) = rig.typed_words().pop().unwrap();
    for _ in 0..erase {
        rig.send(Msg::Typed(Piece::Erase));
    }
    rig.typed(&text);
}

/// Stops `rig` and says, for each of `texts`, whether the emails' blob or English's learned words hold it.
fn stored<const N: usize>(rig: &mut Rig, texts: [&str; N]) -> [bool; N] {
    rig.stop();
    let names = [kx_mod_predict::emails_blob().unwrap(), blob("en")];
    let blobs: Vec<Vec<u8>> = names
        .iter()
        .filter_map(|n| rig.fakes.store.blob(n))
        .collect();
    let holds = |t: &str| {
        blobs
            .iter()
            .any(|b| b.windows(t.len()).any(|w| w == t.as_bytes()))
    };
    texts.map(holds)
}

#[test]
fn the_notebooks_emails_show_after_at_at_and_the_start_of_one_as_saved_chips() {
    let mut rig = knowing(&[], &SAVED);
    rig.typed("@@");
    let want = SAVED.map(|e| (e.to_owned(), ChipKind::Saved));
    assert_eq!(rig.chips(), want);
    let mut rig = knowing(&[], &SAVED);
    rig.typed("le");
    assert_eq!(rig.chips(), [(SAVED[1].to_owned(), ChipKind::Saved)]);
}

#[test]
fn a_new_set_replaces_the_old_and_an_empty_one_removes_them() {
    let mut rig = knowing(&[], &SAVED[..1]);
    rig.typed("@@");
    assert_eq!(rig.chips(), [(SAVED[0].to_owned(), ChipKind::Saved)]);
    rig.send(notebook(&SAVED[1..]));
    assert_eq!(
        rig.chips(),
        [(SAVED[1].to_owned(), ChipKind::Saved)],
        "told again at once"
    );
    rig.send(notebook(&[]));
    assert_eq!(rig.lane().lane, Lane::Empty);
}

#[test]
fn the_same_emails_again_tell_no_new_lane_and_a_pick_of_the_shown_round_still_types() {
    let mut rig = knowing(&[], &SAVED);
    rig.typed("le");
    let shown = rig.lane().round;
    rig.send(notebook(&SAVED));
    assert_eq!(rig.lane().round, shown, "the same emails again");
    rig.send(Msg::Picked(ChipPicked {
        round: shown,
        index: 0,
    }));
    assert_eq!(rig.typed_words().len(), 1, "the saved chip typed");
}

#[test]
fn an_empty_notebook_told_again_leaves_a_word_lane_and_its_pick_alone() {
    let mut rig = Rig::build(|f, _| plant(f, "en", &["hello", "help"]));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.send(notebook(&[]));
    rig.typed("hel");
    let shown = rig.lane().round;
    rig.send(notebook(&[]));
    assert_eq!(rig.lane().round, shown, "no emails before or after");
    rig.send(Msg::Picked(ChipPicked {
        round: shown,
        index: 0,
    }));
    assert_eq!(rig.typed_words(), [(0, "lo ".to_owned())]);
}

#[test]
fn one_email_both_typed_and_saved_shows_once_as_saved() {
    let mut rig = knowing(&["Alex@Notebook.org", "nour@y.com"], &SAVED);
    rig.typed("@@");
    let want = [
        (SAVED[0].to_owned(), ChipKind::Saved),
        ("nour@y.com".to_owned(), ChipKind::Email),
        (SAVED[1].to_owned(), ChipKind::Saved),
    ];
    assert_eq!(rig.chips(), want);
}

#[test]
fn a_saved_chip_picked_after_at_at_or_its_start_is_never_learned_or_stored() {
    for start in ["@@", "al"] {
        let mut rig = knowing(&[], &SAVED);
        rig.typed(start);
        rig.pick(0);
        landed(&mut rig);
        rig.typed(" ");
        rig.typed("nour@y.com ");
        let [saved, typed] = stored(&mut rig, [SAVED[0], "nour@y.com"]);
        assert!(!saved, "{start}: the notebook's email was stored");
        assert!(typed, "{start}: an email typed by hand is learned as usual");
    }
}

#[test]
fn a_saved_email_picked_from_the_list_is_never_learned_or_stored() {
    let many: Vec<String> = (0..7).map(|n| format!("user{n}@notebook.org")).collect();
    let saved: Vec<&str> = many.iter().map(String::as_str).collect();
    let mut rig = knowing(&[], &saved);
    rig.typed("@@");
    rig.ask(WordsAsk::More { rows: ROWS });
    let list = rig.lane().list.unwrap();
    assert!(list.items.iter().all(|c| c.kind == ChipKind::Saved));
    rig.ask(WordsAsk::ListPick(1));
    assert_eq!(rig.typed_words(), [(2, saved[1].to_owned())]);
    landed(&mut rig);
    rig.send(Msg::Pressed(false));
    assert_eq!(stored(&mut rig, [saved[1]]), [false]);
}

#[test]
fn the_same_email_typed_by_hand_after_a_saved_pick_is_learned() {
    let mut rig = knowing(&[], &SAVED);
    rig.typed("@@");
    rig.pick(0);
    landed(&mut rig);
    rig.typed(" ");
    rig.typed(&format!("{} ", SAVED[0]));
    assert_eq!(
        stored(&mut rig, [SAVED[0]]),
        [true],
        "the mark went with its token"
    );
}

#[test]
fn the_notebooks_emails_never_show_in_debug() {
    let shown = format!("{:?}", notebook(&SAVED));
    for email in SAVED {
        assert!(!shown.contains(email), "{shown}");
    }
}

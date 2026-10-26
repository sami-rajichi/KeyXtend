//! The fake's end checks: a selection that holds text never confirms one while a read ends at its start, and the last
//! text asked is kept unshown.

use super::*;

#[test]
fn a_read_with_a_selection_ends_at_its_start_and_a_check_says_no_until_it_is_a_caret() {
    let fake = FakeFieldWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(FieldKind::Text);
    fake.set_before(Some("I really "));
    fake.set_ends(Some(true));
    fake.select("like ap");
    let before = Ok(Some("I really ".to_owned()));
    assert_eq!(
        read(run.as_mut(), 9),
        before,
        "the selected words are not read"
    );
    let no = ended(FieldKind::Text, Some(false));
    assert_eq!(ends(run.as_mut()), no, "a selection never ends with a word");
    fake.select("");
    let yes = ended(FieldKind::Text, Some(true));
    assert_eq!(ends(run.as_mut()), yes, "a caret answers as scripted");
}

#[test]
fn it_keeps_the_last_text_ends_with_was_asked_and_never_shows_it() {
    let fake = FakeFieldWatch::new();
    assert_eq!(fake.last_ends(), None, "nothing asked yet");
    let (mut run, _) = watched(&fake);
    ends(run.as_mut());
    let asked = fake.last_ends().map(Redacted::into_inner);
    assert_eq!(asked.as_deref(), Some("ap"), "asked even of no box");
    run.ends_with(&Redacted::new(String::from("ق")));
    let asked = fake.last_ends().map(Redacted::into_inner);
    assert_eq!(asked.as_deref(), Some("ق"), "the latest wins");
    assert!(!format!("{fake:?}").contains('ق'));
}

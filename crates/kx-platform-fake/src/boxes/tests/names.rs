//! Parts and padlocks as the worker holds them: a box opens only known parts, each once; a seal refuses an unknown or
//! repeated part and a padlock that will not read; and two seals of the same parts differ.

use super::*;

/// A part this version does not know.
const UNKNOWN: &str = "look";

/// The notebook part twice.
fn twice() -> [BoxPart; 2] {
    [part(NOTEBOOK_PART, b"a"), part(NOTEBOOK_PART, b"b")]
}

#[test]
fn a_box_with_an_unknown_part_is_unsupported_and_a_repeated_one_damaged() {
    let (_, mut run) = session();
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let unknown = format::encode(&padlock, &[part(UNKNOWN, b"")]).unwrap();
    let opened = run.open(&unknown, &password(0));
    assert_eq!(opened, Err(BoxError::Unsupported));
    let repeated = format::encode(&padlock, &twice()).unwrap();
    assert_eq!(run.open(&repeated, &password(0)), Err(BoxError::Damaged));
    let wrong = run.open(&unknown, &password(1));
    assert_eq!(wrong, Err(BoxError::WrongPassword), "the password first");
    let empty = format::encode(&padlock, &[]).unwrap();
    assert_eq!(
        run.open(&empty, &password(0)),
        Err(BoxError::Damaged),
        "none"
    );
}

#[test]
fn a_seal_refuses_no_part_or_an_unknown_or_repeated_one_as_the_asks_frame_does() {
    let (_, mut run) = session();
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let unknown = run.seal(&padlock, &[part(UNKNOWN, b"")]);
    assert_eq!(unknown, Err(BoxError::Failed));
    assert_eq!(run.seal(&padlock, &twice()), Err(BoxError::Failed));
    assert_eq!(run.seal(&padlock, &[]), Err(BoxError::Failed), "no part");
}

#[test]
fn a_seal_checks_its_size_before_its_padlock_as_the_platform_does() {
    let (_, mut run) = session();
    let broken = Padlock {
        recipient: Redacted::new("age1-not-fake".to_owned()),
        locked: Redacted::new(Vec::new()),
    };
    let huge = part(NOTEBOOK_PART, &vec![0; MAX_BOX_BYTES + 1]);
    assert_eq!(run.seal(&broken, &[huge]), Err(BoxError::TooLarge));
    assert_eq!(run.seal(&broken, &parts()), Err(BoxError::Damaged));
}

#[test]
fn a_seal_refuses_a_padlock_that_will_not_read_as_damaged() {
    let (_, mut run) = session();
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let recipient = Padlock {
        recipient: Redacted::new("age1-not-fake".to_owned()),
        ..padlock.clone()
    };
    let locked = Padlock {
        locked: Redacted::new(b"not a lock".to_vec()),
        ..padlock
    };
    for bad in [recipient, locked] {
        assert_eq!(run.seal(&bad, &parts()), Err(BoxError::Damaged));
    }
}

#[test]
fn parts_sealed_to_another_padlocks_recipient_will_not_open() {
    let (_, mut run) = session();
    let ours = run.padlock(&password(0), WORK).unwrap();
    let theirs = run.padlock(&password(0), WORK).unwrap();
    let mixed = Padlock {
        recipient: theirs.recipient,
        locked: ours.locked,
    };
    let sealed = run.seal(&mixed, &parts()).unwrap();
    let opened = run.open(sealed.expose(), &password(0));
    assert_eq!(opened, Err(BoxError::Damaged));
}

#[test]
fn two_seals_of_the_same_parts_differ_but_open_the_same() {
    let (_, mut run) = session();
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let one = run.seal(&padlock, &parts()).unwrap();
    let two = run.seal(&padlock, &parts()).unwrap();
    assert_ne!(one, two);
    for sealed in [one, two] {
        let opened = run.open(sealed.expose(), &password(0));
        assert_eq!(opened.map(|o| o.parts), Ok(parts()));
    }
}

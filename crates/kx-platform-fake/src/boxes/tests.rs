//! The fake boxes, held to the port: padlocks that open only their own boxes with their password, boxes that will not
//! open damaged, cut, foreign or newer, scripted failures, counted calls, and no value in `Debug`.

use super::*;
use kx_module_api::ports::{BOX_MARK, BOX_VERSION, KeepassField, MAX_BOX_BYTES, NOTEBOOK_PART};
use std::collections::HashSet;
use std::thread;

mod caps;
mod keepass;
mod names;
mod props;

/// Test passwords that appear nowhere else: a Latin one and an Arabic one ("password").
const PASSWORDS: [&str; 2] = ["zebra-quartz", "كلمة السر"];

/// A test value that appears nowhere else: an Arabic note ("my secret").
const VALUE: &str = "سري الخاص";

/// A `KeePass` file's bytes; the fake reads none of them.
const KDBX: &[u8] = b"a kdbx file";

/// Test settings; the real ones come from the vault module's settings.
const CFG: BoxConfig = BoxConfig {
    deadline_ms: 20_000,
    idle_s: 30,
    restart_ms: 1_000,
    restart_max_ms: 30_000,
};

/// The work factor the tests ask, within the bounds.
const WORK: u8 = MIN_BOX_WORK;

/// Password `n` as a secret.
fn password(n: usize) -> Secret {
    Redacted::new(Zeroizing::new(PASSWORDS[n].to_owned()))
}

/// A secret text.
fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// A part named `name` holding `bytes`.
fn part(name: &str, bytes: &[u8]) -> BoxPart {
    BoxPart {
        name: name.to_owned(),
        bytes: Redacted::new(Zeroizing::new(bytes.to_vec())),
    }
}

/// The notebook, the one part a box holds now.
fn parts() -> Vec<BoxPart> {
    vec![part(NOTEBOOK_PART, VALUE.as_bytes())]
}

/// One `KeePass` entry with a custom field.
fn entries() -> Vec<KeepassEntry> {
    let custom = KeepassField {
        label: Redacted::new("PIN".to_owned()),
        value: secret(VALUE),
    };
    vec![KeepassEntry {
        title: secret("بنك"),
        user: secret("alex"),
        password: secret(VALUE),
        address: secret(""),
        notes: secret("line one\nline two"),
        custom: vec![custom],
    }]
}

/// A new fake and a session on it.
fn session() -> (FakeBoxes, Box<dyn BoxRun>) {
    let boxes = FakeBoxes::new();
    let run = boxes.start(CFG);
    (boxes, run)
}

#[test]
fn a_padlock_opens_only_its_own_boxes_with_its_password() {
    let (_, mut run) = session();
    let ours = run.padlock(&password(0), WORK).unwrap();
    let theirs = run.padlock(&password(1), WORK).unwrap();
    let sealed = run.seal(&ours, &parts()).unwrap();
    let opened = Opened {
        padlock: ours,
        parts: parts(),
    };
    assert_eq!(run.open(sealed.expose(), &password(0)), Ok(opened));
    let wrong = Err(BoxError::WrongPassword);
    assert_eq!(run.open(sealed.expose(), &password(1)), wrong);
    let other = run.seal(&theirs, &parts()).unwrap();
    assert_eq!(run.open(other.expose(), &password(0)), wrong);
    let opened = run.open(other.expose(), &password(1));
    assert_eq!(opened.map(|o| o.padlock), Ok(theirs));
}

#[test]
fn each_padlock_is_plainly_fake_unique_and_never_holds_its_password() {
    let (_, mut here) = session();
    let (_, mut there) = session();
    let made = [
        here.padlock(&password(0), WORK).unwrap(),
        here.padlock(&password(0), WORK).unwrap(),
        there.padlock(&password(0), WORK).unwrap(),
    ];
    let recipients: HashSet<_> = made.iter().map(|p| p.recipient.expose()).collect();
    let locks: HashSet<_> = made.iter().map(|p| p.locked.expose()).collect();
    assert_eq!((recipients.len(), locks.len()), (3, 3), "unique");
    assert!(made[0].recipient.expose().starts_with("fake"));
    let plain = PASSWORDS[0].as_bytes();
    let held = made[0]
        .locked
        .expose()
        .windows(plain.len())
        .any(|w| w == plain);
    assert!(!held, "the locked identity hides the password");
}

#[test]
fn check_tells_whether_the_password_opens_the_padlock() {
    let (_, mut run) = session();
    let padlock = run.padlock(&password(1), WORK).unwrap();
    assert_eq!(run.check(&padlock, &password(1)), Ok(true));
    assert_eq!(run.check(&padlock, &password(0)), Ok(false));
    let broken = Padlock {
        locked: Redacted::new(b"not a lock".to_vec()),
        ..padlock
    };
    assert_eq!(run.check(&broken, &password(1)), Err(BoxError::Damaged));
}

#[test]
fn a_box_opens_on_another_fake_as_on_another_computer() {
    let (_, mut here) = session();
    let (_, mut there) = session();
    let padlock = here.padlock(&password(0), WORK).unwrap();
    let sealed = here.seal(&padlock, &parts()).unwrap();
    let opened = there.open(sealed.expose(), &password(0));
    assert_eq!(opened.map(|o| o.parts), Ok(parts()));
    assert_eq!(there.check(&padlock, &password(0)), Ok(true));
}

#[test]
fn a_damaged_cut_or_foreign_box_is_damaged_and_a_newer_one_unsupported() {
    let (_, mut run) = session();
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let sealed = run.seal(&padlock, &parts()).unwrap().into_inner();
    let head = [&BOX_MARK[..], &[BOX_VERSION]].concat();
    let body = &sealed[head.len()..];
    let value = VALUE.as_bytes();
    let at = sealed.windows(value.len()).position(|w| w == value);
    let mut flipped = sealed.clone();
    flipped[at.unwrap()] ^= 1;
    let damaged = [
        flipped,
        sealed[..sealed.len() - 1].to_vec(),
        [&head[..], b"age-encryption.org/v1"].concat(),
        [&BOX_MARK[..], &[BOX_VERSION - 1], body].concat(),
        head.clone(),
        b"KeePass".to_vec(),
        Vec::new(),
    ];
    for bad in damaged {
        assert_eq!(run.open(&bad, &password(0)), Err(BoxError::Damaged));
    }
    let newer = [&BOX_MARK[..], &[BOX_VERSION + 1], body].concat();
    assert_eq!(run.open(&newer, &password(0)), Err(BoxError::Unsupported));
}

#[test]
fn each_scripted_failure_comes_back_once_as_its_box_error() {
    let (boxes, mut run) = session();
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let sealed = run.seal(&padlock, &parts()).unwrap().into_inner();
    boxes.set_keepass(KDBX, PASSWORDS[0], entries());
    boxes.fail(BoxCall::Padlock, BoxError::TimedOut);
    boxes.fail(BoxCall::Seal, BoxError::Failed);
    boxes.fail(BoxCall::Check, BoxError::TimedOut);
    boxes.fail(BoxCall::Keepass, BoxError::Unsupported);
    let made = run.padlock(&password(0), WORK);
    assert_eq!(made, Err(BoxError::TimedOut));
    assert_eq!(run.seal(&padlock, &parts()), Err(BoxError::Failed));
    let checked = run.check(&padlock, &password(0));
    assert_eq!(checked, Err(BoxError::TimedOut));
    let read = run.keepass(KDBX, &password(0));
    assert_eq!(read, Err(BoxError::Unsupported));
    for error in [
        BoxError::WrongPassword,
        BoxError::Damaged,
        BoxError::Unsupported,
        BoxError::TooLarge,
        BoxError::TimedOut,
        BoxError::Failed,
    ] {
        boxes.fail(BoxCall::Open, error);
        assert_eq!(run.open(&sealed, &password(0)), Err(error));
    }
    assert!(run.padlock(&password(0), WORK).is_ok(), "once");
    assert!(run.seal(&padlock, &parts()).is_ok());
    assert_eq!(run.check(&padlock, &password(0)), Ok(true));
    assert_eq!(run.keepass(KDBX, &password(0)), Ok(entries().into()));
    assert!(run.open(&sealed, &password(0)).is_ok());
}

#[test]
fn sessions_and_calls_are_counted_by_kind_refused_ones_too() {
    let boxes = FakeBoxes::new();
    assert_eq!((boxes.starts(), boxes.config()), (0, None));
    let mut run = boxes.start(CFG);
    assert!(boxes.running());
    boxes.fail(BoxCall::Open, BoxError::Damaged);
    let _ = run.open(b"", &password(0));
    let _ = run.open(b"", &password(0));
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let _ = run.check(&padlock, &password(0));
    let kinds = [
        BoxCall::Padlock,
        BoxCall::Seal,
        BoxCall::Open,
        BoxCall::Check,
        BoxCall::Keepass,
    ];
    assert_eq!(kinds.map(|c| boxes.calls(c)), [1, 0, 2, 1, 0]);
    drop(run);
    assert!(!boxes.running());
    assert_eq!((boxes.starts(), boxes.config()), (1, Some(CFG)));
}

#[test]
fn clones_share_the_fake_and_serve_as_the_port_from_another_thread() {
    let boxes = FakeBoxes::new();
    let port: Arc<dyn Boxes> = Arc::new(boxes.clone());
    let made = thread::spawn(move || port.start(CFG).padlock(&password(0), WORK).is_ok());
    assert!(made.join().unwrap());
    assert_eq!(boxes.calls(BoxCall::Padlock), 1);
}

#[test]
fn debug_counts_the_calls_but_prints_no_password_or_value() {
    let (boxes, mut run) = session();
    boxes.set_keepass(KDBX, PASSWORDS[1], entries());
    let _ = run.padlock(&password(1), WORK);
    let _ = run.keepass(KDBX, &password(1));
    let text = format!("{boxes:?}");
    assert!(text.starts_with("FakeBoxes"), "{text}");
    assert!(text.contains("Padlock"), "{text}");
    for secret in PASSWORDS.iter().chain(&[VALUE]) {
        assert!(!text.contains(secret), "{text} shows {secret}");
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn an_error_the_call_never_returns_is_a_test_mistake() {
    FakeBoxes::new().fail(BoxCall::Seal, BoxError::WrongPassword);
}

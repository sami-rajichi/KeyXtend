//! The vault job through its loop, with in-memory streams: each ask gets its reply in order, a reply past the frame's
//! caps is `TooLarge`, and goodbye ends the loop with nothing after it read.

use super::*;
use crate::serve::serve;
use kx_ipc::{BoxPart, Dropped, MIN_BOX_WORK, NOTEBOOK_PART};
use std::borrow::Cow;

/// The tests' backup password.
const PASSWORD: &[u8] = "zebra quartz كلمة".as_bytes();

/// The frames of `asks`, one after another.
fn frames(asks: &[VaultAsk<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    for ask in asks {
        ask.write(&mut out).unwrap();
    }
    out
}

/// Every reply in `out`.
fn replies(out: &[u8]) -> Vec<VaultReply> {
    let mut written = out;
    std::iter::from_fn(|| VaultReply::read(&mut written).ok()).collect()
}

/// Runs the vault job's loop over `input`, returning its outcome and every reply it wrote.
fn run(input: &[u8]) -> (Result<(), IpcError>, Vec<VaultReply>) {
    let mut out = Vec::new();
    let outcome = serve(&mut &input[..], &mut out, |ask: VaultAsk<'static>| {
        reply(ask)
    });
    (outcome, replies(&out))
}

/// A padlock under `PASSWORD` and a box sealed under it holding `notes`, each made by `reply`.
fn made_box(notes: &'static [u8]) -> (String, Vec<u8>, Vec<u8>) {
    let password = Wiped::Lent(PASSWORD);
    let ask = VaultAsk::Padlock {
        id: 1,
        work: MIN_BOX_WORK,
        password,
    };
    let VaultReply::Padlock {
        recipient, locked, ..
    } = reply(ask)
    else {
        panic!("a padlock for ask 1");
    };
    let part = BoxPart {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes: Wiped::Lent(notes),
    };
    let (to, key) = (
        Cow::Borrowed(recipient.as_str()),
        Cow::Borrowed(&locked[..]),
    );
    let ask = VaultAsk::Seal {
        id: 2,
        recipient: to,
        locked: key,
        parts: vec![part],
    };
    let VaultReply::Sealed { bytes, .. } = reply(ask) else {
        panic!("a box for ask 2");
    };
    (recipient, locked, bytes)
}

#[test]
fn each_ask_is_answered_in_order_and_bye_ends_the_loop() {
    let (recipient, locked, bytes) = made_box(b"entries");
    let password = Wiped::Lent(PASSWORD);
    let open = VaultAsk::Open {
        id: 3,
        password,
        bytes: Cow::Borrowed(&bytes),
    };
    let password = Wiped::Lent(b"wrong");
    let check = VaultAsk::Check {
        id: 4,
        password,
        locked: Cow::Borrowed(&locked),
    };
    let input = frames(&[open, check.clone(), VaultAsk::Bye, check]);
    let (outcome, got) = run(&input);
    assert_eq!(outcome, Ok(()));
    let notes = Wiped::Kept(b"entries".to_vec().into());
    let parts = vec![BoxPart {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes: notes,
    }];
    let opened = VaultReply::Opened {
        id: 3,
        recipient,
        locked,
        parts,
    };
    assert_eq!(got, [opened, VaultReply::Checked { id: 4, yes: false }]);
}

#[test]
fn a_foreign_box_and_a_foreign_keepass_file_each_say_so() {
    let asks = [
        VaultAsk::Open {
            id: 5,
            password: Wiped::Lent(PASSWORD),
            bytes: Cow::Borrowed(b"PK\x03\x04"),
        },
        VaultAsk::Keepass {
            id: 6,
            password: Wiped::Lent(PASSWORD),
            bytes: Cow::Borrowed(b"not a KeePass file"),
        },
    ];
    let (_, got) = run(&frames(&asks));
    let foreign = |id| VaultReply::Failed {
        id,
        fault: Fault::Foreign,
    };
    assert_eq!(got, [foreign(5), foreign(6)]);
}

#[test]
fn a_reply_past_the_caps_is_too_large() {
    let field = "x".repeat(kx_ipc::MAX_FIELD_BYTES + 1);
    let long = kx_ipc::KeepassEntry {
        notes: field.into(),
        ..kx_ipc::KeepassEntry::default()
    };
    let many = vec![kx_ipc::KeepassEntry::default(); kx_ipc::MAX_ENTRIES + 1];
    for entries in [vec![long], many] {
        assert_eq!(
            fit(VaultReply::Entries {
                id: 1,
                entries,
                dropped: Dropped::default()
            }),
            Err(Fault::TooLarge)
        );
    }
    let big = vec![0; kx_ipc::MAX_BOX_BYTES + 1];
    assert_eq!(
        fit(VaultReply::Sealed { id: 1, bytes: big }),
        Err(Fault::TooLarge)
    );
}

#[test]
fn no_input_ends_the_loop_cleanly_and_a_broken_frame_with_its_error() {
    assert_eq!(run(&[]), (Ok(()), Vec::new()));
    assert_eq!(run(b"XXXX not a frame").0, Err(IpcError::BadMagic));
    let media = {
        let mut out = Vec::new();
        kx_ipc::Ask::Bye.write(&mut out).unwrap();
        out
    };
    assert_eq!(run(&media).0, Err(IpcError::BadKind));
}

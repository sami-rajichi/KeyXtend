//! The vault frames, both directions: the wire layout, round trips, the largest box, each job reading only its own
//! kinds, and `Debug` that shows no password or value. Refusals and properties have their own files.

mod cases;
mod keepass;
mod props;
mod refuse;

use super::*;
use crate::vault::wire;
use crate::vault::{ask as vault_ask, reply as vault_reply};
use crate::{
    BoxPart, Dropped, Fault, KeepassEntry, KeepassField, MAX_BOX_BYTES, MAX_BOX_WORK,
    MAX_LOCKED_BYTES, MAX_RECIPIENT_BYTES, MIN_BOX_WORK,
};
use crate::{NOTEBOOK_PART, VaultAsk, VaultReply, Wiped, ask, reply};
use cases::*;
use zeroize::Zeroizing;

#[test]
fn the_padlock_ask_and_a_check_are_little_endian_after_their_kinds() {
    let ask = VaultAsk::Padlock {
        id: 7,
        work: 16,
        password: Wiped::Lent(b"pw"),
    };
    let want = b"KXIP\x01\x07\x07\0\0\0\x05\0\0\0\x10\x02\0pw";
    assert_eq!(vault_ask_bytes(&ask), want);
    let checked = VaultReply::Checked { id: 3, yes: true };
    assert_eq!(
        vault_reply_bytes(&checked),
        b"KXIP\x01\x10\x03\0\0\0\x01\0\0\0\x01"
    );
    let failed = VaultReply::Failed {
        id: 2,
        fault: Fault::WrongPassword,
    };
    assert_eq!(
        vault_reply_bytes(&failed),
        b"KXIP\x01\x12\x02\0\0\0\x01\0\0\0\x01"
    );
}

#[test]
fn a_seal_sends_its_padlock_then_each_part_by_name() {
    let ask = VaultAsk::Seal {
        id: 1,
        recipient: Cow::Borrowed("age1x"),
        locked: Cow::Borrowed(&[9, 9]),
        parts: vec![notebook(b"hi")],
    };
    let want = [
        &b"KXIP\x01\x08\x01\0\0\0\x1a\0\0\0"[..],
        b"\x05age1x\x02\0\x09\x09\x01",
        b"\x08notebook\x02\0\0\0hi",
    ]
    .concat();
    assert_eq!(vault_ask_bytes(&ask), want);
}

#[test]
fn entries_send_the_dropped_counts_then_each_entrys_fields_and_custom_ones() {
    let one = KeepassEntry {
        title: text("t"),
        custom: vec![KeepassField {
            label: text("k"),
            value: text("v"),
        }],
        ..KeepassEntry::default()
    };
    let reply = VaultReply::Entries {
        id: 4,
        entries: vec![one],
        dropped: Dropped {
            entries: 2,
            fields: 3,
        },
    };
    let want = [
        &b"KXIP\x01\x11\x04\0\0\0\x1c\0\0\0"[..],
        b"\x02\0\0\0\x03\0\0\0",
        b"\x01\0\x01\0t\0\0\0\0\0\0\0\0",
        b"\x01\x01\0k\x01\0v",
    ]
    .concat();
    assert_eq!(vault_reply_bytes(&reply), want);
}

#[test]
fn each_vault_message_round_trips() {
    for ask in every_ask() {
        assert_eq!(VaultAsk::read(&mut &vault_ask_bytes(&ask)[..]), Ok(ask));
    }
    for reply in every_reply() {
        let bytes = vault_reply_bytes(&reply);
        assert_eq!(VaultReply::read(&mut &bytes[..]), Ok(reply));
    }
}

#[test]
fn each_work_factor_from_the_lowest_to_the_highest_round_trips() {
    for work in MIN_BOX_WORK..=MAX_BOX_WORK {
        let ask = VaultAsk::Padlock {
            id: 1,
            work,
            password: Wiped::Lent(b"pw"),
        };
        assert_eq!(VaultAsk::read(&mut &vault_ask_bytes(&ask)[..]), Ok(ask));
    }
}

#[test]
fn the_largest_box_round_trips_both_ways() {
    let largest = vec![7; MAX_BOX_BYTES];
    let open = VaultAsk::Open {
        id: 1,
        password: Wiped::Lent(b"pw"),
        bytes: Cow::Borrowed(&largest),
    };
    assert_eq!(VaultAsk::read(&mut &vault_ask_bytes(&open)[..]), Ok(open));
    let sealed = VaultReply::Sealed {
        id: 2,
        bytes: largest,
    };
    let bytes = vault_reply_bytes(&sealed);
    assert_eq!(VaultReply::read(&mut &bytes[..]), Ok(sealed));
}

#[test]
fn the_largest_seal_and_opened_frames_fill_their_cap_and_round_trip() {
    let recipient = "r".repeat(MAX_RECIPIENT_BYTES);
    let (locked, part) = (vec![5; MAX_LOCKED_BYTES], vec![7; MAX_BOX_BYTES]);
    let seal = VaultAsk::Seal {
        id: 1,
        recipient: Cow::Borrowed(&recipient),
        locked: Cow::Borrowed(&locked),
        parts: vec![notebook(&part)],
    };
    let bytes = vault_ask_bytes(&seal);
    assert_eq!(bytes.len(), HEADER_LEN + wire::SEAL_MOST);
    assert_eq!(VaultAsk::read(&mut &bytes[..]), Ok(seal));
    let opened = VaultReply::Opened {
        id: 2,
        recipient,
        locked,
        parts: vec![BoxPart {
            name: Cow::Borrowed(NOTEBOOK_PART),
            bytes: Wiped::Kept(Zeroizing::new(part)),
        }],
    };
    let bytes = vault_reply_bytes(&opened);
    assert_eq!(bytes.len(), HEADER_LEN + wire::SEAL_MOST);
    assert_eq!(VaultReply::read(&mut &bytes[..]), Ok(opened));
}

#[test]
fn the_media_job_refuses_vault_frames_and_the_vault_job_media_frames() {
    for ask in every_ask() {
        let bytes = vault_ask_bytes(&ask);
        assert_eq!(read_ask(&mut &bytes[..]), Err(IpcError::BadKind), "{ask:?}");
        assert_eq!(read_vault_reply(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    for reply in every_reply() {
        let bytes = vault_reply_bytes(&reply);
        assert_eq!(read_reply(&mut &bytes[..]), Err(IpcError::BadKind));
        assert_eq!(read_vault_ask(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    let media = [ask_bytes(&draw(1, b"x")), ask_bytes(&Ask::Bye)];
    for bytes in media {
        assert_eq!(read_vault_ask(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    let failed = reply_bytes(&Reply::Failed {
        id: 1,
        why: Why::Refused,
    });
    assert_eq!(read_vault_reply(&mut &failed[..]), Err(IpcError::BadKind));
}

#[test]
fn the_vault_kinds_are_their_own() {
    for kind in vault_ask::KINDS.iter().chain(vault_reply::KINDS) {
        assert!(!ask::KINDS.contains(kind) && !reply::KINDS.contains(kind));
    }
    assert!(
        vault_ask::KINDS
            .iter()
            .all(|k| !vault_reply::KINDS.contains(k))
    );
}

#[test]
fn a_reply_names_its_ask() {
    let ids: Vec<u32> = every_reply().iter().map(VaultReply::id).collect();
    assert_eq!(ids[..8], [1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn a_reply_that_reads_back_passes_its_check() {
    for reply in every_reply() {
        assert_eq!(reply.check(), Ok(()), "{reply:?}");
    }
}

#[test]
fn debug_shows_no_password_and_no_value() {
    let secrets = [
        "correct horse",
        "my diary",
        "p4ss",
        "sam",
        "1234",
        "line one",
    ];
    let said = format!("{:?}{:?}", every_ask(), every_reply());
    for secret in secrets {
        assert!(!said.contains(secret), "{secret} in {said}");
    }
    let ask = VaultAsk::Open {
        id: 3,
        password: Wiped::Lent(b"pw"),
        bytes: Cow::Borrowed(b"box"),
    };
    assert_eq!(format!("{ask:?}"), "Open { id: 3, len: 3, .. }");
    assert_eq!(format!("{:?}", Wiped::Lent(b"abc")), "Wiped(3)");
}

#[test]
fn a_cut_vault_frame_is_short_and_an_empty_stream_closed() {
    let (asks, replies) = (every_ask(), every_reply());
    let asks = asks
        .iter()
        .map(|a| (vault_ask_bytes(a), read_vault_ask as Reader));
    let replies = replies
        .iter()
        .map(|r| (vault_reply_bytes(r), read_vault_reply as Reader));
    for (bytes, read) in asks.chain(replies) {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            assert_eq!(
                read(&mut &bytes[..cut]),
                Err(IpcError::Short),
                "cut at {cut}"
            );
        }
    }
}

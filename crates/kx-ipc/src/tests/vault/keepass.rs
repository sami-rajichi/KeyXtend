//! The `KeePass` frames' own caps: a file past `MAX_KEEPASS_BYTES`, too many entries or custom fields or too long
//! a field, refused both ways, and the bytes each entry takes.

use super::refuse::{NONE_DROPPED, two, vault_frame};
use super::*;
use crate::{MAX_ENTRIES, MAX_ENTRIES_BYTES, MAX_FIELD_BYTES, MAX_FIELDS, MAX_KEEPASS_BYTES};
use IpcError::TooLong;

/// A `KeePass` ask of `bytes`.
fn keepass(bytes: &[u8]) -> VaultAsk<'_> {
    VaultAsk::Keepass {
        id: 1,
        password: Wiped::Lent(b"pw"),
        bytes: Cow::Borrowed(bytes),
    }
}

#[test]
fn a_keepass_file_past_its_cap_is_refused_both_ways_and_a_box_that_size_is_not() {
    let big = vec![0; MAX_KEEPASS_BYTES + 1];
    let mut out = Vec::new();
    assert_eq!(keepass(&big).write(&mut out), Err(TooLong));
    assert!(out.is_empty());
    let largest = keepass(&big[1..]);
    assert_eq!(
        VaultAsk::read(&mut &vault_ask_bytes(&largest)[..]),
        Ok(largest)
    );
    let open = VaultAsk::Open {
        id: 1,
        password: Wiped::Lent(b"pw"),
        bytes: Cow::Borrowed(&big),
    };
    assert_eq!(VaultAsk::read(&mut &vault_ask_bytes(&open)[..]), Ok(open));
    let len = u32::try_from(big.len()).unwrap();
    let frame_len = len + 8;
    let raw = [
        &head(MAGIC, VERSION, Kind::ReadKeepass.code(), 1, frame_len)[..],
        b"\x02\0pw",
        &len.to_le_bytes(),
    ]
    .concat();
    let mut r = Counted::new((&raw[..]).chain(io::repeat(0)));
    assert_eq!(read_vault_ask(&mut r), Err(TooLong));
    assert_eq!(r.taken, raw.len());
}

#[test]
fn too_many_entries_or_custom_fields_are_refused_both_ways() {
    let many = [&NONE_DROPPED[..], &two(MAX_ENTRIES + 1)].concat();
    let many = vault_frame(Kind::Entries, &many);
    assert_eq!(read_vault_reply(&mut &many[..]), Err(TooLong));
    let custom = [
        &NONE_DROPPED[..],
        b"\x01\0",
        &[0; 10],
        &[u8::try_from(MAX_FIELDS + 1).unwrap()],
    ]
    .concat();
    let bytes = vault_frame(Kind::Entries, &custom);
    assert_eq!(read_vault_reply(&mut &bytes[..]), Err(TooLong));
    let wide = KeepassEntry {
        custom: vec![KeepassField::default(); MAX_FIELDS + 1],
        ..KeepassEntry::default()
    };
    let replies = [
        vec![KeepassEntry::default(); MAX_ENTRIES + 1],
        vec![wide],
        vec![KeepassEntry {
            notes: text(&"x".repeat(MAX_FIELD_BYTES + 1)),
            ..KeepassEntry::default()
        }],
    ];
    for entries in replies {
        let dropped = Dropped::default();
        let reply = VaultReply::Entries {
            id: 1,
            entries,
            dropped,
        };
        assert_eq!(reply.check(), Err(TooLong));
        let mut out = Vec::new();
        assert_eq!(reply.write(&mut out), Err(TooLong));
        assert!(out.is_empty());
    }
}

#[test]
fn an_entrys_wire_len_is_what_it_takes_in_a_reply() {
    let entries = vec![entry(), KeepassEntry::default()];
    let total: usize = entries.iter().map(|e| e.wire_len().unwrap()).sum();
    let reply = VaultReply::Entries {
        id: 1,
        entries,
        dropped: Dropped::default(),
    };
    let head = HEADER_LEN + NONE_DROPPED.len() + two(0).len();
    assert_eq!(vault_reply_bytes(&reply).len(), head + total);
    assert_eq!(
        head - HEADER_LEN + MAX_ENTRIES_BYTES,
        crate::limits::MAX_ENTRIES_LEN
    );
    let long = KeepassEntry {
        user: text(&"x".repeat(MAX_FIELD_BYTES + 1)),
        ..KeepassEntry::default()
    };
    assert_eq!(long.wire_len(), Err(TooLong));
}

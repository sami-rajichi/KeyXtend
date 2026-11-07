//! Vault properties: what is written reads back, and any bytes are refused cleanly, within the frame and the caps.

use super::*;
use crate::tests::props::check_bounded;
use proptest::prelude::*;
use proptest::sample::Index;

/// The longest bytes or text a case sends.
const MAX_CASE_BYTES: usize = 64;
/// The most entries and custom fields a case sends.
const MAX_CASE_ITEMS: usize = 3;
/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 256;
/// How far around each cap a case's length lands.
const NEAR_CAP: u32 = 2;

fn some_bytes(min: usize) -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), min..MAX_CASE_BYTES)
}

fn some_text() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..MAX_CASE_BYTES / 4).prop_map(String::from_iter)
}

fn kept(bytes: Vec<u8>) -> Wiped<'static> {
    Wiped::Kept(Zeroizing::new(bytes))
}

fn a_password() -> impl Strategy<Value = Wiped<'static>> {
    some_text().prop_map(|s| kept(s.into_bytes()))
}

fn some_parts() -> impl Strategy<Value = Vec<BoxPart<'static>>> {
    let part = some_bytes(0).prop_map(|bytes| BoxPart {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes: kept(bytes),
    });
    prop::collection::vec(part, 1..=crate::MAX_PARTS)
}

fn a_vault_ask() -> impl Strategy<Value = VaultAsk<'static>> {
    let id = any::<u32>;
    let padlock = (id(), MIN_BOX_WORK..=MAX_BOX_WORK, a_password())
        .prop_map(|(id, work, password)| VaultAsk::Padlock { id, work, password });
    let seal = (id(), "[a-z0-9]{1,62}", some_bytes(1), some_parts()).prop_map(
        |(id, recipient, locked, parts)| VaultAsk::Seal {
            id,
            recipient: Cow::Owned(recipient),
            locked: Cow::Owned(locked),
            parts,
        },
    );
    let open =
        (id(), a_password(), some_bytes(1)).prop_map(|(id, password, bytes)| VaultAsk::Open {
            id,
            password,
            bytes: Cow::Owned(bytes),
        });
    let check =
        (id(), a_password(), some_bytes(1)).prop_map(|(id, password, locked)| VaultAsk::Check {
            id,
            password,
            locked: Cow::Owned(locked),
        });
    let keepass =
        (id(), a_password(), some_bytes(1)).prop_map(|(id, password, bytes)| VaultAsk::Keepass {
            id,
            password,
            bytes: Cow::Owned(bytes),
        });
    prop_oneof![padlock, seal, open, check, keepass, Just(VaultAsk::Bye)]
}

fn a_secret() -> impl Strategy<Value = Zeroizing<String>> {
    some_text().prop_map(Zeroizing::new)
}

fn an_entry() -> impl Strategy<Value = KeepassEntry> {
    let field = (a_secret(), a_secret()).prop_map(|(label, value)| KeepassField { label, value });
    let custom = prop::collection::vec(field, 0..MAX_CASE_ITEMS);
    let fields = (a_secret(), a_secret(), a_secret(), a_secret(), a_secret());
    (fields, custom).prop_map(
        |((title, user, password, address, notes), custom)| KeepassEntry {
            title,
            user,
            password,
            address,
            notes,
            custom,
        },
    )
}

fn a_vault_reply() -> impl Strategy<Value = VaultReply> {
    let id = any::<u32>;
    let padlock = (id(), "[a-z0-9]{1,62}", some_bytes(1)).prop_map(|(id, recipient, locked)| {
        VaultReply::Padlock {
            id,
            recipient,
            locked,
        }
    });
    let sealed = (id(), some_bytes(1)).prop_map(|(id, bytes)| VaultReply::Sealed { id, bytes });
    let opened = (id(), "[a-z0-9]{1,62}", some_bytes(1), some_parts()).prop_map(
        |(id, recipient, locked, parts)| VaultReply::Opened {
            id,
            recipient,
            locked,
            parts,
        },
    );
    let checked = (id(), any::<bool>()).prop_map(|(id, yes)| VaultReply::Checked { id, yes });
    let some = prop::collection::vec(an_entry(), 0..MAX_CASE_ITEMS);
    let dropped =
        (any::<u32>(), any::<u32>()).prop_map(|(entries, fields)| Dropped { entries, fields });
    let entries = (id(), some, dropped).prop_map(|(id, entries, dropped)| VaultReply::Entries {
        id,
        entries,
        dropped,
    });
    let fault = prop::sample::select(Fault::ALL);
    let failed = (id(), fault).prop_map(|(id, fault)| VaultReply::Failed { id, fault });
    prop_oneof![padlock, sealed, opened, checked, entries, failed]
}

/// Payload lengths: small ones, ones around each vault kind's cap, and any at all.
fn a_len() -> impl Strategy<Value = u32> {
    let near = |kind: Kind| {
        let cap = u32::try_from(*kind.lens().end()).unwrap();
        cap - NEAR_CAP..=cap + NEAR_CAP
    };
    prop_oneof![
        0..64u32,
        near(Kind::Seal),
        near(Kind::OpenBox),
        near(Kind::Entries),
        near(Kind::MakePadlock),
        any::<u32>()
    ]
}

proptest! {
    #[test]
    fn vault_asks_round_trip_and_end_at_their_frame(ask in a_vault_ask()) {
        let bytes = vault_ask_bytes(&ask);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(VaultAsk::read(&mut r), Ok(ask));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn vault_replies_round_trip_and_end_at_their_frame(reply in a_vault_reply()) {
        let bytes = vault_reply_bytes(&reply);
        let mut r = Counted::new(&bytes[..]);
        prop_assert_eq!(VaultReply::read(&mut r), Ok(reply));
        prop_assert_eq!(r.taken, bytes.len());
    }

    #[test]
    fn random_bytes_never_panic_a_vault_reader(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        for read in [read_vault_ask as Reader, read_vault_reply] {
            let mut r = &junk[..];
            while read(&mut r).is_ok() {}
        }
    }

    #[test]
    fn a_good_vault_header_is_never_read_past_its_frame_or_cap(
        kind in any::<u8>(),
        len in a_len(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        check_bounded(kind, len, &tail, read_vault_ask, vault_ask::KINDS)?;
        check_bounded(kind, len, &tail, read_vault_reply, vault_reply::KINDS)?;
    }

    #[test]
    fn a_changed_vault_ask_never_panics_and_what_reads_writes_back_the_same(
        ask in a_vault_ask(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = vault_ask_bytes(&ask);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = VaultAsk::read(&mut r) {
            prop_assert_eq!(vault_ask_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_changed_vault_reply_never_panics_and_what_reads_writes_back_the_same(
        reply in a_vault_reply(),
        pick in any::<Index>(),
        flip in 1..=u8::MAX,
    ) {
        let mut bytes = vault_reply_bytes(&reply);
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        let mut r = Counted::new(&bytes[..]);
        if let Ok(read) = VaultReply::read(&mut r) {
            prop_assert_eq!(vault_reply_bytes(&read), &bytes[..r.taken]);
        }
    }

    #[test]
    fn a_cut_vault_frame_fails_cleanly(ask in a_vault_ask(), reply in a_vault_reply(), pick in any::<Index>()) {
        let cases: [(Vec<u8>, Reader); 2] =
            [(vault_ask_bytes(&ask), read_vault_ask), (vault_reply_bytes(&reply), read_vault_reply)];
        for (bytes, read) in cases {
            let cut = pick.index(bytes.len());
            let want = if cut == 0 { IpcError::Closed } else { IpcError::Short };
            prop_assert_eq!(read(&mut &bytes[..cut]), Err(want));
        }
    }
}

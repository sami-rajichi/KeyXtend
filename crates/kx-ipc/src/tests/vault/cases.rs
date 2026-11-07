//! The vault tests' cases and helpers: one ask and one reply of each kind, and the readers each side uses.

use super::super::*;
use crate::{
    BoxPart, Dropped, Fault, KeepassEntry, KeepassField, MIN_BOX_WORK, NOTEBOOK_PART, VaultAsk,
    VaultReply, Wiped,
};
use zeroize::Zeroizing;

pub(super) fn vault_ask_bytes(ask: &VaultAsk) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

pub(super) fn vault_reply_bytes(reply: &VaultReply) -> Vec<u8> {
    let mut out = Vec::new();
    reply.write(&mut out).unwrap();
    out
}

pub(super) fn read_vault_ask(r: &mut dyn Read) -> Result<(), IpcError> {
    VaultAsk::read(r).map(drop)
}

pub(super) fn read_vault_reply(r: &mut dyn Read) -> Result<(), IpcError> {
    VaultReply::read(r).map(drop)
}

/// The notebook part holding `bytes`.
pub(super) fn notebook(bytes: &[u8]) -> BoxPart<'_> {
    BoxPart {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes: Wiped::Lent(bytes),
    }
}

pub(super) fn text(s: &str) -> Zeroizing<String> {
    Zeroizing::new(s.to_string())
}

/// An entry with every field set and two custom ones.
pub(super) fn entry() -> KeepassEntry {
    let field = |label: &str, value: &str| KeepassField {
        label: text(label),
        value: text(value),
    };
    KeepassEntry {
        title: text("Mail"),
        user: text("sam"),
        password: text("p4ss"),
        address: text("https://mail.example"),
        notes: text("line one\nسطر"),
        custom: vec![field("PIN", "1234"), field("رمز", "٥٦")],
    }
}

/// One ask of each kind.
pub(super) fn every_ask() -> Vec<VaultAsk<'static>> {
    let password = || Wiped::Lent(b"correct horse".as_slice());
    vec![
        VaultAsk::Padlock {
            id: 1,
            work: MIN_BOX_WORK,
            password: password(),
        },
        VaultAsk::Seal {
            id: 2,
            recipient: Cow::Borrowed("age1recipient"),
            locked: Cow::Borrowed(b"locked identity"),
            parts: vec![notebook(b"my diary")],
        },
        VaultAsk::Open {
            id: 3,
            password: password(),
            bytes: Cow::Borrowed(b"KXBOX box"),
        },
        VaultAsk::Check {
            id: 4,
            password: Wiped::Lent(b""),
            locked: Cow::Borrowed(b"locked"),
        },
        VaultAsk::Keepass {
            id: u32::MAX,
            password: password(),
            bytes: Cow::Borrowed(b"kdbx"),
        },
        VaultAsk::Bye,
    ]
}

/// An opened box with `parts`.
pub(super) fn opened(id: u32, parts: Vec<BoxPart<'static>>) -> VaultReply {
    VaultReply::Opened {
        id,
        recipient: "age1recipient".to_string(),
        locked: b"locked".to_vec(),
        parts,
    }
}

/// One reply of each kind, and a failure for each reason.
pub(super) fn every_reply() -> Vec<VaultReply> {
    let part = |bytes: &[u8]| BoxPart {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes: Wiped::Kept(Zeroizing::new(bytes.to_vec())),
    };
    let mut replies = vec![
        VaultReply::Padlock {
            id: 1,
            recipient: "age1recipient".to_string(),
            locked: b"locked".to_vec(),
        },
        VaultReply::Sealed {
            id: 2,
            bytes: b"KXBOX\x01".to_vec(),
        },
        opened(3, vec![part(b"my diary")]),
        opened(4, vec![part(b"")]),
        VaultReply::Checked { id: 5, yes: true },
        VaultReply::Checked { id: 6, yes: false },
        VaultReply::Entries {
            id: 7,
            entries: vec![entry(), KeepassEntry::default()],
            dropped: Dropped::default(),
        },
        VaultReply::Entries {
            id: 8,
            entries: Vec::new(),
            dropped: Dropped {
                entries: 2,
                fields: u32::MAX,
            },
        },
    ];
    let failed = Fault::ALL
        .iter()
        .map(|&fault| VaultReply::Failed { id: 9, fault });
    replies.extend(failed);
    replies
}

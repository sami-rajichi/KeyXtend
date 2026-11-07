//! The boxes port: its gate, the box facts it shares with the worker, the box's file name, `Debug` that never shows a
//! password, a value or a part, and the errors.

use super::*;
use crate::{REDACTED_MARKER, Zeroizing};
use std::collections::HashSet;

/// Texts that appear nowhere else in the output: a Latin one and an Arabic one ("password").
const SECRETS: [&str; 2] = ["zebra-quartz", "كلمة السر"];

/// Secret `n` as a secret text.
fn secret(n: usize) -> Secret {
    Secret::new(Zeroizing::new(SECRETS[n].to_owned()))
}

/// Secret `n`'s bytes as secret bytes.
fn bytes(n: usize) -> SecretBytes {
    SecretBytes::new(Zeroizing::new(SECRETS[n].as_bytes().to_vec()))
}

/// A padlock whose recipient and locked identity are the two secrets.
fn padlock() -> Padlock {
    Padlock {
        recipient: Redacted::new(SECRETS[0].to_owned()),
        locked: Redacted::new(SECRETS[1].as_bytes().to_vec()),
    }
}

#[test]
fn boxes_need_vault_data() {
    assert_eq!(BoxesKey::ID.as_str(), "boxes");
    assert_eq!(BoxesKey::CAPABILITY, Some(Capability::VaultData));
}

#[test]
fn the_box_facts_come_unchanged_from_the_workers_frames_crate() {
    assert_eq!(BOX_MARK, kx_ipc::BOX_MARK);
    assert_eq!(BOX_VERSION, kx_ipc::BOX_VERSION);
    assert_eq!(BOX_NAME, kx_ipc::BOX_NAME);
    assert_eq!(BOX_EXT, kx_ipc::BOX_EXT);
    assert_eq!(MAX_BOX_BYTES, kx_ipc::MAX_BOX_BYTES);
    assert_eq!(MIN_BOX_WORK, kx_ipc::MIN_BOX_WORK);
    assert_eq!(MAX_BOX_WORK, kx_ipc::MAX_BOX_WORK);
    assert_eq!(VAULT_MEMORY_MB, kx_ipc::VAULT_MEMORY_MB);
    assert_eq!(BOX_PARTS, kx_ipc::BOX_PARTS);
    assert_eq!(NOTEBOOK_PART, kx_ipc::NOTEBOOK_PART);
    assert_eq!(MAX_KEEPASS_BYTES, kx_ipc::MAX_KEEPASS_BYTES);
    assert_eq!(MAX_PASSWORD_BYTES, kx_ipc::MAX_PASSWORD_BYTES);
    assert_eq!(MAX_FIELD_CHARS, kx_ipc::MAX_FIELD_CHARS);
    assert_eq!(MAX_VAULT_ENTRIES, kx_ipc::MAX_ENTRIES);
    assert_eq!(MAX_VAULT_FIELDS, kx_ipc::MAX_FIELDS);
}

#[test]
fn the_box_file_is_its_name_and_extension() {
    assert_eq!(box_file(), "KeyXtend Backup.kxbox");
}

#[test]
fn debug_never_shows_a_password_a_value_or_a_part() {
    let part = BoxPart {
        name: "notebook".to_owned(),
        bytes: bytes(1),
    };
    let entry = KeepassEntry {
        title: secret(0),
        user: secret(1),
        password: secret(0),
        address: secret(1),
        notes: secret(0),
        custom: vec![KeepassField {
            label: Redacted::new(SECRETS[1].to_owned()),
            value: secret(0),
        }],
    };
    let opened = Opened {
        padlock: padlock(),
        parts: vec![part],
    };
    let texts = SECRETS.map(str::to_owned);
    let raw = SECRETS.map(|s| format!("{:?}", s.as_bytes()));
    for shown in [format!("{opened:?}"), format!("{entry:?}")] {
        assert!(shown.contains(REDACTED_MARKER), "{shown}");
        for secret in texts.iter().chain(&raw) {
            assert!(!shown.contains(secret.as_str()), "{shown} shows {secret}");
        }
    }
    assert!(
        format!("{opened:?}").contains("notebook"),
        "a part's name is no secret"
    );
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        BoxError::WrongPassword,
        BoxError::Damaged,
        BoxError::Unsupported,
        BoxError::TooLarge,
        BoxError::TimedOut,
        BoxError::Failed,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn an_import_of_entries_alone_dropped_nothing() {
    let import = KeepassImport::from(Vec::new());
    assert_eq!(import, KeepassImport::default());
    assert_eq!(
        import.dropped,
        Dropped {
            entries: 0,
            fields: 0
        }
    );
}

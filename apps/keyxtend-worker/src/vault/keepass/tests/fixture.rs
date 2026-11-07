//! `KeePass` test files: KDBX 4 databases written here with `keepass`'s own writer and a light Argon2, and small
//! hand-made headers; the format bytes are spelled out from the KDBX spec, apart from the code under test. No real
//! person's data.

use keepass::config::KdfConfig;
use keepass::db::{EntryMut, GroupMut, fields};
use keepass::{Database, DatabaseKey};

/// The test files' password, with an Arabic word in it.
pub const PASSWORD: &str = "correct horse بطارية";
/// The light Argon2 the test files use: 64 KiB, one pass, one lane.
const LIGHT_MEMORY: u64 = 64 * 1024;

/// The KDBX signature, the format ids of `KeePass` 1, KDBX 2 and KDBX 3 and 4, and the minor version written.
const SIGNATURE: [u8; 4] = [0x03, 0xd9, 0xa2, 0x9a];
const KEEPASS_1: u32 = 0xb54b_fb65;
const KDBX_2: u32 = 0xb54b_fb66;
const KDBX: u32 = 0xb54b_fb67;
const MINOR: u16 = 1;
/// Outer header ids: the end, the cipher, compression, both seeds, KDBX 3's rounds, the IV, the inner stream's key,
/// the start bytes and inner stream, and KDBX 4's key derivation and custom data.
const END: u8 = 0;
const CIPHER_ID: u8 = 2;
pub const COMPRESSION: u8 = 3;
const MASTER_SEED: u8 = 4;
const TRANSFORM_SEED: u8 = 5;
pub const ROUNDS: u8 = 6;
const IV: u8 = 7;
const STREAM_KEY: u8 = 8;
const STREAM_START: u8 = 9;
pub const INNER_STREAM: u8 = 10;
pub const KDF: u8 = 11;
pub const CUSTOM_DATA: u8 = 12;
/// The AES-256 cipher's id.
const AES_256: [u8; 16] = *b"\x31\xc1\xf2\xe6\xbf\x71\x43\x50\xbe\x58\x05\x21\x6a\xfc\x5a\xff";
/// What the end field holds.
const END_BYTES: &[u8] = b"\r\n\r\n";
/// Variant dictionary: its version and end, and the value types used here.
const DICT_VERSION: u16 = 0x100;
pub const DICT_U32: u8 = 0x04;
pub const DICT_U64: u8 = 0x05;
pub const DICT_BYTES: u8 = 0x42;
/// Key derivation ids: AES-KDF as KDBX 4 writes it, and Argon2d.
pub const AES_ID: [u8; 16] = *b"\x7c\x02\xbb\x82\x79\xa7\x4a\xc0\x92\x7d\x11\x4a\x00\x64\x82\x38";
const ARGON2D_ID: [u8; 16] = *b"\xef\x63\x6d\xdf\x8c\x29\x44\x4b\x91\xf7\xa9\xa4\x03\xe3\x0a\x0c";
/// Argon2's version 1.3 and a seed's length.
const ARGON2_13: u32 = 0x13;
const SEED: [u8; 32] = [7; 32];

/// A database being built.
pub struct Kdbx(Database);

impl Kdbx {
    /// A database with nothing in it and a light key derivation.
    pub fn empty() -> Self {
        let mut db = Database::new();
        if let KdfConfig::Argon2 {
            iterations,
            memory,
            parallelism,
            ..
        } = &mut db.config.kdf_config
        {
            (*iterations, *memory, *parallelism) = (1, LIGHT_MEMORY, 1);
        }
        Self(db)
    }

    /// Two root entries, a group with a subgroup, the recycle bin, then one more group.
    pub fn sample() -> Self {
        let mut kdbx = Self::empty();
        let mut root = kdbx.0.root_mut();
        add(
            &mut root,
            &[(fields::TITLE, "Mail"), (fields::USERNAME, "sam")],
            |e| {
                e.set_protected(fields::PASSWORD, "p4ss word");
                e.set_unprotected(fields::URL, "https://mail.example");
                e.set_unprotected(fields::NOTES, "line one\nسطر اثنان");
                e.set_protected("رمز", "٥٦٧");
                e.set_unprotected("PIN", "1234");
            },
        );
        add(&mut root, &[(fields::TITLE, "Bank")], |_| {});
        let mut home = root.add_group();
        home.name = "Home".into();
        add(
            &mut home,
            &[(fields::TITLE, "Wi-Fi"), (fields::PASSWORD, "router")],
            |_| {},
        );
        let mut deep = home.add_group();
        add(&mut deep, &[(fields::TITLE, "Deep")], |_| {});
        let mut bin = root.add_group();
        bin.name = "Recycle Bin".into();
        add(&mut bin, &[(fields::TITLE, "Thrown away")], |_| {});
        let bin = bin.id().uuid();
        let mut shop = root.add_group();
        add(&mut shop, &[(fields::TITLE, "Shop")], |_| {});
        kdbx.0.meta.recyclebin_uuid = Some(bin);
        kdbx
    }

    /// Adds `n` entries to the root.
    pub fn entries(&mut self, n: usize) {
        let mut root = self.0.root_mut();
        for _ in 0..n {
            add(&mut root, &[(fields::TITLE, "x")], |_| {});
        }
    }

    /// Adds an entry whose notes are `text`.
    pub fn note(&mut self, text: &str) {
        add(&mut self.0.root_mut(), &[(fields::NOTES, text)], |_| {});
    }

    /// Adds an entry with the custom fields `custom`, each a name and a value.
    pub fn custom(&mut self, custom: &[(&str, &str)]) {
        add(&mut self.0.root_mut(), custom, |_| {});
    }

    /// The database as built, unsaved.
    pub fn db(&self) -> &Database {
        &self.0
    }

    /// The file, locked by `PASSWORD`.
    pub fn save(&self) -> Vec<u8> {
        self.save_as(DatabaseKey::new().with_password(PASSWORD))
    }

    /// The file, locked by `PASSWORD` and the key file `key`.
    pub fn save_with_key_file(&self, mut key: &[u8]) -> Vec<u8> {
        let key = DatabaseKey::new()
            .with_password(PASSWORD)
            .with_keyfile(&mut key);
        self.save_as(key.unwrap())
    }

    fn save_as(&self, key: DatabaseKey) -> Vec<u8> {
        let mut out = Vec::new();
        self.0.save(&mut out, key).unwrap();
        out
    }
}

/// Adds an entry to `group` with unprotected `texts`, then lets `more` fill it.
fn add(group: &mut GroupMut<'_>, texts: &[(&str, &str)], more: impl FnOnce(&mut EntryMut<'_>)) {
    let mut entry = group.add_entry();
    for (key, value) in texts {
        entry.set_unprotected(*key, *value);
    }
    more(&mut entry);
}

/// The 12 bytes before a header: the signature, `format` and the version.
fn version(format: u32, major: u16) -> Vec<u8> {
    [
        &SIGNATURE[..],
        &format.to_le_bytes(),
        &MINOR.to_le_bytes(),
        &major.to_le_bytes(),
    ]
    .concat()
}

/// A `KeePass` 1 file's first bytes.
pub fn keepass1_head() -> Vec<u8> {
    [version(KEEPASS_1, 1), vec![0; 112]].concat()
}

/// A KDBX 2 file's first bytes.
pub fn kdbx2_head() -> Vec<u8> {
    [version(KDBX_2, 2), vec![0; 64]].concat()
}

/// A KDBX 4 header holding only the key derivation `params` and the end.
pub fn kdbx4_head(params: &[u8]) -> Vec<u8> {
    kdbx4_fields(&[(KDF, params)])
}

/// A KDBX 4 header holding `fields`, each an id and its bytes, then the end.
pub fn kdbx4_fields(fields: &[(u8, &[u8])]) -> Vec<u8> {
    let mut out = version(KDBX, 4);
    for &(id, bytes) in fields.iter().chain([&(END, END_BYTES)]) {
        out.push(id);
        out.extend_from_slice(&u32::try_from(bytes.len()).unwrap().to_le_bytes());
        out.extend_from_slice(bytes);
    }
    out
}

/// A KDBX 3 header holding only AES-KDF's `rounds` and the end.
pub fn kdbx3_head(rounds: u64) -> Vec<u8> {
    kdbx3_fields(&[(ROUNDS, &rounds.to_le_bytes())])
}

/// A KDBX 3 header holding `fields`, each an id and its bytes, then the end.
pub fn kdbx3_fields(fields: &[(u8, &[u8])]) -> Vec<u8> {
    let mut out = version(KDBX, 3);
    for &(id, bytes) in fields.iter().chain([&(END, END_BYTES)]) {
        out.push(id);
        out.extend_from_slice(&u16::try_from(bytes.len()).unwrap().to_le_bytes());
        out.extend_from_slice(bytes);
    }
    out
}

/// A whole KDBX 3.1 file whose payload, two AES blocks of one byte repeated, no key opens: AES-256, no
/// compression, one AES-KDF round and plain inner values.
pub fn kdbx3_file() -> Vec<u8> {
    let none = 0_u32.to_le_bytes();
    let head = kdbx3_fields(&[
        (CIPHER_ID, &AES_256),
        (COMPRESSION, &none),
        (MASTER_SEED, &[1; 32]),
        (TRANSFORM_SEED, &[2; 32]),
        (ROUNDS, &1_u64.to_le_bytes()),
        (IV, &[3; 16]),
        (STREAM_KEY, &[4; 32]),
        (STREAM_START, &[5; 32]),
        (INNER_STREAM, &none),
    ]);
    [head, vec![6; 32]].concat()
}

/// A variant dictionary of `(type, key, value)` items.
pub fn dict(items: &[(u8, &[u8], &[u8])]) -> Vec<u8> {
    let mut out = DICT_VERSION.to_le_bytes().to_vec();
    for (ty, key, value) in items {
        out.push(*ty);
        for bytes in [key, value] {
            out.extend_from_slice(&u32::try_from(bytes.len()).unwrap().to_le_bytes());
            out.extend_from_slice(bytes);
        }
    }
    out.push(END);
    out
}

/// Argon2d parameters of `memory` bytes, `passes` and `lanes`.
pub fn argon2(memory: u64, passes: u64, lanes: u32) -> Vec<u8> {
    dict(&[
        (DICT_BYTES, b"$UUID", &ARGON2D_ID),
        (DICT_U64, b"M", &memory.to_le_bytes()),
        (DICT_U64, b"I", &passes.to_le_bytes()),
        (DICT_U32, b"P", &lanes.to_le_bytes()),
        (DICT_U32, b"V", &ARGON2_13.to_le_bytes()),
        (DICT_BYTES, b"S", &SEED),
    ])
}

/// AES-KDF parameters of `rounds`.
pub fn aes(rounds: u64) -> Vec<u8> {
    dict(&[
        (DICT_BYTES, b"$UUID", &AES_ID),
        (DICT_U64, b"R", &rounds.to_le_bytes()),
        (DICT_BYTES, b"S", &SEED),
    ])
}

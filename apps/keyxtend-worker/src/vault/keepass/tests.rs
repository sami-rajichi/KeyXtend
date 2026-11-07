//! `KeePass` import: a KDBX 4 file made here maps every field in the file's order, groups flattened and the recycle bin
//! left out, with entries and fields past the caps dropped and counted; a wrong password, a key-file database, a KDBX
//! 3.1 payload no key opens, `KeePass` 1 and KDBX 2 headers, too heavy a key derivation and random bytes each fail.

mod fixture;
mod props;

use super::*;
use fixture::{Kdbx, PASSWORD, kdbx2_head, kdbx3_head, kdbx4_head, keepass1_head};
use kx_ipc::{
    Dropped, MAX_AES_ROUNDS, MAX_ENTRIES, MAX_FIELD_BYTES, MAX_FIELDS, MAX_KDF_BYTES,
    MAX_KDF_LANES, MAX_KDF_PASSES, MAX_KEEPASS_BYTES,
};

/// A variant dictionary value type no `KeePass` version writes.
const UNKNOWN_TYPE: u8 = 0x77;

#[test]
fn every_field_maps_and_custom_fields_follow_by_name() {
    let (entries, _) = read(&Kdbx::sample().save(), PASSWORD).unwrap();
    let mail = &entries[0];
    assert_eq!(mail.title.as_str(), "Mail");
    assert_eq!(mail.user.as_str(), "sam");
    assert_eq!(mail.password.as_str(), "p4ss word");
    assert_eq!(mail.address.as_str(), "https://mail.example");
    assert_eq!(mail.notes.as_str(), "line one\nسطر اثنان");
    let custom: Vec<(&str, &str)> = mail
        .custom
        .iter()
        .map(|f| (f.label.as_str(), f.value.as_str()))
        .collect();
    assert_eq!(custom, [("PIN", "1234"), ("رمز", "٥٦٧")]);
}

#[test]
fn entries_come_in_the_files_order_with_groups_flattened_and_the_bin_left_out() {
    let (entries, _) = read(&Kdbx::sample().save(), PASSWORD).unwrap();
    let titles: Vec<&str> = entries.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["Mail", "Bank", "Wi-Fi", "Deep", "Shop"]);
}

#[test]
fn a_field_an_entry_lacks_is_empty() {
    let (entries, _) = read(&Kdbx::sample().save(), PASSWORD).unwrap();
    let wifi = &entries[2];
    assert!(wifi.user.is_empty() && wifi.address.is_empty() && wifi.notes.is_empty());
    assert!(wifi.custom.is_empty());
}

#[test]
fn a_wrong_password_says_so() {
    let bytes = Kdbx::sample().save();
    assert_eq!(read(&bytes, "p4ss").err(), Some(Fault::WrongPassword));
}

/// A KDBX file cannot tell a missing key file from a wrong password, so a file that needs one says wrong password.
#[test]
fn a_key_file_database_opened_with_its_password_alone_says_wrong_password() {
    let bytes = Kdbx::sample().save_with_key_file(b"a key file of my own");
    assert_eq!(read(&bytes, PASSWORD).err(), Some(Fault::WrongPassword));
}

#[test]
fn keepass_1_and_kdbx_2_are_unsupported_and_other_files_foreign() {
    assert_eq!(
        read(&keepass1_head(), PASSWORD).err(),
        Some(Fault::Unsupported)
    );
    assert_eq!(
        read(&kdbx2_head(), PASSWORD).err(),
        Some(Fault::Unsupported)
    );
    for junk in [
        &b"PK\x03\x04 a zip file"[..],
        b"KXBOX\x01 a box, not KeePass",
    ] {
        assert_eq!(read(junk, PASSWORD).err(), Some(Fault::Foreign));
    }
}

#[test]
fn a_cut_or_changed_file_is_damaged() {
    let bytes = Kdbx::sample().save();
    for cut in [4, 12, 40, bytes.len() / 2, bytes.len() - 1] {
        assert_eq!(
            read(&bytes[..cut], PASSWORD).err(),
            Some(Fault::Damaged),
            "{cut}"
        );
    }
    let mut changed = bytes;
    let last = changed.len() - 1;
    changed[last] ^= 1;
    assert_eq!(read(&changed, PASSWORD).err(), Some(Fault::Damaged));
}

#[test]
fn too_heavy_a_key_derivation_is_unsupported_before_it_runs() {
    let heavy = [
        kdbx4_head(&fixture::argon2(MAX_KDF_BYTES + 1, 1, 1)),
        kdbx4_head(&fixture::argon2(MAX_KDF_BYTES, MAX_KDF_PASSES + 1, 1)),
        kdbx4_head(&fixture::argon2(1 << 20, u64::MAX, 1)),
        kdbx4_head(&fixture::argon2(1 << 20, 1, MAX_KDF_LANES + 1)),
        kdbx4_head(&fixture::aes(MAX_AES_ROUNDS + 1)),
        kdbx4_head(&fixture::aes(u64::MAX)),
        kdbx3_head(u64::MAX),
    ];
    for bytes in heavy {
        assert_eq!(kdf::check(&bytes).err(), Some(Fault::Unsupported));
        assert_eq!(read(&bytes, PASSWORD).err(), Some(Fault::Unsupported));
    }
}

#[test]
fn a_light_key_derivation_passes_the_check() {
    assert_eq!(kdf::check(&Kdbx::sample().save()), Ok(()));
    let light = [
        kdbx4_head(&fixture::argon2(
            MAX_KDF_BYTES,
            MAX_KDF_PASSES,
            MAX_KDF_LANES,
        )),
        kdbx4_head(&fixture::aes(MAX_AES_ROUNDS)),
        kdbx3_head(MAX_AES_ROUNDS),
    ];
    for bytes in light {
        assert_eq!(kdf::check(&bytes), Ok(()));
    }
}

#[test]
fn a_header_with_short_numbers_or_no_key_derivation_is_damaged() {
    let short_rounds = fixture::dict(&[
        (fixture::DICT_BYTES, b"$UUID", &fixture::AES_ID),
        (fixture::DICT_U64, b"R", &[1, 0]),
    ]);
    let no_kdf = fixture::dict(&[(fixture::DICT_U32, b"V", &[1, 0, 0, 0])]);
    for params in [short_rounds, no_kdf, b"\x00\x01\x00".to_vec(), Vec::new()] {
        assert_eq!(kdf::check(&kdbx4_head(&params)), Err(Fault::Damaged));
    }
}

#[test]
fn every_key_derivation_field_must_read_and_the_last_one_counts() {
    let short = fixture::dict(&[
        (fixture::DICT_BYTES, b"$UUID", &fixture::AES_ID),
        (fixture::DICT_U64, b"R", &[1, 0]),
    ]);
    let (light, heavy) = (fixture::aes(1), fixture::aes(u64::MAX));
    let kdf = |first: &[u8], last: &[u8]| {
        kdf::check(&fixture::kdbx4_fields(&[
            (fixture::KDF, first),
            (fixture::KDF, last),
        ]))
    };
    assert_eq!(kdf(&short, &light), Err(Fault::Damaged));
    assert_eq!(kdf(&heavy, &light), Ok(()));
    assert_eq!(kdf(&light, &heavy), Err(Fault::Unsupported));
}

/// `keepass` panics on a number of the wrong width, so each must be refused before it reads the file.
#[test]
fn a_kdbx_3_header_with_a_short_number_or_no_rounds_is_damaged() {
    let (none, rounds) = (0_u32.to_le_bytes(), 1_u64.to_le_bytes());
    let whole = fixture::kdbx3_fields(&[
        (fixture::COMPRESSION, &none),
        (fixture::INNER_STREAM, &none),
        (fixture::ROUNDS, &rounds),
    ]);
    assert_eq!(kdf::check(&whole), Ok(()));
    let bad: [&[(u8, &[u8])]; 4] = [
        &[(fixture::COMPRESSION, &[0, 0]), (fixture::ROUNDS, &rounds)],
        &[(fixture::INNER_STREAM, &[0, 0]), (fixture::ROUNDS, &rounds)],
        &[(fixture::ROUNDS, &[1, 0])],
        &[(fixture::COMPRESSION, &none)],
    ];
    for fields in bad {
        let bytes = fixture::kdbx3_fields(fields);
        assert_eq!(kdf::check(&bytes), Err(Fault::Damaged), "{fields:?}");
        assert_eq!(read(&bytes, PASSWORD).err(), Some(Fault::Damaged));
    }
}

#[test]
fn a_kdbx_4_header_with_a_short_compression_or_malformed_custom_data_is_damaged() {
    let (none, kdf_params) = (0_u32.to_le_bytes(), fixture::aes(1));
    let kdbx4 =
        |id: u8, bytes: &[u8]| fixture::kdbx4_fields(&[(fixture::KDF, &kdf_params), (id, bytes)]);
    let good = fixture::dict(&[(fixture::DICT_U32, b"key", &none)]);
    assert_eq!(kdf::check(&kdbx4(fixture::CUSTOM_DATA, &good)), Ok(()));
    assert_eq!(kdf::check(&kdbx4(fixture::COMPRESSION, &none)), Ok(()));
    let mut no_end = fixture::dict(&[]);
    no_end.pop();
    let custom = [
        Vec::new(),
        no_end,
        fixture::dict(&[(UNKNOWN_TYPE, b"key", b"value")]),
        fixture::dict(&[(fixture::DICT_U32, b"key", &[1, 0])]),
    ];
    let bad = custom
        .iter()
        .map(|c| kdbx4(fixture::CUSTOM_DATA, c))
        .chain([kdbx4(fixture::COMPRESSION, &[0, 0])]);
    for bytes in bad {
        assert_eq!(kdf::check(&bytes), Err(Fault::Damaged));
        assert_eq!(read(&bytes, PASSWORD).err(), Some(Fault::Damaged));
    }
}

#[test]
fn an_unknown_key_derivation_is_unsupported() {
    let unknown = fixture::dict(&[(fixture::DICT_BYTES, b"$UUID", &[9; 16])]);
    let bytes = kdbx4_head(&unknown);
    assert_eq!(kdf::check(&bytes), Err(Fault::Unsupported));
    assert_eq!(read(&bytes, PASSWORD).err(), Some(Fault::Unsupported));
}

#[test]
fn entries_past_the_cap_are_dropped_and_counted() {
    let mut many = Kdbx::empty();
    many.entries(MAX_ENTRIES + 2);
    let (entries, dropped) = read(&many.save(), PASSWORD).unwrap();
    assert_eq!(entries.len(), MAX_ENTRIES);
    let two = Dropped {
        entries: 2,
        fields: 0,
    };
    assert_eq!(dropped, two);
}

#[test]
fn fields_past_their_caps_are_dropped_and_counted_and_their_entry_kept() {
    let long = "x".repeat(MAX_FIELD_BYTES + 1);
    let names: Vec<String> = (0..=MAX_FIELDS).map(|n| format!("c{n:03}")).collect();
    let mut custom: Vec<(&str, &str)> = names.iter().map(|n| (n.as_str(), "v")).collect();
    custom.push(("a long one", &long));
    let mut kdbx = Kdbx::empty();
    kdbx.note(&long);
    kdbx.custom(&custom);
    let (entries, dropped) = read(&kdbx.save(), PASSWORD).unwrap();
    assert!(entries[0].notes.is_empty());
    assert_eq!(entries[1].custom.len(), MAX_FIELDS);
    assert_eq!(entries[1].custom[0].label.as_str(), "c000");
    let three = Dropped {
        entries: 0,
        fields: 3,
    };
    assert_eq!(dropped, three);
}

#[test]
fn entries_past_the_replys_size_are_dropped_and_their_fields_not_counted() {
    let mut kdbx = Kdbx::empty();
    kdbx.entries(2);
    kdbx.note(&"x".repeat(MAX_FIELD_BYTES + 1));
    let (all, _) = entries(kdbx.db(), CAPS);
    let one = all[0].wire_len().unwrap();
    let (kept, dropped) = entries(
        kdbx.db(),
        Caps {
            bytes: 2 * one,
            ..CAPS
        },
    );
    assert_eq!(kept.len(), 2);
    let one_entry = Dropped {
        entries: 1,
        fields: 0,
    };
    assert_eq!(dropped, one_entry);
}

#[test]
fn a_file_past_its_cap_is_too_large_before_it_is_read() {
    let big = vec![0; MAX_KEEPASS_BYTES + 1];
    assert_eq!(read(&big, PASSWORD).err(), Some(Fault::TooLarge));
}

#[test]
fn a_kdbx_3_1_payload_the_key_does_not_open_fails_its_padding_and_says_wrong_password() {
    let bytes = fixture::kdbx3_file();
    let key = DatabaseKey::new().with_password(PASSWORD);
    let error = Database::parse(&bytes, key).err();
    let padding = matches!(
        error,
        Some(DatabaseOpenError::Cryptography(
            CryptographyError::InvalidPadding(_)
        ))
    );
    assert!(padding, "{error:?}");
    assert_eq!(read(&bytes, PASSWORD).err(), Some(Fault::WrongPassword));
}

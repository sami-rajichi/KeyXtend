//! The backup box's facts, set once for the app and the worker (ADR-0038 decision 5), and the vault job's memory cap.

use crate::limits::{MAGIC, MAX_PART_NAME, MIB, SCRYPT_BYTES};
use crate::{
    BOX_EXT, BOX_MARK, BOX_NAME, BOX_PARTS, BOX_VERSION, MAX_BOX_BYTES, MAX_FIELD_BYTES,
    MAX_FIELD_CHARS, MAX_KDF_BYTES, MAX_PARTS, MAX_PASSWORD_BYTES, NOTEBOOK_PART, VAULT_MEMORY_MB,
};

#[test]
fn the_box_is_keyxtend_backup_kxbox() {
    assert_eq!(format!("{BOX_NAME}.{BOX_EXT}"), "KeyXtend Backup.kxbox");
}

#[test]
fn the_extension_is_lower_case_ascii_so_a_find_folds_only_the_name() {
    assert!(BOX_EXT.bytes().all(|b| b.is_ascii_lowercase()), "{BOX_EXT}");
}

#[test]
fn a_box_never_reads_as_a_frame_nor_a_zeroed_file_as_a_box() {
    let (mark, magic) = (&BOX_MARK[..], &MAGIC[..]);
    assert!(!mark.starts_with(magic) && !magic.starts_with(mark));
    assert!(mark.iter().all(|&b| b != 0), "{mark:?}");
    assert_ne!(BOX_VERSION, 0);
}

#[test]
fn the_vault_job_opens_the_largest_box_at_the_top_work_factor() {
    let cap = usize::try_from(VAULT_MEMORY_MB).unwrap() * MIB;
    assert!(cap >= SCRYPT_BYTES + MAX_BOX_BYTES, "{cap} bytes");
}

#[test]
fn the_box_parts_are_short_lower_case_ascii_names_each_once() {
    assert!(BOX_PARTS.contains(&NOTEBOOK_PART));
    for (i, name) in BOX_PARTS.iter().enumerate() {
        assert!(!name.is_empty() && name.len() <= MAX_PART_NAME, "{name}");
        assert!(name.bytes().all(|b| b.is_ascii_lowercase()), "{name}");
        assert!(!BOX_PARTS[..i].contains(name), "{name} twice");
    }
    assert_eq!(MAX_PARTS, BOX_PARTS.len());
}

#[test]
fn a_keepass_key_derivation_fits_the_vault_jobs_memory_cap() {
    assert_eq!(MAX_KDF_BYTES, u64::try_from(SCRYPT_BYTES).unwrap());
    let cap = u64::from(VAULT_MEMORY_MB) * u64::try_from(MIB).unwrap();
    let file = u64::try_from(MAX_BOX_BYTES).unwrap();
    assert!(cap >= MAX_KDF_BYTES + file, "{cap} bytes");
}

#[test]
fn a_field_holds_its_most_characters_in_any_script() {
    let arabic = "ب".repeat(MAX_FIELD_CHARS);
    let wide = "😀".repeat(MAX_FIELD_CHARS);
    assert!(arabic.len() <= MAX_FIELD_BYTES && wide.len() == MAX_FIELD_BYTES);
    assert_eq!(MAX_PASSWORD_BYTES, MAX_FIELD_BYTES);
}

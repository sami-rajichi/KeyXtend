use super::*;

const KEY: [u8; KEY_LEN] = [7; KEY_LEN];
const OTHER: [u8; KEY_LEN] = [8; KEY_LEN];
const AAD: &[u8] = b"predict-en";
const PLAIN: &[u8] = b"bonjour \xd9\x85\xd8\xb1\xd8\xad\xd8\xa8\xd8\xa7";

fn sealed() -> Vec<u8> {
    seal(&KEY, AAD, PLAIN).unwrap()
}

#[test]
fn seal_then_open_gives_the_bytes() {
    assert_eq!(open(&KEY, AAD, &sealed()).unwrap().as_slice(), PLAIN);
}

#[test]
fn an_empty_blob_round_trips() {
    let sealed = seal(&KEY, AAD, b"").unwrap();
    assert_eq!(sealed.len(), OVERHEAD);
    assert!(open(&KEY, AAD, &sealed).unwrap().is_empty());
}

#[test]
fn the_file_starts_with_the_magic_and_version_and_hides_the_bytes() {
    let sealed = sealed();
    assert_eq!(&sealed[..MAGIC.len()], MAGIC);
    assert_eq!(sealed[MAGIC.len()], VERSION);
    assert_eq!(sealed.len(), PLAIN.len() + OVERHEAD);
    assert!(!sealed.windows(PLAIN.len()).any(|w| w == PLAIN));
}

#[test]
fn each_seal_takes_a_fresh_nonce() {
    let (a, b) = (sealed(), sealed());
    assert_ne!(
        a[MAGIC.len() + 1..HEADER_LEN],
        b[MAGIC.len() + 1..HEADER_LEN]
    );
    assert_ne!(a, b);
}

#[test]
fn a_flipped_bit_anywhere_is_refused() {
    let good = sealed();
    for bit in 0..good.len() * 8 {
        let mut bad = good.clone();
        bad[bit / 8] ^= 1 << (bit % 8);
        let want = if bit / 8 <= MAGIC.len() {
            CryptoError::BadMagic
        } else {
            CryptoError::Refused
        };
        assert_eq!(open(&KEY, AAD, &bad).unwrap_err(), want, "bit {bit}");
    }
}

#[test]
fn a_wrong_key_is_refused() {
    assert_eq!(
        open(&OTHER, AAD, &sealed()).unwrap_err(),
        CryptoError::Refused
    );
}

#[test]
fn a_wrong_aad_is_refused() {
    let err = open(&KEY, b"predict-fr", &sealed()).unwrap_err();
    assert_eq!(err, CryptoError::Refused);
}

#[test]
fn a_truncated_file_is_refused() {
    let good = sealed();
    for len in 0..good.len() {
        let want = if len < OVERHEAD {
            CryptoError::TooShort
        } else {
            CryptoError::Refused
        };
        assert_eq!(
            open(&KEY, AAD, &good[..len]).unwrap_err(),
            want,
            "length {len}"
        );
    }
}

#[test]
fn an_empty_file_is_too_short() {
    assert_eq!(open(&KEY, AAD, b"").unwrap_err(), CryptoError::TooShort);
}

#[test]
fn a_bad_magic_or_version_is_refused() {
    let mut magic = sealed();
    magic[..MAGIC.len()].copy_from_slice(b"KXSX");
    assert_eq!(open(&KEY, AAD, &magic).unwrap_err(), CryptoError::BadMagic);
    let mut version = sealed();
    version[MAGIC.len()] = VERSION + 1;
    assert_eq!(
        open(&KEY, AAD, &version).unwrap_err(),
        CryptoError::BadMagic
    );
}

#[test]
fn a_blob_over_the_limit_is_refused_both_ways() {
    let big = vec![0; MAX_BLOB_BYTES + 1];
    assert_eq!(seal(&KEY, AAD, &big).unwrap_err(), CryptoError::TooLarge);
    let huge = vec![0; MAX_BLOB_BYTES + OVERHEAD + 1];
    assert_eq!(open(&KEY, AAD, &huge).unwrap_err(), CryptoError::TooLarge);
}

#[test]
fn a_file_at_the_limit_is_read_not_refused_for_its_size() {
    let full = vec![0; MAX_BLOB_BYTES + OVERHEAD];
    assert_eq!(open(&KEY, AAD, &full).unwrap_err(), CryptoError::BadMagic);
}

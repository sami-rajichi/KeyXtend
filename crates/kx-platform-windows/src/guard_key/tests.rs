//! The DPAPI guard: keys go round for this user, and anything else is refused.

use kx_crypto::KEY_LEN;
use proptest::prelude::*;
use windows::Win32::Foundation::{E_FAIL, RPC_E_DISCONNECTED};

use super::*;

/// Random keys or junk blobs per test: each case is one or two DPAPI calls of a few milliseconds.
const CASES: u32 = 16;
/// The longest junk blob tried.
const MAX_JUNK: usize = 512;
/// Another program's entropy, which our guard must not accept.
const OTHER_ENTROPY: &[u8] = b"another program";
/// A byte that is not zero, to see the wipe.
const FILL: u8 = 0xA5;
/// A cut shorter than DPAPI's header, which Windows calls an invalid parameter.
const SHORT: usize = 40;

/// A fixed key of the cipher's length.
fn key() -> Vec<u8> {
    (0..KEY_LEN).map(|i| u8::try_from(i).unwrap()).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn a_random_key_wraps_to_other_bytes_and_unwraps_back(
        key in prop::collection::vec(any::<u8>(), KEY_LEN),
    ) {
        let wrapped = WinKeyGuard.wrap(&key).unwrap();
        prop_assert!(wrapped.len() > key.len());
        prop_assert!(!wrapped.windows(key.len()).any(|w| w == key.as_slice()), "the key shows in clear");
        let back = WinKeyGuard.unwrap(&wrapped).unwrap();
        prop_assert_eq!(&back[..], &key[..]);
    }

    #[test]
    fn random_bytes_are_refused(junk in prop::collection::vec(any::<u8>(), 1..MAX_JUNK)) {
        prop_assert_eq!(WinKeyGuard.unwrap(&junk), Err(GuardError::Refused));
    }
}

#[test]
fn two_wraps_of_one_key_differ_but_both_unwrap() {
    let (first, second) = (
        WinKeyGuard.wrap(&key()).unwrap(),
        WinKeyGuard.wrap(&key()).unwrap(),
    );
    assert_ne!(first, second, "DPAPI salts every wrap");
    assert_eq!(&WinKeyGuard.unwrap(&second).unwrap()[..], &key()[..]);
}

#[test]
fn a_damaged_wrap_is_refused() {
    let mut wrapped = WinKeyGuard.wrap(&key()).unwrap();
    let last = wrapped.len() - 1;
    wrapped[last] ^= FILL;
    assert_eq!(WinKeyGuard.unwrap(&wrapped), Err(GuardError::Refused));
}

#[test]
fn an_empty_or_short_blob_is_refused() {
    let wrapped = WinKeyGuard.wrap(&key()).unwrap();
    for len in [0, 1, SHORT, wrapped.len() / 2, wrapped.len() - 1] {
        assert_eq!(
            WinKeyGuard.unwrap(&wrapped[..len]),
            Err(GuardError::Refused),
            "{len} bytes"
        );
    }
}

#[test]
fn a_wrap_under_another_entropy_is_refused() {
    let theirs = protect(&key(), OTHER_ENTROPY).unwrap().bytes().to_vec();
    assert_eq!(WinKeyGuard.unwrap(&theirs), Err(GuardError::Refused));
    let back = unprotect(&theirs, OTHER_ENTROPY).unwrap();
    assert_eq!(back.bytes(), key().as_slice(), "it is only the entropy");
}

#[test]
fn declined_data_is_refused_and_any_other_failure_unavailable() {
    for code in REFUSED {
        assert_eq!(guard_error(code), GuardError::Refused, "{code:?}");
    }
    for code in [E_FAIL, RPC_E_DISCONNECTED] {
        assert_eq!(guard_error(code), GuardError::Unavailable, "{code:?}");
    }
}

#[test]
fn the_wipe_zeroes_every_byte() {
    let mut bytes = [FILL; KEY_LEN];
    wipe(&mut bytes);
    assert_eq!(bytes, [0; KEY_LEN]);
}

#[test]
fn the_guard_works_as_a_trait_object_on_another_thread() {
    let guard: Box<dyn KeyGuard> = Box::new(WinKeyGuard);
    let back = std::thread::spawn(move || guard.unwrap(&guard.wrap(&key())?)).join();
    assert_eq!(back.unwrap().map(|k| k.to_vec()), Ok(key()));
}

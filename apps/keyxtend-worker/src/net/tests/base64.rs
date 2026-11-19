//! Base64 as Gemini's request writes the sound: RFC 4648's own vectors, four characters for each three bytes rounded
//! up, sizes on each side of a chunk's edge, and any bytes reading back as themselves.

use crate::net::base64::Base64;
use proptest::prelude::*;

/// RFC 4648 section 10's test vectors.
const VECTORS: [(&str, &str); 7] = [
    ("", ""),
    ("f", "Zg=="),
    ("fo", "Zm8="),
    ("foo", "Zm9v"),
    ("foob", "Zm9vYg=="),
    ("fooba", "Zm9vYmE="),
    ("foobar", "Zm9vYmFy"),
];
/// The alphabet a test reads Base64 back with, apart from the one under test.
const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
/// What pads a last group.
const PAD: u8 = b'=';
/// Bytes in a group, and the characters that write it.
const GROUP_IN: usize = 3;
const GROUP_OUT: usize = 4;
/// Bits each character holds.
const SEXTET: u32 = 6;

/// `text` read back from Base64; `None` for text that is not.
pub(super) fn decoded(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(GROUP_OUT) {
        return None;
    }
    let mut out = Vec::new();
    for group in bytes.chunks(GROUP_OUT) {
        let pad = group.iter().rev().take_while(|&&b| b == PAD).count();
        let mut word = 0u32;
        for &b in group.get(..GROUP_OUT - pad.min(2))? {
            let at = ALPHABET.iter().position(|&a| a == b)?;
            word = word << SEXTET | u32::try_from(at).ok()?;
        }
        word <<= SEXTET * u32::try_from(pad).ok()?;
        out.extend_from_slice(&word.to_be_bytes()[1..=GROUP_IN - pad]);
    }
    Some(out)
}

/// `bytes` as the encoder writes them.
fn encoded(bytes: &[u8]) -> String {
    Base64(bytes).to_string()
}

/// The characters `n` bytes take: four for each three, rounded up.
fn len(n: usize) -> usize {
    n.div_ceil(GROUP_IN) * GROUP_OUT
}

#[test]
fn rfcs_vectors_read_as_it_gives_them() {
    for (plain, want) in VECTORS {
        assert_eq!(encoded(plain.as_bytes()), want, "{plain:?}");
        assert_eq!(decoded(want).as_deref(), Some(plain.as_bytes()));
    }
}

#[test]
fn sizes_on_each_side_of_a_chunks_edge_read_back_whole() {
    for n in [3071, 3072, 3073, 6143, 6144, 6145, 100_000] {
        let bytes: Vec<u8> = (0..n).map(|i| u8::try_from(i % 251).unwrap()).collect();
        let text = encoded(&bytes);
        assert_eq!(text.len(), len(n), "{n}");
        assert_eq!(decoded(&text), Some(bytes), "{n}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn any_bytes_read_back_as_themselves_four_characters_to_each_three(
        bytes in proptest::collection::vec(any::<u8>(), 0..4096),
    ) {
        let text = encoded(&bytes);
        prop_assert_eq!(text.len(), len(bytes.len()));
        prop_assert!(text.bytes().all(|b| ALPHABET.contains(&b) || b == PAD));
        prop_assert_eq!(decoded(&text), Some(bytes));
    }
}

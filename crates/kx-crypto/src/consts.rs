//! Every file name, the magic and the limits of the crate, in one place.

use chacha20poly1305::aead::array::typenum::Unsigned;
use chacha20poly1305::{AeadCore, KeySizeUser, XChaCha20Poly1305};

/// The bytes a sealed file starts with.
pub(crate) const MAGIC: &[u8; 4] = b"KXSL";
/// The format version after the magic; any other version is refused.
pub(crate) const VERSION: u8 = 1;
/// The magic then the version: the start of every sealed file and of the AAD its tag covers.
pub(crate) const MARK: [u8; MAGIC.len() + 1] = mark();
/// The data key's length in bytes, as the cipher takes it.
pub const KEY_LEN: usize = <<XChaCha20Poly1305 as KeySizeUser>::KeySize as Unsigned>::USIZE;
/// The nonce's length in bytes, as the cipher takes it.
pub(crate) const NONCE_LEN: usize = <<XChaCha20Poly1305 as AeadCore>::NonceSize as Unsigned>::USIZE;
/// The tag's length in bytes, as the cipher makes it.
pub(crate) const TAG_LEN: usize = <<XChaCha20Poly1305 as AeadCore>::TagSize as Unsigned>::USIZE;
/// The bytes before the ciphertext: the mark and the nonce.
pub(crate) const HEADER_LEN: usize = MARK.len() + NONCE_LEN;
/// What sealing adds to a blob: the header and the tag.
pub(crate) const OVERHEAD: usize = HEADER_LEN + TAG_LEN;

/// The largest blob that is sealed or opened, 16 MiB.
pub const MAX_BLOB_BYTES: usize = 16 * 1024 * 1024;
/// The largest key file that is read; a wrapped 32-byte key is far smaller.
pub(crate) const MAX_KEY_FILE_BYTES: usize = 64 * 1024;

/// The file in the folder that holds the wrapped data key.
pub(crate) const KEY_FILE: &str = "sealed.key";
/// The extension of a blob's sealed file.
pub(crate) const SEALED_EXT: &str = "sealed";
/// The extension of a blob set aside because it would not open.
pub(crate) const DAMAGED_EXT: &str = "damaged";
/// The extension a save writes first, before the file takes the old one's place.
pub(crate) const TEMP_EXT: &str = "tmp";

/// Builds `MARK` from `MAGIC` and `VERSION`, so neither is written twice.
const fn mark() -> [u8; MAGIC.len() + 1] {
    let mut mark = [VERSION; MAGIC.len() + 1];
    let mut i = 0;
    while i < MAGIC.len() {
        mark[i] = MAGIC[i];
        i += 1;
    }
    mark
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mark_is_the_magic_then_the_version() {
        assert_eq!(&MARK, b"KXSL\x01");
    }

    #[test]
    fn the_cipher_takes_a_32_byte_key_and_a_24_byte_nonce() {
        assert_eq!((KEY_LEN, NONCE_LEN, TAG_LEN), (32, 24, 16));
        assert_eq!(OVERHEAD, 4 + 1 + 24 + 16);
    }

    #[test]
    fn the_extensions_differ_and_the_key_file_is_no_blob_file() {
        let exts = [SEALED_EXT, DAMAGED_EXT, TEMP_EXT];
        assert!(
            exts.iter()
                .enumerate()
                .all(|(i, a)| !exts[i + 1..].contains(a))
        );
        let key_ext = std::path::Path::new(KEY_FILE).extension();
        assert!(exts.iter().all(|ext| key_ext != Some(ext.as_ref())));
    }
}

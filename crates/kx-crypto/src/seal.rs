//! Seal and open one blob: XChaCha20-Poly1305 with a fresh nonce, after a magic and a version byte.
//! Layout: magic, version, nonce, ciphertext, tag; the AAD binds the blob to its name.

use crate::consts::{
    HEADER_LEN, KEY_LEN, MAGIC, MAX_BLOB_BYTES, NONCE_LEN, OVERHEAD, TAG_LEN, VERSION,
};
use chacha20poly1305::{AeadInOut, Key, KeyInit, Tag, XChaCha20Poly1305, XNonce};
use thiserror::Error;
use zeroize::Zeroizing;

/// Why sealed bytes would not open, or a blob would not seal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum CryptoError {
    /// The bytes are shorter than the header and the tag.
    #[error("the sealed bytes are too short")]
    TooShort,
    /// The bytes do not start with the magic and the version byte.
    #[error("the sealed bytes do not start with the magic and version")]
    BadMagic,
    /// Authentication failed: a wrong key or name, or changed bytes.
    #[error("the sealed bytes failed authentication")]
    Refused,
    /// The blob is over `MAX_BLOB_BYTES`.
    #[error("the blob is over the size limit")]
    TooLarge,
    /// The system gave no random bytes for the nonce.
    #[error("the system gave no random bytes")]
    NoRandom,
}

/// The cipher under `key`; its copy of the key is wiped when it drops.
fn cipher(key: &[u8; KEY_LEN]) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(<&Key>::from(key))
}

/// Seals `plain` under `key`, bound to `aad`, with a fresh random nonce.
///
/// # Errors
/// `TooLarge` over `MAX_BLOB_BYTES`, `NoRandom` when the system gives no random bytes.
pub fn seal(key: &[u8; KEY_LEN], aad: &[u8], plain: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if plain.len() > MAX_BLOB_BYTES {
        return Err(CryptoError::TooLarge);
    }
    let mut nonce = [0; NONCE_LEN];
    getrandom::fill(&mut nonce).map_err(|_| CryptoError::NoRandom)?;
    // The buffer holds the plain bytes until they are encrypted in place, so it is wiped if that fails.
    let mut out = Zeroizing::new(Vec::with_capacity(OVERHEAD + plain.len()));
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(plain);
    let tag = cipher(key)
        .encrypt_inout_detached(
            <&XNonce>::from(&nonce),
            aad,
            (&mut out[HEADER_LEN..]).into(),
        )
        .map_err(|_| CryptoError::TooLarge)?;
    out.extend_from_slice(&tag);
    Ok(std::mem::take(&mut *out))
}

/// Opens bytes that `seal` made under `key` and `aad`.
///
/// # Errors
/// `TooLarge`, `TooShort` or `BadMagic` from the size and header, `Refused` when authentication fails.
pub fn open(
    key: &[u8; KEY_LEN],
    aad: &[u8],
    sealed: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if sealed.len() > MAX_BLOB_BYTES + OVERHEAD {
        return Err(CryptoError::TooLarge);
    }
    let Some((head, rest)) = sealed.split_first_chunk::<HEADER_LEN>() else {
        return Err(CryptoError::TooShort);
    };
    let Some((body, tag)) = rest.split_last_chunk::<TAG_LEN>() else {
        return Err(CryptoError::TooShort);
    };
    let Some((mark, nonce)) = head.split_last_chunk::<NONCE_LEN>() else {
        return Err(CryptoError::TooShort);
    };
    if mark.split_last() != Some((&VERSION, MAGIC.as_slice())) {
        return Err(CryptoError::BadMagic);
    }
    let mut plain = Zeroizing::new(body.to_vec());
    cipher(key)
        .decrypt_inout_detached(
            <&XNonce>::from(nonce),
            aad,
            plain.as_mut_slice().into(),
            <&Tag>::from(tag),
        )
        .map_err(|_| CryptoError::Refused)?;
    Ok(plain)
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;

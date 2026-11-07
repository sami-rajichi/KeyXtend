//! The box's `age` files: the identity locked by scrypt under the password, and every part sealed to the recipient
//! as one file, so no part can be cut, moved or added unseen. Opened bytes sit in buffers wiped when dropped.

use super::layout;
use age::secrecy::{ExposeSecret, SecretString};
use age::{DecryptError, Decryptor, Encryptor, scrypt, x25519};
use kx_ipc::{BoxPart, Fault, MAX_BOX_WORK};
use std::io::{Read, Write};
use std::iter;
use zeroize::Zeroizing;

/// Locks `identity` under `password` at scrypt work factor `work`.
///
/// # Errors
/// `Internal` when `age` fails.
pub(super) fn lock(
    identity: &x25519::Identity,
    password: &str,
    work: u8,
) -> Result<Vec<u8>, Fault> {
    let mut recipient = scrypt::Recipient::new(secret(password));
    recipient.set_work_factor(work);
    let mut out = Vec::new();
    seal(
        &recipient,
        &[identity.to_string().expose_secret().as_bytes()],
        &mut out,
    )?;
    Ok(out)
}

/// True when `locked` reads as an identity locked by a password.
pub(super) fn is_locked(locked: &[u8]) -> bool {
    Decryptor::new_buffered(locked).is_ok_and(|d| d.is_scrypt())
}

/// The identity `locked` holds, opened with `password`; a work factor past `MAX_BOX_WORK` is never run.
///
/// # Errors
/// `WrongPassword`, or `Damaged` for an identity that will not read.
pub(super) fn unlock(locked: &[u8], password: &str) -> Result<x25519::Identity, Fault> {
    let mut identity = scrypt::Identity::new(secret(password));
    identity.set_max_work_factor(MAX_BOX_WORK);
    let plain = open(&identity, locked, Fault::WrongPassword)?;
    let text = std::str::from_utf8(&plain).map_err(|_| Fault::Damaged)?;
    text.parse().map_err(|_| Fault::Damaged)
}

/// Seals `parts` to `recipient` as one `age` file, each after its name and length, appended to `out`.
///
/// # Errors
/// `Internal` for a name or part past its length's width, or when `age` fails.
pub(super) fn seal_parts(
    recipient: &x25519::Recipient,
    parts: &[BoxPart<'_>],
    out: &mut Vec<u8>,
) -> Result<(), Fault> {
    let heads = parts
        .iter()
        .map(|p| layout::head(&p.name, p.bytes.len()))
        .collect::<Result<Vec<_>, _>>()?;
    let pieces: Vec<&[u8]> = heads
        .iter()
        .zip(parts)
        .flat_map(|(head, part)| [&head[..], &part.bytes])
        .collect();
    seal(recipient, &pieces, out)
}

/// The plain bytes of a box's parts, opened by `identity`.
///
/// # Errors
/// `Damaged` for an `age` file that will not open, or is cut, spliced or longer.
pub(super) fn open_parts(
    identity: &x25519::Identity,
    sealed: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Fault> {
    open(identity, sealed, Fault::Damaged)
}

/// `plain` as given, sealed to `recipient` as one `age` file, for a test that forges a box's parts.
#[cfg(test)]
pub(super) fn seal_raw(
    recipient: &x25519::Recipient,
    plain: &[u8],
    out: &mut Vec<u8>,
) -> Result<(), Fault> {
    seal(recipient, &[plain], out)
}

/// Writes `pieces` as one `age` file to `recipient`, appended to `out`.
fn seal(recipient: &dyn age::Recipient, pieces: &[&[u8]], out: &mut Vec<u8>) -> Result<(), Fault> {
    let encryptor =
        Encryptor::with_recipients(iter::once(recipient)).map_err(|_| Fault::Internal)?;
    let mut w = encryptor.wrap_output(out).map_err(|_| Fault::Internal)?;
    for piece in pieces {
        w.write_all(piece).map_err(|_| Fault::Internal)?;
    }
    w.finish().map_err(|_| Fault::Internal)?;
    Ok(())
}

/// The plain bytes of the `age` file `sealed`, opened by `identity`; `wrong` when its key does not open it.
fn open(
    identity: &dyn age::Identity,
    sealed: &[u8],
    wrong: Fault,
) -> Result<Zeroizing<Vec<u8>>, Fault> {
    let decryptor = Decryptor::new_buffered(sealed).map_err(|_| Fault::Damaged)?;
    let mut reader = decryptor
        .decrypt(iter::once(identity))
        .map_err(|e| match e {
            DecryptError::DecryptionFailed => wrong,
            _ => Fault::Damaged,
        })?;
    // The plain bytes are shorter than the sealed ones, so the buffer never moves and leaves no copy behind.
    let mut plain = Zeroizing::new(Vec::with_capacity(sealed.len()));
    reader.read_to_end(&mut plain).map_err(|_| Fault::Damaged)?;
    Ok(plain)
}

/// `password` as `age` takes it, wiped when dropped.
fn secret(password: &str) -> SecretString {
    SecretString::from(password.to_owned())
}

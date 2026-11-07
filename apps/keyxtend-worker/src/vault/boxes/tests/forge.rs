//! Forged boxes for the tests: parts sealed as given under the shared padlock, boxes large enough for two `age`
//! chunks, and opening with the shared identity so no test waits on scrypt.

use super::*;

/// The plain bytes in one `age` STREAM chunk, and the tag each chunk carries (age spec, "Payload").
const AGE_CHUNK: usize = 64 * 1024;
const AGE_TAG: usize = 16;
/// A notebook that fills more than one chunk, the last one short.
const BIG_NOTEBOOK: usize = 100_000;

/// A box under the shared padlock whose parts are `parts`, each a name and bytes, sealed as given.
pub(super) fn forged(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let p = padlock_once();
    let recipient = p.recipient.parse().unwrap();
    let parts: Vec<BoxPart<'_>> = parts
        .iter()
        .map(|&(name, bytes)| BoxPart {
            name: Cow::Borrowed(name),
            bytes: Wiped::Lent(bytes),
        })
        .collect();
    let mut out = layout::begin(&p.locked).unwrap();
    cipher::seal_parts(&recipient, &parts, &mut out).unwrap();
    out
}

/// A box under the shared padlock whose sealed bytes are `plain` as given, so a test can cut its parts.
pub(super) fn raw(plain: &[u8]) -> Vec<u8> {
    let p = padlock_once();
    let recipient: x25519::Recipient = p.recipient.parse().unwrap();
    let mut out = layout::begin(&p.locked).unwrap();
    cipher::seal_raw(&recipient, plain, &mut out).unwrap();
    out
}

/// The parts of `bytes`, opened with the shared padlock's identity, unlocked once.
pub(super) fn open_with(bytes: &[u8]) -> Result<Vec<BoxPart<'static>>, Fault> {
    static ONE: OnceLock<x25519::Identity> = OnceLock::new();
    let identity = ONE.get_or_init(|| cipher::unlock(&padlock_once().locked, PASSWORD).unwrap());
    open_parts(identity, layout::read(bytes)?.sealed)
}

/// A box whose notebook is `BIG_NOTEBOOK` bytes of `fill`.
pub(super) fn big(fill: u8) -> Vec<u8> {
    sealed_with(&vec![fill; BIG_NOTEBOOK])
}

/// The sealed bytes of a big box's last chunk.
pub(super) fn last_chunk() -> usize {
    let plain = layout::head(NOTEBOOK_PART, BIG_NOTEBOOK).unwrap().len() + BIG_NOTEBOOK;
    plain % AGE_CHUNK + AGE_TAG
}

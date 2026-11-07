//! A KDBX file's key derivation, read from its outer header before any key is derived: AES-KDF's rounds, or Argon2's
//! memory, passes and lanes, refused past the vault job's caps. The walk is `keepass` 0.17's, with the last value
//! winning as there, and it refuses a number of the wrong width, which `keepass` would panic on.

use kx_ipc::{Fault, MAX_AES_ROUNDS, MAX_KDF_BYTES, MAX_KDF_LANES, MAX_KDF_PASSES};

/// The bytes every KDBX file starts with.
const SIGNATURE: [u8; 4] = [0x03, 0xd9, 0xa2, 0x9a];
/// Bytes before the outer header: the signature, the format id and the minor and major versions.
const VERSION_HEAD: usize = 12;
/// Where the major version sits, and the two read here.
const MAJOR_AT: usize = 10;
const KDBX3: u16 = 3;
const KDBX4: u16 = 4;
/// Outer header ids: the end, the compression flags, KDBX 3's rounds and inner stream, and KDBX 4's dictionaries.
const END: u8 = 0;
const COMPRESSION: u8 = 3;
const ROUNDS: u8 = 6;
const INNER_STREAM: u8 = 10;
const KDF_PARAMS: u8 = 11;
const CUSTOM_DATA: u8 = 12;
/// A variant dictionary's version and end mark, and the bytes `keepass` needs left to read another item.
const DICT_VERSION: [u8; 2] = 0x100_u16.to_le_bytes();
const DICT_END: u8 = 0;
const DICT_ITEM_MIN: usize = 9;
/// Variant dictionary value types, with the width a number takes; the others take any length.
const DICT_U32: u8 = 0x04;
const DICT_U64: u8 = 0x05;
const DICT_BOOL: u8 = 0x08;
const DICT_I32: u8 = 0x0c;
const DICT_I64: u8 = 0x0d;
const DICT_STR: u8 = 0x18;
const DICT_BYTES: u8 = 0x42;
/// The key derivation's dictionary keys.
const KDF_ID: &[u8] = b"$UUID";
const MEMORY: &[u8] = b"M";
const PASSES: &[u8] = b"I";
const LANES: &[u8] = b"P";
const AES_ROUNDS: &[u8] = b"R";
/// Key derivation ids: AES-KDF as KDBX 3 and KDBX 4 write it, Argon2d and Argon2id.
const AES_KDBX3: [u8; 16] = *b"\xc9\xd9\xf3\x9a\x62\x8a\x44\x60\xbf\x74\x0d\x08\xc1\x8a\x4f\xea";
const AES_KDBX4: [u8; 16] = *b"\x7c\x02\xbb\x82\x79\xa7\x4a\xc0\x92\x7d\x11\x4a\x00\x64\x82\x38";
const ARGON2D: [u8; 16] = *b"\xef\x63\x6d\xdf\x8c\x29\x44\x4b\x91\xf7\xa9\xa4\x03\xe3\x0a\x0c";
const ARGON2ID: [u8; 16] = *b"\x9e\x29\x8b\x19\x56\xdb\x47\x73\xb2\x3d\xfc\x3e\xc6\xf0\xa1\xe6";

/// True when `bytes` are a cut start of a KDBX file: too short to hold its version, with the signature as far as
/// they go.
pub(super) fn cut_signature(bytes: &[u8]) -> bool {
    bytes.len() < VERSION_HEAD && SIGNATURE.iter().zip(bytes).all(|(a, b)| a == b)
}

/// The dictionary keys the check reads.
const KEYS: &[&[u8]] = &[KDF_ID, MEMORY, PASSES, LANES, AES_ROUNDS];

/// The last type and value given under each of `KEYS`; the walk keeps nothing else.
#[derive(Default)]
struct Params<'a>([Option<(u8, &'a [u8])>; KEYS.len()]);

impl<'a> Params<'a> {
    /// Keeps `value` of type `ty` when `key` is one the check reads.
    fn keep(&mut self, key: &[u8], ty: u8, value: &'a [u8]) {
        let at = KEYS.iter().position(|k| *k == key);
        if let Some(slot) = at.and_then(|at| self.0.get_mut(at)) {
            *slot = Some((ty, value));
        }
    }

    /// The value under `key`, `Damaged` when it is missing or not of type `ty`.
    fn typed(&self, key: &[u8], ty: u8) -> Result<&'a [u8], Fault> {
        let at = KEYS.iter().position(|k| *k == key);
        match at.and_then(|at| self.0.get(at)).copied().flatten() {
            Some((t, value)) if t == ty => Ok(value),
            _ => Err(Fault::Damaged),
        }
    }
}

/// Refuses a KDBX 3 or 4 file whose key derivation would pass the vault job's caps.
///
/// # Errors
/// `Unsupported` past a cap or for an unknown derivation or major version; `Damaged` for a cut or malformed header.
pub(super) fn check(bytes: &[u8]) -> Result<(), Fault> {
    let major = bytes
        .get(MAJOR_AT..VERSION_HEAD)
        .and_then(|b| b.try_into().ok());
    match u16::from_le_bytes(major.ok_or(Fault::Damaged)?) {
        KDBX3 => check_rounds(kdbx3_rounds(bytes)?),
        KDBX4 => check_params(&kdbx4_params(bytes)?),
        _ => Err(Fault::Unsupported),
    }
}

/// Walks the header fields after the version up to the end field, each an id, a `width`-byte length and the bytes.
fn walk<'a>(
    bytes: &'a [u8],
    width: usize,
    mut each: impl FnMut(u8, &'a [u8]) -> Result<(), Fault>,
) -> Result<(), Fault> {
    let mut rest = bytes.get(VERSION_HEAD..).ok_or(Fault::Damaged)?;
    loop {
        let (&id, after) = rest.split_first().ok_or(Fault::Damaged)?;
        let (len, after) = after.split_at_checked(width).ok_or(Fault::Damaged)?;
        let len = len.iter().rev().fold(0, |n, &b| n << 8 | usize::from(b));
        let (value, after) = after.split_at_checked(len).ok_or(Fault::Damaged)?;
        if id == END {
            return Ok(());
        }
        each(id, value)?;
        rest = after;
    }
}

/// A KDBX 3 file's AES-KDF rounds, refusing short numbers in the fields `keepass` reads as numbers.
fn kdbx3_rounds(bytes: &[u8]) -> Result<u64, Fault> {
    let mut rounds = None;
    walk(bytes, size_of::<u16>(), |id, value| {
        match id {
            COMPRESSION | INNER_STREAM => {
                number(value, size_of::<u32>())?;
            }
            ROUNDS => rounds = Some(number(value, size_of::<u64>())?),
            _ => {}
        }
        Ok(())
    })?;
    rounds.ok_or(Fault::Damaged)
}

/// A KDBX 4 file's key derivation, the last dictionary winning; `keepass` reads every dictionary, so each must read.
fn kdbx4_params(bytes: &[u8]) -> Result<Params<'_>, Fault> {
    let mut params = None;
    walk(bytes, size_of::<u32>(), |id, value| {
        match id {
            COMPRESSION => {
                number(value, size_of::<u32>())?;
            }
            KDF_PARAMS => {
                let mut p = Params::default();
                dict(value, |key, ty, v| p.keep(key, ty, v))?;
                params = Some(p);
            }
            CUSTOM_DATA => dict(value, |_, _, _| {})?,
            _ => {}
        }
        Ok(())
    })?;
    params.ok_or(Fault::Damaged)
}

/// Walks a variant dictionary as `keepass` reads it: an item needs ten bytes left, and the end mark follows.
fn dict<'a>(bytes: &'a [u8], mut each: impl FnMut(&'a [u8], u8, &'a [u8])) -> Result<(), Fault> {
    let mut c = bytes
        .strip_prefix(&DICT_VERSION[..])
        .ok_or(Fault::Damaged)?;
    while c.len() > DICT_ITEM_MIN {
        let (&ty, after) = c.split_first().ok_or(Fault::Damaged)?;
        let (key, after) = sized(after)?;
        let (value, after) = sized(after)?;
        let width = match ty {
            DICT_U32 | DICT_I32 => Some(size_of::<u32>()),
            DICT_U64 | DICT_I64 => Some(size_of::<u64>()),
            DICT_BOOL | DICT_STR | DICT_BYTES => None,
            _ => return Err(Fault::Damaged),
        };
        if width.is_some_and(|w| w != value.len()) {
            return Err(Fault::Damaged);
        }
        each(key, ty, value);
        c = after;
    }
    match c.first() {
        Some(&DICT_END) => Ok(()),
        _ => Err(Fault::Damaged),
    }
}

/// Splits off bytes after their four-byte length.
fn sized(bytes: &[u8]) -> Result<(&[u8], &[u8]), Fault> {
    let (len, rest) = bytes.split_first_chunk::<4>().ok_or(Fault::Damaged)?;
    let len = usize::try_from(u32::from_le_bytes(*len)).map_err(|_| Fault::Damaged)?;
    rest.split_at_checked(len).ok_or(Fault::Damaged)
}

/// A little-endian number of exactly `width` bytes.
fn number(value: &[u8], width: usize) -> Result<u64, Fault> {
    if value.len() != width {
        return Err(Fault::Damaged);
    }
    Ok(value.iter().rev().fold(0, |n, &b| n << 8 | u64::from(b)))
}

/// Refuses AES-KDF rounds past `MAX_AES_ROUNDS`.
fn check_rounds(rounds: u64) -> Result<(), Fault> {
    if rounds > MAX_AES_ROUNDS {
        Err(Fault::Unsupported)
    } else {
        Ok(())
    }
}

/// Refuses a derivation that is unknown or past the memory, pass or lane caps; the host's deadline bounds its time.
fn check_params(p: &Params<'_>) -> Result<(), Fault> {
    let id = p.typed(KDF_ID, DICT_BYTES)?;
    let u64_of = |key| number(p.typed(key, DICT_U64)?, size_of::<u64>());
    if id == AES_KDBX3 || id == AES_KDBX4 {
        return check_rounds(u64_of(AES_ROUNDS)?);
    }
    if id != ARGON2D && id != ARGON2ID {
        return Err(Fault::Unsupported);
    }
    let (memory, passes) = (u64_of(MEMORY)?, u64_of(PASSES)?);
    let lanes = number(p.typed(LANES, DICT_U32)?, size_of::<u32>())?;
    if memory > MAX_KDF_BYTES || passes > MAX_KDF_PASSES || lanes > u64::from(MAX_KDF_LANES) {
        return Err(Fault::Unsupported);
    }
    Ok(())
}

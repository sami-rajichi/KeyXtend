//! The box's bytes, little-endian: the mark and the format version, the locked identity after its two-byte length,
//! then one `age` file holding every part, to the end. Inside it each part is its name after a one-byte length, then
//! its bytes after a four-byte length, so `age` catches any part cut, dropped, moved or added.

use kx_ipc::{BOX_MARK, BOX_VERSION, Fault, MAX_LOCKED_BYTES};
use std::iter;
use std::ops::Range;

/// Where a box's sections sit in its bytes.
pub(super) struct Layout<'a> {
    /// The locked identity.
    pub locked: &'a [u8],
    /// The `age` file of the parts.
    pub sealed: &'a [u8],
}

/// A box's bytes before its parts: the mark, the version and the `locked` identity.
///
/// # Errors
/// `Damaged` for an empty locked identity or one past `MAX_LOCKED_BYTES`.
pub(super) fn begin(locked: &[u8]) -> Result<Vec<u8>, Fault> {
    if locked.is_empty() || locked.len() > MAX_LOCKED_BYTES {
        return Err(Fault::Damaged);
    }
    let len = u16::try_from(locked.len()).map_err(|_| Fault::Damaged)?;
    Ok([&BOX_MARK[..], &[BOX_VERSION], &len.to_le_bytes(), locked].concat())
}

/// The bytes before a part's own inside the `age` file: its name and its length.
///
/// # Errors
/// `Internal` for a name past one length byte or bytes past a four-byte length.
pub(super) fn head(name: &str, len: usize) -> Result<Vec<u8>, Fault> {
    let name_len = u8::try_from(name.len()).map_err(|_| Fault::Internal)?;
    let len = u32::try_from(len).map_err(|_| Fault::Internal)?;
    Ok([&[name_len][..], name.as_bytes(), &len.to_le_bytes()].concat())
}

/// Reads a box's layout, before any key is derived.
///
/// # Errors
/// `Foreign` without the mark, `Unsupported` for a newer version, and `Damaged` for anything cut or out of range.
pub(super) fn read(bytes: &[u8]) -> Result<Layout<'_>, Fault> {
    let Some(rest) = bytes.strip_prefix(&BOX_MARK[..]) else {
        return Err(if BOX_MARK.starts_with(bytes) {
            Fault::Damaged
        } else {
            Fault::Foreign
        });
    };
    let mut c = Cursor(rest);
    let [version] = c.array()?;
    if version > BOX_VERSION {
        return Err(Fault::Unsupported);
    } else if version != BOX_VERSION {
        return Err(Fault::Damaged);
    }
    let len = usize::from(u16::from_le_bytes(c.array()?));
    if len == 0 || len > MAX_LOCKED_BYTES {
        return Err(Fault::Damaged);
    }
    let locked = c.take(len)?;
    if c.0.is_empty() {
        return Err(Fault::Damaged);
    }
    Ok(Layout {
        locked,
        sealed: c.0,
    })
}

/// Each part in a box's opened bytes, in order, read only as far as it is asked: its name's bytes and where its own
/// bytes sit, or `Damaged` for a part cut short, which ends the walk.
pub(super) fn parts(plain: &[u8]) -> impl Iterator<Item = Result<Part<'_>, Fault>> {
    let mut c = Cursor(plain);
    iter::from_fn(move || {
        let part = (!c.0.is_empty()).then(|| c.part(plain.len()));
        if matches!(part, Some(Err(_))) {
            c.0 = &[];
        }
        part
    })
}

/// A part's name and where its bytes sit.
pub(super) type Part<'a> = (&'a [u8], Range<usize>);

/// The bytes of a box not read yet.
struct Cursor<'a>(&'a [u8]);

impl<'a> Cursor<'a> {
    /// The next `n` bytes, `Damaged` when fewer are left.
    fn take(&mut self, n: usize) -> Result<&'a [u8], Fault> {
        let (now, later) = self.0.split_at_checked(n).ok_or(Fault::Damaged)?;
        self.0 = later;
        Ok(now)
    }

    /// The next `N` bytes as an array.
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Fault> {
        self.take(N)?.try_into().map_err(|_| Fault::Damaged)
    }

    /// The next part of opened bytes `whole` long: its name's bytes and where its own bytes sit.
    fn part(&mut self, whole: usize) -> Result<Part<'a>, Fault> {
        let [name_len] = self.array()?;
        let name = self.take(usize::from(name_len))?;
        let len = u32::from_le_bytes(self.array()?);
        let start = whole - self.0.len();
        self.take(usize::try_from(len).map_err(|_| Fault::Damaged)?)?;
        Ok((name, start..whole - self.0.len()))
    }
}

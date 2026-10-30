//! The frame header and the bounded reads under it; little-endian throughout.

use crate::code::{Code, Kind};
use crate::error::IpcError;
use crate::limits::{HEADER_LEN, MAGIC, MAX_SIDE, READ_CHUNK, VERSION};
use std::io::{ErrorKind, Read, Write};

/// A checked header: a kind the reader takes, and a length within that kind's range.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Head {
    /// What the payload carries.
    pub kind: Kind,
    /// The request this frame belongs to.
    pub id: u32,
    /// The payload's length in bytes.
    pub len: usize,
}

impl Head {
    /// Reads and checks a header, before any payload byte is read or allocated.
    pub(crate) fn read<R: Read + ?Sized>(r: &mut R, kinds: &[Kind]) -> Result<Self, IpcError> {
        let mut raw = [0; HEADER_LEN];
        match fill(r, &mut raw)? {
            0 => return Err(IpcError::Closed),
            n if n < HEADER_LEN => return Err(IpcError::Short),
            _ => {}
        }
        let [m0, m1, m2, m3, version, kind, rest @ ..] = raw;
        let [i0, i1, i2, i3, l0, l1, l2, l3] = rest;
        if [m0, m1, m2, m3] != MAGIC {
            return Err(IpcError::BadMagic);
        }
        if version != VERSION {
            return Err(IpcError::BadVersion);
        }
        let kind = Kind::from_code(kind)
            .filter(|k| kinds.contains(k))
            .ok_or(IpcError::BadKind)?;
        let len = u32::from_le_bytes([l0, l1, l2, l3]);
        let len = usize::try_from(len).map_err(|_| IpcError::TooLong)?;
        check_len(kind, len)?;
        let id = u32::from_le_bytes([i0, i1, i2, i3]);
        Ok(Self { kind, id, len })
    }
}

/// Writes a header for `len` payload bytes, refusing a length its kind may not take.
pub(crate) fn write_head<W: Write + ?Sized>(
    w: &mut W,
    kind: Kind,
    id: u32,
    len: usize,
) -> Result<(), IpcError> {
    check_len(kind, len)?;
    let len = u32::try_from(len).map_err(|_| IpcError::TooLong)?;
    let fields = [VERSION, kind.code()];
    let raw = [&MAGIC[..], &fields, &id.to_le_bytes(), &len.to_le_bytes()].concat();
    w.write_all(&raw)?;
    Ok(())
}

/// Refuses a payload length outside `kind`'s range: `TooLong` above it, `Inconsistent` below.
fn check_len(kind: Kind, len: usize) -> Result<(), IpcError> {
    let lens = kind.lens();
    if len > *lens.end() {
        Err(IpcError::TooLong)
    } else if len < *lens.start() {
        Err(IpcError::Inconsistent)
    } else {
        Ok(())
    }
}

/// `n` as a side, when it is from 1 to `MAX_SIDE` pixels.
pub(crate) fn check_side(n: u32) -> Result<u32, IpcError> {
    if (1..=MAX_SIDE).contains(&n) {
        Ok(n)
    } else {
        Err(IpcError::BadField)
    }
}

/// Reads exactly `N` payload bytes.
pub(crate) fn read_array<const N: usize, R: Read + ?Sized>(r: &mut R) -> Result<[u8; N], IpcError> {
    let mut buf = [0; N];
    if fill(r, &mut buf)? < N {
        return Err(IpcError::Short);
    }
    Ok(buf)
}

/// Reads the payload's last `len` bytes, growing the buffer only as bytes arrive.
pub(crate) fn read_tail<R: Read + ?Sized>(r: &mut R, len: usize) -> Result<Vec<u8>, IpcError> {
    let mut buf = Vec::new();
    while buf.len() < len {
        let have = buf.len();
        let next = grow(have, len);
        buf.reserve_exact(next - have);
        buf.resize(next, 0);
        if fill(r, &mut buf[have..])? < next - have {
            return Err(IpcError::Short);
        }
    }
    Ok(buf)
}

/// The next buffer size: twice what arrived, at least `READ_CHUNK`, never past `len`.
pub(crate) fn grow(have: usize, len: usize) -> usize {
    have.saturating_mul(2).max(READ_CHUNK).min(len)
}

/// Reads into `buf` until it is full or the stream ends, and returns the bytes read.
fn fill<R: Read + ?Sized>(r: &mut R, buf: &mut [u8]) -> Result<usize, IpcError> {
    let mut done = 0;
    while done < buf.len() {
        match r.read(&mut buf[done..]) {
            Ok(0) => break,
            Ok(n) => done += n,
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(done)
}

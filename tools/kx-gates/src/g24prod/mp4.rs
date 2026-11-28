//! An MP4 file's length, read from its movie header (`moov`, then `mvhd`) without decoding a frame.

use std::io::{Read, Seek, SeekFrom};

use kx_module_api::MS_PER_S;

/// The movie box, which holds the movie header.
const MOOV: [u8; 4] = *b"moov";
/// The movie header, which holds the time scale and the length.
const MVHD: [u8; 4] = *b"mvhd";
/// Bytes of a box's size and kind.
const HEAD: u64 = 8;
/// Bytes of a box's long size, when its size reads 1.
const LONG: u64 = 8;
/// A size of 1: the long size follows the kind.
const LONG_SIZE: u32 = 1;
/// A size of 0: the box runs to the end of the file.
const TO_END: u32 = 0;
/// Bytes of the version and flags before the times.
const VERSION: u64 = 4;
/// Bytes of the made and changed times in a version 1 header, which counts in 64 bits.
const TIMES_V1: u64 = 16;
/// Bytes of those times in a version 0 header.
const TIMES_V0: u64 = 8;
/// Bytes of the time scale.
const SCALE: u64 = 4;
/// Bytes of the length in a version 1 header, and in a version 0 one.
const UNITS_V1: u64 = 8;
const UNITS_V0: u64 = 4;

/// A box: its kind, where its body starts and where it ends.
struct Boxed {
    kind: [u8; 4],
    body: u64,
    end: u64,
}

/// Any read failure, as text.
#[allow(clippy::needless_pass_by_value, reason = "It fits `map_err` as it is.")]
fn io(e: std::io::Error) -> String {
    format!("the video: {e}")
}

/// Reads `N` bytes.
fn take<const N: usize, R: Read>(r: &mut R) -> Result<[u8; N], String> {
    let mut b = [0u8; N];
    r.read_exact(&mut b).map_err(io)?;
    Ok(b)
}

/// The box starting at `at`, within a parent ending at `limit`; none at the parent's end.
fn next<R: Read + Seek>(r: &mut R, at: u64, limit: u64) -> Result<Option<Boxed>, String> {
    if at.saturating_add(HEAD) > limit {
        return Ok(None);
    }
    r.seek(SeekFrom::Start(at)).map_err(io)?;
    let size = u32::from_be_bytes(take(r)?);
    let kind = take(r)?;
    let (body, end) = match size {
        LONG_SIZE => (
            at + HEAD + LONG,
            at.saturating_add(u64::from_be_bytes(take(r)?)),
        ),
        TO_END => (at + HEAD, limit),
        n => (at + HEAD, at.saturating_add(u64::from(n))),
    };
    if end < body || end > limit {
        return Err("the video holds a box past its end".into());
    }
    Ok(Some(Boxed { kind, body, end }))
}

/// The first box of kind `kind` from `at` to `limit`.
fn find<R: Read + Seek>(
    r: &mut R,
    kind: [u8; 4],
    mut at: u64,
    limit: u64,
) -> Result<Boxed, String> {
    while let Some(b) = next(r, at, limit)? {
        if b.kind == kind {
            return Ok(b);
        }
        at = b.end;
    }
    Err(format!(
        "the video has no {}",
        String::from_utf8_lossy(&kind)
    ))
}

/// The length of the MP4 movie `r`, in ms, from its movie header's time scale and units.
pub fn length_ms<R: Read + Seek>(r: &mut R) -> Result<u64, String> {
    let file_end = r.seek(SeekFrom::End(0)).map_err(io)?;
    let moov = find(r, MOOV, 0, file_end)?;
    let mvhd = find(r, MVHD, moov.body, moov.end)?;
    r.seek(SeekFrom::Start(mvhd.body)).map_err(io)?;
    let [version, ..] = take::<4, _>(r)?;
    let v1 = version == 1;
    let (times, units_len) = if v1 {
        (TIMES_V1, UNITS_V1)
    } else {
        (TIMES_V0, UNITS_V0)
    };
    let at = mvhd.body + VERSION + times;
    if at + SCALE + units_len > mvhd.end {
        return Err("the video's movie header is too short for its length".into());
    }
    r.seek(SeekFrom::Start(at)).map_err(io)?;
    let scale = u32::from_be_bytes(take(r)?);
    let units = if v1 {
        u64::from_be_bytes(take(r)?)
    } else {
        u64::from(u32::from_be_bytes(take(r)?))
    };
    if scale == 0 || units == 0 {
        return Err("the video's header gives no length".into());
    }
    let ms = (u128::from(units) * u128::from(MS_PER_S) + u128::from(scale) / 2) / u128::from(scale);
    u64::try_from(ms).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    /// A box of kind `kind` around `body`.
    fn boxed(kind: &[u8], body: &[u8]) -> Vec<u8> {
        let size = u32::try_from(body.len() + 8).expect("fits");
        [&size.to_be_bytes()[..], kind, body].concat()
    }

    /// A version 0 movie header of `scale` units a second and `units` long.
    fn mvhd0(scale: u32, units: u32) -> Vec<u8> {
        let body = [
            &[0u8; 4][..],
            &[0; 8],
            &scale.to_be_bytes(),
            &units.to_be_bytes(),
            &[0; 80],
        ]
        .concat();
        boxed(b"mvhd", &body)
    }

    /// A version 1 movie header.
    fn mvhd1(scale: u32, units: u64) -> Vec<u8> {
        let body = [
            &[1u8, 0, 0, 0][..],
            &[0; 16],
            &scale.to_be_bytes(),
            &units.to_be_bytes(),
            &[0; 80],
        ]
        .concat();
        boxed(b"mvhd", &body)
    }

    fn length(file: Vec<u8>) -> Result<u64, String> {
        length_ms(&mut Cursor::new(file))
    }

    #[test]
    fn the_length_is_the_movie_headers_units_over_its_scale() {
        let ftyp = boxed(b"ftyp", b"isom");
        let mdat = boxed(b"mdat", &[7; 64]);
        let moov = boxed(
            b"moov",
            &[boxed(b"trak", &[0; 8]), mvhd0(600, 3_000)].concat(),
        );
        assert_eq!(
            length([ftyp.clone(), mdat.clone(), moov].concat()),
            Ok(5_000)
        );
        let moov = boxed(b"moov", &mvhd1(90_000, 2_700_000));
        assert_eq!(length([ftyp, moov, mdat].concat()), Ok(30_000));
    }

    #[test]
    fn a_large_box_is_stepped_over_by_its_long_size() {
        let mut big = [&1u32.to_be_bytes()[..], b"mdat", &24u64.to_be_bytes()].concat();
        big.extend([0u8; 8]);
        let moov = boxed(b"moov", &mvhd0(1_000, 1_234));
        assert_eq!(length([big, moov].concat()), Ok(1_234));
    }

    #[test]
    fn a_file_with_no_length_is_an_error() {
        assert!(length(boxed(b"ftyp", b"isom")).is_err(), "no movie");
        assert!(length(boxed(b"moov", &mvhd0(0, 5))).is_err(), "no scale");
        assert!(
            length(boxed(b"moov", &mvhd0(1_000, 0))).is_err(),
            "no units"
        );
        let mut cut = boxed(b"moov", &mvhd0(1_000, 5));
        cut.truncate(20);
        assert!(length(cut).is_err(), "cut short");
        assert!(
            length(vec![0, 0, 0, 4, b'f', b'r', b'e', b'e']).is_err(),
            "a box shorter than its header"
        );
    }

    #[test]
    fn a_movie_header_too_short_for_its_length_is_an_error_not_the_next_boxs_bytes() {
        let short = boxed(b"mvhd", &[0u8; 12]);
        let next = boxed(b"trak", &[0, 0, 3, 232, 0, 0, 19, 136, 0, 0, 0, 0]);
        assert!(length(boxed(b"moov", &[short, next].concat())).is_err());
        let short1 = boxed(b"mvhd", &[[1u8, 0, 0, 0].as_slice(), &[0; 20]].concat());
        assert!(length(boxed(b"moov", &short1)).is_err(), "version 1");
    }
}

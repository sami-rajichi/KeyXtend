//! The pictures port: its open gate, pixels that check their size, `Debug` that never shows bytes, and the errors.

use super::*;
use crate::REDACTED_MARKER;
use crate::ports::SHOT_CAP_MB;
use std::collections::HashSet;

/// The longest side the tests ask for.
const SIDE: u32 = 3;

/// `w` by `h` pixels of `len` bytes, asked within `SIDE`.
fn pixels(w: u32, h: u32, len: usize) -> Option<Pixels> {
    Pixels::new(w, h, vec![0; len], SIDE)
}

#[test]
fn pictures_need_no_capability() {
    assert_eq!(PicturesKey::ID.as_str(), "pictures");
    assert_eq!(PicturesKey::CAPABILITY, None);
}

#[test]
fn pixels_take_width_times_height_rgba_bytes() {
    let ok = pixels(2, 3, 2 * 3 * RGBA_BYTES).unwrap();
    assert_eq!((ok.width(), ok.height()), (2, 3));
    assert_eq!(ok.rgba().expose().len(), 2 * 3 * RGBA_BYTES);
    for len in [0, 2 * 3 * RGBA_BYTES - 1, 2 * 3 * RGBA_BYTES + 1] {
        assert_eq!(pixels(2, 3, len), None, "{len} bytes");
    }
}

#[test]
fn pixels_refuse_a_zero_side_or_one_past_the_ask() {
    assert_eq!(pixels(0, 3, 0), None);
    assert_eq!(pixels(2, 0, 0), None);
    let over = usize::try_from(SIDE + 1).unwrap();
    assert_eq!(pixels(SIDE + 1, 1, over * RGBA_BYTES), None);
    assert_eq!(pixels(1, SIDE + 1, over * RGBA_BYTES), None);
    let most = usize::try_from(SIDE * SIDE).unwrap();
    assert!(pixels(SIDE, SIDE, most * RGBA_BYTES).is_some());
}

#[test]
fn a_huge_size_is_refused_without_overflow() {
    assert_eq!(Pixels::new(u32::MAX, u32::MAX, Vec::new(), u32::MAX), None);
}

#[test]
fn debug_never_shows_the_bytes() {
    let bytes = vec![7_u8, 9, 11, 13];
    let raw = format!("{bytes:?}");
    let ask = PictureAsk {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(bytes.clone()),
        side: SIDE,
    };
    let dib = DibAsk {
        png: Redacted::new(bytes.clone()),
        max_bytes: 9,
    };
    let drawn = Pixels::new(1, 1, bytes, SIDE).unwrap();
    for shown in [format!("{ask:?}"), format!("{dib:?}"), format!("{drawn:?}")] {
        assert!(shown.contains(REDACTED_MARKER), "{shown}");
        assert!(!shown.contains(&raw), "{shown} shows {raw}");
    }
    assert_eq!(
        format!("{drawn:?}"),
        format!("Pixels {{ width: 1, height: 1, rgba: {REDACTED_MARKER} }}")
    );
}

#[test]
fn an_encode_ask_holds_at_most_the_largest_frozen_copy() {
    let top = usize::try_from(*SHOT_CAP_MB.end()).unwrap() * kx_ipc::MIB;
    assert_eq!(EncodeAsk::MAX_BYTES, top, "one source of truth");
    // The rows of a square-ish picture that fill the top of `shot_cap_mb`, and not one row more.
    let pixels = top / BGRX_BYTES;
    let side = pixels.isqrt();
    let (side, rows) = (
        u32::try_from(side).unwrap(),
        u32::try_from(pixels / side).unwrap(),
    );
    assert!(fits(side, rows) && !fits(side, rows + 1));
    assert!(!fits(u32::MAX, u32::MAX), "without overflow");
    let one = Shot::new(1, 1, vec![0; BGRX_BYTES]).unwrap();
    assert_eq!(EncodeAsk { shot: one }.check(), Ok(()));
}

#[test]
fn an_encode_ask_never_shows_its_pixels() {
    let bgrx = vec![201_u8, 202, 203, 204];
    let raw = format!("{bgrx:?}");
    let ask = EncodeAsk {
        shot: Shot::new(1, 1, bgrx).unwrap(),
    };
    let shown = format!("{ask:?}");
    assert!(shown.contains(REDACTED_MARKER), "{shown}");
    assert!(!shown.contains(raw.trim_matches(['[', ']'])), "{shown}");
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        PictureError::Refused,
        PictureError::Timeout,
        PictureError::Gone,
        PictureError::Down {
            retry_in: std::time::Duration::ZERO,
        },
        PictureError::TooBig,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn an_encodes_memory_covers_its_pixels_and_png_with_the_measured_peak() {
    // An 8192 by 8192 noise picture, 256 MiB of pixels, peaked at 455 MiB in the media job.
    let (side, peak_mb) = (8_192, 455);
    let mb = memory_mb(side, side);
    assert!(mb >= peak_mb, "{mb} MB is under the measured peak");
    let raw = u64::from(side) * u64::from(side) * u64::try_from(BGRX_BYTES).unwrap();
    let raw_mb = u32::try_from(raw / crate::ports::MB).unwrap();
    assert!(mb < 2 * raw_mb, "{mb} MB holds less than twice the pixels");
    assert!(
        memory_mb(1, 1) > 0,
        "a tiny picture still has the job's base"
    );
    assert_eq!(memory_mb(u32::MAX, u32::MAX), u32::MAX, "without overflow");
    let one = Shot::new(1, 1, vec![0; BGRX_BYTES]).unwrap();
    assert_eq!(EncodeAsk { shot: one }.memory_mb(), memory_mb(1, 1));
}

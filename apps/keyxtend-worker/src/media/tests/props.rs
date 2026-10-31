//! Properties: any bytes, a changed or cut PNG, and a DIB with any header fields never panic, and what comes out
//! always fits the asked side in an exact allocation; a declared size past the limit is always too big.

use crate::fixture::{BGRA, BI_BITFIELDS, BI_RGB, Dib, INFO, Png, V4, V5, le};
use crate::media::{Picture, draw};
use ::png::{BitDepth, ColorType};
use kx_ipc::{Codec, RGBA_BYTES, Why};
use proptest::prelude::*;
use proptest::sample::Index;

/// The longest random input a case reads.
const MAX_JUNK: usize = 512;
/// The longest side of a case's own images.
const MAX_CASE_SIDE: u32 = 12;
/// The most pixels a case allows.
const MAX_CASE_PIXELS: u64 = 4096;

/// Checks a draw's outcome: pixels within `side` in an exact allocation, or a clean refusal.
fn check(outcome: Result<Picture, Why>, side: u32) -> Result<(), TestCaseError> {
    match outcome {
        Ok(p) => {
            prop_assert!((1..=side).contains(&p.width) && (1..=side).contains(&p.height));
            let len = p.width as usize * p.height as usize * RGBA_BYTES;
            prop_assert_eq!(p.rgba.len(), len);
            prop_assert_eq!(p.rgba.capacity(), len);
        }
        Err(why) => prop_assert_ne!(why, Why::Internal),
    }
    Ok(())
}

fn a_codec() -> impl Strategy<Value = Codec> {
    prop_oneof![Just(Codec::Png), Just(Codec::Dib)]
}

/// A small PNG of any colour type, with matching random rows.
fn a_png() -> impl Strategy<Value = Vec<u8>> {
    let kinds = [
        (ColorType::Rgba, BitDepth::Eight, 32),
        (ColorType::Rgb, BitDepth::Sixteen, 48),
        (ColorType::Grayscale, BitDepth::One, 1),
        (ColorType::GrayscaleAlpha, BitDepth::Eight, 16),
        (ColorType::Indexed, BitDepth::Four, 4),
    ];
    let sides = (
        1..=MAX_CASE_SIDE,
        1..=MAX_CASE_SIDE,
        prop::sample::select(kinds.to_vec()),
    );
    sides.prop_flat_map(|(w, h, (color, depth, bits))| {
        let len = (w as usize * bits).div_ceil(8) * h as usize;
        prop::collection::vec(any::<u8>(), len).prop_map(move |raw| {
            let mut png = Png::new(w, h, color, depth);
            if color == ColorType::Indexed {
                png.palette = Some((0..48).collect());
            }
            png.encode(&raw)
        })
    })
}

/// A DIB whose header fields are each good, odd or extreme, with random bytes after it.
fn a_dib() -> impl Strategy<Value = Vec<u8>> {
    let size = prop_oneof![Just(INFO), Just(V4), Just(V5), any::<u32>()];
    let span = prop_oneof![-20..20_i32, any::<i32>()];
    let bits = prop::sample::select(&[24_u16, 32, 16, 8, 0][..]);
    let comp = prop_oneof![Just(BI_RGB), Just(BI_BITFIELDS), any::<u32>()];
    let colours = prop_oneof![0..4_u32, any::<u32>()];
    let mask = prop_oneof![prop::sample::select(&BGRA[..]), Just(0), any::<u32>()];
    let masks = [mask.clone(), mask.clone(), mask.clone(), mask];
    let tail = prop::collection::vec(any::<u8>(), 0..MAX_JUNK);
    let head = (size, span.clone(), span, bits, comp, colours);
    (head, masks, tail).prop_map(
        |((size, width, height, bits, compression, colours), masks, tail)| {
            let mut dib = Dib::new(width, height, bits, tail);
            (dib.size, dib.compression, dib.colours, dib.masks) =
                (size, compression, colours, masks);
            dib.after = le(&masks[..3]);
            dib.bytes()
        },
    )
}

proptest! {
    #[test]
    fn random_bytes_never_panic_in_either_reader(
        codec in a_codec(),
        bytes in prop::collection::vec(any::<u8>(), 0..MAX_JUNK),
        side in 1..=MAX_CASE_SIDE,
        max in 1..=MAX_CASE_PIXELS,
    ) {
        check(draw(codec, &bytes, side, max), side)?;
    }

    #[test]
    fn a_changed_or_cut_png_never_panics(
        png in a_png(),
        at in any::<Index>(),
        flip in 1..=u8::MAX,
        cut in any::<Index>(),
        side in 1..=MAX_CASE_SIDE,
    ) {
        check(draw(Codec::Png, &png, side, MAX_CASE_PIXELS), side)?;
        let mut changed = png.clone();
        changed[at.index(png.len())] ^= flip;
        check(draw(Codec::Png, &changed, side, MAX_CASE_PIXELS), side)?;
        check(draw(Codec::Png, &png[..cut.index(png.len())], side, MAX_CASE_PIXELS), side)?;
    }

    #[test]
    fn a_dib_with_any_header_fields_never_panics(
        dib in a_dib(),
        side in 1..=MAX_CASE_SIDE,
        max in prop_oneof![1..=MAX_CASE_PIXELS, Just(u64::MAX)],
    ) {
        check(draw(Codec::Dib, &dib, side, max), side)?;
    }

    #[test]
    fn a_declared_size_past_the_limit_is_always_too_big(
        png in a_png(),
        width in 1..=i32::MAX,
        height in prop_oneof![1..=i32::MAX, i32::MIN..=-1],
    ) {
        let declared = u64::from(width.unsigned_abs()) * u64::from(height.unsigned_abs());
        let max = declared - 1;
        let dib = Dib::new(width, height, 24, Vec::new()).bytes();
        prop_assume!(max > 0);
        prop_assert_eq!(draw(Codec::Dib, &dib, MAX_CASE_SIDE, max), Err(Why::TooBig));
        let (w, h) = png_size(&png);
        prop_assume!(w * h > 1);
        prop_assert_eq!(draw(Codec::Png, &png, MAX_CASE_SIDE, w * h - 1), Err(Why::TooBig));
    }
}

/// The width and height in a PNG's header.
fn png_size(png: &[u8]) -> (u64, u64) {
    let field = |at: usize| u64::from(u32::from_be_bytes(png[at..at + 4].try_into().unwrap()));
    (field(16), field(20))
}

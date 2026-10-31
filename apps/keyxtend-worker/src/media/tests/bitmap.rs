//! A PNG made into a `CF_DIB`: the exact bytes of the published layout, grey with alpha, the cap checked before any
//! row is decoded, the pixel limit, and damaged, cut or interlaced PNGs refused.

use super::png::IHDR_END;
use crate::fixture::{Png, rgba};
use crate::media::{draw, make_dib};
use ::png::{BitDepth, ColorType};
use kx_ipc::{Codec, Why};

/// The most pixels the tests allow.
const MAX: u64 = 1_000_000;
/// A cap well over any test's DIB.
const ROOMY: usize = 1 << 20;

fn png(width: u32, height: u32, color: ColorType) -> Png {
    Png::new(width, height, color, BitDepth::Eight)
}

/// A 2 by 2 RGBA PNG: top left, top right, bottom left, bottom right.
fn square() -> Vec<u8> {
    let pixels = [
        [1, 2, 3, 255],
        [4, 5, 6, 128],
        [7, 8, 9, 1],
        [10, 11, 12, 64],
    ];
    png(2, 2, ColorType::Rgba).encode(&rgba(&pixels))
}

#[test]
fn a_small_png_becomes_its_exact_dib() {
    let head = [
        &40u32.to_le_bytes()[..],
        &2i32.to_le_bytes(),
        &2i32.to_le_bytes(),
        &1u16.to_le_bytes(),
        &32u16.to_le_bytes(),
        &0u32.to_le_bytes(),
        &16u32.to_le_bytes(),
        &[0; 16],
    ]
    .concat();
    let bottom_up = [
        [9, 8, 7, 1],
        [12, 11, 10, 64],
        [3, 2, 1, 255],
        [6, 5, 4, 128],
    ];
    let want = [head, rgba(&bottom_up)].concat();
    assert_eq!(make_dib(&square(), ROOMY, MAX), Ok(want));
}

#[test]
fn grey_with_alpha_comes_out_as_equal_channels_and_its_alpha() {
    let bytes = png(1, 1, ColorType::GrayscaleAlpha).encode(&[100, 50]);
    let made = make_dib(&bytes, ROOMY, MAX).unwrap();
    assert_eq!(made[40..], [100, 100, 100, 50]);
}

#[test]
fn the_dib_reads_back_as_the_pngs_pixels() {
    let made = make_dib(&square(), ROOMY, MAX).unwrap();
    let pixels = [
        [1, 2, 3, 255],
        [4, 5, 6, 128],
        [7, 8, 9, 1],
        [10, 11, 12, 64],
    ];
    assert_eq!(
        draw(Codec::Dib, &made, 8, MAX).map(|p| p.rgba),
        Ok(rgba(&pixels))
    );
}

#[test]
fn a_dib_past_the_cap_is_too_big_and_one_at_it_is_made() {
    let exact = 40 + 2 * 2 * 4;
    assert_eq!(make_dib(&square(), exact - 1, MAX), Err(Why::TooBig));
    assert_eq!(make_dib(&square(), exact, MAX).map(|d| d.len()), Ok(exact));
}

/// A PNG cut right after its header has no rows to decode, so only a check before decoding can say `TooBig`.
#[test]
fn the_cap_is_checked_before_any_row_is_decoded() {
    let header_only = &square()[..IHDR_END];
    assert_eq!(make_dib(header_only, 40, MAX), Err(Why::TooBig));
    assert_eq!(make_dib(header_only, ROOMY, MAX), Err(Why::Refused));
}

#[test]
fn too_many_pixels_are_too_big() {
    assert_eq!(make_dib(&square(), ROOMY, 3), Err(Why::TooBig));
}

#[test]
fn a_bad_cut_or_interlaced_png_is_refused() {
    let whole = square();
    let mut interlaced = png(2, 2, ColorType::Rgba);
    interlaced.interlaced = true;
    let laced = interlaced.encode(&[0; 16]);
    for bytes in [&b"not a png"[..], &whole[..whole.len() - 20], &laced] {
        assert_eq!(make_dib(bytes, ROOMY, MAX), Err(Why::Refused));
    }
}

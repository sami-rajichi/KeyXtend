//! A snip's raw pixels made into a PNG: the media job reads it back to the same pixels whatever the stride, its buffer
//! and even noise stay within `png_most` of its RGB rows, and a picture past the pixel limit or a PNG past its cap is
//! too big.

use super::super::encoder::{COLOR, FILTER_BYTES};
use super::*;
use kx_ipc::{BGRX_BYTES, png_most};
use kx_test_support::noise::noise;
use proptest::prelude::*;
use std::borrow::Cow;

/// The most pixels the tests allow an encode, draw or DIB.
const MAX_PIXELS: u64 = 1 << 24;
/// What each pixel's unused byte and a row's padding hold, which no PNG keeps.
const JUNK: u8 = 0x77;
/// The side of a noise picture, 16 MiB of pixels: large enough that a stream deflate that cannot fall back to stored
/// blocks, such as `fdeflate`'s, passes its bound.
const NOISE_SIDE: u32 = 2048;
/// The longest side of a property case's picture, and the most padding after a row.
const MAX_CASE_SIDE: u32 = 24;
const MAX_PAD: u32 = 12;

/// `width` by `height` pixels, `pad` junk bytes after each row, whose blue, green and red come from `colour`.
fn raw(width: u32, height: u32, pad: u32, colour: impl Fn(u32, u32) -> [u8; 3]) -> (u32, Vec<u8>) {
    let stride = width * u32::try_from(BGRX_BYTES).unwrap() + pad;
    let mut bgrx = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let [b, g, r] = colour(x, y);
            bgrx.extend([b, g, r, JUNK]);
        }
        bgrx.extend(std::iter::repeat_n(JUNK, pad as usize));
    }
    (stride, bgrx)
}

/// The RGBA a draw gives for `bgrx`, read `width` pixels a row of `stride` bytes.
fn expected(width: u32, stride: u32, bgrx: &[u8]) -> Vec<u8> {
    let row = width as usize * BGRX_BYTES;
    let rows = bgrx.chunks_exact(stride as usize).map(|r| &r[..row]);
    let pixels = rows.flat_map(|r| r.as_chunks::<BGRX_BYTES>().0);
    pixels
        .flat_map(|&[blue, green, red, _]| [red, green, blue, u8::MAX])
        .collect()
}

/// The bytes of a `width` by `height` 8-bit RGB PNG's rows as deflate takes them: a filter byte, then RGB per pixel.
fn rgb_rows(width: u32, height: u32) -> usize {
    height as usize * (width as usize * COLOR.samples() + FILTER_BYTES)
}

/// The pixels a PNG of `width` by `height` reads back to, drawn at its own size.
fn read_back(png: &[u8], width: u32, height: u32) -> Vec<u8> {
    let drawn = draw(Codec::Png, png, width.max(height), MAX_PIXELS).unwrap();
    assert_eq!((drawn.width, drawn.height), (width, height));
    drawn.rgba
}

/// A draw within the tests' pixel limit.
fn plain_draw(codec: Codec, bytes: &[u8], side: u32) -> Result<Picture, Why> {
    draw(codec, bytes, side, MAX_PIXELS)
}

/// A DIB within the tests' pixel limit.
fn plain_dib(png: &[u8], max_bytes: usize) -> Result<Vec<u8>, Why> {
    make_dib(png, max_bytes, MAX_PIXELS)
}

/// A bright pattern with a step at each pixel.
fn pattern(x: u32, y: u32) -> [u8; 3] {
    let [a, b, ..] = (x * 37 + y * 11).to_le_bytes();
    [a, b, a ^ b]
}

#[test]
fn a_png_reads_back_to_the_same_pixels_dropping_the_unused_bytes() {
    let (width, height) = (5, 3);
    let (stride, bgrx) = raw(width, height, 8, pattern);
    let raw = Raw {
        width,
        height,
        stride,
        bgrx: &bgrx,
    };
    let png = encode(&raw, MAX_PIXELS).unwrap();
    assert_eq!(
        read_back(&png, width, height),
        expected(width, stride, &bgrx)
    );
}

#[test]
fn the_png_buffer_is_reserved_at_png_most_of_its_rgb_rows_not_of_the_raw_pixels() {
    let side = 64;
    let (stride, bgrx) = raw(side, side, 0, pattern);
    let raw = Raw {
        width: side,
        height: side,
        stride,
        bgrx: &bgrx,
    };
    let most = png_most(rgb_rows(side, side));
    assert_eq!(encoder::most(side, side), Some(most));
    assert_eq!(encode(&raw, MAX_PIXELS).unwrap().capacity(), most);
    assert_eq!(encoder::most(u32::MAX, u32::MAX), None, "without overflow");
}

#[test]
fn noise_stays_within_png_most_of_its_rgb_rows_and_reads_back() {
    let (stride, bgrx) = raw(NOISE_SIDE, NOISE_SIDE, 0, noise);
    let raw = Raw {
        width: NOISE_SIDE,
        height: NOISE_SIDE,
        stride,
        bgrx: &bgrx,
    };
    let png = encode(&raw, MAX_PIXELS).unwrap();
    let most = png_most(rgb_rows(NOISE_SIDE, NOISE_SIDE));
    assert!(png.len() <= most, "{} past {most}", png.len());
    let (back, want) = (
        read_back(&png, NOISE_SIDE, NOISE_SIDE),
        expected(NOISE_SIDE, stride, &bgrx),
    );
    let first_off = back.iter().zip(&want).position(|(got, was)| got != was);
    assert_eq!((back.len(), first_off), (want.len(), None));
}

#[test]
fn a_picture_past_the_most_pixels_or_a_png_past_its_cap_is_too_big() {
    let (stride, bgrx) = raw(4, 4, 0, pattern);
    let raw = Raw {
        width: 4,
        height: 4,
        stride,
        bgrx: &bgrx,
    };
    assert_eq!(encode(&raw, 15), Err(Why::TooBig));
    assert!(encode(&raw, 16).is_ok());
    assert_eq!(encoder::make(&raw, 10, MAX_PIXELS), Err(Why::TooBig));
}

#[test]
fn rows_shorter_than_their_stride_or_fewer_than_asked_are_refused() {
    let (stride, bgrx) = raw(4, 4, 0, pattern);
    let short = Raw {
        width: 4,
        height: 4,
        stride: stride - 1,
        bgrx: &bgrx,
    };
    assert_eq!(encode(&short, MAX_PIXELS), Err(Why::Refused));
    let cut = Raw {
        width: 4,
        height: 4,
        stride,
        bgrx: &bgrx[1..],
    };
    assert_eq!(encode(&cut, MAX_PIXELS), Err(Why::Refused));
}

#[test]
fn an_encode_ask_is_answered_with_its_png() {
    let (stride, bgrx) = raw(2, 2, 0, pattern);
    let ask = |id, height| Ask::Encode {
        id,
        width: 2,
        height,
        stride,
        bgrx: Cow::Borrowed(&bgrx[..(stride * height) as usize]),
    };
    let encoding = |raw: &Raw<'_>| encode(raw, 2);
    let answer = |ask| reply(ask, plain_draw, plain_dib, encoding);
    let Reply::Png { id: 7, bytes } = answer(ask(7, 1)) else {
        panic!("a PNG for ask 7");
    };
    assert_eq!(
        read_back(&bytes, 2, 1),
        expected(2, stride, &bgrx[..stride as usize])
    );
    let too_big = Reply::Failed {
        id: 8,
        why: Why::TooBig,
    };
    assert_eq!(answer(ask(8, 2)), too_big);
}

proptest! {
    #[test]
    fn any_picture_at_any_stride_reads_back_to_its_pixels(
        width in 1..=MAX_CASE_SIDE,
        height in 1..=MAX_CASE_SIDE,
        pad in 0..=MAX_PAD,
        seed in any::<u32>(),
    ) {
        let (stride, bgrx) = raw(width, height, pad, |x, y| noise(x ^ seed, y));
        let raw = Raw { width, height, stride, bgrx: &bgrx };
        let png = encode(&raw, MAX_PIXELS).unwrap();
        prop_assert!(png.len() <= png_most(rgb_rows(width, height)));
        prop_assert_eq!(read_back(&png, width, height), expected(width, stride, &bgrx));
    }
}

//! PNG: every colour type and depth comes out as RGBA8, a large image is read row by row, and damage is refused.

use crate::fixture::{Png, rgba};
use crate::media::png::draw;
use crate::media::{DECODE_BYTES, Picture};
use ::png::{BitDepth, ColorType};
use kx_ipc::{RGBA_BYTES, Why};

/// The side the tests draw within.
const SIDE: u32 = 64;
/// The most pixels the tests allow.
const MAX: u64 = 1_000_000;

fn png(width: u32, height: u32, color: ColorType, depth: BitDepth) -> Png {
    Png::new(width, height, color, depth)
}

fn pixels(bytes: &[u8]) -> Vec<u8> {
    draw(bytes, SIDE, MAX).unwrap().rgba
}

#[test]
fn rgba_decodes_to_its_pixels() {
    let want = [
        [1, 2, 3, 255],
        [4, 5, 6, 128],
        [7, 8, 9, 1],
        [200, 201, 202, 255],
    ];
    let bytes = png(2, 2, ColorType::Rgba, BitDepth::Eight).encode(&rgba(&want));
    let picture = draw(&bytes, SIDE, MAX).unwrap();
    let full = Picture {
        width: 2,
        height: 2,
        rgba: rgba(&want),
    };
    assert_eq!(picture, full);
}

#[test]
fn rgb_comes_out_opaque() {
    let bytes = png(2, 1, ColorType::Rgb, BitDepth::Eight).encode(&[1, 2, 3, 4, 5, 6]);
    assert_eq!(pixels(&bytes), rgba(&[[1, 2, 3, 255], [4, 5, 6, 255]]));
}

#[test]
fn grey_comes_out_as_equal_channels() {
    let bytes = png(3, 1, ColorType::Grayscale, BitDepth::Eight).encode(&[0, 128, 255]);
    let want = [[0, 0, 0, 255], [128, 128, 128, 255], [255, 255, 255, 255]];
    assert_eq!(pixels(&bytes), rgba(&want));
}

#[test]
fn low_bit_grey_is_stretched_to_eight_bits() {
    let bytes = png(4, 1, ColorType::Grayscale, BitDepth::Two).encode(&[0b00_01_10_11]);
    let grey = |v: u8| [v, v, v, 255];
    assert_eq!(
        pixels(&bytes),
        rgba(&[grey(0), grey(85), grey(170), grey(255)])
    );
}

#[test]
fn grey_with_alpha_keeps_its_alpha() {
    let bytes = png(2, 1, ColorType::GrayscaleAlpha, BitDepth::Eight).encode(&[50, 200, 100, 255]);
    assert_eq!(
        pixels(&bytes),
        rgba(&[[50, 50, 50, 200], [100, 100, 100, 255]])
    );
}

#[test]
fn a_palette_expands_with_its_transparency() {
    let mut header = png(3, 1, ColorType::Indexed, BitDepth::Eight);
    header.palette = Some(vec![255, 0, 0, 0, 255, 0, 0, 0, 255]);
    header.trns = Some(vec![255, 128]);
    let want = [[255, 0, 0, 255], [0, 255, 0, 128], [0, 0, 255, 255]];
    assert_eq!(pixels(&header.encode(&[0, 1, 2])), rgba(&want));
}

#[test]
fn sixteen_bits_keep_their_high_bytes() {
    let raw = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xFF, 0xFF];
    let bytes = png(1, 1, ColorType::Rgba, BitDepth::Sixteen).encode(&raw);
    assert_eq!(pixels(&bytes), vec![0x12, 0x56, 0x9A, 0xFF]);
}

#[test]
fn an_rgb_transparent_colour_becomes_clear() {
    let mut header = png(2, 1, ColorType::Rgb, BitDepth::Eight);
    header.trns = Some(vec![0, 10, 0, 20, 0, 30]);
    let bytes = header.encode(&[10, 20, 30, 40, 50, 60]);
    assert_eq!(pixels(&bytes), rgba(&[[0, 0, 0, 0], [40, 50, 60, 255]]));
}

#[test]
fn a_png_is_scaled_within_the_side() {
    let bytes = png(10, 4, ColorType::Rgb, BitDepth::Eight).encode(&[7; 10 * 4 * 3]);
    let picture = draw(&bytes, 5, MAX).unwrap();
    assert_eq!((picture.width, picture.height), (5, 2));
    assert!(picture.rgba.chunks(RGBA_BYTES).all(|p| p == [7, 7, 7, 255]));
}

#[test]
fn a_large_png_is_read_row_by_row_within_the_decoder_budget() {
    let (width, height) = (8000_u32, 6000_u32);
    let full = width as usize * height as usize * RGBA_BYTES;
    assert!(
        DECODE_BYTES < full / 8,
        "the budget must be far below a full frame"
    );
    let red = [255, 0, 0].repeat(width as usize);
    let blue = [0, 0, 255].repeat(width as usize);
    let header = png(width, height, ColorType::Rgb, BitDepth::Eight);
    let bytes = header.stream(|y| {
        if y < height / 2 {
            red.clone()
        } else {
            blue.clone()
        }
    });
    let picture = draw(&bytes, 128, u64::from(width) * u64::from(height)).unwrap();
    assert_eq!((picture.width, picture.height), (128, 96));
    let rows: Vec<&[u8]> = picture.rgba.chunks(128 * RGBA_BYTES).collect();
    assert!(
        rows[..48]
            .iter()
            .all(|r| r.chunks(RGBA_BYTES).all(|p| p == [255, 0, 0, 255]))
    );
    assert!(
        rows[48..]
            .iter()
            .all(|r| r.chunks(RGBA_BYTES).all(|p| p == [0, 0, 255, 255]))
    );
}

#[test]
fn too_many_pixels_are_refused_from_the_header_alone() {
    let bytes = png(1000, 1000, ColorType::Grayscale, BitDepth::Eight).encode(&vec![0; 1_000_000]);
    assert_eq!(draw(&bytes, SIDE, 999_999), Err(Why::TooBig));
    let header_only = &bytes[..IHDR_END];
    assert_eq!(draw(header_only, SIDE, 999_999), Err(Why::TooBig));
    assert_eq!(draw(header_only, SIDE, 1_000_000), Err(Why::Refused));
}

/// The signature and the IHDR chunk: 8 bytes, then length, type, 13 bytes of fields and the checksum.
const IHDR_END: usize = 8 + 4 + 4 + 13 + 4;

#[test]
fn rows_wider_than_the_decoder_budget_are_too_big() {
    let width = u32::try_from(DECODE_BYTES / RGBA_BYTES + 1).unwrap();
    let mut header = png(width, 1, ColorType::Indexed, BitDepth::One);
    header.palette = Some(vec![0, 0, 0, 255, 255, 255]);
    let bytes = header.encode(&vec![0; (width as usize).div_ceil(8)]);
    assert_eq!(draw(&bytes, SIDE, u64::from(width)), Err(Why::TooBig));
}

#[test]
fn an_interlaced_png_is_refused() {
    let mut header = png(2, 2, ColorType::Rgb, BitDepth::Eight);
    header.interlaced = true;
    assert_eq!(draw(&header.encode(&[0; 12]), SIDE, MAX), Err(Why::Refused));
}

#[test]
fn a_png_cut_before_its_last_row_is_refused() {
    let raw: Vec<u8> = (0..=255).cycle().take(16 * 16 * 3).collect();
    let bytes = png(16, 16, ColorType::Rgb, BitDepth::Eight).encode(&raw);
    let end = bytes.windows(4).position(|w| w == b"IEND").unwrap() - 4;
    for cut in 0..end {
        assert_eq!(
            draw(&bytes[..cut], SIDE, MAX),
            Err(Why::Refused),
            "cut at {cut}"
        );
    }
}

#[test]
fn bytes_that_are_not_a_png_are_refused() {
    for bytes in [&b""[..], b"\x89PNG", b"GIF89a", &[0; 64]] {
        assert_eq!(draw(bytes, SIDE, MAX), Err(Why::Refused), "{bytes:?}");
    }
}

#[test]
fn a_changed_byte_under_a_checksum_is_refused() {
    let mut bytes = png(2, 2, ColorType::Rgb, BitDepth::Eight).encode(&[9; 12]);
    bytes[IHDR_END - 1] ^= 1;
    assert_eq!(draw(&bytes, SIDE, MAX), Err(Why::Refused));
}

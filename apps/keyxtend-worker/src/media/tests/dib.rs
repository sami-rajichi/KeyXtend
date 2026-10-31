//! DIB: the three headers, 24- and 32-bit rows either way up, masks, zero alpha, and every refusal before a read.

use crate::fixture::{
    BGRA, BI_BITFIELDS, BI_JPEG, BI_PNG, BI_RGB, BI_RLE8, Dib, INFO, V4, V5, le, rgba,
};
use crate::media::dib::draw;
use kx_ipc::{RGBA_BYTES, Why};

/// The side the tests draw within.
const SIDE: u32 = 64;
/// The most pixels the tests allow.
const MAX: u64 = 1_000_000;
/// Masks of rows stored red, green, blue, alpha.
const RGBA_MASKS: [u32; 4] = [0x0000_00FF, 0x0000_FF00, 0x00FF_0000, 0xFF00_0000];

fn pixels(dib: &Dib) -> Result<Vec<u8>, Why> {
    draw(&dib.bytes(), SIDE, MAX).map(|p| p.rgba)
}

/// A 2 by 1 top-down 32-bit DIB in `masks`' byte order, with `rows` as stored.
fn masked(size: u32, masks: [u32; 4], rows: Vec<u8>) -> Dib {
    let mut dib = Dib::new(2, -1, 32, rows);
    dib.size = size;
    dib.compression = BI_BITFIELDS;
    dib.masks = masks;
    dib
}

#[test]
fn a_24_bit_bottom_up_dib_skips_its_padding_and_comes_out_top_first() {
    let pad = [0xEE; 3];
    let bottom = [[31, 21, 11], [61, 51, 41], [91, 81, 71]].concat();
    let top = [[30, 20, 10], [60, 50, 40], [90, 80, 70]].concat();
    let dib = Dib::new(3, 2, 24, [bottom, pad.to_vec(), top, pad.to_vec()].concat());
    let want = [
        [10, 20, 30, 255],
        [40, 50, 60, 255],
        [70, 80, 90, 255],
        [11, 21, 31, 255],
        [41, 51, 61, 255],
        [71, 81, 91, 255],
    ];
    assert_eq!(pixels(&dib), Ok(rgba(&want)));
}

#[test]
fn a_32_bit_top_down_dib_keeps_its_alpha() {
    let rows = [
        [3, 2, 1, 255],
        [6, 5, 4, 128],
        [9, 8, 7, 64],
        [12, 11, 10, 1],
    ]
    .concat();
    let dib = Dib::new(2, -2, 32, rows);
    let want = [
        [1, 2, 3, 255],
        [4, 5, 6, 128],
        [7, 8, 9, 64],
        [10, 11, 12, 1],
    ];
    assert_eq!(pixels(&dib), Ok(rgba(&want)));
}

#[test]
fn a_v5_dib_with_bitfields_reads_its_masks() {
    let dib = masked(V5, RGBA_MASKS, vec![1, 2, 3, 255, 4, 5, 6, 200]);
    assert_eq!(pixels(&dib), Ok(rgba(&[[1, 2, 3, 255], [4, 5, 6, 200]])));
}

#[test]
fn masks_repeated_after_a_v4_or_v5_header_are_skipped() {
    for size in [V4, V5] {
        let mut dib = masked(size, RGBA_MASKS, vec![1, 2, 3, 255, 4, 5, 6, 200]);
        dib.after = le(&RGBA_MASKS[..3]);
        assert_eq!(
            pixels(&dib),
            Ok(rgba(&[[1, 2, 3, 255], [4, 5, 6, 200]])),
            "{size}"
        );
    }
}

#[test]
fn a_plain_header_with_bitfields_takes_its_masks_after_it_and_no_alpha() {
    let mut dib = masked(INFO, [0; 4], vec![3, 2, 1, 0x77, 6, 5, 4, 0]);
    dib.after = le(&BGRA[..3]);
    assert_eq!(pixels(&dib), Ok(rgba(&[[1, 2, 3, 255], [4, 5, 6, 255]])));
}

#[test]
fn a_32_bit_dib_whose_alpha_is_all_zero_is_opaque() {
    let plain = Dib::new(2, 1, 32, vec![3, 2, 1, 0, 6, 5, 4, 0]);
    let v5 = masked(V5, BGRA, vec![3, 2, 1, 0, 6, 5, 4, 0]);
    for dib in [plain, v5] {
        assert_eq!(pixels(&dib), Ok(rgba(&[[1, 2, 3, 255], [4, 5, 6, 255]])));
    }
}

#[test]
fn one_alpha_above_zero_keeps_every_alpha() {
    let dib = Dib::new(2, 1, 32, vec![3, 2, 1, 255, 6, 5, 4, 0]);
    assert_eq!(pixels(&dib), Ok(rgba(&[[1, 2, 3, 255], [0, 0, 0, 0]])));
}

#[test]
fn colours_after_the_header_are_skipped() {
    let mut dib = Dib::new(1, 1, 24, vec![3, 2, 1, 0]);
    dib.colours = 2;
    dib.after = vec![0xAB; 8];
    assert_eq!(pixels(&dib), Ok(rgba(&[[1, 2, 3, 255]])));
}

#[test]
fn a_v4_header_reads_like_a_plain_one() {
    let mut dib = Dib::new(1, 1, 24, vec![3, 2, 1, 0]);
    dib.size = V4;
    assert_eq!(pixels(&dib), Ok(rgba(&[[1, 2, 3, 255]])));
}

#[test]
fn bytes_after_the_rows_are_ignored() {
    let dib = Dib::new(1, 1, 24, vec![3, 2, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(pixels(&dib), Ok(rgba(&[[1, 2, 3, 255]])));
}

#[test]
fn a_dib_is_scaled_within_the_side() {
    let dib = Dib::new(4, 4, 24, vec![9; 4 * 12]);
    let picture = draw(&dib.bytes(), 2, MAX).unwrap();
    assert_eq!((picture.width, picture.height), (2, 2));
    assert!(picture.rgba.chunks(RGBA_BYTES).all(|p| p == [9, 9, 9, 255]));
}

#[test]
fn other_header_sizes_are_refused() {
    for size in [0, 12, 39, 41, 52, 56, 64, 107, 125] {
        let mut dib = Dib::new(1, 1, 24, vec![0; 4]);
        dib.size = size;
        assert_eq!(pixels(&dib), Err(Why::Refused), "{size}");
    }
}

#[test]
fn a_header_past_the_buffer_is_refused() {
    let mut v5 = Dib::new(1, 1, 24, Vec::new());
    v5.size = V5;
    let bytes = v5.bytes();
    for cut in [0, 3, 4, 39, 40, 100, 123] {
        assert_eq!(draw(&bytes[..cut], SIDE, MAX), Err(Why::Refused), "{cut}");
    }
}

#[test]
fn an_empty_or_negative_size_is_refused() {
    for (width, height) in [(0, 1), (1, 0), (-1, 1), (i32::MIN, 1)] {
        let dib = Dib::new(width, height, 24, vec![0; 64]);
        assert_eq!(pixels(&dib), Err(Why::Refused), "{width} x {height}");
    }
}

#[test]
fn planes_other_than_one_are_refused() {
    for planes in [0, 2, u16::MAX] {
        let mut dib = Dib::new(1, 1, 24, vec![0; 4]);
        dib.planes = planes;
        assert_eq!(pixels(&dib), Err(Why::Refused), "{planes}");
    }
}

#[test]
fn other_depths_and_compressions_are_refused() {
    let cases = [
        (1, BI_RGB),
        (4, BI_RGB),
        (8, BI_RGB),
        (16, BI_RGB),
        (48, BI_RGB),
        (16, BI_BITFIELDS),
        (24, BI_BITFIELDS),
        (8, BI_RLE8),
        (32, BI_RLE8),
        (32, BI_JPEG),
        (32, BI_PNG),
        (32, 6),
    ];
    for (bits, compression) in cases {
        let mut dib = Dib::new(1, 1, bits, vec![0; 64]);
        dib.compression = compression;
        dib.after = le(&BGRA[..3]);
        assert_eq!(
            pixels(&dib),
            Err(Why::Refused),
            "{bits} bits, {compression}"
        );
    }
}

#[test]
fn masks_that_are_not_distinct_whole_bytes_are_refused() {
    let cases = [
        [0x00FF_F000, 0x0000_0FF0, 0x0000_000F, 0],
        [0x0000_00FF, 0x0000_00FF, 0x00FF_0000, 0],
        [0, 0x0000_FF00, 0x0000_00FF, 0],
        [0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0x0F00_0000],
        [0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0x00FF_0000],
    ];
    for masks in cases {
        assert_eq!(
            pixels(&masked(V5, masks, vec![0; 8])),
            Err(Why::Refused),
            "{masks:x?}"
        );
    }
}

#[test]
fn rows_past_the_buffer_are_refused() {
    let short = Dib::new(3, 2, 24, vec![0; 23]);
    assert_eq!(pixels(&short), Err(Why::Refused));
    let mut far = Dib::new(1, 1, 24, vec![0; 4]);
    far.colours = 1000;
    assert_eq!(pixels(&far), Err(Why::Refused));
}

#[test]
fn huge_fields_never_overflow() {
    let mut colours = Dib::new(1, 1, 24, vec![0; 4]);
    colours.colours = u32::MAX;
    assert_eq!(pixels(&colours), Err(Why::Refused));
    let huge = Dib::new(i32::MAX, i32::MIN, 32, vec![0; 64]);
    assert_eq!(draw(&huge.bytes(), SIDE, u64::MAX), Err(Why::Refused));
}

#[test]
fn too_many_pixels_are_too_big_before_the_rest_is_read() {
    let dib = Dib::new(1000, 1000, 24, Vec::new());
    assert_eq!(draw(&dib.bytes(), SIDE, 999_999), Err(Why::TooBig));
    assert_eq!(draw(&dib.bytes(), SIDE, 1_000_000), Err(Why::Refused));
    let huge = Dib::new(i32::MAX, i32::MIN, 13, Vec::new());
    assert_eq!(draw(&huge.bytes(), SIDE, MAX), Err(Why::TooBig));
}

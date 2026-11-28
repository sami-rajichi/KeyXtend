//! The `CF_DIB` a Copy puts beside the PNG: a 40-byte header, 24-bit rows from the bottom, each padded to four bytes,
//! and each pixel's unused byte dropped.

#![allow(
    clippy::unwrap_used,
    reason = "a test fails on a picture it cannot build"
)]

use super::*;

/// The little-endian `u32` at `at`.
fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// The little-endian `u16` at `at`.
fn half(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}

#[test]
fn the_header_names_the_size_the_planes_the_bits_and_the_rows() {
    let shot = Shot::new(3, 2, vec![7; 3 * 2 * BGRX_BYTES]).unwrap();
    let dib = of(&shot);
    let bytes = dib.expose();
    // Three 24-bit pixels take 9 bytes, padded to 12.
    let image = 12 * 2;
    assert_eq!(bytes.len(), 40 + image);
    assert_eq!(word(bytes, 0), 40, "BITMAPINFOHEADER");
    assert_eq!(
        (word(bytes, 4), word(bytes, 8)),
        (3, 2),
        "rows from the bottom"
    );
    assert_eq!((half(bytes, 12), half(bytes, 14)), (1, 24));
    assert_eq!(word(bytes, 16), 0, "BI_RGB");
    assert_eq!(word(bytes, 20), u32::try_from(image).unwrap());
    assert!(bytes[24..40].iter().all(|b| *b == 0));
}

#[test]
fn rows_go_bottom_up_without_the_unused_byte_and_padded_with_zeros() {
    // Top row: pixels (1,2,3,x) and (4,5,6,x); bottom row: (7,8,9,x) and (10,11,12,x).
    let bgrx = vec![1, 2, 3, 99, 4, 5, 6, 99, 7, 8, 9, 99, 10, 11, 12, 99];
    let shot = Shot::new(2, 2, bgrx).unwrap();
    let dib = of(&shot);
    let rows = &dib.expose()[40..];
    assert_eq!(rows, [7, 8, 9, 10, 11, 12, 0, 0, 1, 2, 3, 4, 5, 6, 0, 0]);
}

#[test]
fn a_row_already_on_four_bytes_takes_no_padding_and_the_bytes_never_print() {
    let shot = Shot::new(4, 1, vec![201; 4 * BGRX_BYTES]).unwrap();
    let dib = of(&shot);
    assert_eq!(dib.expose().len(), 40 + 12);
    assert!(!format!("{dib:?}").contains("201"));
}

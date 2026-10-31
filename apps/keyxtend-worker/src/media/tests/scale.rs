//! The box filter: output sizes, box means by alpha, row checks, and a large image held one row of sums at a time.

use crate::fixture::rgba;
use crate::media::Picture;
use crate::media::scale::{Rgba, Scaler, fit};
use kx_ipc::{RGBA_BYTES, Why};
use std::iter::repeat_n;

const RED: Rgba = [255, 0, 0, 255];
const BLUE: Rgba = [0, 0, 255, 255];

/// Shrinks `pixels`, `width` to a row, within `side`.
fn shrink(width: u32, height: u32, side: u32, pixels: &[Rgba]) -> Result<Picture, Why> {
    let mut scaler = Scaler::new(width, height, side)?;
    for row in pixels.chunks(width as usize) {
        scaler.row(row.iter().copied())?;
    }
    scaler.finish()
}

#[test]
fn the_longest_side_becomes_the_side_and_the_other_keeps_its_ratio() {
    assert_eq!(fit(8000, 6000, 128), (128, 96));
    assert_eq!(fit(6000, 8000, 128), (96, 128));
    assert_eq!(fit(300, 300, 128), (128, 128));
    assert_eq!(fit(1000, 1, 128), (128, 1));
    assert_eq!(fit(1, 1000, 128), (1, 128));
    assert_eq!(fit(3, 1000, 2), (1, 2));
    assert_eq!(fit(u32::MAX, 3, 4096), (4096, 1));
}

#[test]
fn no_side_is_ever_past_the_protocol_s_largest() {
    let max = kx_ipc::MAX_SIDE;
    assert_eq!(fit(5000, 2500, max + 1), (max, max / 2));
    assert_eq!(fit(u32::MAX, u32::MAX, u32::MAX), (max, max));
    assert_eq!(fit(max, 1, u32::MAX), (max, 1));
    let scaler = Scaler::new(10_000, 1, u32::MAX).unwrap();
    assert!(scaler.held() <= max as usize * (RGBA_BYTES + RGBA_BYTES * size_of::<u64>()));
}

#[test]
fn a_small_image_is_never_scaled_up() {
    assert_eq!(fit(100, 50, 128), (100, 50));
    assert_eq!(fit(128, 128, 128), (128, 128));
    assert_eq!(fit(1, 1, 4096), (1, 1));
}

#[test]
fn an_image_within_the_side_comes_out_unchanged() {
    let pixels = [
        [1, 2, 3, 255],
        [4, 5, 6, 200],
        [7, 8, 9, 1],
        [250, 251, 252, 255],
    ];
    let picture = shrink(2, 2, 2, &pixels).unwrap();
    assert_eq!((picture.width, picture.height), (2, 2));
    assert_eq!(picture.rgba, rgba(&pixels));
}

#[test]
fn each_output_pixel_is_the_rounded_mean_of_its_box() {
    let pixels = [
        [10, 20, 30, 255],
        [20, 30, 40, 255],
        [100, 0, 0, 255],
        [101, 0, 0, 255],
        [30, 40, 50, 255],
        [40, 50, 61, 255],
        [0, 0, 0, 255],
        [1, 0, 1, 255],
    ];
    let picture = shrink(4, 2, 2, &pixels).unwrap();
    assert_eq!((picture.width, picture.height), (2, 1));
    // Box 1: red 202 / 4 = 50.5 rounds up, blue 1 / 4 = 0.25 rounds down.
    assert_eq!(picture.rgba, rgba(&[[25, 35, 45, 255], [51, 0, 0, 255]]));
}

#[test]
fn uneven_boxes_cover_every_input_pixel_once() {
    let grey = |v: u8| [v, v, v, 255];
    let three = shrink(3, 1, 2, &[grey(10), grey(20), grey(30)]).unwrap();
    assert_eq!(three.rgba, rgba(&[grey(15), grey(30)]));
    let five: Vec<Rgba> = [0, 10, 20, 30, 40].map(grey).to_vec();
    let five = shrink(5, 1, 3, &five).unwrap();
    assert_eq!(five.rgba, rgba(&[grey(5), grey(25), grey(40)]));
}

#[test]
fn clear_pixels_do_not_darken_their_neighbours() {
    let picture = shrink(2, 1, 1, &[RED, [0, 0, 0, 0]]).unwrap();
    assert_eq!(picture.rgba, rgba(&[[255, 0, 0, 128]]));
}

#[test]
fn a_fully_clear_box_is_clear_black() {
    let picture = shrink(2, 1, 1, &[[10, 20, 30, 0], [40, 50, 60, 0]]).unwrap();
    assert_eq!(picture.rgba, rgba(&[[0, 0, 0, 0]]));
}

#[test]
fn a_row_of_the_wrong_length_is_refused() {
    let mut scaler = Scaler::new(3, 1, 8).unwrap();
    assert_eq!(scaler.row([RED; 2]), Err(Why::Refused));
    let mut scaler = Scaler::new(3, 1, 8).unwrap();
    assert_eq!(scaler.row([RED; 4]), Err(Why::Refused));
}

#[test]
fn rows_past_the_height_are_refused() {
    let mut scaler = Scaler::new(1, 1, 8).unwrap();
    assert_eq!(scaler.row([RED]), Ok(()));
    assert_eq!(scaler.row([RED]), Err(Why::Refused));
}

#[test]
fn missing_rows_are_refused_at_the_end() {
    let mut scaler = Scaler::new(1, 3, 2).unwrap();
    scaler.row([RED]).unwrap();
    assert_eq!(scaler.finish(), Err(Why::Refused));
    assert_eq!(Scaler::new(1, 3, 2).unwrap().finish(), Err(Why::Refused));
}

#[test]
fn an_empty_image_is_refused() {
    assert!(matches!(Scaler::new(0, 5, 8), Err(Why::Refused)));
    assert!(matches!(Scaler::new(5, 0, 8), Err(Why::Refused)));
}

#[test]
fn a_large_image_scales_holding_only_the_output_and_one_row_of_sums() {
    let (width, height) = (8000, 6000);
    let (out_w, out_h) = (128, 96);
    let budget = out_w * out_h * RGBA_BYTES + out_w * RGBA_BYTES * size_of::<u64>();
    let mut scaler = Scaler::new(width, height, 128).unwrap();
    assert!(scaler.held() <= budget, "{} > {budget}", scaler.held());
    for y in 0..height {
        let colour = if y < height / 2 { RED } else { BLUE };
        scaler.row(repeat_n(colour, width as usize)).unwrap();
    }
    assert!(scaler.held() <= budget, "{} > {budget}", scaler.held());
    let picture = scaler.finish().unwrap();
    assert_eq!((picture.width, picture.height), (128, 96));
    let rows: Vec<&[u8]> = picture.rgba.chunks(out_w * RGBA_BYTES).collect();
    assert_eq!(rows.len(), out_h);
    for (y, row) in rows.iter().enumerate() {
        let colour = if y < out_h / 2 { RED } else { BLUE };
        assert!(row.chunks(RGBA_BYTES).all(|p| p == colour), "row {y}");
    }
}

//! The media job: the size check and the codec choice here; each reader, the scaler and the properties have files.

mod dib;
mod png;
mod props;
mod scale;

use super::*;
use crate::fixture::{Dib, Png, rgba};
use ::png::{BitDepth, ColorType};

#[test]
fn an_empty_image_is_refused() {
    assert_eq!(check_size(0, 5, 100), Err(Why::Refused));
    assert_eq!(check_size(5, 0, 100), Err(Why::Refused));
}

#[test]
fn more_pixels_than_the_limit_are_too_big() {
    assert_eq!(check_size(10, 10, 100), Ok(()));
    assert_eq!(check_size(10, 10, 99), Err(Why::TooBig));
    assert_eq!(check_size(u32::MAX, u32::MAX, u64::MAX), Ok(()));
    assert_eq!(
        check_size(u32::MAX, u32::MAX, u64::MAX / 2),
        Err(Why::TooBig)
    );
}

#[test]
fn each_codec_goes_to_its_reader() {
    let pixel = [10, 20, 30, 255];
    let png = Png::new(1, 1, ColorType::Rgba, BitDepth::Eight).encode(&pixel);
    let dib = Dib::new(1, 1, 24, vec![30, 20, 10, 0]).bytes();
    let want = Picture {
        width: 1,
        height: 1,
        rgba: rgba(&[pixel]),
    };
    assert_eq!(draw(Codec::Png, &png, 8, 100), Ok(want));
    assert_eq!(
        draw(Codec::Dib, &dib, 8, 100).map(|p| p.rgba),
        Ok(pixel.to_vec())
    );
    assert_eq!(draw(Codec::Png, &dib, 8, 100), Err(Why::Refused));
    assert_eq!(draw(Codec::Dib, &png, 8, 100), Err(Why::Refused));
}

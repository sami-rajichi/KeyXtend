//! Rectangles and points in physical pixels, shared by the face and the UI ports.
//! The UI sends reals, so a fraction rounds to the nearest pixel when read.

use serde::de::{self, Deserializer, Unexpected, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// What a pixel reader expects, for serde's error text.
const EXPECTING: &str = "a pixel count within the i32 range";

/// The DPI at which one logical pixel is one physical pixel (100 %).
pub const BASE_DPI: u32 = 96;

/// `px` logical pixels at `dpi`, in physical pixels; not-a-number gives 0 and a size past the range stops at its end.
#[must_use]
pub fn physical(px: f64, dpi: u32) -> i32 {
    let scaled = (px * f64::from(dpi) / f64::from(BASE_DPI)).round();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "`as` saturates at the ends of the range and gives 0 for NaN"
    )]
    let px = scaled as i32;
    px
}

/// A rectangle in physical pixels.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    /// Left edge.
    #[serde(deserialize_with = "px")]
    pub x: i32,
    /// Top edge.
    #[serde(deserialize_with = "px")]
    pub y: i32,
    /// Width.
    #[serde(deserialize_with = "px")]
    pub w: i32,
    /// Height.
    #[serde(deserialize_with = "px")]
    pub h: i32,
}

/// A point in physical pixels.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    /// Distance from the left.
    #[serde(deserialize_with = "px")]
    pub x: i32,
    /// Distance from the top.
    #[serde(deserialize_with = "px")]
    pub y: i32,
}

/// One screen's usable area, its DPI and whether it is the primary screen.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screen {
    /// The work area, without the taskbar, in physical pixels.
    pub rect: Rect,
    /// Dots per inch; `BASE_DPI` is 100 %.
    pub dpi: u32,
    /// The primary screen.
    pub primary: bool,
}

/// Reads a pixel from any number, rounding a fraction to the nearest pixel.
pub(super) fn px<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    deserializer.deserialize_any(PxVisitor)
}

/// Turns a JSON or TOML number into a pixel, refusing values outside `i32`.
struct PxVisitor;

impl Visitor<'_> for PxVisitor {
    type Value = i32;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(EXPECTING)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<i32, E> {
        i32::try_from(v).map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<i32, E> {
        i32::try_from(v).map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &self))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<i32, E> {
        let whole = v.round();
        let range = f64::from(i32::MIN)..=f64::from(i32::MAX);
        if !range.contains(&whole) {
            return Err(E::invalid_value(Unexpected::Float(v), &self));
        }
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the value is whole and inside the i32 range"
        )]
        Ok(whole as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_pixels_scale_with_the_dpi() {
        assert_eq!((physical(72.0, 96), physical(72.0, 120)), (72, 90));
        assert_eq!(physical(33.0, 144), 50, "49.5 rounds up");
        assert_eq!(
            (physical(f64::NAN, 144), physical(f64::INFINITY, 96)),
            (0, i32::MAX)
        );
        assert_eq!((physical(-10.0, 192), physical(10.0, 0)), (-20, 0));
    }

    #[test]
    fn a_screen_carries_its_dpi_and_whether_it_is_primary() {
        let text = r#"{"rect":{"x":1920,"y":0,"w":2560,"h":1400},"dpi":144,"primary":false}"#;
        let s: Screen = serde_json::from_str(text).unwrap();
        let rect = Rect {
            x: 1920,
            y: 0,
            w: 2560,
            h: 1400,
        };
        let want = Screen {
            rect,
            dpi: 144,
            primary: false,
        };
        assert_eq!(s, want);
        assert_eq!(serde_json::to_string(&s).unwrap(), text);
        let spare = r#"{"rect":{"x":0,"y":0,"w":1,"h":1},"dpi":96,"primary":true,"scale":1}"#;
        assert!(serde_json::from_str::<Screen>(spare).is_err());
    }

    #[test]
    fn a_fraction_rounds_to_the_nearest_pixel() {
        let p: Point = serde_json::from_str(r#"{"x":10.5,"y":-3.4}"#).unwrap();
        assert_eq!(p, Point { x: 11, y: -3 });
        let r: Rect = serde_json::from_str(r#"{"x":0.4,"y":1,"w":899.6,"h":300}"#).unwrap();
        let want = Rect {
            x: 0,
            y: 1,
            w: 900,
            h: 300,
        };
        assert_eq!(r, want);
    }

    #[test]
    fn a_value_outside_the_pixel_range_is_refused() {
        let bad = [
            r#"{"x":1e12,"y":0}"#,
            r#"{"x":-3000000000,"y":0}"#,
            r#"{"x":3000000000,"y":0}"#,
            r#"{"x":"1","y":0}"#,
        ];
        for text in bad {
            assert!(serde_json::from_str::<Point>(text).is_err(), "{text}");
        }
    }

    #[test]
    fn pixels_are_written_as_integers_and_read_from_toml() {
        let json = serde_json::to_string(&Point { x: -8, y: 40 }).unwrap();
        assert_eq!(json, r#"{"x":-8,"y":40}"#);
        let r: Rect = toml::from_str("x = 1\ny = 2.5\nw = 3\nh = 4").unwrap();
        let want = Rect {
            x: 1,
            y: 3,
            w: 3,
            h: 4,
        };
        assert_eq!(r, want);
    }
}

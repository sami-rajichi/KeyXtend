//! `g8 clip`'s copies: the plan of texts, pictures and short tail copies, each text and picture made in memory, and
//! their clipboard writes, which keep out of the cloud as every gate copy does.
#![cfg(windows)]

use kx_module_api::ports::{ClipError, ClipWait};
use kx_platform_windows::clip::{self, open};
use serde::Deserialize;
use windows::Win32::Graphics::Gdi::{BI_RGB, BITMAPINFOHEADER};
use windows::Win32::System::Ole::CF_DIB;

use crate::p8cfg::G8Clip;

/// The product's clipboard table (kx-platform-windows), for the cloud switch every gate copy turns off.
const TABLE: &str = include_str!("../../../crates/kx-platform-windows/src/clip.toml");
/// The 32-bit number that turns a Windows switch format off.
pub(crate) const OFF: [u8; 4] = 0u32.to_ne_bytes();
/// A DIB's colour planes, which Windows requires to be one.
const PLANES: u16 = 1;
/// Bits per pixel of the gate's pictures: blue, green, red and alpha.
const BITS: u16 = 32;
/// Bytes per pixel at those bits.
const PIXEL: u32 = 4;
/// The alpha of every pixel: opaque.
const OPAQUE: u8 = u8::MAX;
/// What parts two copies of a long text.
const GAP: char = ' ';

/// What one copy of the fill is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// A text of this many characters.
    Text(usize),
    /// A picture of this side, in px.
    Image(u32),
    /// The prefix and the copy's number alone, which the clean-up reads.
    Short,
}

/// The only key `clip.toml` is read for here.
#[derive(Deserialize)]
struct Table {
    /// The cloud switch every gate copy sets to 0.
    cloud_off: String,
}

/// The cloud switch's format name in the product's table.
pub(crate) fn cloud_off() -> Result<String, String> {
    let t: Table = toml::from_str(TABLE).map_err(|e| format!("the clipboard table: {e}"))?;
    Ok(t.cloud_off)
}

/// Each copy of `g`'s fill in order: every `image_every`-th a picture, the sides in turn, the others texts of the
/// lengths in turn, and the last `forget_tail` short.
pub fn plan(g: &G8Clip) -> Vec<Fill> {
    let long = g.entries.saturating_sub(g.forget_tail);
    let (mut pictures, mut texts) = (g.image_px.iter().cycle(), g.text_chars.iter().cycle());
    let fill = |i: usize| {
        if i >= long {
            Some(Fill::Short)
        } else if (i + 1).is_multiple_of(g.image_every) {
            pictures.next().map(|px| Fill::Image(*px))
        } else {
            texts.next().map(|n| Fill::Text(*n))
        }
    };
    (0..g.entries)
        .map(fill)
        .collect::<Option<_>>()
        .unwrap_or_default()
}

/// The unit a long text repeats: the prefix and the copy's number, then a gap.
fn unit(prefix: &str, n: usize) -> String {
    format!("{prefix}{n}{GAP}")
}

/// Copy `n`'s text: `prefix` and `n` alone for none, else that unit repeated and cut to `chars` characters.
pub fn text(prefix: &str, n: usize, chars: Option<usize>) -> String {
    match chars {
        None => format!("{prefix}{n}"),
        Some(chars) => unit(prefix, n).chars().cycle().take(chars).collect(),
    }
}

/// The bytes of copy `n`'s text of `chars` characters, worked out without making it.
pub fn text_bytes(prefix: &str, n: usize, chars: usize) -> u64 {
    let unit = unit(prefix, n);
    let count = unit.chars().count().max(1);
    let rest: usize = unit.chars().take(chars % count).map(char::len_utf8).sum();
    let whole = (chars / count) as u64;
    whole
        .saturating_mul(unit.len() as u64)
        .saturating_add(rest as u64)
}

/// The fewest characters a long text needs to hold its prefix and the largest of `entries` numbers.
pub fn least_chars(prefix: &str, entries: usize) -> usize {
    prefix.chars().count() + entries.to_string().len()
}

/// The bytes of a square picture of side `side`: its header and its pixels.
pub fn dib_len(side: u32) -> u64 {
    let pixels = u64::from(side) * u64::from(side) * u64::from(PIXEL);
    size_of::<BITMAPINFOHEADER>() as u64 + pixels
}

/// Copy `n`'s picture: a bottom-up 32-bit DIB of side `side`, whose blue and green hold `n` and red the pixel's place.
pub fn dib(side: u32, n: usize) -> Vec<u8> {
    let image = side.saturating_mul(side).saturating_mul(PIXEL);
    let wide = i32::try_from(side).unwrap_or(i32::MAX);
    let mut out = Vec::with_capacity(usize::try_from(dib_len(side)).unwrap_or_default());
    let header = u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap_or_default();
    out.extend(header.to_le_bytes());
    out.extend(wide.to_le_bytes());
    out.extend(wide.to_le_bytes());
    out.extend(PLANES.to_le_bytes());
    out.extend(BITS.to_le_bytes());
    out.extend(BI_RGB.0.to_le_bytes());
    out.extend(image.to_le_bytes());
    out.resize(size_of::<BITMAPINFOHEADER>(), 0);
    let [b, g, ..] = n.to_le_bytes();
    let count = image / PIXEL;
    out.extend((0..count).flat_map(|p| [b, g, p.to_le_bytes()[0], OPAQUE]));
    out
}

/// Puts picture `bytes` on the clipboard as one change, with the cloud switch off.
fn write_image(bytes: &[u8], wait: ClipWait) -> Result<(), String> {
    let cloud = cloud_off()?;
    let put = || -> Result<(), ClipError> {
        let open = open::open(wait)?;
        open::empty(&open)?;
        open::put(&open, u32::from(CF_DIB.0), bytes)?;
        open::put(&open, open::format_id(&cloud)?, &OFF)
    };
    put().map_err(|e| e.to_string())
}

/// Writes copy `n` of `g`'s fill, made as `fill` says.
pub fn copy(g: &G8Clip, n: usize, fill: Fill, wait: ClipWait) -> Result<(), String> {
    let written = match fill {
        Fill::Image(px) => return write_image(&dib(px, n), wait),
        Fill::Text(chars) => clip::write_text(Some(&text(&g.prefix, n, Some(chars))), &[], wait),
        Fill::Short => clip::write_text(Some(&text(&g.prefix, n, None)), &[], wait),
    };
    written.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small fill: 20 copies, a picture every 4th, the last 5 short.
    fn small() -> G8Clip {
        let mut g = crate::config::load().expect("loads").g8_clip;
        g.entries = 20;
        g.image_every = 4;
        g.forget_tail = 5;
        g.image_px = vec![8, 16];
        g.text_chars = vec![30, 50];
        g
    }

    #[test]
    fn the_plan_puts_pictures_in_turn_and_never_in_the_short_tail() {
        let got = plan(&small());
        let want = [
            Fill::Text(30),
            Fill::Text(50),
            Fill::Text(30),
            Fill::Image(8),
            Fill::Text(50),
            Fill::Text(30),
            Fill::Text(50),
            Fill::Image(16),
            Fill::Text(30),
            Fill::Text(50),
            Fill::Text(30),
            Fill::Image(8),
            Fill::Text(50),
            Fill::Text(30),
            Fill::Text(50),
            Fill::Short,
            Fill::Short,
            Fill::Short,
            Fill::Short,
            Fill::Short,
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn the_shipped_plan_fills_every_entry_with_short_copies_last() {
        let g = crate::config::load().expect("loads").g8_clip;
        let got = plan(&g);
        assert_eq!(got.len(), g.entries);
        let tail = &got[g.entries - g.forget_tail..];
        assert!(tail.iter().all(|f| *f == Fill::Short));
        let pictures = got.iter().filter(|f| matches!(f, Fill::Image(_))).count();
        assert!(pictures > 0, "some pictures");
    }

    #[test]
    fn each_text_holds_its_number_has_its_length_and_differs() {
        assert_eq!(text("kx-", 7, None), "kx-7");
        let long = text("kx-", 7, Some(30));
        assert_eq!(long.chars().count(), 30);
        assert!(long.starts_with("kx-7 kx-7 "), "{long}");
        assert_ne!(text("kx-", 7, Some(30)), text("kx-", 8, Some(30)));
        assert_eq!(text_bytes("kx-", 7, 30), 30);
        assert_eq!(text_bytes("é-", 3, 7), text("é-", 3, Some(7)).len() as u64);
        assert_eq!(least_chars("kx-g8clip-", 1000), 14);
        assert_eq!(least_chars("kx-", 9), 4);
    }

    /// The little-endian number at `at` in `bytes`, `N` bytes wide.
    fn le<const N: usize>(bytes: &[u8], at: usize) -> [u8; N] {
        bytes[at..at + N].try_into().expect("in range")
    }

    #[test]
    fn a_picture_is_a_32_bit_dib_of_its_side_that_differs_by_number() {
        let bytes = dib(8, 3);
        assert_eq!(bytes.len() as u64, dib_len(8));
        assert_eq!(dib_len(8), 40 + 8 * 8 * 4);
        assert_eq!(u32::from_le_bytes(le(&bytes, 0)), 40, "header size");
        assert_eq!(i32::from_le_bytes(le(&bytes, 4)), 8, "width");
        assert_eq!(i32::from_le_bytes(le(&bytes, 8)), 8, "height");
        assert_eq!(u16::from_le_bytes(le(&bytes, 12)), 1, "planes");
        assert_eq!(u16::from_le_bytes(le(&bytes, 14)), 32, "bits");
        assert_eq!(u32::from_le_bytes(le(&bytes, 16)), 0, "uncompressed");
        assert_eq!(u32::from_le_bytes(le(&bytes, 20)), 8 * 8 * 4, "image size");
        assert_eq!(dib(8, 3), bytes, "the same number makes the same picture");
        assert_ne!(dib(8, 4), bytes, "another number makes another");
    }

    #[test]
    fn the_cloud_switch_comes_from_the_products_table_and_marks_nothing() {
        let cloud = cloud_off().expect("the table parses");
        assert!(!cloud.is_empty());
        let marks = kx_platform_windows::clip::markers();
        assert!(!marks.is_empty() && !marks.contains(&cloud), "{cloud}");
    }
}

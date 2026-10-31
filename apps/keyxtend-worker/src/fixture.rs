//! Test images: PNGs made with the `png` crate's encoder, and DIBs built by hand from the format's published layout.

use ::png::{BitDepth, ColorType, Compression, Encoder, Info};
use kx_ipc::RGBA_BYTES;

/// `BITMAPINFOHEADER`'s size.
pub const INFO: u32 = 40;
/// `BITMAPV4HEADER`'s size.
pub const V4: u32 = 108;
/// `BITMAPV5HEADER`'s size.
pub const V5: u32 = 124;
/// Plain rows.
pub const BI_RGB: u32 = 0;
/// Run-length coded 8-bit rows.
pub const BI_RLE8: u32 = 1;
/// Rows with bit masks.
pub const BI_BITFIELDS: u32 = 3;
/// A JPEG inside the DIB.
pub const BI_JPEG: u32 = 4;
/// A PNG inside the DIB.
pub const BI_PNG: u32 = 5;
/// Red, green, blue and alpha masks of rows stored blue, green, red, alpha: Windows' usual order.
pub const BGRA: [u32; 4] = [0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0xFF00_0000];
/// Where the masks sit in a V4 or V5 header.
const MASKS_AT: usize = 40;
/// The longest header a fixture writes, so a huge size field stays a small buffer.
const MAX_PAD: usize = 2 * V5 as usize;

/// A DIB's header fields and the bytes after the header.
#[derive(Clone, Debug)]
pub struct Dib {
    pub size: u32,
    pub width: i32,
    pub height: i32,
    pub planes: u16,
    pub bits: u16,
    pub compression: u32,
    pub colours: u32,
    /// Red, green, blue and alpha masks, written inside a V4 or V5 header.
    pub masks: [u32; 4],
    /// Bytes between the header and the rows: masks after a plain header, or colours.
    pub after: Vec<u8>,
    pub rows: Vec<u8>,
}

impl Dib {
    /// A plain `BITMAPINFOHEADER` DIB with no masks or colours.
    pub fn new(width: i32, height: i32, bits: u16, rows: Vec<u8>) -> Self {
        Self {
            size: INFO,
            width,
            height,
            planes: 1,
            bits,
            compression: BI_RGB,
            colours: 0,
            masks: [0; 4],
            after: Vec::new(),
            rows,
        }
    }

    /// The DIB's bytes: the header cut or padded to `size`, at most `MAX_PAD` bytes, then `after`, then `rows`.
    pub fn bytes(&self) -> Vec<u8> {
        let mut head = vec![0; V5 as usize];
        let mut put = |at: usize, field: &[u8]| head[at..at + field.len()].copy_from_slice(field);
        put(0, &self.size.to_le_bytes());
        put(4, &self.width.to_le_bytes());
        put(8, &self.height.to_le_bytes());
        put(12, &self.planes.to_le_bytes());
        put(14, &self.bits.to_le_bytes());
        put(16, &self.compression.to_le_bytes());
        put(32, &self.colours.to_le_bytes());
        put(MASKS_AT, &le(&self.masks));
        head.resize((self.size as usize).min(MAX_PAD), 0);
        [head, self.after.clone(), self.rows.clone()].concat()
    }
}

/// `words` as little-endian bytes.
pub fn le(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// A PNG's header fields.
#[derive(Clone, Debug)]
pub struct Png {
    pub width: u32,
    pub height: u32,
    pub color: ColorType,
    pub depth: BitDepth,
    pub palette: Option<Vec<u8>>,
    pub trns: Option<Vec<u8>>,
    pub interlaced: bool,
}

impl Png {
    /// A plain PNG header.
    pub fn new(width: u32, height: u32, color: ColorType, depth: BitDepth) -> Self {
        Self {
            width,
            height,
            color,
            depth,
            palette: None,
            trns: None,
            interlaced: false,
        }
    }

    /// The PNG of `raw`, its rows as the format packs them.
    pub fn encode(&self, raw: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut writer = self.encoder(&mut out).write_header().unwrap();
        writer.write_image_data(raw).unwrap();
        writer.finish().unwrap();
        out
    }

    /// The PNG whose row `y` is `row(y)`, written one row at a time so no full-size buffer exists.
    pub fn stream(&self, row: impl Fn(u32) -> Vec<u8>) -> Vec<u8> {
        let mut out = Vec::new();
        let mut writer = self.encoder(&mut out).write_header().unwrap();
        let mut rows = writer.stream_writer().unwrap();
        for y in 0..self.height {
            std::io::Write::write_all(&mut rows, &row(y)).unwrap();
        }
        rows.finish().unwrap();
        writer.finish().unwrap();
        out
    }

    fn encoder<'a>(&self, out: &'a mut Vec<u8>) -> Encoder<'static, &'a mut Vec<u8>> {
        let mut info = Info::with_size(self.width, self.height);
        info.color_type = self.color;
        info.bit_depth = self.depth;
        info.interlaced = self.interlaced;
        let mut encoder = Encoder::with_info(out, info).unwrap();
        encoder.set_compression(Compression::Fast);
        if let Some(palette) = &self.palette {
            encoder.set_palette(palette.clone());
        }
        if let Some(trns) = &self.trns {
            encoder.set_trns(trns.clone());
        }
        encoder
    }
}

/// Pixels as RGBA8 bytes.
pub fn rgba(pixels: &[[u8; RGBA_BYTES]]) -> Vec<u8> {
    pixels.concat()
}

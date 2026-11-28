//! One-byte wire codes for the frame kinds, image codecs and failure reasons; code 0 is never used. The voice, fetch,
//! net, record and pick jobs keep their own codes beside their frames.

mod kind;
mod lens;

pub(crate) use kind::Kind;

/// A small enum sent as one byte.
pub(crate) trait Code: Copy + PartialEq + 'static {
    /// Every value, so a byte maps back without a second table.
    const ALL: &'static [Self];

    /// The value's byte on the wire.
    fn code(self) -> u8;

    /// The value whose byte is `code`, if any.
    fn from_code(code: u8) -> Option<Self> {
        Self::ALL.iter().copied().find(|v| v.code() == code)
    }
}

/// How a draw's image bytes are coded.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Codec {
    /// PNG.
    Png = 1,
    /// A Windows device-independent bitmap, as `CF_DIB` holds it.
    Dib = 2,
}

impl Code for Codec {
    const ALL: &'static [Self] = &[Self::Png, Self::Dib];

    fn code(self) -> u8 {
        self as u8
    }
}

/// Why the media job made no pixels, DIB or PNG; a fixed reason, never the decoder's or encoder's own words.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Why {
    /// The bytes are not an image the worker reads.
    Refused = 1,
    /// The image holds more pixels than the worker's limit, its DIB would pass the asked cap, or its PNG `png_most`.
    TooBig = 2,
    /// The worker failed inside.
    Internal = 3,
}

impl Code for Why {
    const ALL: &'static [Self] = &[Self::Refused, Self::TooBig, Self::Internal];

    fn code(self) -> u8 {
        self as u8
    }
}

/// Why the vault job made nothing; a fixed reason, never a library's own words.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Fault {
    /// The password does not open it.
    WrongPassword = 1,
    /// It is damaged or cut short.
    Damaged = 2,
    /// It is not a box or a `KeePass` file at all.
    Foreign = 3,
    /// It, or what it holds, is over the vault job's caps.
    TooLarge = 4,
    /// A box from a newer version, or a `KeePass` kind this version does not read.
    Unsupported = 5,
    /// The worker failed inside.
    Internal = 6,
}

impl Code for Fault {
    const ALL: &'static [Self] = &[
        Self::WrongPassword,
        Self::Damaged,
        Self::Foreign,
        Self::TooLarge,
        Self::Unsupported,
        Self::Internal,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}

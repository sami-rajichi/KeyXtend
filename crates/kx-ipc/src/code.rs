//! One-byte wire codes for the frame kinds, image codecs and failure reasons; code 0 is never used.

use crate::limits::{
    DRAW_HEAD, FAILED_LEN, MAKE_HEAD, MAX_ASK_LEN, MAX_IMAGE_BYTES, MAX_REPLY_LEN, MIN_IMAGE_BYTES,
    PIXELS_HEAD, RGBA_BYTES,
};
use crate::vault::wire;
use std::ops::RangeInclusive;

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

/// What a frame carries; each side reads only the other side's kinds, and each job only its own.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Kind {
    /// The host asks for pixels.
    Draw = 1,
    /// The host tells the media job to end.
    Bye = 2,
    /// The worker sends pixels.
    Pixels = 3,
    /// The worker drew nothing.
    Failed = 4,
    /// The host asks for a DIB made from a PNG.
    MakeDib = 5,
    /// The worker sends the DIB.
    Dib = 6,
    /// The host asks for a new padlock.
    MakePadlock = 7,
    /// The host asks for parts sealed into a box.
    Seal = 8,
    /// The host asks for a box opened.
    OpenBox = 9,
    /// The host asks whether a password opens a locked identity.
    CheckPassword = 10,
    /// The host asks for a `KeePass` file's entries.
    ReadKeepass = 11,
    /// The host tells the vault job to end.
    VaultBye = 12,
    /// The worker sends a padlock.
    Padlock = 13,
    /// The worker sends a sealed box.
    Sealed = 14,
    /// The worker sends an opened box.
    Opened = 15,
    /// The worker says whether the password opens the identity.
    Checked = 16,
    /// The worker sends `KeePass` entries.
    Entries = 17,
    /// The vault job made nothing.
    VaultFailed = 18,
}

impl Code for Kind {
    const ALL: &'static [Self] = &[
        Self::Draw,
        Self::Bye,
        Self::Pixels,
        Self::Failed,
        Self::MakeDib,
        Self::Dib,
        Self::MakePadlock,
        Self::Seal,
        Self::OpenBox,
        Self::CheckPassword,
        Self::ReadKeepass,
        Self::VaultBye,
        Self::Padlock,
        Self::Sealed,
        Self::Opened,
        Self::Checked,
        Self::Entries,
        Self::VaultFailed,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}

impl Kind {
    /// The payload lengths this kind may take: a longer one is `TooLong`, a shorter one `Inconsistent`.
    pub(crate) const fn lens(self) -> RangeInclusive<usize> {
        match self {
            Self::Draw => DRAW_HEAD + MIN_IMAGE_BYTES..=MAX_ASK_LEN,
            Self::Bye | Self::VaultBye => 0..=0,
            Self::Pixels => PIXELS_HEAD + RGBA_BYTES..=MAX_REPLY_LEN,
            Self::Failed | Self::VaultFailed => FAILED_LEN..=FAILED_LEN,
            Self::Checked => wire::ONE..=wire::ONE,
            Self::MakeDib => MAKE_HEAD + MIN_IMAGE_BYTES..=MAKE_HEAD + MAX_IMAGE_BYTES,
            Self::Dib => MIN_IMAGE_BYTES..=MAX_IMAGE_BYTES,
            Self::MakePadlock => wire::MAKE_LEAST..=wire::MAKE_MOST,
            Self::Seal | Self::Opened => wire::SEAL_LEAST..=wire::SEAL_MOST,
            Self::OpenBox => wire::OPEN_LEAST..=wire::OPEN_MOST,
            Self::ReadKeepass => wire::KEEPASS_LEAST..=wire::KEEPASS_MOST,
            Self::CheckPassword => wire::CHECK_LEAST..=wire::CHECK_MOST,
            Self::Padlock => wire::PADLOCK_LEAST..=wire::PADLOCK_MOST,
            Self::Sealed => wire::BYTES.least()..=wire::BYTES.most(),
            Self::Entries => wire::ENTRIES_LEAST..=wire::ENTRIES_MOST,
        }
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

/// Why the worker drew no pixels; a fixed reason, never the decoder's own words.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Why {
    /// The bytes are not an image the worker reads.
    Refused = 1,
    /// The image holds more pixels than the worker's limit, or its DIB would pass the asked cap.
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

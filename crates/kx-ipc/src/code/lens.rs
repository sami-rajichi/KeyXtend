//! The payload lengths each kind may take, checked right after the header before any payload byte is read.

use super::Kind;
use crate::fetch::wire as fetch;
use crate::limits::{
    DRAW_HEAD, FAILED_LEN, MAKE_HEAD, MAX_ASK_LEN, MAX_IMAGE_BYTES, MAX_REPLY_LEN, MIN_IMAGE_BYTES,
    PIXELS_HEAD, RGBA_BYTES,
};
use crate::vault::wire as vault;
use crate::voice::wire as voice;
use crate::wire::ONE;
use std::ops::RangeInclusive;

/// No payload: a goodbye, an ask that carries only its id, or the fetched note.
const NONE: RangeInclusive<usize> = 0..=0;
/// A payload of one byte: a yes or no, a level, a stage or a percentage.
const ONE_BYTE: RangeInclusive<usize> = ONE..=ONE;
/// A failure's payload: its reason.
const REASON: RangeInclusive<usize> = FAILED_LEN..=FAILED_LEN;

impl Kind {
    /// True for a kind with no payload.
    pub(crate) const fn bare(self) -> bool {
        matches!(
            self,
            Self::Bye
                | Self::VaultBye
                | Self::VoiceBye
                | Self::FetchBye
                | Self::VoiceStop
                | Self::VoiceCancel
                | Self::VoiceKeep
                | Self::VoiceClose
                | Self::VoiceDevices
                | Self::FetchPause
                | Self::FetchResume
                | Self::FetchCancel
                | Self::Fetched
        )
    }

    /// The payload lengths this kind may take: a longer one is `TooLong`, a shorter one `Inconsistent`.
    pub(crate) const fn lens(self) -> RangeInclusive<usize> {
        match self {
            Self::Draw => DRAW_HEAD + MIN_IMAGE_BYTES..=MAX_ASK_LEN,
            Self::Checked | Self::Level | Self::Heard | Self::Stage | Self::Warm => ONE_BYTE,
            Self::Failed | Self::VaultFailed | Self::VoiceFailed | Self::FetchFailed => REASON,
            Self::VoiceStart | Self::VoiceAgain => voice::SETUP_LEAST..=voice::SETUP_MOST,
            Self::VoiceFacts => voice::PATH.least()..=voice::PATH.most(),
            Self::VoiceCheck => voice::CHECK_LEAST..=voice::CHECK_MOST,
            Self::SoundIn | Self::SoundOut => voice::SOUND_LEAST..=voice::SOUND_MOST,
            Self::Piece => voice::PIECE_LEAST..=voice::PIECE_MOST,
            Self::Done => voice::DONE_LEN..=voice::DONE_LEN,
            Self::DeviceList => voice::DEVICES_LEAST..=voice::DEVICES_MOST,
            Self::FileFacts => voice::FACTS_LEAST..=voice::FACTS_MOST,
            Self::CardChecked => voice::CHECKED_LEAST..=voice::CHECKED_MOST,
            Self::Fetch => fetch::GET_LEAST..=fetch::GET_MOST,
            Self::Progress => fetch::PROGRESS_LEN..=fetch::PROGRESS_LEN,
            Self::Pixels => PIXELS_HEAD + RGBA_BYTES..=MAX_REPLY_LEN,
            Self::MakeDib => MAKE_HEAD + MIN_IMAGE_BYTES..=MAKE_HEAD + MAX_IMAGE_BYTES,
            Self::Dib => MIN_IMAGE_BYTES..=MAX_IMAGE_BYTES,
            Self::MakePadlock => vault::MAKE_LEAST..=vault::MAKE_MOST,
            Self::Seal | Self::Opened => vault::SEAL_LEAST..=vault::SEAL_MOST,
            Self::OpenBox => vault::OPEN_LEAST..=vault::OPEN_MOST,
            Self::ReadKeepass => vault::KEEPASS_LEAST..=vault::KEEPASS_MOST,
            Self::CheckPassword => vault::CHECK_LEAST..=vault::CHECK_MOST,
            Self::Padlock => vault::PADLOCK_LEAST..=vault::PADLOCK_MOST,
            Self::Sealed => vault::BYTES.least()..=vault::BYTES.most(),
            Self::Entries => vault::ENTRIES_LEAST..=vault::ENTRIES_MOST,
            // Only the bare kinds are left; a payload kind missing above fails the debug build's tests here.
            other => {
                debug_assert!(other.bare(), "a payload kind has no lengths");
                NONE
            }
        }
    }
}

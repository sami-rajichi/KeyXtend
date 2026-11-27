//! The payload lengths each kind may take, checked right after the header before any payload byte is read.

use super::Kind;
use crate::fetch::wire as fetch;
use crate::limits::snip::BGRX_BYTES;
use crate::limits::{
    DRAW_HEAD, ENCODE_HEAD, FAILED_LEN, MAKE_HEAD, MAX_DRAW_LEN, MAX_ENCODE_LEN, MAX_IMAGE_BYTES,
    MAX_PIXELS_LEN, MAX_PNG_BYTES, MIN_IMAGE_BYTES, PIXELS_HEAD, RGBA_BYTES,
};
use crate::net::wire as net;
use crate::pick::wire as pick;
use crate::record::wire as record;
use crate::vault::wire as vault;
use crate::voice::wire as voice;
use crate::wire::{ONE, PATH};
use std::ops::RangeInclusive;

/// No payload: a goodbye, an ask that carries only its id, the fetched or net done note, a dialog's Cancel or refusal.
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
                | Self::NetBye
                | Self::NetCancel
                | Self::NetDone
                | Self::RecordStop
                | Self::RecordCancel
                | Self::RecordBye
                | Self::PickBye
                | Self::PickCancelled
                | Self::PickRefused
        )
    }

    /// The payload lengths this kind may take: a longer one is `TooLong`, a shorter one `Inconsistent`.
    pub(crate) const fn lens(self) -> RangeInclusive<usize> {
        match self {
            Self::Draw => DRAW_HEAD + MIN_IMAGE_BYTES..=MAX_DRAW_LEN,
            Self::Checked | Self::Level | Self::Heard | Self::Stage | Self::Warm => ONE_BYTE,
            Self::Failed | Self::VaultFailed | Self::VoiceFailed | Self::FetchFailed => REASON,
            Self::VoiceStart | Self::VoiceAgain => voice::SETUP_LEAST..=voice::SETUP_MOST,
            Self::VoiceFacts => PATH.least()..=PATH.most(),
            Self::VoiceCheck => voice::CHECK_LEAST..=voice::CHECK_MOST,
            Self::SoundIn | Self::SoundOut => voice::SOUND_LEAST..=voice::SOUND_MOST,
            Self::SpeechOut => voice::SPEECH_LEAST..=voice::SPEECH_MOST,
            Self::Piece => voice::PIECE_LEAST..=voice::PIECE_MOST,
            Self::Done => voice::DONE_LEN..=voice::DONE_LEN,
            Self::DeviceList => voice::DEVICES_LEAST..=voice::DEVICES_MOST,
            Self::FileFacts => voice::FACTS_LEAST..=voice::FACTS_MOST,
            Self::CardChecked => voice::CHECKED_LEAST..=voice::CHECKED_MOST,
            Self::Fetch => fetch::GET_LEAST..=fetch::GET_MOST,
            Self::Progress => fetch::PROGRESS_LEN..=fetch::PROGRESS_LEN,
            Self::NetSend | Self::NetSound | Self::NetTest | Self::NetText | Self::NetFailed => {
                self.net_lens()
            }
            Self::Pixels => PIXELS_HEAD + RGBA_BYTES..=MAX_PIXELS_LEN,
            Self::MakeDib => MAKE_HEAD + MIN_IMAGE_BYTES..=MAKE_HEAD + MAX_IMAGE_BYTES,
            Self::Dib => MIN_IMAGE_BYTES..=MAX_IMAGE_BYTES,
            Self::Encode => ENCODE_HEAD + BGRX_BYTES..=MAX_ENCODE_LEN,
            Self::Png => MIN_IMAGE_BYTES..=MAX_PNG_BYTES,
            Self::MakePadlock => vault::MAKE_LEAST..=vault::MAKE_MOST,
            Self::Seal | Self::Opened => vault::SEAL_LEAST..=vault::SEAL_MOST,
            Self::OpenBox => vault::OPEN_LEAST..=vault::OPEN_MOST,
            Self::ReadKeepass => vault::KEEPASS_LEAST..=vault::KEEPASS_MOST,
            Self::CheckPassword => vault::CHECK_LEAST..=vault::CHECK_MOST,
            Self::Padlock => vault::PADLOCK_LEAST..=vault::PADLOCK_MOST,
            Self::Sealed => vault::BYTES.least()..=vault::BYTES.most(),
            Self::Entries => vault::ENTRIES_LEAST..=vault::ENTRIES_MOST,
            // The record and pick kinds, and the bare kinds, are left.
            other => other.snip_lens(),
        }
    }

    /// The payload lengths of the net kinds that carry one; `NONE` for any other kind.
    const fn net_lens(self) -> RangeInclusive<usize> {
        match self {
            Self::NetSend => net::SEND_LEAST..=net::SEND_MOST,
            Self::NetSound => net::SOUND_LEAST..=net::SOUND_MOST,
            Self::NetTest => net::TEST_LEAST..=net::TEST_MOST,
            Self::NetText => net::TEXT_LEAST..=net::TEXT_MOST,
            Self::NetFailed => net::FAULT_LEAST..=net::FAULT_MOST,
            _ => NONE,
        }
    }

    /// The payload lengths of the record and pick kinds that carry one, and `NONE` for a bare kind.
    const fn snip_lens(self) -> RangeInclusive<usize> {
        match self {
            Self::RecordStart => record::START_LEAST..=record::START_MOST,
            Self::RecordMic | Self::RecordSound => ONE_BYTE,
            Self::RecordSpeed => record::SPEED_LEAST..=record::SPEED_MOST,
            Self::RecordTime => record::TIME_LEN..=record::TIME_LEN,
            Self::Recorded => record::END_LEN..=record::END_LEN,
            Self::RecordFailed => REASON,
            Self::RecordSwitched => record::SWITCHED_LEN..=record::SWITCHED_LEN,
            Self::RecordLost => record::LOST_LEN..=record::LOST_LEN,
            Self::PickSave => pick::SAVE_LEAST..=pick::SAVE_MOST,
            Self::PickFolder | Self::Picked => PATH.least()..=PATH.most(),
            Self::PickFailed => pick::FAILED_LEN..=pick::FAILED_LEN,
            // Only the bare kinds are left; a payload kind missing above fails the debug build's tests here.
            other => {
                debug_assert!(other.bare(), "a payload kind has no lengths");
                NONE
            }
        }
    }
}

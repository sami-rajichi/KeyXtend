//! The frame kinds, one byte each, numbered without a gap; a new kind takes the next number.

use super::Code;

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
    /// The host starts a dictation.
    VoiceStart = 19,
    /// The host stops listening; the rest is transcribed.
    VoiceStop = 20,
    /// The host drops the run; the recording stays until closed.
    VoiceCancel = 21,
    /// The host asks for the held recording transcribed again.
    VoiceAgain = 22,
    /// The host asks for the held recording back, in pieces.
    VoiceKeep = 23,
    /// The host closes the run; its recording is wiped.
    VoiceClose = 24,
    /// The host asks for the devices.
    VoiceDevices = 25,
    /// The host asks for a model file's facts.
    VoiceFacts = 26,
    /// The host asks for a card's check.
    VoiceCheck = 27,
    /// The host hands a piece of a recording to the voice job.
    SoundIn = 28,
    /// The host tells the voice job to end.
    VoiceBye = 29,
    /// The voice job sends a sound level.
    Level = 30,
    /// The voice job heard speech start or stop.
    Heard = 31,
    /// The voice job reached a stage.
    Stage = 32,
    /// The voice job sends a card's warm-up progress.
    Warm = 33,
    /// The voice job sends a piece's text.
    Piece = 34,
    /// The voice job finished a run.
    Done = 35,
    /// The voice job hit a fault.
    VoiceFailed = 36,
    /// The voice job sends the devices.
    DeviceList = 37,
    /// The voice job sends a model file's facts.
    FileFacts = 38,
    /// The voice job sends a card's check result.
    CardChecked = 39,
    /// The voice job hands a piece of its recording to the host.
    SoundOut = 40,
    /// The host asks for a model file downloaded.
    Fetch = 41,
    /// The host pauses the download.
    FetchPause = 42,
    /// The host resumes the download.
    FetchResume = 43,
    /// The host cancels the download.
    FetchCancel = 44,
    /// The host tells the fetch job to end.
    FetchBye = 45,
    /// The fetch job sends its progress.
    Progress = 46,
    /// The fetch job checked and named the file.
    Fetched = 47,
    /// The fetch job stopped, for a fixed reason.
    FetchFailed = 48,
    /// The host sends a dictation's pieces to a cloud service.
    NetSend = 49,
    /// The host hands a part of a piece's sound to the speech-net job.
    NetSound = 50,
    /// The host asks whether a key works.
    NetTest = 51,
    /// The host drops a run or a test.
    NetCancel = 52,
    /// The host tells the speech-net job to end.
    NetBye = 53,
    /// The speech-net job sends a piece's text.
    NetText = 54,
    /// The speech-net job told every piece, or the key works.
    NetDone = 55,
    /// The speech-net job stopped, for a fixed reason.
    NetFailed = 56,
    /// The voice job hands over a piece of a listen-only run's speech at Stop.
    SpeechOut = 57,
    /// The host asks for a snip's raw pixels made into a PNG.
    Encode = 58,
    /// The media job sends the PNG.
    Png = 59,
    /// The host starts a recording.
    RecordStart = 60,
    /// The host switches a recording's microphone on or off.
    RecordMic = 61,
    /// The host switches a recording's computer sound on or off.
    RecordSound = 62,
    /// The host stops a recording, keeping its video.
    RecordStop = 63,
    /// The host stops a recording and deletes its video.
    RecordCancel = 64,
    /// The host asks for a re-timed copy of a video.
    RecordSpeed = 65,
    /// The host tells the record job to end.
    RecordBye = 66,
    /// The record job tells how long it has recorded.
    RecordTime = 67,
    /// The record job ended a video, which is whole and kept.
    Recorded = 68,
    /// The record job hit a fault.
    RecordFailed = 69,
    /// The host asks for the Save dialog.
    PickSave = 70,
    /// The host asks for the folder dialog.
    PickFolder = 71,
    /// The host tells the pick job to end.
    PickBye = 72,
    /// The pick job sends the path the user chose.
    Picked = 73,
    /// The user cancelled the dialog.
    PickCancelled = 74,
    /// The dialog could not show, with the system's code.
    PickFailed = 75,
    /// The record job switched a sound as asked.
    RecordSwitched = 76,
    /// The record job lost a sound's device mid-run and records silence for it.
    RecordLost = 77,
    /// The dialog answered a path the ask's check refuses.
    PickRefused = 78,
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
        Self::VoiceStart,
        Self::VoiceStop,
        Self::VoiceCancel,
        Self::VoiceAgain,
        Self::VoiceKeep,
        Self::VoiceClose,
        Self::VoiceDevices,
        Self::VoiceFacts,
        Self::VoiceCheck,
        Self::SoundIn,
        Self::VoiceBye,
        Self::Level,
        Self::Heard,
        Self::Stage,
        Self::Warm,
        Self::Piece,
        Self::Done,
        Self::VoiceFailed,
        Self::DeviceList,
        Self::FileFacts,
        Self::CardChecked,
        Self::SoundOut,
        Self::Fetch,
        Self::FetchPause,
        Self::FetchResume,
        Self::FetchCancel,
        Self::FetchBye,
        Self::Progress,
        Self::Fetched,
        Self::FetchFailed,
        Self::NetSend,
        Self::NetSound,
        Self::NetTest,
        Self::NetCancel,
        Self::NetBye,
        Self::NetText,
        Self::NetDone,
        Self::NetFailed,
        Self::SpeechOut,
        Self::Encode,
        Self::Png,
        Self::RecordStart,
        Self::RecordMic,
        Self::RecordSound,
        Self::RecordStop,
        Self::RecordCancel,
        Self::RecordSpeed,
        Self::RecordBye,
        Self::RecordTime,
        Self::Recorded,
        Self::RecordFailed,
        Self::PickSave,
        Self::PickFolder,
        Self::PickBye,
        Self::Picked,
        Self::PickCancelled,
        Self::PickFailed,
        Self::RecordSwitched,
        Self::RecordLost,
        Self::PickRefused,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}

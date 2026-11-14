//! One-byte wire codes for the frame kinds, image codecs and failure reasons; code 0 is never used. The voice and
//! fetch jobs keep their own codes beside their frames.

mod lens;

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
    ];

    fn code(self) -> u8 {
        self as u8
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

//! The worker's command words and exit codes, read by the host that starts it and by the worker itself.

/// The job that draws pictures and makes snips' PNGs, the first argument.
pub const MEDIA_JOB: &str = "media";
/// The flag before the most pixels an image may hold.
pub const MAX_PIXELS_FLAG: &str = "--max-pixels";
/// The job that makes padlocks, seals and opens boxes and reads `KeePass` files, alone on its command line.
pub const VAULT_JOB: &str = "vault";
/// The job that opens the microphone and runs the detector and the engine, with no network (ADR-0042).
pub const VOICE_JOB: &str = "voice";
/// The job that downloads model files, with no microphone (ADR-0042).
pub const FETCH_JOB: &str = "fetch";
/// The job that sends a dictation's sound to the cloud service the user switched on, with no microphone (ADR-0047).
pub const NET_JOB: &str = "speech-net";
/// The job that records parts of the screen with their sound into videos and writes re-timed copies, with no network
/// (ADR-0056 decision 3).
pub const RECORD_JOB: &str = "record";
/// The job that shows the system's Save and folder dialogs, with no network (ADR-0056 decision 6).
pub const PICK_JOB: &str = "pick";
/// Gate builds only: the flag after `VOICE_JOB` that names a WAV file played in place of the microphone; the stage
/// refuses any program holding its text.
pub const SOUND_FLAG: &str = "--sound";
/// The exit code after goodbye or the end of input; every OS reads 0 as success.
pub const CLEAN_EXIT: u8 = 0;
/// The exit code after a refused command line or a broken frame.
pub const REFUSED_EXIT: u8 = 1;

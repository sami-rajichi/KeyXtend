//! The frames between the keyboard and its worker processes, the worker's command words and exit codes, and the backup
//! box's facts, one copy for both sides (ADR-0004, ADR-0034 decision 5, ADR-0038, ADR-0042). A frame has a fixed header
//! and a payload capped by its kind, and every field is checked before it is trusted or a buffer is allocated.

#![forbid(unsafe_code)]

mod ask;
mod code;
mod command;
mod error;
mod fetch;
mod frame;
mod limits;
mod reply;
mod rules;
mod vault;
mod voice;
mod wiped;
mod wire;

pub use ask::Ask;
pub use code::{Codec, Fault, Why};
pub use command::{
    CLEAN_EXIT, FETCH_JOB, MAX_PIXELS_FLAG, MEDIA_JOB, REFUSED_EXIT, VAULT_JOB, VOICE_JOB,
};
pub use error::IpcError;
pub use fetch::ask::FetchAsk;
pub use fetch::note::FetchNote;
pub use fetch::{FetchFault, part_path};
pub use limits::{
    BOX_EXT, BOX_MARK, BOX_NAME, BOX_PARTS, BOX_VERSION, DECODE_BYTES, MAX_AES_ROUNDS, MAX_ASK_LEN,
    MAX_BOX_BYTES, MAX_BOX_WORK, MAX_ENTRIES, MAX_ENTRIES_BYTES, MAX_FIELD_BYTES, MAX_FIELD_CHARS,
    MAX_FIELDS, MAX_IMAGE_BYTES, MAX_KDF_BYTES, MAX_KDF_LANES, MAX_KDF_PASSES, MAX_KEEPASS_BYTES,
    MAX_LOCKED_BYTES, MAX_PARTS, MAX_PASSWORD_BYTES, MAX_RECIPIENT_BYTES, MAX_REPLY_LEN,
    MAX_RGBA_BYTES, MAX_SIDE, MIN_BOX_WORK, NOTEBOOK_PART, READ_TIMES, RGBA_BYTES, VAULT_MEMORY_MB,
    VERSION,
};
pub use limits::{
    HTTPS, LEVEL_HZ, MAX_DEVICES, MAX_HOST_BYTES, MAX_HOSTS, MAX_LANGS, MAX_MODEL_BYTES,
    MAX_NAME_BYTES, MAX_PATH_BYTES, MAX_PERCENT, MAX_PIECES, MAX_RECORD_BYTES, MAX_SOUND_BYTES,
    MAX_SOUND_MS, MAX_SOUND_PIECES, MAX_TAG_BYTES, MAX_TEXT_BYTES, MAX_URL_BYTES, MAX_URLS, PAD_MS,
    PART_EXT, PIECE_S, RECORD_MAX_S, SAMPLE_BYTES, SHA256_BYTES, SILENCE_STOP_S, SOUND_HZ, STALL_S,
    THREADS,
};
pub use reply::Reply;
pub use vault::ask::VaultAsk;
pub use vault::reply::VaultReply;
pub use vault::{BoxPart, Dropped, KeepassEntry, KeepassField};
pub use voice::ask::VoiceAsk;
pub use voice::note::VoiceNote;
pub use voice::{Device, DeviceKind, Model, Setup, SoundPiece, Stage, Target, VoiceFault};
pub use wiped::Wiped;

#[cfg(test)]
mod tests;

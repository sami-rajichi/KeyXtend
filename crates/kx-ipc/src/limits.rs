//! The protocol's and the backup box's magic, versions, names and limits, in one place.

mod media;
pub(crate) mod snip;

pub use media::{
    DECODE_BYTES, MAX_ASK_LEN, MAX_IMAGE_BYTES, MAX_PNG_BYTES, MAX_REPLY_LEN, MAX_RGBA_BYTES,
    MAX_SIDE, READ_TIMES, RGBA_BYTES, png_most,
};
pub(crate) use media::{
    DRAW_HEAD, ENCODE_HEAD, FAILED_LEN, MAKE_HEAD, MAX_DRAW_LEN, MAX_ENCODE_LEN, MAX_PIXELS_LEN,
    MIN_IMAGE_BYTES, PIXELS_HEAD, READ_CHUNK,
};

use crate::vault::wire;
use std::ops::RangeInclusive;

/// A kibibyte, the unit of the read chunk and of the locked identity's cap.
const KIB: usize = 1024;
/// A mebibyte, the unit of the image, decoder, box and vault job caps, and of every job's memory cap.
pub const MIB: usize = KIB * KIB;

/// The bytes every frame starts with.
pub(crate) const MAGIC: [u8; 4] = *b"KXIP";
/// The protocol version; a frame with any other is refused.
pub const VERSION: u8 = 1;
/// The fixed header: magic, version, kind, request id and payload length.
pub(crate) const HEADER_LEN: usize = MAGIC.len() + 2 * size_of::<u8>() + 2 * size_of::<u32>();

/// The bytes every backup box starts with, before its format version (ADR-0038 decision 5).
pub const BOX_MARK: [u8; 5] = *b"KXBOX";
/// The backup box's format version, the byte after its mark; a box with a newer one is refused.
pub const BOX_VERSION: u8 = 1;
/// The backup box's file name, before its extension.
pub const BOX_NAME: &str = "KeyXtend Backup";
/// The backup box's extension, lower case.
pub const BOX_EXT: &str = "kxbox";
/// The largest box the worker seals or opens: the top of the `box_mb` setting, 64 MiB.
pub const MAX_BOX_BYTES: usize = 64 * MIB;
/// The lowest scrypt work factor (log N) a padlock is made with: the bottom of the `box_work` setting.
pub const MIN_BOX_WORK: u8 = 16;
/// The highest scrypt work factor (log N) the worker makes or opens, so a forged box cannot make it spin.
pub const MAX_BOX_WORK: u8 = 20;
/// scrypt's block size `r`, which `age` fixes at 8.
const SCRYPT_R: usize = 8;
/// What scrypt takes at the top work factor: 128 · r · 2^N bytes, 1 GiB.
pub(crate) const SCRYPT_BYTES: usize = (128 * SCRYPT_R) << MAX_BOX_WORK;
/// The copies of a box the vault job holds at once: the frame as its read grows, the opened parts and the reply.
const BOX_COPIES: usize = READ_TIMES + 2;
/// The vault job's memory beside scrypt and the boxes: its code, stacks, pipe buffers and `age`'s own.
const VAULT_HEADROOM: usize = 64 * MIB;
/// The vault job's whole budget, in bytes.
const VAULT_MEMORY_BYTES: usize = SCRYPT_BYTES + BOX_COPIES * MAX_BOX_BYTES + VAULT_HEADROOM;
/// The most memory the vault job may take, in MB: scrypt at the top work factor, the box's copies and a headroom.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a const assert below keeps it within `u32`"
)]
pub const VAULT_MEMORY_MB: u32 = VAULT_MEMORY_BYTES.div_ceil(MIB) as u32;

/// The part of a box that holds the Quick-fill notebook.
pub const NOTEBOOK_PART: &str = "notebook";
/// Every part a box may hold, by its short ASCII name; a box with any other name, or one name twice, is refused.
pub const BOX_PARTS: &[&str] = &[NOTEBOOK_PART];
/// The most parts a box holds: each known part once.
pub const MAX_PARTS: usize = BOX_PARTS.len();
/// The longest part name, in bytes.
pub(crate) const MAX_PART_NAME: usize = longest(BOX_PARTS);
/// The largest locked identity a padlock carries; `age` locks one in about 250 bytes.
pub const MAX_LOCKED_BYTES: usize = KIB;
/// The longest open padlock; an `age` X25519 recipient is 62 ASCII bytes, and one length byte holds it.
pub const MAX_RECIPIENT_BYTES: usize = u8::MAX as usize;
/// The most characters a saved value holds: the top of the `field_chars` setting.
pub const MAX_FIELD_CHARS: usize = 4000;
/// The longest saved value or `KeePass` field in UTF-8 bytes, at most 4 a character.
pub const MAX_FIELD_BYTES: usize = MAX_FIELD_CHARS * char::MAX_LEN_UTF8;
/// The longest password a vault ask carries; it is typed in a field.
pub const MAX_PASSWORD_BYTES: usize = MAX_FIELD_BYTES;
/// The most entries a `KeePass` reply holds: the top of the `max_entries` setting.
pub const MAX_ENTRIES: usize = 10_000;
/// The most custom fields a `KeePass` entry carries: the top of the `max_fields` setting.
pub const MAX_FIELDS: usize = 100;
/// The largest `KeePass` reply, the box's cap; the job drops the entries past it.
pub(crate) const MAX_ENTRIES_LEN: usize = MAX_BOX_BYTES;
/// The most bytes a `KeePass` reply's entries take together, after its counts; see `KeepassEntry::wire_len`.
pub const MAX_ENTRIES_BYTES: usize = MAX_ENTRIES_LEN - wire::ENTRIES_LEAST;
/// The largest `KeePass` file the vault job reads, so the copies `keepass` makes of it fit the job's memory.
pub const MAX_KEEPASS_BYTES: usize = 16 * MIB;
/// The copies of a `KeePass` file the job holds once its key is derived: decrypted, unzipped, its XML and entries.
const KEEPASS_COPIES: usize = 32;

/// The most memory a `KeePass` file's Argon2 may take: scrypt's share of the vault job's memory.
pub const MAX_KDF_BYTES: u64 = SCRYPT_BYTES as u64;
/// The most Argon2 passes, a sanity cap at `keepass`'s 32-bit count; the host's deadline bounds the time.
pub const MAX_KDF_PASSES: u64 = u32::MAX as u64;
/// The most Argon2 lanes, each a thread while it runs.
pub const MAX_KDF_LANES: u32 = 64;
/// The most AES-KDF rounds, a sanity cap at a 32-bit count; the host's deadline bounds the time.
pub const MAX_AES_ROUNDS: u64 = u32::MAX as u64;

/// The `threads` setting's range: processor threads, 0 for the real cores.
pub const THREADS: RangeInclusive<u8> = 0..=64;
/// The `pad_ms` setting's range: silence kept around speech, in ms.
pub const PAD_MS: RangeInclusive<u16> = 0..=2000;
/// The `piece_s` setting's range: seconds of sound before a cut at the next pause.
pub const PIECE_S: RangeInclusive<u8> = 10..=120;
/// The `record_max_s` setting's range: the longest dictation, in seconds.
pub const RECORD_MAX_S: RangeInclusive<u16> = 10..=3600;
/// The `level_hz` setting's range: sound levels per second.
pub const LEVEL_HZ: RangeInclusive<u8> = 5..=60;
/// The `silence_stop_s` setting's range: seconds of silence after speech that stop a dictation.
pub const SILENCE_STOP_S: RangeInclusive<u8> = 1..=30;
/// The `stall_s` setting's range: seconds without a byte before a download stops.
pub const STALL_S: RangeInclusive<u16> = 5..=300;

/// The sound hand-over's rate: 16 kHz mono, the form the models hear and the store keeps.
pub const SOUND_HZ: usize = 16_000;
/// Bytes in one sound sample: signed 16-bit, little-endian.
pub const SAMPLE_BYTES: usize = size_of::<i16>();
/// Bytes of hand-over sound in one second.
pub const SOUND_BYTES_PER_S: usize = SOUND_HZ * SAMPLE_BYTES;
/// The most sound one frame carries, in whole samples: about 33 seconds.
pub const MAX_SOUND_BYTES: usize = MIB;
/// The longest recording in bytes: `record_max_s` at its top, as hand-over sound.
pub const MAX_RECORD_BYTES: usize = *RECORD_MAX_S.end() as usize * SOUND_BYTES_PER_S;
/// The most pieces a recording is handed over in, so a piece's index stays bounded.
pub const MAX_SOUND_PIECES: usize = MAX_RECORD_BYTES.div_ceil(MAX_SOUND_BYTES);
/// Milliseconds in a second.
pub const MS_PER_S: u32 = 1000;
/// The longest recording in ms, which a done note's sound may not pass.
pub const MAX_SOUND_MS: u32 = *RECORD_MAX_S.end() as u32 * MS_PER_S;
/// The most text pieces a dictation is cut into: each but the last is at least `piece_s` long.
pub const MAX_PIECES: usize = (*RECORD_MAX_S.end() / *PIECE_S.start() as u16) as usize + 1;
/// The most speech pieces a listen-only Stop hands over, and so the most one cloud run sends: two in a row hold more
/// than the shortest `piece_s`.
pub const MAX_SPEECH_PIECES: usize = 2 * MAX_PIECES;
/// The longest text of one piece in UTF-8 bytes: far past two minutes of speech in any script.
pub const MAX_TEXT_BYTES: usize = 64_000;
/// The longest file path in UTF-8 bytes.
pub const MAX_PATH_BYTES: usize = 4096;
/// The longest short ASCII name: a language code, a runner or a card's id.
pub const MAX_TAG_BYTES: usize = 64;
/// The longest device name or driver version in UTF-8 bytes.
pub const MAX_NAME_BYTES: usize = u8::MAX as usize;
/// The most devices a list holds: the processor and its cards.
pub const MAX_DEVICES: usize = 16;
/// The most language codes a file's facts name.
pub const MAX_LANGS: usize = 128;
/// The top of a warm-up's progress.
pub const MAX_PERCENT: u8 = 100;

/// The most addresses a download tries, in the table's order.
pub const MAX_URLS: usize = 8;
/// The longest download address.
pub const MAX_URL_BYTES: usize = 2048;
/// The most hosts a download may be redirected to.
pub const MAX_HOSTS: usize = 16;
/// The longest host name, as DNS allows.
pub const MAX_HOST_BYTES: usize = 253;
/// A gibibyte, the unit of the model file cap.
const GIB: u64 = (MIB * KIB) as u64;
/// The largest model file a download takes.
pub const MAX_MODEL_BYTES: u64 = 64 * GIB;
/// Bytes in a SHA-256 fingerprint.
pub const SHA256_BYTES: usize = 32;
/// The start every download address must have.
pub const HTTPS: &str = "https://";
/// What a download's partial file adds to the final name.
pub const PART_EXT: &str = ".part";

/// The `net_wait_s` setting's range: seconds to wait for one cloud request's answer.
pub const NET_WAIT_S: RangeInclusive<u16> = 5..=300;
/// The `net_tries` setting's range: tries per piece before a cloud fault is told.
pub const NET_TRIES: RangeInclusive<u8> = 1..=5;
/// A service row's megabytes a request: up to 8 GiB, far past the largest cap a service names.
pub const NET_MAX_MB: RangeInclusive<u32> = 1..=8_192;
/// A service row's requests a minute: up to 100 a second, a sanity ceiling far past any service's own rate.
pub const NET_PER_MIN: RangeInclusive<u16> = 1..=6_000;
/// A service row's requests at once: up to 16, so one dictation never floods the user's link.
pub const NET_AT_ONCE: RangeInclusive<u8> = 1..=16;
/// The longest cloud key a header carries: 1 KiB, far past any service's key of a few hundred bytes.
pub const MAX_KEY_BYTES: usize = KIB;
/// The most fixed fields an upload form adds: a keep-nothing switch needs one, and 4 leaves room for a new row's own
/// without a frame change.
pub const MAX_FIXED: usize = 4;
/// The most reply fields a form's key test reads for a management key: `OpenRouter` names two, and 4 leaves room.
pub const MAX_ADMIN: usize = 4;
/// The longest cloud piece, in seconds: ten minutes, 19.2 MB raw, under the ~25 MB a request most services take; a
/// longer dictation goes in more pieces.
const NET_PIECE_MAX_S: u16 = 600;
/// A service row's seconds a request: from the shortest piece the `voice` job cuts to `NET_PIECE_MAX_S`.
pub const NET_MAX_S: RangeInclusive<u16> = (*PIECE_S.start() as u16)..=NET_PIECE_MAX_S;
/// Seconds in a day, the unit of a renewal's cap.
const DAY_S: u32 = 24 * 60 * 60;
/// The most days an allowance's renewal may be away: a month, the longest period a service's free allowance names.
const RENEW_DAYS: u32 = 31;
/// How far away an allowance's renewal may be, in seconds.
pub const NET_RENEW_S: RangeInclusive<u32> = 1..=RENEW_DAYS * DAY_S;
/// The longest text of one cloud piece in UTF-8 bytes: `MAX_TEXT_BYTES` for each top `piece_s` the longest piece holds.
pub const MAX_NET_TEXT_BYTES: usize =
    MAX_TEXT_BYTES * (*NET_MAX_S.end() / *PIECE_S.end() as u16) as usize;
/// The most text bytes one cloud run tells in all, as for the longest dictation; the host enforces it.
pub const MAX_NET_RUN_TEXT_BYTES: usize =
    MAX_TEXT_BYTES * (*RECORD_MAX_S.end() / *PIECE_S.end() as u16) as usize;
/// The largest reply `speech-net` reads from a service: ten minutes' words with their timings stay under 1 MiB, so
/// this leaves room yet bounds a hostile reply.
pub const MAX_NET_REPLY_BYTES: usize = 4 * MIB;
/// The span a row's `per_min` counts requests in, in seconds; a request may wait this long for its turn.
pub const NET_PACE_S: u16 = 60;
/// The waits of `net_wait_s` one try may spend beside its upload: connecting, the service's answer and its reply.
pub const NET_TRY_WAITS: u32 = 3;
/// How many times its own length a piece's upload may take: a link at half the sound's rate still sends it.
pub const NET_UPLOAD_TIMES: u32 = 2;
/// A key's test sends once: a wrong key is told at once, and a 429 still waits once.
pub const NET_TEST_TRIES: u8 = 1;

/// The most bytes one cloud piece may hold within a row's `max_mb` and `max_s`, in whole samples.
#[must_use]
pub fn piece_bytes(max_mb: u32, max_s: u16) -> usize {
    let by_size = u64::from(max_mb).saturating_mul(MIB as u64);
    let by_size = usize::try_from(by_size).unwrap_or(usize::MAX);
    let by_time = usize::from(max_s) * SOUND_BYTES_PER_S;
    let most = by_size.min(by_time);
    most - most % SAMPLE_BYTES
}

/// The longest of `names`, in bytes.
const fn longest(names: &[&str]) -> usize {
    let (mut at, mut most) = (0, 0);
    while at < names.len() {
        if names[at].len() > most {
            most = names[at].len();
        }
        at += 1;
    }
    most
}

// The largest box fits the header's `u32` field; `media.rs` checks the media payloads.
const _: () = assert!(MAX_BOX_BYTES <= u32::MAX as usize);
// A `KeePass` read fits the vault job's memory: Argon2 beside the file as it is read, then the file's copies.
const _: () = assert!(
    MAX_KDF_BYTES <= (VAULT_MEMORY_BYTES - READ_TIMES * MAX_KEEPASS_BYTES - VAULT_HEADROOM) as u64
);
const _: () = assert!(KEEPASS_COPIES * MAX_KEEPASS_BYTES + VAULT_HEADROOM <= VAULT_MEMORY_BYTES);
const _: () = assert!(MAX_KEEPASS_BYTES <= MAX_BOX_BYTES);
// The work factors rise, and the vault job's cap fits the `u32` MB a job is started with.
const _: () = assert!(MIN_BOX_WORK <= MAX_BOX_WORK);
const _: () = assert!(VAULT_MEMORY_BYTES.div_ceil(MIB) <= u32::MAX as usize);
// Each vault cap fits the width it is sent in: one byte for names, padlocks and small counts, two for the rest.
const _: () = assert!(MAX_PART_NAME <= u8::MAX as usize);
const _: () = assert!(MAX_PARTS <= u8::MAX as usize);
const _: () = assert!(MAX_FIELDS <= u8::MAX as usize);
const _: () = assert!(MAX_ENTRIES <= u16::MAX as usize);
const _: () = assert!(MAX_FIELD_BYTES <= u16::MAX as usize);
const _: () = assert!(MAX_LOCKED_BYTES <= u16::MAX as usize);
const _: () = assert!(MAX_PASSWORD_BYTES <= u16::MAX as usize);
// A sound piece is whole samples, each index fits its two bytes, 0 is free for no silence stop, and a `.part` name
// fits a path; `Spec::new` checks the other voice and fetch widths.
const _: () = assert!(MAX_SOUND_BYTES.is_multiple_of(SAMPLE_BYTES));
const _: () = assert!(MAX_SOUND_PIECES <= u16::MAX as usize);
const _: () = assert!(MAX_PIECES <= u16::MAX as usize);
const _: () = assert!(MAX_SPEECH_PIECES <= u16::MAX as usize);
const _: () = assert!(*SILENCE_STOP_S.start() > 0);
const _: () = assert!(PART_EXT.len() < MAX_PATH_BYTES);
// A service's reply holds the longest text it may yield, a run holds its longest piece's text, and the longest piece
// lies between the longest `piece_s` and the longest dictation.
const _: () = assert!(MAX_NET_REPLY_BYTES >= MAX_NET_TEXT_BYTES);
const _: () = assert!(MAX_NET_RUN_TEXT_BYTES >= MAX_NET_TEXT_BYTES);
const _: () =
    assert!(NET_PIECE_MAX_S <= *RECORD_MAX_S.end() && NET_PIECE_MAX_S >= *PIECE_S.end() as u16);

//! The protocol's and the backup box's magic, versions, names and limits, in one place.

use crate::vault::wire;

/// A kibibyte, the unit of the read chunk and of the locked identity's cap.
const KIB: usize = 1024;
/// A mebibyte, the unit of the image, decoder, box and vault job caps.
pub(crate) const MIB: usize = KIB * KIB;

/// The bytes every frame starts with.
pub(crate) const MAGIC: [u8; 4] = *b"KXIP";
/// The protocol version; a frame with any other is refused.
pub const VERSION: u8 = 1;
/// The fixed header: magic, version, kind, request id and payload length.
pub(crate) const HEADER_LEN: usize = MAGIC.len() + 2 * size_of::<u8>() + 2 * size_of::<u32>();

/// Bytes in one RGBA8 pixel.
pub const RGBA_BYTES: usize = 4;
/// The longest side, in pixels, that a draw may ask for and a reply may hold.
pub const MAX_SIDE: u32 = 4096;
/// The largest image a draw may carry: the largest clipboard entry the settings allow, 64 MiB.
pub const MAX_IMAGE_BYTES: usize = 64 * MIB;
/// The largest pixels a reply may carry: `MAX_SIDE` by `MAX_SIDE` in RGBA8, 64 MiB.
pub const MAX_RGBA_BYTES: usize = MAX_SIDE as usize * MAX_SIDE as usize * RGBA_BYTES;

/// A draw's fields before its image: the codec and the longest side.
pub(crate) const DRAW_HEAD: usize = size_of::<u8>() + size_of::<u32>();
/// A DIB ask's field before its PNG: the most bytes the DIB may hold.
pub(crate) const MAKE_HEAD: usize = size_of::<u32>();
/// The fewest image bytes a draw carries: an empty image is refused.
pub(crate) const MIN_IMAGE_BYTES: usize = 1;
/// A pixel reply's fields before its pixels: the width and the height.
pub(crate) const PIXELS_HEAD: usize = 2 * size_of::<u32>();
/// A failure reply's only field: the reason.
pub(crate) const FAILED_LEN: usize = size_of::<u8>();
/// The largest media ask payload: a draw's fields and the largest image.
pub const MAX_ASK_LEN: usize = DRAW_HEAD + MAX_IMAGE_BYTES;
/// The largest media reply payload: a pixel reply's fields and the largest pixels.
pub const MAX_REPLY_LEN: usize = PIXELS_HEAD + MAX_RGBA_BYTES;

/// The first buffer a payload read takes; it then doubles only as bytes arrive, up to the declared length.
pub(crate) const READ_CHUNK: usize = 64 * KIB;
/// The most times its length a payload read holds while its buffer grows: the old and new buffers together.
pub const READ_TIMES: usize = 2;
/// What the worker's PNG decoder may allocate for its rows and chunks: 16 MiB, rows up to 4 million pixels wide.
pub const DECODE_BYTES: usize = 16 * MIB;

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

// Every payload length fits the header's `u32` field, and so does the largest box.
const _: () = assert!(MAX_ASK_LEN <= u32::MAX as usize);
const _: () = assert!(MAX_REPLY_LEN <= u32::MAX as usize);
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
// A DIB ask and its reply fit within the largest payloads.
const _: () = assert!(MAKE_HEAD + MAX_IMAGE_BYTES <= MAX_ASK_LEN);
const _: () = assert!(MAX_IMAGE_BYTES <= MAX_REPLY_LEN);

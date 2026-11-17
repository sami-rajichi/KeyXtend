//! The vault frames' fields and their caps, on the shared field machinery in `crate::wire`.

use crate::limits::{
    MAX_BOX_BYTES, MAX_ENTRIES, MAX_ENTRIES_LEN, MAX_FIELD_BYTES, MAX_FIELDS, MAX_KEEPASS_BYTES,
    MAX_LOCKED_BYTES, MAX_PART_NAME, MAX_PARTS, MAX_PASSWORD_BYTES, MAX_RECIPIENT_BYTES,
};
pub(crate) use crate::wire::{FOUR, ONE, Payload, Put, Sink, Size, Spec, TWO};

/// A password: UTF-8, possibly empty.
pub(crate) const PASSWORD: Spec = Spec::new(TWO, 0, MAX_PASSWORD_BYTES);
/// An open padlock: an `age` recipient's text.
pub(crate) const RECIPIENT: Spec = Spec::new(ONE, 1, MAX_RECIPIENT_BYTES);
/// A locked identity.
pub(crate) const LOCKED: Spec = Spec::new(TWO, 1, MAX_LOCKED_BYTES);
/// A box's bytes.
pub(crate) const BYTES: Spec = Spec::new(FOUR, 1, MAX_BOX_BYTES);
/// A `KeePass` file's bytes.
pub(crate) const KEEPASS: Spec = Spec::new(FOUR, 1, MAX_KEEPASS_BYTES);
/// The count of a box's parts; a box holds at least one.
pub(crate) const PARTS: Spec = Spec::new(ONE, 1, MAX_PARTS);
/// A part's name, one of `BOX_PARTS`.
pub(crate) const NAME: Spec = Spec::new(ONE, 1, MAX_PART_NAME);
/// A part's plain bytes; all the parts together hold `MAX_BOX_BYTES` at most.
pub(crate) const PART: Spec = Spec::new(FOUR, 0, MAX_BOX_BYTES);
/// The count of `KeePass` entries.
pub(crate) const ENTRIES: Spec = Spec::new(TWO, 0, MAX_ENTRIES);
/// One field of an entry, as UTF-8.
pub(crate) const FIELD: Spec = Spec::new(TWO, 0, MAX_FIELD_BYTES);
/// The count of an entry's custom fields.
pub(crate) const CUSTOM: Spec = Spec::new(ONE, 0, MAX_FIELDS);
/// A count of what a `KeePass` import dropped.
pub(crate) const DROPPED: Spec = Spec::new(FOUR, 0, u32::MAX as usize);

/// A padlock: the recipient, then the locked identity.
pub(crate) const PADLOCK_LEAST: usize = RECIPIENT.least() + LOCKED.least();
pub(crate) const PADLOCK_MOST: usize = RECIPIENT.most() + LOCKED.most();
/// A padlock and a part list: each known part's name and length, and all the parts' bytes together.
pub(crate) const SEAL_LEAST: usize = PADLOCK_LEAST + PARTS.width + NAME.least() + PART.least();
pub(crate) const SEAL_MOST: usize =
    PADLOCK_MOST + PARTS.width + MAX_PARTS * (NAME.most() + PART.width) + MAX_BOX_BYTES;
/// A work factor, then a password.
pub(crate) const MAKE_LEAST: usize = ONE + PASSWORD.least();
pub(crate) const MAKE_MOST: usize = ONE + PASSWORD.most();
/// A password, then a box's bytes.
pub(crate) const OPEN_LEAST: usize = PASSWORD.least() + BYTES.least();
pub(crate) const OPEN_MOST: usize = PASSWORD.most() + BYTES.most();
/// A password, then a `KeePass` file's bytes.
pub(crate) const KEEPASS_LEAST: usize = PASSWORD.least() + KEEPASS.least();
pub(crate) const KEEPASS_MOST: usize = PASSWORD.most() + KEEPASS.most();
/// A password, then a locked identity.
pub(crate) const CHECK_LEAST: usize = PASSWORD.least() + LOCKED.least();
pub(crate) const CHECK_MOST: usize = PASSWORD.most() + LOCKED.most();
/// A `KeePass` reply: the dropped entries and fields, then the count and the entries, within `MAX_ENTRIES_LEN`.
pub(crate) const ENTRIES_LEAST: usize = 2 * DROPPED.width + ENTRIES.width;
pub(crate) const ENTRIES_MOST: usize = MAX_ENTRIES_LEN;

// Every vault payload fits the header's `u32` length.
const _: () = assert!(SEAL_MOST <= u32::MAX as usize);
const _: () = assert!(OPEN_MOST <= u32::MAX as usize);

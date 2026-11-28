//! The pick frames' fields and their caps, on the shared field machinery in `crate::wire`.

use crate::limits::snip::MAX_FILE_NAME_BYTES;
pub(crate) use crate::wire::PATH;
use crate::wire::{FOUR, ONE, Spec, TWO};

/// A proposed file name: one byte at least, within `MAX_FILE_NAME_BYTES`.
pub(crate) const NAME: Spec = Spec::new(TWO, 1, MAX_FILE_NAME_BYTES);
/// A Save ask: the file kind, the folder, then the proposed name.
pub(crate) const SAVE_LEAST: usize = ONE + PATH.least() + NAME.least();
pub(crate) const SAVE_MOST: usize = ONE + PATH.most() + NAME.most();
/// A dialog's failure: the system's code.
pub(crate) const FAILED_LEN: usize = FOUR;

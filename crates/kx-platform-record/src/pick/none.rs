//! Systems with no dialogs yet (P15): no dialog shows, so the app hears a failure and saves nothing.

use super::{Dialogs, FileKind, Picked, Unshown};

/// The code a system without dialogs fails with: `E_NOTIMPL`'s bits, "not built yet".
pub(super) const NOT_BUILT: u32 = 0x8000_4001;

/// Dialogs that never show.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoDialogs;

impl Dialogs for NoDialogs {
    fn save(&self, _: &str, _: &str, _: FileKind) -> Result<Picked, Unshown> {
        Err(Unshown { code: NOT_BUILT })
    }

    fn folder(&self, _: &str) -> Result<Picked, Unshown> {
        Err(Unshown { code: NOT_BUILT })
    }
}

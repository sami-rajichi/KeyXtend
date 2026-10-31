//! Setting a list aside (Ruling 26): a list that will not read at this version is kept in its folder as aside blobs,
//! which no clean or list read takes, and the folder's content stays as orphans until its day drops.

use std::collections::BTreeSet;

use kx_module_api::ports::{BlobName, SealedError};

use super::{Store, list};

impl Store {
    /// Keeps `bytes`, `folder`'s list that would not read, as its next aside, then removes the list blobs among
    /// `names`; from then on no load cleans the folder. A damaged head gives no bytes: its aside only marks the folder.
    ///
    /// # Errors
    /// The shelf's error when a piece would not save; the pieces saved are removed and the list stays.
    pub(super) fn set_aside(
        &mut self,
        folder: &BlobName,
        names: &BTreeSet<BlobName>,
        bytes: &[u8],
    ) -> Result<(), SealedError> {
        let last = names.iter().filter_map(list::aside_of).max();
        let n = last.map_or(1, |n| n.saturating_add(1));
        let mut pieces: Vec<&[u8]> = bytes.chunks(self.blob).collect();
        if pieces.is_empty() {
            pieces.push(&[]);
        }
        for (k, piece) in pieces.iter().enumerate() {
            let saved =
                list::aside_name(n, k).and_then(|name| self.shelf.save(folder, &name, piece));
            if let Err(e) = saved {
                for name in (0..k).filter_map(|k| list::aside_name(n, k).ok()) {
                    let _ = self.shelf.remove(folder, &name);
                }
                return Err(e);
            }
        }
        self.kept.insert(folder.clone());
        // The head sorts first; once it is gone, an extra blob whose removal failed is never read.
        for name in names.iter().filter(|n| list::is_list(n)) {
            let _ = self.shelf.remove(folder, name);
        }
        Ok(())
    }
}

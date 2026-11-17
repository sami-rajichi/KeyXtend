//! The shelf's folders and blob names: `texts/<id>`, `sounds/<id>-<n>` and `numbers/{counts,checks,scores}`. Ids are
//! plain decimal, so a name reads back to one number only.

use kx_module_api::ports::{BlobName, SealedError};

/// The folder of the texts, one blob each.
pub(super) const TEXTS: &str = "texts";
/// The folder of the recordings, each in blobs.
pub(super) const SOUNDS: &str = "sounds";
/// The folder of the numbers.
pub(super) const NUMBERS: &str = "numbers";
/// The counts' blob.
pub(super) const COUNTS: &str = "counts";
/// The card checks' blob.
pub(super) const CHECKS: &str = "checks";
/// Compare's results' blob.
pub(super) const SCORES: &str = "scores";
/// What parts a recording's id from its blob's number.
const SEP: char = '-';

/// The checked name `text`.
pub(super) fn name(text: &str) -> Result<BlobName, SealedError> {
    BlobName::new(text)
}

/// The blob of text `id`.
pub(super) fn text_name(id: u64) -> Result<BlobName, SealedError> {
    BlobName::new(&id.to_string())
}

/// Blob `n`, from 0, of text `id`'s recording.
pub(super) fn sound_name(id: u64, n: u32) -> Result<BlobName, SealedError> {
    BlobName::new(&format!("{id}{SEP}{n}"))
}

/// The text id `name` holds, written plainly.
pub(super) fn text_id(name: &BlobName) -> Option<u64> {
    plain(name.as_str())
}

/// The text id and blob number `name` holds, when it names a recording's blob.
pub(super) fn sound_of(name: &BlobName) -> Option<(u64, u32)> {
    let (id, n) = name.as_str().split_once(SEP)?;
    Some((plain(id)?, plain(n)?))
}

/// The number `text` writes in plain decimal, with no sign or leading zero.
fn plain<T: std::str::FromStr + ToString>(text: &str) -> Option<T> {
    let n: T = text.parse().ok()?;
    (n.to_string() == text).then_some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_read_back_to_their_numbers_only() {
        let text = text_name(u64::MAX).unwrap();
        assert_eq!(text_id(&text), Some(u64::MAX));
        let sound = sound_name(u64::MAX, u32::MAX).unwrap();
        assert_eq!(sound_of(&sound), Some((u64::MAX, u32::MAX)));
        assert_eq!(sound_of(&text), None);
        for odd in ["07", "1-01", "1-", "-1", "counts", "1-2-3"] {
            let odd = name(odd).unwrap();
            assert!(
                text_id(&odd).is_none() && sound_of(&odd).is_none(),
                "{odd:?}"
            );
        }
    }
}

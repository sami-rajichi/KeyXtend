//! Properties: random file bytes never open or panic, and the AAD names exactly one folder and blob.

use super::*;
use crate::consts::MARK;
use kx_module_api::ports::{MAX_BLOB_LEN, MIN_BLOB_LEN};
use proptest::prelude::*;

/// Cases per file property; each one writes files, so fewer than the default.
const CASES: u32 = 64;
/// The longest random file a case writes.
const MAX_JUNK_BYTES: usize = 256;

/// A text `BlobName` takes: lower-case letters, digits and dashes within its length limits.
fn text() -> impl Strategy<Value = String> {
    let pattern = format!("[a-z0-9-]{{{MIN_BLOB_LEN},{MAX_BLOB_LEN}}}");
    prop::string::string_regex(&pattern).unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_file_bytes_never_open(
        junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
        good_head in any::<bool>(),
    ) {
        let c = case();
        c.shelf.save(&name(DAY), &name("e1"), b"words").unwrap();
        let bytes = if good_head { [MARK.as_slice(), &junk].concat() } else { junk };
        fs::write(c.file(DAY, "e1", SEALED_EXT), &bytes).unwrap();
        prop_assert_eq!(c.shelf.load(&name(DAY), &name("e1")), Err(SealedError::Damaged));
        prop_assert_eq!(c.shelf.load(&name(DAY), &name("e1")), Ok(None));
    }
}

proptest! {
    #[test]
    fn the_aad_names_one_folder_and_blob(a in text(), b in text(), cut in any::<prop::sample::Index>()) {
        // Another split of the same letters is the case a plain join of both names would confuse.
        let joined = format!("{a}{b}");
        let (x, y) = joined.split_at(1 + cut.index(joined.len() - 1));
        prop_assume!(BlobName::new(x).is_ok() && BlobName::new(y).is_ok());
        let same = (a.as_str(), b.as_str()) == (x, y);
        prop_assert_eq!(aad(&name(&a), &name(&b)) == aad(&name(x), &name(y)), same);
    }
}

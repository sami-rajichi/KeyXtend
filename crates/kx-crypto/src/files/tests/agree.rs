//! The real store and the fake agree on names: `BlobName` gates both, so both keep every name it builds, and no device
//! or ill-formed name ever reaches either.

use super::*;
use kx_module_api::ports::RESERVED_NAMES;
use kx_platform_fake::FakeSealedStore;

/// Names both stores keep, a device name's lookalikes among them.
const GOOD: [&str; 4] = ["predict-en", "emails", "con-1", "lpt10"];
/// Names of the wrong form.
const BAD: [&str; 4] = ["", "Predict-en", "a/b", "CON"];

#[test]
fn the_fake_and_the_real_store_keep_the_same_names() {
    let c = case();
    let fake = FakeSealedStore::new();
    let texts = GOOD.iter().chain(&BAD).chain(RESERVED_NAMES);
    let names: Vec<BlobName> = texts.filter_map(|t| BlobName::new(t).ok()).collect();
    assert_eq!(names.len(), GOOD.len(), "only the good names build");
    let stores: [&dyn SealedStore; 2] = [&c.store, &fake];
    for name in &names {
        let bytes = name.as_str().as_bytes();
        for store in stores {
            store.save(name, bytes).unwrap();
            assert_eq!(store.load(name), Ok(Some(bytes.to_vec())), "{name:?}");
        }
    }
    let mut kept: Vec<_> = GOOD.iter().map(|n| format!("{n}.{SEALED_EXT}")).collect();
    kept.push(KEY_FILE.to_owned());
    kept.sort();
    assert_eq!(c.names(), kept);
    assert_eq!(fake.saves(), GOOD.len());
}

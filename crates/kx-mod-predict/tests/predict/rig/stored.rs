//! The sealed blobs the tests plant and read back: learned words as an earlier run saved them.

use super::{DAY, Fakes};
use kx_mod_predict::PredictSettings;
use kx_mod_predict::learn::Learner;
use kx_module_api::ports::BlobName;

/// The blob of language `iso`'s learned words.
pub fn blob(iso: &str) -> BlobName {
    kx_mod_predict::learned_blob(iso).unwrap()
}

/// A learner of `iso` that learned each of `words` on `day` as often as the default settings need to offer it.
pub fn learner(iso: &str, words: &[&str], day: u32) -> Learner {
    let cfg = PredictSettings::from_defaults().unwrap().learn_config();
    let mut l = Learner::new(iso, cfg).unwrap();
    for word in words {
        for _ in 0..cfg.min_uses {
            l.learn(None, word, &[], day);
        }
    }
    l
}

/// Plants a learner of `iso` offering each of `words`, as an earlier run saved it on `DAY`.
pub fn plant(fakes: &Fakes, iso: &str, words: &[&str]) {
    let bytes = learner(iso, words, DAY).to_bytes();
    fakes.store.plant(&blob(iso), bytes);
}

/// The learner saved for `iso`, read back with the default settings.
pub fn saved(fakes: &Fakes, iso: &str) -> Option<Learner> {
    let cfg = PredictSettings::from_defaults().unwrap().learn_config();
    let bytes = fakes.store.blob(&blob(iso))?;
    Learner::from_bytes(&bytes, cfg).ok()
}

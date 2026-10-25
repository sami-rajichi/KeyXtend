//! Held blobs: one that opens but does not read is set aside once and never written over, and one whose load fails
//! with `Io` is never written that run, so no learned word or email is ever wiped.

use crate::rig::{DAY, Rig, blob, learner, plant};
use kx_mod_predict::email::{self, Emails};
use kx_mod_predict::{aside_blob, emails_blob, learn};
use kx_module_api::keys;
use kx_module_api::ports::{BlobName, FieldKind, SealedError};
use std::time::Duration;

/// `bytes` as a newer build would write them: the version after `magic` raised past `version`.
fn newer(mut bytes: Vec<u8>, magic: usize, version: u16) -> Vec<u8> {
    let next = (version + 1).to_le_bytes();
    bytes.splice(magic..magic + next.len(), next);
    bytes
}

/// A rig in English, in a text box, that typed `text`, waited past `save_s`, switched language and stopped.
fn typed_and_stopped(setup: impl FnOnce(&crate::rig::Fakes), text: &str) -> Rig {
    let mut rig = Rig::build(|f, _| setup(f));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.typed(text);
    rig.advance(Duration::from_secs(rig.settings.save_s.into()));
    rig.tick();
    rig.lang("fr");
    rig.stop();
    rig
}

/// Asserts `name` still holds `planted` and its aside copy holds `aside`.
fn kept(rig: &Rig, name: &BlobName, planted: &[u8], aside: &[u8]) {
    let store = &rig.fakes.store;
    assert_eq!(
        store.blob(name).as_deref(),
        Some(planted),
        "never written over"
    );
    let copy = store.blob(&aside_blob(name).unwrap());
    assert_eq!(copy.as_deref(), Some(aside), "set aside");
    assert!(rig.notices().contains(&keys::WORDS_DAMAGED));
}

#[test]
fn a_learner_of_a_newer_format_or_another_language_is_set_aside_and_never_written_over() {
    let en = learner("en", &["hello"], DAY).to_bytes();
    let newer = newer(en, learn::MAGIC.len(), learn::VERSION);
    let other = learner("fr", &["bonjour"], DAY).to_bytes();
    for planted in [newer, other] {
        let bytes = planted.clone();
        let rig = typed_and_stopped(|f| f.store.plant(&blob("en"), bytes), "hello again ");
        kept(&rig, &blob("en"), &planted, &planted);
    }
}

#[test]
fn emails_of_a_newer_format_are_set_aside_and_never_written_over() {
    let known = Emails::new(&["sami@x.org".to_owned()], usize::MAX).to_bytes();
    let planted = newer(known, email::MAGIC.len(), email::VERSION);
    let name = emails_blob().unwrap();
    let (at, bytes) = (name.clone(), planted.clone());
    let rig = typed_and_stopped(|f| f.store.plant(&at, bytes), "nour@y.com ");
    kept(&rig, &name, &planted, &planted);
}

#[test]
fn an_earlier_aside_copy_is_kept() {
    let planted = newer(
        learner("en", &[], DAY).to_bytes(),
        learn::MAGIC.len(),
        learn::VERSION,
    );
    let earlier = b"an earlier copy".to_vec();
    let (bytes, old) = (planted.clone(), earlier.clone());
    let rig = typed_and_stopped(
        |f| {
            f.store.plant(&blob("en"), bytes);
            f.store.plant(&aside_blob(&blob("en")).unwrap(), old);
        },
        "hello ",
    );
    kept(&rig, &blob("en"), &planted, &earlier);
}

#[test]
fn learned_words_whose_load_fails_with_io_are_never_written_that_run() {
    let planted = learner("en", &["hello"], DAY).to_bytes();
    let mut rig = Rig::build(|f, _| plant(f, "en", &["hello"]));
    rig.fakes.store.fail_load(SealedError::Io);
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.typed("again ");
    rig.advance(Duration::from_secs(rig.settings.save_s.into()));
    rig.tick();
    rig.stop();
    assert_eq!(rig.fakes.store.saves(), 0);
    assert_eq!(rig.fakes.store.blob(&blob("en")), Some(planted));
}

#[test]
fn emails_whose_load_fails_with_io_are_never_written_that_run() {
    let planted = Emails::new(&["sami@x.org".to_owned()], usize::MAX).to_bytes();
    let name = emails_blob().unwrap();
    let (at, bytes) = (name.clone(), planted.clone());
    let rig = typed_and_stopped(
        |f| {
            f.store.plant(&at, bytes);
            f.store.fail_load(SealedError::Io);
        },
        "nour@y.com ",
    );
    assert_eq!(rig.fakes.store.saves(), 0);
    assert_eq!(rig.fakes.store.blob(&name), Some(planted));
}

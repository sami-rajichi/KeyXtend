//! Saving: a changed blob saves `save_s` after its first change and at stop; a damaged blob starts empty with one
//! notice; a locked store starts empty and saves nothing that run; a refused save keeps running; a new day purges;
//! while learning is paused, neither purges nor timed saves write.

use crate::rig::{DAY, Rig, blob, learner, plant, saved};
use kx_mod_predict::score::Learned;
use kx_module_api::keys;
use kx_module_api::ports::{FieldKind, SealedError, WordsAsk};
use std::time::Duration;

/// One millisecond, the clock's step past a deadline.
const MS: Duration = Duration::from_millis(1);

/// A rig in English, in a text box, with `setup` done first.
fn english(setup: impl FnOnce(&crate::rig::Fakes)) -> Rig {
    let mut rig = Rig::build(|f, _| setup(f));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

#[test]
fn a_change_saves_save_s_after_it_and_not_before() {
    let mut rig = english(|_| {});
    let save = Duration::from_secs(rig.settings.save_s.into());
    assert_eq!(rig.wait(), None, "nothing to save, no timer");
    rig.typed("hello ");
    assert_eq!(rig.wait(), Some(save));
    rig.advance(save.checked_sub(MS).unwrap());
    rig.typed("again ");
    rig.tick();
    assert_eq!(
        (rig.fakes.store.saves(), rig.wait()),
        (0, Some(MS)),
        "the first change sets the time"
    );
    rig.advance(MS);
    rig.tick();
    assert_eq!(rig.fakes.store.saves(), 1);
    assert_eq!(rig.wait(), None);
    assert_eq!(
        saved(&rig.fakes, "en").unwrap().view(None).count("again"),
        1
    );
}

#[test]
fn a_stop_saves_what_changed_and_nothing_else() {
    let mut rig = english(|_| {});
    rig.stop();
    assert_eq!(rig.fakes.store.saves(), 0);
    let mut rig = english(|_| {});
    rig.typed("hello ");
    rig.stop();
    assert_eq!(rig.fakes.store.saves(), 1);
    assert!(rig.fakes.store.blob(&blob("en")).is_some());
}

#[test]
fn a_damaged_blob_starts_empty_and_tells_once() {
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", &["hello"]);
        f.store.plant(&blob("fr"), b"KXL1 broken".to_vec());
    });
    rig.fakes.store.fail_load(SealedError::Damaged);
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.typed("hel");
    assert!(rig.words().is_empty(), "the damaged learner starts empty");
    rig.lang("fr");
    assert_eq!(rig.notices(), [keys::WORDS_DAMAGED], "once for the run");
}

#[test]
fn a_bad_byte_form_or_another_languages_file_is_damaged() {
    let mut rig = Rig::build(|f, _| {
        f.store
            .plant(&blob("en"), learner("fr", &["bonjour"], DAY).to_bytes());
    });
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.typed("bon");
    assert!(rig.words().is_empty());
    assert!(rig.notices().contains(&keys::WORDS_DAMAGED));
}

#[test]
fn a_locked_store_starts_empty_tells_once_and_saves_nothing_that_run() {
    let mut rig = Rig::build(|f, _| f.store.fail_load(SealedError::Locked));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.typed("hello sami@x.org ");
    rig.advance(Duration::from_secs(rig.settings.save_s.into()));
    rig.tick();
    rig.lang("fr");
    rig.stop();
    assert_eq!(rig.fakes.store.saves(), 0);
    assert_eq!(
        rig.notices()
            .iter()
            .filter(|k| **k == keys::WORDS_LOCKED)
            .count(),
        1
    );
}

#[test]
fn a_refused_save_keeps_running_and_tries_again_after_save_s() {
    let mut rig = english(|_| {});
    let save = Duration::from_secs(rig.settings.save_s.into());
    rig.typed("hello ");
    rig.fakes.store.fail_save(SealedError::Io);
    rig.advance(save);
    rig.tick();
    assert_eq!(rig.fakes.store.saves(), 1, "tried");
    assert_eq!(rig.wait(), Some(save), "tries again later");
    rig.advance(save);
    rig.tick();
    assert_eq!(rig.fakes.store.saves(), 2);
    assert!(rig.fakes.store.blob(&blob("en")).is_some());
}

#[test]
fn a_switch_whose_save_fails_keeps_the_words_and_saves_them_save_s_later() {
    let mut rig = english(|_| {});
    let save = Duration::from_secs(rig.settings.save_s.into());
    rig.typed("hello ");
    rig.fakes.store.fail_save(SealedError::Io);
    rig.lang("fr");
    assert_eq!(rig.wait(), Some(save), "tries again later");
    rig.advance(save);
    rig.tick();
    assert_eq!(rig.fakes.store.saves(), 2);
    assert_eq!(
        saved(&rig.fakes, "en").unwrap().view(None).count("hello"),
        1
    );
    assert_eq!(rig.wait(), None);
}

#[test]
fn words_whose_switch_save_failed_come_back_with_their_language_and_save_at_stop() {
    let mut rig = english(|_| {});
    rig.typed("hello ");
    rig.fakes.store.fail_save(SealedError::Io);
    rig.lang("fr");
    rig.lang("en");
    rig.typed("hel");
    assert_eq!(
        rig.words(),
        Vec::<String>::new(),
        "one use, not offered yet"
    );
    rig.typed("lo ");
    rig.stop();
    assert_eq!(
        saved(&rig.fakes, "en").unwrap().view(None).count("hello"),
        2
    );
    let mut rig = english(|_| {});
    rig.typed("hello ");
    rig.fakes.store.fail_save(SealedError::Io);
    rig.lang("fr");
    rig.stop();
    assert_eq!(
        saved(&rig.fakes, "en").unwrap().view(None).count("hello"),
        1,
        "at stop"
    );
}

#[test]
fn a_locked_save_stops_saving_for_the_run() {
    let mut rig = english(|_| {});
    rig.typed("hello ");
    rig.fakes.store.fail_save(SealedError::Locked);
    rig.advance(Duration::from_secs(rig.settings.save_s.into()));
    rig.tick();
    rig.typed("again ");
    rig.stop();
    assert_eq!(rig.fakes.store.saves(), 1);
    assert_eq!(rig.notices(), [keys::WORDS_LOCKED]);
}

#[test]
fn a_paused_learner_neither_purges_nor_saves_on_time_until_it_resumes() {
    let mut rig = english(|f| plant(f, "en", &["old"]));
    let save = Duration::from_secs(rig.settings.save_s.into());
    rig.typed("hello ");
    rig.ask(WordsAsk::Pause(true));
    assert_eq!(rig.wait(), None, "no timer while paused");
    rig.set_day(DAY + rig.settings.forget_days + 1);
    rig.advance(save);
    rig.tick();
    assert_eq!(rig.fakes.store.saves(), 0, "no purge, no timed save");
    rig.ask(WordsAsk::Pause(false));
    assert_eq!(rig.fakes.store.saves(), 1, "the save was due");
    let learner = saved(&rig.fakes, "en").unwrap();
    assert_eq!(learner.view(None).count("old"), 0, "purged on resume");
}

#[test]
fn a_forget_while_paused_waits_for_the_stop() {
    let mut rig = english(|f| plant(f, "en", &["hello", "help"]));
    rig.ask(WordsAsk::Pause(true));
    rig.typed("hel");
    rig.ask(WordsAsk::Forget);
    rig.pick(0);
    rig.advance(Duration::from_secs(rig.settings.save_s.into()));
    rig.tick();
    assert_eq!(rig.fakes.store.saves(), 0, "no timed save");
    rig.stop();
    let learner = saved(&rig.fakes, "en").unwrap();
    assert_eq!(learner.view(None).count("hello"), 0, "the stop saves it");
}

#[test]
fn a_new_day_purges_what_went_unused_too_long() {
    let mut rig = english(|f| plant(f, "en", &["old"]));
    let days = rig.settings.forget_days;
    rig.set_day(DAY + days);
    rig.tick();
    rig.stop();
    assert_eq!(rig.fakes.store.saves(), 0, "still within forget_days");
    let mut rig = english(|f| plant(f, "en", &["old"]));
    rig.set_day(DAY + days + 1);
    rig.tick();
    rig.stop();
    assert_eq!(saved(&rig.fakes, "en").unwrap().view(None).count("old"), 0);
}

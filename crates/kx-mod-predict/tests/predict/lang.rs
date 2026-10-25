//! The language in front: a switch saves the old learner and loads the new one, only that language is opened, and a
//! missing full pack is told once per language and run.

use crate::rig::{Rig, blob, plant, saved};
use kx_mod_predict::Flow;
use kx_mod_predict::score::Learned;
use kx_module_api::keys;
use kx_module_api::ports::FieldKind;

#[test]
fn a_switch_saves_the_old_learner_and_loads_the_new_one() {
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", &["hello"]);
        plant(f, "fr", &["bonjour"]);
    });
    assert_eq!(rig.lang("en"), Flow::Switch);
    rig.focus(FieldKind::Text);
    rig.typed("hello ");
    let loads = rig.fakes.store.loads();
    rig.lang("fr");
    assert_eq!(
        rig.fakes.store.saves(),
        1,
        "the English learner saved at once"
    );
    let en = saved(&rig.fakes, "en").unwrap();
    assert_eq!(en.view(None).count("hello"), 3);
    assert_eq!(
        rig.fakes.store.loads(),
        loads + 1,
        "the French learner loaded"
    );
    rig.typed("bon");
    assert_eq!(rig.words(), ["bonjour"]);
    rig.typed("jour ");
    assert_eq!(rig.lang("fr"), Flow::Go, "the same language stays");
    rig.stop();
    let fr = saved(&rig.fakes, "fr").unwrap();
    assert_eq!(fr.view(None).count("bonjour"), 3);
}

#[test]
fn a_switch_mid_word_ends_it_unlearned() {
    let mut rig = Rig::build(|f, _| plant(f, "en", &["hello"]));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.typed("hello");
    rig.lang("fr");
    assert_eq!(rig.fakes.store.saves(), 0, "nothing changed, nothing saved");
    rig.typed(" ");
    rig.stop();
    assert!(
        rig.fakes.store.blob(&blob("fr")).is_none(),
        "no word ended in French"
    );
}

#[test]
fn only_the_language_in_front_is_opened_once_per_switch() {
    let mut rig = Rig::new();
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.typed("hello world ");
    assert_eq!(rig.fakes.packs.opens("en"), 1);
    assert_eq!(rig.fakes.packs.opens("fr"), 0);
    rig.lang("fr");
    rig.typed("bonjour ");
    assert_eq!(
        (rig.fakes.packs.opens("en"), rig.fakes.packs.opens("fr")),
        (1, 1)
    );
}

#[test]
fn a_missing_shipped_pack_is_told_once_per_language_and_an_unlisted_language_never() {
    let mut rig = Rig::build(|f, _| {
        for iso in ["en", "fr"] {
            f.packs.ship(iso);
        }
    });
    rig.lang("en");
    rig.lang("de");
    rig.lang("fr");
    rig.lang("en");
    let missing: Vec<&str> = rig.notices();
    assert_eq!(missing, [keys::WORDS_MISSING; 2]);
}

#[test]
fn the_missing_notice_names_its_language_by_code() {
    use crate::rig::Heard;
    use kx_module_api::{NoticeArg, args};
    let mut rig = Rig::build(|f, _| f.packs.ship("ar"));
    rig.lang("ar");
    let notes: Vec<_> = rig
        .take()
        .into_iter()
        .filter_map(|h| match h {
            Heard::Notice(n) => Some(n),
            _ => None,
        })
        .collect();
    let lang = notes
        .first()
        .and_then(|n| n.args.iter().find(|a| a.0 == args::LANG));
    assert_eq!(
        lang.map(|a| a.1.clone()),
        Some(NoticeArg::Text("ar".to_owned()))
    );
}

#[test]
fn the_actor_asks_for_the_layout_when_it_begins() {
    use crate::rig::Heard;
    let rig = Rig::new();
    assert!(rig.take().contains(&Heard::Ask));
}

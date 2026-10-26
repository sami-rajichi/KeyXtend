//! The packs: chips come from the language in front's pack, a learner's strays merge into the pack's spelling when the
//! pack loads, and a pack that does not read, or reads as another language, counts as missing.

#![cfg(feature = "build")]

use crate::rig::{Rig, plant, saved};
use kx_mod_predict::score::Learned;
use kx_mod_predict::{Flags, PackWriter};
use kx_module_api::keys;
use kx_module_api::ports::FieldKind;

/// The followers each test pack keeps per context.
const FOLLOWERS: u8 = 8;
/// A frequency byte for each test word.
const FREQ: u8 = 150;

/// A pack of `iso` holding `words`, each as frequent.
fn pack(iso: &str, words: &[&str]) -> Vec<u8> {
    let mut writer = PackWriter::new(iso, Flags::default(), FOLLOWERS).unwrap();
    for word in words {
        writer.add_word(word, FREQ).unwrap();
    }
    writer.finish().unwrap()
}

#[test]
fn chips_come_from_the_pack_in_its_spelling() {
    let mut rig = Rig::build(|f, _| f.packs.add("fr", pack("fr", &["école", "écoute"])));
    rig.lang("fr");
    rig.focus(FieldKind::Text);
    rig.typed("eco");
    assert_eq!(rig.words(), ["école", "écoute"]);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(3, "école ".to_owned())]);
    assert!(rig.notices().is_empty());
}

#[test]
fn a_learners_stray_merges_into_the_packs_spelling_when_the_pack_loads() {
    let mut rig = Rig::build(|f, _| {
        f.packs.add("fr", pack("fr", &["école"]));
        plant(f, "fr", &["ecole"]);
    });
    rig.lang("fr");
    rig.focus(FieldKind::Text);
    rig.typed("ec");
    assert_eq!(rig.words(), ["école"], "one chip, in the pack's spelling");
}

#[test]
fn a_pack_that_does_not_read_or_names_another_language_counts_as_missing() {
    let mut rig = Rig::build(|f, _| {
        for iso in ["en", "fr"] {
            f.packs.ship(iso);
        }
        f.packs.add("en", b"not a pack".to_vec());
        f.packs.add("fr", pack("ar", &["من"]));
    });
    rig.lang("en");
    rig.lang("fr");
    assert_eq!(rig.notices(), [keys::WORDS_MISSING; 2]);
}

#[test]
fn a_pack_that_cannot_be_opened_counts_as_missing() {
    let mut rig = Rig::build(|f, _| {
        f.packs.ship("fr");
        f.packs.fail_open();
    });
    rig.lang("fr");
    assert_eq!(rig.notices(), [keys::WORDS_MISSING]);
}

#[test]
fn a_croatian_layout_takes_the_serbo_croatian_pack_and_keeps_its_own_learned_file() {
    let mut rig = Rig::build(|f, _| {
        f.packs.ship("sh");
        f.packs.alias("sh", "hr");
        f.packs.add("sh", pack("sh", &["hvala", "hvatati"]));
    });
    rig.lang("hr");
    rig.focus(FieldKind::Text);
    rig.typed("hva");
    assert_eq!(rig.words(), ["hvala", "hvatati"], "the pack's words");
    assert!(rig.notices().is_empty(), "nothing is missing");
    rig.typed("la ");
    rig.stop();
    assert_eq!(
        rig.fakes.packs.opens("hr"),
        0,
        "no file of the layout's code"
    );
    let hr = saved(&rig.fakes, "hr").unwrap();
    assert_eq!(
        hr.view(None).count("hvala"),
        1,
        "learned under the layout's code"
    );
}

#[test]
fn a_shipped_alias_whose_pack_is_gone_is_told_missing_and_serbian_never() {
    let mut rig = Rig::build(|f, _| {
        f.packs.ship("sh");
        f.packs.alias("sh", "bs");
    });
    rig.lang("bs");
    rig.lang("sr");
    assert_eq!(rig.notices(), [keys::WORDS_MISSING], "Bosnian only");
}

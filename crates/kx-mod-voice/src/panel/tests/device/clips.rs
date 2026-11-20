//! The clip a card's check reads through the panel (ADR-0052): the language in front's when its model knows it, else
//! the first shipped clip that model knows.

use kx_module_api::ports::{VoiceClick, VoiceTab};

use super::super::super::rig::{Rig, card, row};
use super::check_card;

/// The rig on the Device page with `id`'s file in the folder, a roomy card and `iso` in front.
fn on_device(id: &str, iso: &str) -> Rig {
    let mut rig = Rig::foldered();
    rig.devices.push(card(4_096));
    rig.holds(&[row(id)]);
    rig.front.iso = iso.into();
    rig.click(&VoiceClick::Open(VoiceTab::Device));
    rig
}

/// The clip a check reads, with the model's code the ask names and the clip the ask carries.
fn clip_of(rig: &mut Rig) -> (String, String, String) {
    let check = check_card(rig);
    (check.clip, check.ask.model.lang, check.ask.clip)
}

/// What a check on the clip of `iso` carries.
fn on(iso: &str) -> (String, String, String) {
    (iso.into(), iso.into(), iso.into())
}

#[test]
fn arabic_in_front_checks_its_model_on_the_arabic_clip_not_english() {
    let mut rig = on_device("cohere-arabic-q8", "ar");
    assert_eq!(clip_of(&mut rig), on("ar"), "it knows English first");
}

#[test]
fn french_in_front_checks_parakeet_on_the_french_clip() {
    let mut rig = on_device("parakeet-v3-q8", "fr");
    assert_eq!(clip_of(&mut rig), on("fr"));
}

#[test]
fn a_language_without_a_clip_checks_on_the_first_clip_the_model_knows() {
    let mut rig = on_device("parakeet-v3-q8", "de");
    rig.clips = vec!["ar".into(), "fr".into(), "en".into()];
    assert_eq!(clip_of(&mut rig), on("fr"), "Parakeet knows no Arabic");
}

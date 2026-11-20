//! The clip a card's check reads through the panel (ADR-0052): the language in front's when its model knows it, else
//! the first shipped clip that model knows; its result counts for that clip's language alone.

use kx_module_api::ports::{
    CardCheck, CardId, Date, DeviceItem, Hundredths, Verdict, VoiceClick, VoiceTab,
};

use super::super::super::rig::{Rig, card, row};
use super::{Do, Panel, check_card, device, passed};
use crate::counts::Checked;

const PASS: Verdict = Verdict::Passed {
    gain: Hundredths(300),
};

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

/// Parakeet's check on the card under its driver, kept in `lang`.
pub(in crate::panel::tests) fn parakeet_in(lang: Option<&str>) -> Checked {
    Checked {
        card: CardId::new("Vulkan0").unwrap(),
        driver: "1.0".into(),
        model: row("parakeet-v3-q8").print,
        lang: lang.map(str::to_owned),
        verdict: PASS,
        day: Date::new(2026, 11, 20).unwrap(),
    }
}

/// The clip of the one check `recheck` asks for.
fn rechecked(rig: &mut Rig) -> String {
    let out = rig.with(Panel::recheck);
    let [Do::Check(check)] = &out[..] else {
        panic!("{out:?}");
    };
    check.clip.clone()
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

#[test]
fn edge_1_a_result_is_kept_for_its_clips_language_though_the_front_changed() {
    let mut rig = on_device("cohere-arabic-q8", "ar");
    let check = check_card(&mut rig);
    rig.front.iso = "en".into();
    let out = rig.with(|p, f| p.checked(&check, Ok(passed()), f));
    let [Do::Checked(checked)] = &out[..] else {
        panic!("{out:?}");
    };
    assert_eq!(checked.lang.as_deref(), Some("ar"));
}

#[test]
fn the_cards_state_is_its_result_for_the_language_in_front() {
    let mut rig = on_device("parakeet-v3-q8", "en");
    rig.checks.put(parakeet_in(Some("en")));
    let state = |rig: &mut Rig| match device(rig).rows.swap_remove(2).item {
        DeviceItem::Card { check, .. } => check,
        other => panic!("{other:?}"),
    };
    assert_eq!(state(&mut rig), CardCheck::Done(PASS));
    rig.front.iso = "fr".into();
    assert_eq!(
        state(&mut rig),
        CardCheck::Unchecked,
        "French has no result"
    );
}

#[test]
fn a_picked_card_checked_in_english_is_checked_again_for_french() {
    let mut rig = on_device("parakeet-v3-q8", "en");
    rig.settings.device = "Vulkan0".into();
    rig.checks.put(parakeet_in(Some("en")));
    assert!(rig.with(Panel::recheck).is_empty(), "English is checked");
    rig.front.iso = "fr".into();
    assert_eq!(rechecked(&mut rig), "fr");
}

#[test]
fn automatic_checks_again_a_card_whose_old_result_names_no_language() {
    let mut rig = on_device("parakeet-v3-q8", "en");
    rig.checks.put(parakeet_in(None));
    assert_eq!(rechecked(&mut rig), "en");
}

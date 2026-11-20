//! Automatic's checks on the actor: a card that passed one model's check under its present driver is checked for each
//! newly used model, in turn and one at a time; a card that passed none, or failed, is never checked by a switch.

use kx_module_api::ports::{
    CardId, Date, Device, Hundredths, PanelButton, SpeechError, Verdict, VoiceClick,
};

use super::Msg;
use super::cards::{
    CARD, CARD2, arabic, arabic_layout, build, card_id, checked, devices, dictate_past_arabic,
    files, finish, passed, served, start_compare, two_cards, written,
};
use super::rig::{Rig, english, english_layout};
use super::taps::{held, waiting};
use crate::counts::Checked;
use crate::models::Row;

/// The rig's cards' driver now.
const DRIVER: &str = "1.0";
/// A driver before an update.
const OLD: &str = "0.9";
/// A passed check's verdict.
const PASS: Verdict = Verdict::Passed {
    gain: Hundredths(300),
};

/// A check an earlier run kept: card `card` under `driver` for `row`'s file, on the clip of `lang`.
pub(super) fn kept(card: &str, driver: &str, row: &Row, lang: &str, verdict: Verdict) -> Checked {
    Checked {
        card: CardId::new(card).unwrap(),
        driver: driver.into(),
        model: row.print,
        lang: Some(lang.into()),
        verdict,
        day: Date::new(2026, 11, 14).unwrap(),
    }
}

/// A begun rig on Automatic over `devices`, with `kept` from an earlier run and each check passing.
fn auto(kept: Vec<Checked>, devices: Vec<Device>) -> Rig {
    build("auto", Some(devices), Ok(passed()), kept)
}

/// The rig on Automatic with one card that passed English's model before.
fn passed_english() -> Rig {
    auto(vec![kept(CARD, DRIVER, english(), "en", PASS)], devices())
}

/// The card each check was asked on, in order.
fn cards(rig: &Rig) -> Vec<String> {
    let asks = rig.fake.speech.check_asks();
    asks.iter().map(|a| a.card.id.as_str().to_owned()).collect()
}

#[test]
fn automatic_checks_a_new_model_on_a_card_that_passed_another_then_dictates_on_it() {
    let mut rig = passed_english();
    assert!(checked(&rig).is_empty(), "English's model passed before");
    rig.send(Msg::Layout(arabic_layout()));
    assert_eq!(checked(&rig), [files().1]);
    assert_eq!(cards(&rig), [CARD]);
    rig.listen();
    assert_eq!(served(&rig), Some(card_id()));
}

#[test]
fn automatic_checks_only_the_card_that_passed_before() {
    let mut rig = auto(
        vec![kept(CARD2, DRIVER, english(), "en", PASS)],
        two_cards(),
    );
    rig.send(Msg::Layout(arabic_layout()));
    assert_eq!(cards(&rig), [CARD2]);
}

#[test]
fn automatic_checks_each_card_that_passed_in_turn_one_at_a_time() {
    let rows = [CARD, CARD2].map(|c| kept(c, DRIVER, english(), "en", PASS));
    let mut rig = auto(rows.to_vec(), two_cards());
    rig.actor.ports.spawn = held;
    rig.send(Msg::Layout(arabic_layout()));
    finish(&mut rig);
    assert_eq!(cards(&rig), [CARD], "one check at a time");
    assert!(waiting(), "the second card's check follows");
    finish(&mut rig);
    assert_eq!(cards(&rig), [CARD, CARD2]);
    assert!(!waiting());
}

#[test]
fn automatic_never_checks_a_new_model_on_a_card_whose_check_failed() {
    let english_failed = vec![kept(CARD, DRIVER, english(), "en", Verdict::Failed)];
    let arabic_failed = vec![
        kept(CARD, DRIVER, english(), "en", PASS),
        kept(CARD, DRIVER, arabic(), "ar", Verdict::Failed),
    ];
    for rows in [english_failed, arabic_failed] {
        let mut rig = auto(rows, devices());
        for layout in [arabic_layout(), english_layout(), arabic_layout()] {
            rig.send(Msg::Layout(layout));
        }
        assert!(checked(&rig).is_empty());
        rig.listen();
        assert_eq!(served(&rig), None);
    }
}

#[test]
fn after_a_driver_update_the_model_checked_before_goes_first_and_only_a_new_pass_counts() {
    let (en, ar) = files();
    let old = vec![kept(CARD, OLD, english(), "en", PASS)];
    let fault = Err(SpeechError::Timeout);
    let mut rig = build("auto", Some(devices()), fault, old.clone());
    rig.send(Msg::Layout(arabic_layout()));
    assert_eq!(checked(&rig), [en], "an old pass checks no new model");
    let mut rig = auto(old, devices());
    rig.send(Msg::Layout(arabic_layout()));
    assert_eq!(checked(&rig), [en, ar], "a new pass lets Arabic follow");
}

#[test]
fn quick_switches_on_automatic_pile_up_no_checks() {
    let mut rig = passed_english();
    rig.actor.ports.spawn = held;
    let (ar, en) = (arabic_layout, english_layout);
    for layout in [ar(), en(), ar(), en(), ar()] {
        rig.send(Msg::Layout(layout));
    }
    finish(&mut rig);
    assert!(!waiting(), "no check piles up");
    assert_eq!(checked(&rig), [files().1]);
}

#[test]
fn automatic_waits_for_a_dictation_and_for_compare() {
    let ar = files().1;
    let mut rig = passed_english();
    dictate_past_arabic(&mut rig);
    assert!(checked(&rig).is_empty(), "nothing while it dictates");
    written(&mut rig);
    assert_eq!(checked(&rig), [ar]);
    let mut rig = passed_english();
    start_compare(&mut rig);
    rig.send(Msg::Layout(arabic_layout()));
    assert!(checked(&rig).is_empty(), "nothing while Compare runs");
    rig.act(VoiceClick::Button(PanelButton::Cancel));
    assert_eq!(checked(&rig), [ar]);
}

//! The cards' checks by language on the actor (ADR-0052): Arabic's model, passed on the English clip before, is
//! checked on the Arabic clip before the card writes Arabic; a card that differs there keeps English; and an old result
//! of no language is checked again.

use kx_module_api::ports::{Hundredths, SpeechError, Verdict};

use super::super::Msg;
use super::super::auto::kept;
use super::super::rig::{Rig, english, english_layout};
use super::{
    BOTH, CARD, arabic, arabic_layout, build, build_picked, card_id, devices, differs, passed,
    served,
};
use crate::counts::Checked;

/// The rig's cards' driver.
const DRIVER: &str = "1.0";
const PASS: Verdict = Verdict::Passed {
    gain: Hundredths(300),
};

/// The clip each check read, in order.
fn clips(rig: &Rig) -> Vec<String> {
    let asks = rig.fake.speech.check_asks();
    asks.into_iter().map(|a| a.clip).collect()
}

/// Arabic's model passed on the English clip, as ADR-0044 kept it.
fn english_pass() -> Vec<Checked> {
    vec![kept(CARD, DRIVER, arabic(), "en", PASS)]
}

#[test]
fn arabics_model_passed_in_english_is_checked_on_the_arabic_clip_before_the_card_writes_arabic() {
    let answers = [
        (Err(SpeechError::Timeout), None),
        (Ok(passed()), Some(card_id())),
    ];
    for (check, card) in answers {
        let mut rig = build_picked(CARD, Some(devices()), check, english_pass(), Some(BOTH));
        assert!(clips(&rig).is_empty(), "English was checked");
        rig.send(Msg::Layout(arabic_layout()));
        assert_eq!(clips(&rig), ["ar"]);
        rig.listen();
        assert_eq!(served(&rig), card);
    }
}

/// The card picked after Arabic's check differed on the Arabic clip.
fn differed_in_arabic() -> Rig {
    let check = Ok(differs());
    let mut rig = build_picked(CARD, Some(devices()), check, english_pass(), Some(BOTH));
    rig.send(Msg::Layout(arabic_layout()));
    rig
}

#[test]
fn edge_3_a_card_differing_on_the_arabic_clip_writes_arabic_on_the_processor_alone() {
    let mut rig = differed_in_arabic();
    rig.listen();
    assert_eq!(served(&rig), None, "Arabic");
    let mut rig = differed_in_arabic();
    rig.send(Msg::Layout(english_layout()));
    rig.listen();
    assert_eq!(clips(&rig), ["ar"], "English is not checked again");
    assert_eq!(served(&rig), Some(card_id()), "English keeps the card");
}

#[test]
fn an_old_result_of_no_language_serves_nothing_and_is_checked_again() {
    let old = Checked {
        lang: None,
        ..kept(CARD, DRIVER, english(), "en", PASS)
    };
    for device in [CARD, "auto"] {
        let fault = Err(SpeechError::Timeout);
        let mut rig = build(device, Some(devices()), fault, vec![old.clone()]);
        assert_eq!(clips(&rig), ["en"], "{device}: checked again");
        rig.listen();
        assert_eq!(served(&rig), None, "{device}: the old pass serves nothing");
        let mut rig = build(device, Some(devices()), Ok(passed()), vec![old.clone()]);
        rig.listen();
        assert_eq!(served(&rig), Some(card_id()), "{device}: a new pass serves");
    }
}

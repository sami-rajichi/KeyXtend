//! The cards' checks by language on the actor (ADR-0052): Arabic's model, passed on the English clip before, is
//! checked on the Arabic clip before the card writes Arabic; a card that differs there keeps English; an old result of
//! no language is checked again; and a failure in English never stops a model's other languages.

use kx_module_api::ports::{LayoutChanged, LayoutId, SpeechError, Verdict};

use super::super::Msg;
use super::super::auto::{DRIVER, PASS, kept};
use super::super::rig::{Rig, english, english_layout};
use super::{
    BOTH, CARD, arabic, arabic_layout, build, build_picked, card_id, devices, differs, passed,
    served,
};
use crate::counts::Checked;
use crate::models::Row;

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

/// The French layout.
fn french_layout() -> LayoutChanged {
    LayoutChanged {
        id: LayoutId::new(3),
        iso: "fr".into(),
        name: "Français".into(),
        rtl: false,
    }
}

/// Automatic over a card whose check of `row`'s model failed on the English clip, with `picks`.
fn failed_in_english(row: &Row, picks: Option<&str>) -> Rig {
    let failed = vec![kept(CARD, DRIVER, row, "en", Verdict::Failed)];
    build_picked("auto", Some(devices()), Ok(passed()), failed, picks)
}

#[test]
fn a_failure_in_english_never_stops_automatic_checking_that_model_in_arabic_or_french() {
    let cases = [
        (failed_in_english(arabic(), Some(BOTH)), arabic_layout()),
        (failed_in_english(english(), None), french_layout()),
    ];
    for (mut rig, layout) in cases {
        assert!(clips(&rig).is_empty(), "English failed");
        let lang = layout.iso.clone();
        rig.send(Msg::Layout(layout));
        assert_eq!(clips(&rig), [lang]);
        rig.listen();
        assert_eq!(served(&rig), Some(card_id()));
    }
}

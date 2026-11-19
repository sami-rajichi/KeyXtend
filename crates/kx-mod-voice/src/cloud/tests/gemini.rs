//! Gemini through the cloud rules: its card carries the free allowance's warning, the training line in the warning tone
//! and its rule for the EEA and for age; a pick of Gemini 3.5 Transcribe sends the keyboard language's BCP-47 code in
//! its own shape, and a language its set lacks cannot pick it.

use super::*;
use crate::services::tests::gemini::FREE_USE;
use kx_module_api::ports::CloudShape;

/// The model the tests pick [N2].
const TRANSCRIBE: &str = "models/gemini-3.5-transcribe";
/// The keyboard languages and the codes they send: Arabic as Modern Standard Arabic's usual locale.
const SENT: [(&str, &str); 3] = [("en", "en-US"), ("fr", "fr-FR"), ("ar", "ar-SA")];

#[test]
fn its_card_carries_the_free_allowances_warning_the_training_tone_and_its_rule() {
    let card = rules().card(&on(), row("gemini"), today());
    assert_eq!(card.service, ServiceName::Named("Gemini".into()));
    let warn = card.warn.as_ref().map(|w| (w.id.as_str(), w.warn));
    assert_eq!(warn, Some((FREE_USE, true)), "the card's warning band");
    assert!(card.trains.warn, "the training line takes the warning tone");
    let rule = card.rule.as_ref().map(|r| r.id.as_str());
    assert_eq!(rule, Some("eea-paid-18"));
    assert_eq!(card.site.as_deref(), Some("aistudio.google.com"));
    assert!(card.untested && card.on.is_none(), "asks to switch it on");
}

#[test]
fn a_pick_sends_the_keyboard_languages_bcp_47_code_in_its_own_shape() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "gemini", today()));
    for (lang, code) in SENT {
        assert!(rules().pick(&mut s, lang, "gemini", TRANSCRIBE), "{lang}");
        let choice = rules().choice(&s, lang).unwrap();
        assert_eq!(choice.code, code, "{lang}");
        let Chosen::Listed(model) = choice.model else {
            panic!("{lang}: a listed model")
        };
        assert_eq!(model.id, TRANSCRIBE);
        let shape = choice.service.shape_for(Some(model), true);
        assert_eq!(shape, CloudShape::Gemini, "{lang}: no form, no keep field");
        let warned = choice.warn.as_ref().map(|w| w.id.as_str());
        assert_eq!(
            warned,
            Some(FREE_USE),
            "{lang}: the first dictation's reminder"
        );
    }
    assert!(
        !rules().pick(&mut s, "de", "gemini", TRANSCRIBE),
        "a code its pages do not name"
    );
}

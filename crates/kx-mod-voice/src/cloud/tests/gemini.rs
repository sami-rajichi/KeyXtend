//! Gemini through the cloud rules: its card carries the free allowance's warning, the training line in the warning tone
//! and its rule for the EEA and for age; a pick of Gemini 3.5 Transcribe sends the keyboard language's BCP-47 code in
//! its own shape, and a language its set lacks cannot pick it.

use super::*;
use kx_module_api::ports::CloudShape;

/// The model the tests pick: the row's first.
fn transcribe() -> &'static Model {
    &row("gemini").models[0]
}

#[test]
fn its_card_carries_the_free_allowances_warning_the_training_tone_and_its_rule() {
    let card = rules().card(&on(), row("gemini"), today());
    assert_eq!(card.service, row("gemini").shown());
    let warn = card.warn.as_ref().map(|w| (w.id.as_str(), w.warn));
    assert_eq!(
        warn,
        Some((warn_id("gemini"), true)),
        "the card's warning band"
    );
    assert!(card.trains.warn, "the training line takes the warning tone");
    let rule = card.rule.as_ref().map(|r| r.id.as_str());
    assert_eq!(rule, id_of(row("gemini").facts.rule.as_ref()));
    assert_eq!(card.site.as_deref(), site("gemini"));
    assert!(card.untested && card.on.is_none(), "asks to switch it on");
}

#[test]
fn a_pick_sends_the_keyboard_languages_bcp_47_code_in_its_own_shape() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "gemini", today()));
    for lang in ["en", "fr", "ar"] {
        let id = &transcribe().id;
        assert!(rules().pick(&mut s, lang, "gemini", id), "{lang}");
        let choice = rules().choice(&s, lang).unwrap();
        assert_eq!(
            Some(choice.code.as_str()),
            transcribe().code(lang),
            "{lang}"
        );
        let Chosen::Listed(model) = choice.model else {
            panic!("{lang}: a listed model")
        };
        assert_eq!(model.id, transcribe().id);
        let shape = choice.service.shape_for(Some(model), true);
        assert_eq!(shape, CloudShape::Gemini, "{lang}: no form, no keep field");
        let warned = choice.warn.as_ref().map(|w| w.id.as_str());
        assert_eq!(
            warned,
            Some(warn_id("gemini")),
            "{lang}: the first dictation's reminder"
        );
    }
    assert!(
        !rules().pick(
            &mut s,
            lacked("gemini", &transcribe().id),
            "gemini",
            &transcribe().id
        ),
        "a code its pages do not name"
    );
}

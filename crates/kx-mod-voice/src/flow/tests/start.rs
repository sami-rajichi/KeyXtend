//! Starting a dictation: two quick clicks (edge case 1), the language fixed at Start (2), no model yet or a vanished
//! file (17), and a language the model or the table does not know (18).

use super::*;

/// The debounce in the defaults, in ms.
fn debounce() -> u64 {
    VoiceSettings::from_defaults().unwrap().debounce_ms.into()
}

/// Arabic as the layouts name it.
fn arabic() -> LayoutChanged {
    layout("ar", "العربية", true)
}

#[test]
fn the_key_opens_the_microphone_at_once_with_the_settings_ask() {
    let mut rig = Rig::new();
    let out = rig.key();
    let settings = VoiceSettings::from_defaults().unwrap();
    let want = settings.ask(Some(ready("parakeet", "fr").model), None);
    assert_eq!(opened(&out), [want]);
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    let VoiceBar::Listening(shown) = rig.shown() else {
        panic!("no Listening bar: {:?}", rig.shown());
    };
    assert_eq!(shown.lang, french());
    assert_eq!(shown.wave_ms, settings.wave_ms);
    assert!(!shown.nothing);
}

#[test]
fn the_key_lights_only_once_the_microphone_opened() {
    let mut rig = Rig::new();
    rig.key();
    assert!(!rig.lit);
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    assert!(rig.lit);
}

#[test]
fn two_clicks_within_the_debounce_count_as_one() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.wait(debounce() - 1);
    let out = rig.key();
    assert!(calls(&out).is_empty() && opened(&out).is_empty());
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)));
    rig.wait(1);
    assert_eq!(calls(&rig.key()), [RunCall::Stop]);
}

#[test]
fn the_debounce_comes_from_the_settings() {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    settings.debounce_ms *= 2;
    let mut rig = Rig::with(settings, false);
    rig.key();
    rig.speak();
    rig.wait(debounce());
    assert!(calls(&rig.key()).is_empty());
    rig.wait(debounce());
    assert_eq!(calls(&rig.key()), [RunCall::Stop]);
}

#[test]
fn a_layout_change_while_listening_keeps_the_language() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.on(In::Layout(layout("en", "English", false)));
    let VoiceBar::Listening(shown) = rig.shown() else {
        panic!("no Listening bar");
    };
    assert_eq!(shown.lang, french());
    rig.wait(debounce());
    rig.key();
    let out = rig.write(&["bonjour"]);
    assert_eq!(kept(&out)[0].lang, "fr");
    rig.wait(debounce());
    let out = rig.key();
    assert_eq!(opened(&out)[0].model.as_ref().unwrap().lang, "en");
}

#[test]
fn no_model_yet_shows_a_note_with_download_and_opens_nothing() {
    let mut rig = Rig::new();
    rig.on(In::Layout(arabic()));
    let out = rig.key();
    assert!(opened(&out).is_empty());
    assert!(!rig.lit);
    let lang = VoiceLang {
        name: "العربية".into(),
        iso: "ar".into(),
        rtl: true,
        arabic: true,
    };
    let hint = BarHint::NoModel { lang, bytes: BYTES };
    assert_eq!(note(&rig.shown()), Some((hint, Some(BarButton::Download))));
    let out = rig.bar(BarButton::Download);
    assert!(out.contains(&Out::Download("ar".into())));
}

#[test]
fn a_download_click_on_another_languages_note_is_dropped() {
    let mut rig = Rig::new();
    rig.book
        .0
        .insert("en".into(), Err(Lack::Missing { bytes: BYTES }));
    rig.on(In::Layout(arabic()));
    rig.key();
    let round = rig.round();
    rig.on(In::Layout(layout("en", "English", false)));
    rig.wait(debounce());
    rig.key();
    let click = VoiceClick::Bar(BarButton::Download);
    let out = rig.on(In::Act(VoiceAct { round, click }));
    assert!(!out.iter().any(|o| matches!(o, Out::Download(_))));
    let out = rig.bar(BarButton::Download);
    assert!(out.contains(&Out::Download("en".into())));
}

#[test]
fn a_vanished_file_tells_the_folder_and_offers_download() {
    let mut rig = Rig::new();
    rig.key();
    let out = rig.tell(SpeechNote::Fault(SpeechError::NoFile));
    assert!(out.contains(&Out::Lost));
    assert_eq!(calls(&out), [RunCall::Close]);
    let hint = BarHint::NoModel {
        lang: french(),
        bytes: BYTES,
    };
    assert_eq!(note(&rig.shown()), Some((hint, Some(BarButton::Download))));
    let out = rig.bar(BarButton::Download);
    assert!(out.contains(&Out::Download("fr".into())));
}

#[test]
fn a_model_that_does_not_know_the_language_points_to_the_gear() {
    let mut rig = Rig::new();
    rig.book.0.insert("fr".into(), Err(Lack::Stranger));
    let out = rig.key();
    assert!(opened(&out).is_empty());
    let want = Some((BarHint::Language(french()), None));
    assert_eq!(note(&rig.shown()), want);
}

#[test]
fn a_picked_file_still_read_says_it_is_being_checked() {
    let mut rig = Rig::new();
    rig.book.0.insert("fr".into(), Err(Lack::Checking));
    let out = rig.key();
    assert!(opened(&out).is_empty());
    assert_eq!(note(&rig.shown()), Some((BarHint::Checking, None)));
}

#[test]
fn the_worker_refusing_the_language_points_to_the_gear() {
    let mut rig = Rig::new();
    rig.key();
    let out = rig.tell(SpeechNote::Fault(SpeechError::Language));
    assert_eq!(calls(&out), [RunCall::Close]);
    let want = Some((BarHint::Language(french()), None));
    assert_eq!(note(&rig.shown()), want);
}

#[test]
fn a_language_no_listed_model_knows_says_so() {
    let mut rig = Rig::new();
    rig.on(In::Layout(layout("sw", "Kiswahili", false)));
    let out = rig.key();
    assert!(opened(&out).is_empty());
    let lang = VoiceLang {
        name: "Kiswahili".into(),
        iso: "sw".into(),
        rtl: false,
        arabic: false,
    };
    assert_eq!(note(&rig.shown()), Some((BarHint::NoLanguage(lang), None)));
}

#[test]
fn a_model_too_big_now_says_so() {
    let mut rig = Rig::new();
    rig.book.0.insert("fr".into(), Err(Lack::NoFit));
    rig.key();
    assert_eq!(note(&rig.shown()), Some((BarHint::NoFit, None)));
}

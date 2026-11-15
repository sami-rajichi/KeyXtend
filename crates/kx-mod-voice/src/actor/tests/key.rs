//! The Dictation key drives the flow on the actor, and `VoiceLit` follows the microphone exactly: on when the run says
//! it opened, off at Stop, at its end and when it shuts.

use std::path::Path;

use kx_module_api::ports::{SpeechNote, SpeechStage, VoiceBar};

use super::Msg;
use super::rig::{DIR, Rig, english};
use super::taps::Heard;

#[test]
fn the_key_opens_the_microphone_and_the_light_follows_it_exactly() {
    let mut rig = Rig::new();
    rig.listen();
    assert_eq!(rig.fake.speech.runs().len(), 1);
    assert_eq!(rig.lits(), [false, true]);
    assert!(matches!(rig.bar(), VoiceBar::Listening(_)));
    rig.key();
    assert_eq!(rig.fake.speech.runs()[0].stops, 1);
    assert_eq!(rig.lits(), [false, true, false]);
    assert_eq!(rig.bar(), VoiceBar::Transcribing);
}

#[test]
fn the_light_waits_for_the_microphone_and_goes_out_when_it_shuts() {
    let mut rig = Rig::new();
    rig.key();
    assert_eq!(rig.fake.speech.runs().len(), 1);
    assert_eq!(
        rig.lits(),
        [false],
        "the run has not said the microphone opened"
    );
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    rig.tell(SpeechNote::Stage(SpeechStage::Shut));
    assert_eq!(rig.lits(), [false, true, false]);
}

#[test]
fn the_run_opens_with_the_model_of_the_language_in_front() {
    let mut rig = Rig::new();
    rig.key();
    let ask = rig.fake.speech.runs()[0].ask.clone();
    let model = ask.model.unwrap();
    let row = english();
    assert_eq!(model.file.expose(), &Path::new(DIR).join(&row.file));
    assert_eq!(Some(model.lang.as_str()), row.code("en"));
    assert_eq!(ask.job, rig.actor.settings().job());
}

#[test]
fn a_late_starter_hears_the_view_and_the_light_again() {
    let mut rig = Rig::new();
    rig.listen();
    let (told, page) = (rig.heard().len(), rig.views().pop().unwrap());
    rig.send(Msg::Ask);
    assert_eq!(rig.heard()[told..], [Heard::View(page), Heard::Lit(true)]);
}

#[test]
fn the_level_reaches_the_bar_while_listening() {
    let mut rig = Rig::new();
    rig.listen();
    rig.tell(SpeechNote::Level(90));
    assert_eq!(rig.heard().last(), Some(&Heard::Level(90)));
}

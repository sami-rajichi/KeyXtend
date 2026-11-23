//! The Dictation key drives the flow on the actor, and `VoiceLit` follows the microphone exactly: amber from the click,
//! red when the run says it opened, off at Stop, at its end and when it shuts.

use std::path::Path;

use kx_module_api::ports::VoiceLit::{Off, Prep, Rec};
use kx_module_api::ports::{SpeechNote, SpeechStage, VoiceBar};

use super::Msg;
use super::rig::{DIR, Rig, english};
use super::taps::Heard;

#[test]
fn the_key_opens_the_microphone_and_the_light_follows_it_exactly() {
    let mut rig = Rig::new();
    rig.listen();
    assert_eq!(rig.fake.speech.runs().len(), 1);
    assert_eq!(rig.lits(), [Off, Prep, Rec]);
    assert!(matches!(rig.bar(), VoiceBar::Listening(_)));
    rig.key();
    assert_eq!(rig.fake.speech.runs()[0].stops, 1);
    assert_eq!(rig.lits(), [Off, Prep, Rec, Off]);
    assert_eq!(rig.bar(), VoiceBar::Transcribing);
}

#[test]
fn the_light_waits_for_the_microphone_and_goes_out_when_it_shuts() {
    let mut rig = Rig::new();
    rig.key();
    assert_eq!(rig.fake.speech.runs().len(), 1);
    assert_eq!(
        (rig.lits(), rig.bar()),
        (vec![Off, Prep], VoiceBar::Hidden),
        "amber with no bar until the run says the microphone opened"
    );
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    rig.tell(SpeechNote::Stage(SpeechStage::Shut));
    assert_eq!(rig.lits(), [Off, Prep, Rec, Off]);
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
fn a_late_starter_hears_the_view_the_light_and_the_panels_line_again() {
    let mut rig = Rig::new();
    rig.listen();
    let (told, page) = (rig.heard().len(), rig.views().pop().unwrap());
    rig.send(Msg::Ask);
    let again = [Heard::View(page), Heard::Lit(Rec), Heard::Line(None)];
    assert_eq!(rig.heard()[told..], again);
}

#[test]
fn the_level_reaches_the_bar_while_listening() {
    let mut rig = Rig::new();
    rig.listen();
    rig.tell(SpeechNote::Level(90));
    assert_eq!(rig.heard().last(), Some(&Heard::Level(90)));
}

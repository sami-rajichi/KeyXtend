//! A warned take's card warning through Stop: the hub keeps its fact from the last Listening page and words it on
//! Transcribing while the same service writes, in the keyboard's language; another chip, a text, a note or a hide drops
//! it.

use super::{heard, json, listening_bar, page};
use crate::Rig;
use crate::texts::{rig, switch};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarHint, BarMark, BarNote, BarText, Fact, FactArg, FactValue, Listening, ServiceName, VoiceBar,
    VoicePage, VoiceText,
};
use kx_ui::i18n::keys::FACT_WARN;
use serde_json::Value;

/// The card's warning the tests' service carries, with the switch that stops it.
fn fact() -> Fact {
    let switch = FactArg {
        name: "switch".to_owned(),
        value: FactValue::Text("Data use".to_owned()),
    };
    Fact {
        id: "trains".to_owned(),
        args: vec![switch],
        warn: true,
    }
}

/// The service the warned take goes to, and the model on this PC a password box sends it to.
fn gemini() -> BarMark {
    BarMark::Cloud(ServiceName::Named("Gemini".to_owned()))
}
fn device() -> BarMark {
    BarMark::Device(Redacted::new("Whisper Turbo".to_owned()))
}

/// Page `round` with `bar` and the chip `mark`.
fn marked(round: u32, bar: VoiceBar, mark: BarMark) -> VoicePage {
    VoicePage {
        mark: Some(mark),
        ..page(round, bar, None)
    }
}

/// Listening, with the card's warning when `warned`.
fn listening(warned: bool) -> VoiceBar {
    let VoiceBar::Listening(plain) = listening_bar() else {
        panic!("a listening bar")
    };
    VoiceBar::Listening(Listening {
        warn: warned.then(fact),
        ..plain
    })
}

/// The warning on the bar QML reads, in the words of `rig`'s keyboard language.
fn shown(rig: &Rig) -> (Value, String) {
    let worded = rig.hub.i18n().voice_fact(FACT_WARN, &fact()).text;
    (json(rig)["bar"]["warn"].clone(), worded)
}

#[test]
fn transcribing_words_the_takes_warning_in_the_keyboards_language_while_its_service_writes() {
    let (rig, face, _host) = rig("en", "en", "auto");
    let heard = heard(&rig);
    heard.view(marked(1, listening(true), gemini()));
    heard.view(marked(2, VoiceBar::Transcribing, gemini()));
    let mut words = Vec::new();
    for iso in ["en", "ar", "fr"] {
        switch(&rig, &face, iso);
        let (warn, worded) = shown(&rig);
        assert_eq!(warn, worded.as_str(), "{iso}: kept while Gemini writes");
        words.push(worded);
    }
    words.sort();
    words.dedup();
    assert_eq!(words.len(), 3, "each language words it its own way");
}

#[test]
fn another_chip_a_text_a_note_or_a_hide_drops_the_warning() {
    let text = VoiceBar::Text(BarText {
        text: VoiceText::Masked,
        hint: None,
        undo: false,
    });
    let note = VoiceBar::Note(BarNote {
        hint: BarHint::Fault,
        act: None,
        mic: false,
    });
    let cases = [
        (None, device(), "the model on this PC writes"),
        (Some(text), gemini(), "Try again after a text"),
        (Some(note), gemini(), "Try again after a note"),
        (Some(VoiceBar::Hidden), gemini(), "after a hide"),
    ];
    for (between, writer, why) in cases {
        let rig = Rig::new();
        let heard = heard(&rig);
        heard.view(marked(1, listening(true), gemini()));
        if let Some(bar) = between {
            heard.view(marked(2, bar, gemini()));
        }
        heard.view(marked(3, VoiceBar::Transcribing, writer));
        assert_eq!(json(&rig)["bar"]["warn"], Value::Null, "{why}");
    }
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.view(marked(1, listening(false), gemini()));
    heard.view(marked(2, VoiceBar::Transcribing, gemini()));
    assert_eq!(json(&rig)["bar"]["warn"], Value::Null, "none to keep");
}

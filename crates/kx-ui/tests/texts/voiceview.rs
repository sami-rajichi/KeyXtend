//! The dictation bar's views the Qt Quick tests draw (`qmltest/fixtures/voice.js`), as the hub writes them in each
//! language: each state, a long text, the password dots, each kind of note, and the lines a text can carry; and the
//! click kinds QML sends to `Voice.act`, by the name QML holds each under (VoiceNames.qml).

use crate::qml::{HEAD, LANGS, fixture, hub, var};
use kx_module_api::ports::{
    BarButton, BarHint, BarNote, BarPlace, BarText, Listening, VoiceBar, VoiceLang, VoicePage,
    VoiceText, VoiceView,
};
use kx_module_api::{Bus, Redacted, Zeroizing};
use kx_test_support::host::FakeCore;
use kx_ui::voice_acts as acts;
use serde_json::{Map, Value, json};
use std::sync::Arc;

/// The fixture's file name.
pub const FILE: &str = "voice.js";
/// How long the bar has listened in each Listening view, in ms.
const ELAPSED_MS: u32 = 1200;
/// The wave's smoothing, as the module's default.
const WAVE_MS: u16 = 300;
/// The default model's size in the no-model note: 740 MB.
const MODEL_BYTES: u64 = 740 << 20;
/// How many times the long view says its sentence, so it runs past the bar's lines.
const LONG: usize = 3;

/// Each click kind by the name QML holds it under.
fn kinds() -> Value {
    json!({
        "bar": acts::BAR, "gear": acts::GEAR, "over": acts::OVER, "open": acts::OPEN, "row": acts::ROW,
        "page": acts::PAGE, "wheel": acts::WHEEL, "fetch": acts::FETCH, "flip": acts::FLIP, "step": acts::STEP,
        "plus": acts::PLUS, "minus": acts::MINUS, "button": acts::BUTTON, "compare": acts::COMPARE,
        "confirm": acts::CONFIRM,
    })
}

/// The language the keyboard in `tag` dictates in, as Windows names it, and a sentence said in it (the mock-up's).
fn spoken(tag: &str) -> (VoiceLang, &'static str) {
    let lang = |name: &str, rtl| VoiceLang {
        name: name.to_owned(),
        iso: tag.to_owned(),
        rtl,
        arabic: rtl,
    };
    match tag {
        "fr" => (
            lang("français", false),
            "Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine.",
        ),
        "ar" => (
            lang("العربية", true),
            "شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل.",
        ),
        _ => (
            lang("English", false),
            "Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week.",
        ),
    }
}

/// The Text state showing `said` with `hint`, Undo insert acting while `undo`.
fn text(said: &str, lang: &VoiceLang, hint: Option<BarHint>, undo: bool) -> VoiceBar {
    let text = VoiceText::Shown {
        text: Redacted::new(Zeroizing::new(said.to_owned())),
        rtl: lang.rtl,
        arabic: lang.arabic,
    };
    VoiceBar::Text(BarText { text, hint, undo })
}

/// The Note state for `hint` with `act`, the microphone open while `mic`.
fn note(hint: BarHint, act: Option<BarButton>, mic: bool) -> VoiceBar {
    VoiceBar::Note(BarNote { hint, act, mic })
}

/// The Listening state in `lang`, with nothing heard yet while `nothing`.
fn listening(lang: &VoiceLang, nothing: bool) -> VoiceBar {
    VoiceBar::Listening(Listening {
        lang: lang.clone(),
        elapsed_ms: ELAPSED_MS,
        nothing,
        wave_ms: WAVE_MS,
    })
}

/// Every bar the tests draw in language `tag`, by name.
fn bars(tag: &str) -> Vec<(&'static str, VoiceBar)> {
    let (lang, said) = spoken(tag);
    let repeated = [said; LONG].join(" ");
    let no_model = BarHint::NoModel {
        lang: lang.clone(),
        bytes: MODEL_BYTES,
    };
    vec![
        ("hidden", VoiceBar::Hidden),
        ("listening", listening(&lang, false)),
        ("nothing", listening(&lang, true)),
        ("transcribing", VoiceBar::Transcribing),
        ("text", text(said, &lang, None, true)),
        ("long", text(&repeated, &lang, None, true)),
        ("undone", text(said, &lang, None, false)),
        ("stopped", text(said, &lang, Some(BarHint::Stopped), true)),
        (
            "copyFailed",
            text(said, &lang, Some(BarHint::CopyFailed), true),
        ),
        ("password", text_masked()),
        (
            "heardNothing",
            note(BarHint::Nothing, Some(BarButton::Listen), false),
        ),
        (
            "cancelled",
            note(BarHint::Cancelled, Some(BarButton::Anyway), false),
        ),
        ("noModel", note(no_model, Some(BarButton::Download), false)),
        ("checking", note(BarHint::Checking, None, true)),
        ("taken", note(BarHint::Taken, None, false)),
        ("fault", note(BarHint::Fault, None, false)),
    ]
}

/// A password box took the text: dots, its line, and Undo insert.
fn text_masked() -> VoiceBar {
    let hint = Some(BarHint::Password);
    VoiceBar::Text(BarText {
        text: VoiceText::Masked,
        hint,
        undo: true,
    })
}

/// Every view in language `tag`, by name, each its own round from 1, above the keyboard with sounds.
fn views(tag: &str) -> Value {
    let (hub, _host) = hub(tag);
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).expect("the hub listens");
    let shown = (1..).zip(bars(tag)).map(|(round, (name, bar))| {
        let page = VoicePage {
            round,
            bar,
            panel: None,
            place: BarPlace::Above,
            sounds: true,
            cloud: None,
        };
        bus.publish(&VoiceView::new(page))
            .expect("the view is sent");
        let v: Value = serde_json::from_str(&hub.voice_json()).expect("voice JSON");
        (name.to_owned(), v)
    });
    Value::Object(shown.collect::<Map<_, _>>())
}

#[test]
fn the_voice_fixture_matches_the_hub() {
    let all: Map<String, Value> = LANGS.iter().map(|t| ((*t).to_owned(), views(t))).collect();
    let text = format!(
        "{HEAD}{}{}",
        var("acts", &kinds(), true),
        var("views", &Value::Object(all), true)
    );
    fixture(FILE.to_owned(), &text);
}

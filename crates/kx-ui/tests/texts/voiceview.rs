//! The dictation bar's views the Qt Quick tests draw (`qmltest/fixtures/voice.js`), as the hub writes them in each
//! language: each state, a long text, the password dots, each kind of note, the lines a text can carry, the cloud
//! mark with a cloud note's buttons, Open service under a note's line, a card's warning and the chip naming the model
//! on this PC; and every click kind QML sends to `Voice.act`, by the name QML holds each under (VoiceNames.qml).

use crate::qml::{HEAD, LANGS, fixture, hub, var};
use kx_mod_voice::models;
use kx_mod_voice::services::table;
use kx_module_api::ports::{
    BarButton, BarHint, BarMark, BarNote, BarPlace, BarText, CloudWhy, Fact, Listening, LocalUse,
    ServiceName, VoiceBar, VoiceLang, VoicePage, VoiceText, VoiceView,
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
/// The default model's size in the no-model note and a cloud note's Download: 740 MB.
const MODEL_BYTES: u64 = 740 << 20;
/// How many times the long view says its sentence, so it runs past the bar's lines.
const LONG: usize = 3;
/// The start of a view's name that shows the cloud mark, and the service it names.
const CLOUD: &str = "cloud";
const SERVICE: &str = "Groq";
/// The view of a first dictation to a service whose card warns.
const WARNED: &str = "cloudWarned";
/// The view whose cloud mark names Custom, the longest service name in French.
const CUSTOM: &str = "cloudCustom";
/// The start of a view's name whose chip names the model on this PC: the table's longest short name.
const DEVICE: &str = "device";
/// The view whose chip names a user's own file, too long to show whole.
const OWN: &str = "deviceOwn";
const OWN_FILE: &str = "whisper-large-v3-turbo-german-and-french-finetune-q5_k_m";

/// Each click kind by the name QML holds it under.
fn kinds() -> Value {
    json!({
        "bar": acts::BAR, "gear": acts::GEAR, "over": acts::OVER, "open": acts::OPEN, "row": acts::ROW,
        "page": acts::PAGE, "wheel": acts::WHEEL, "fetch": acts::FETCH, "flip": acts::FLIP, "step": acts::STEP,
        "plus": acts::PLUS, "minus": acts::MINUS, "button": acts::BUTTON, "compare": acts::COMPARE,
        "confirm": acts::CONFIRM, "cloud": acts::CLOUD, "service": acts::SERVICE, "keep": acts::KEEP,
        "side": acts::SIDE, "local": acts::LOCAL, "save": acts::SAVE, "caret": acts::CARET,
        "pick": acts::PICK, "paste": acts::PASTE,
    })
}

/// The hub's click kinds, each a `pub const` text there.
const ACTS: &str = include_str!("../../src/hub/voice/acts.rs");

/// Every click kind the hub takes has a name in `kinds()`, so a QML test can send each one.
#[test]
fn every_click_kind_has_a_name_for_qml() {
    let named = kinds();
    let named: Vec<&Value> = named.as_object().expect("names").values().collect();
    let all = ACTS.lines().filter_map(|l| {
        let (_, text) = l.strip_prefix("pub const ")?.split_once(": &str = \"")?;
        text.strip_suffix("\";")
    });
    let all: Vec<&str> = all.collect();
    let missing: Vec<&&str> = all
        .iter()
        .filter(|k| !named.iter().any(|v| v == *k))
        .collect();
    assert!(!all.is_empty() && missing.is_empty(), "{missing:?}");
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

/// A cloud note on `why` with `local`, and Try again while `again`.
fn cloud(why: CloudWhy, local: LocalUse, again: bool) -> VoiceBar {
    let act = again.then_some(BarButton::Again);
    note(BarHint::Cloud { why, local }, act, false)
}

/// The Listening state in `lang`, with nothing heard yet while `nothing`, and a service's card warning if any.
fn listening(lang: &VoiceLang, nothing: bool, warn: Option<Fact>) -> VoiceBar {
    VoiceBar::Listening(Listening {
        lang: lang.clone(),
        elapsed_ms: ELAPSED_MS,
        nothing,
        wave_ms: WAVE_MS,
        warn,
    })
}

/// The first offered service with a card warning: its cloud mark, and the warning its first dictation shows.
fn warned() -> (ServiceName, Fact) {
    let services = table().expect("the services table reads");
    let first = services
        .offered()
        .find_map(|s| Some((s.shown(), s.facts.warn.clone()?)));
    first.expect("an offered service warns")
}

/// The models table's longest short name, which the bar must fit whole.
fn longest() -> String {
    let rows = models::table().expect("the models table reads").rows();
    let most = rows.iter().max_by_key(|r| r.short.chars().count());
    most.expect("a model").short.clone()
}

/// The chip view `name` shows: the warning service's on `WARNED`, Custom on `CUSTOM`, Groq's on another cloud view, the
/// user's own file on `OWN`, the longest short name on another device view, none otherwise.
fn mark(name: &str) -> Option<BarMark> {
    let device = |n: String| Some(BarMark::Device(Redacted::new(n)));
    match name {
        WARNED => Some(BarMark::Cloud(warned().0)),
        CUSTOM => Some(BarMark::Cloud(ServiceName::Custom)),
        OWN => device(OWN_FILE.to_owned()),
        _ if name.starts_with(DEVICE) => device(longest()),
        _ if name.starts_with(CLOUD) => Some(BarMark::Cloud(ServiceName::Named(SERVICE.into()))),
        _ => None,
    }
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
        ("listening", listening(&lang, false, None)),
        ("nothing", listening(&lang, true, None)),
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
    .into_iter()
    .chain(cloud_bars(&lang))
    .chain(device_bars(&lang))
    .collect()
}

/// The bars a dictation by a model on this PC draws in `lang`, each named with `DEVICE` first; nothing heard yet is the
/// widest.
fn device_bars(lang: &VoiceLang) -> [(&'static str, VoiceBar); 4] {
    [
        ("deviceListening", listening(lang, false, None)),
        ("deviceNothing", listening(lang, true, None)),
        ("deviceTranscribing", VoiceBar::Transcribing),
        (OWN, listening(lang, false, None)),
    ]
}

/// The bars a cloud dictation draws in `lang`, each named with `CLOUD` first; the Off note with Download carries the
/// most acts under its line, and the account note the longest line over its acts.
fn cloud_bars(lang: &VoiceLang) -> [(&'static str, VoiceBar); 9] {
    let fetch = LocalUse::Download { bytes: MODEL_BYTES };
    [
        ("cloudListening", listening(lang, false, None)),
        ("cloudNothing", listening(lang, true, None)),
        (CUSTOM, listening(lang, false, None)),
        (WARNED, listening(lang, false, Some(warned().1))),
        ("cloudKey", cloud(CloudWhy::Key, LocalUse::Ready, true)),
        ("cloudNet", cloud(CloudWhy::Net, fetch, true)),
        ("cloudOff", cloud(CloudWhy::Off, fetch, true)),
        (
            "cloudAccount",
            cloud(CloudWhy::Account, LocalUse::Ready, true),
        ),
        (
            "cloudNoLocal",
            cloud(CloudWhy::NoLocal, LocalUse::Dimmed, false),
        ),
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
            mark: mark(name),
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

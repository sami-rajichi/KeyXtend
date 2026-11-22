//! Voice typing's texts in each language: every id the hub words has its text, the owner's and the mock-up's words read
//! as checked, and a note names its language. Its numbers are in `numbers`, and the port's newer notes in `newer`.

use crate::{all, lang};
use kx_module_api::ports::{
    BarButton, BarHint, BarPlace, FetchButton, PanelButton, VoiceLang, VoiceTab,
};
use kx_ui::i18n::keys::{
    LANG_IN, LANG_NAME, TRAY_VOICE, VOICE_COL, VOICE_COLS, VOICE_MARK, VOICE_NOTE, VOICE_PLACE,
    VOICE_TITLE, VOICE_WORD, VoiceMark, bar_button_id, bar_hint_id, fetch_button_id, mark_id,
    panel_button_id, place_id, voice_tab_id,
};

mod approved;
mod assembly;
mod cloud;
mod eleven;
mod gemini;
mod icons;
mod lines;
mod money;
mod newer;
mod numbers;
pub(crate) mod own;
mod renewal;
mod sentences;

/// A language as the module names it, by its own name.
fn named(name: &str) -> VoiceLang {
    VoiceLang {
        name: name.to_owned(),
        iso: String::new(),
        rtl: false,
        arabic: false,
    }
}

/// Every hint, each with a language where it names one.
fn hints() -> Vec<BarHint> {
    let lang = named("English");
    vec![
        BarHint::Nothing,
        BarHint::Cancelled,
        BarHint::NoMic,
        BarHint::MicBusy,
        BarHint::MicRefused,
        BarHint::Unplugged,
        BarHint::NoModel {
            lang: lang.clone(),
            bytes: 1,
        },
        BarHint::Language(lang.clone()),
        BarHint::Checking,
        BarHint::NoLanguage(lang),
        BarHint::NoFit,
        BarHint::Fault,
        BarHint::Pieces,
        BarHint::TooLong,
        BarHint::NoField,
        BarHint::Stopped,
        BarHint::CopyFailed,
        BarHint::Password,
        BarHint::Fell,
        BarHint::Print,
        BarHint::Taken,
        BarHint::Full,
        BarHint::Folder,
        BarHint::Net,
    ]
}

/// Every key the hub builds from an id: notes, bar and download buttons, pages, places, marks, columns, languages.
fn keys() -> Vec<String> {
    let word = |id: &str| format!("{VOICE_WORD}{id}");
    let mut keys: Vec<String> = hints()
        .iter()
        .map(|h| format!("{VOICE_NOTE}{}", bar_hint_id(h)))
        .collect();
    keys.extend(BarButton::ALL.map(|b| word(bar_button_id(b))));
    keys.extend(FetchButton::ALL.map(|b| word(fetch_button_id(b))));
    keys.extend(PanelButton::ALL.map(|b| word(panel_button_id(b))));
    let tabs = VoiceTab::ALL.into_iter();
    keys.extend(tabs.map(|t| word(voice_tab_id(t))));
    keys.extend(BarPlace::ALL.map(|p| format!("{VOICE_PLACE}{}", place_id(p))));
    keys.extend(VoiceMark::ALL.map(|m| format!("{VOICE_MARK}{}", mark_id(m))));
    keys.extend(VOICE_COLS.map(|c| format!("{VOICE_COL}{c}")));
    for group in [LANG_NAME, LANG_IN] {
        keys.extend(["en", "fr", "ar"].map(|iso| format!("{group}.{iso}")));
    }
    keys
}

#[test]
fn every_id_the_hub_words_has_its_text_in_each_language() {
    let keys = keys();
    assert_eq!(hints().len(), 24, "every hint is listed");
    for t in all() {
        let missing: Vec<&String> = keys.iter().filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
}

#[test]
fn the_key_its_label_the_tray_row_and_the_panel_say_the_owners_word() {
    let want = [("en", "Dictation"), ("fr", "Dictée"), ("ar", "إملاء")];
    for (tag, word) in want {
        let t = lang(tag);
        for key in ["key.mic", "label.mic", TRAY_VOICE, VOICE_TITLE] {
            assert_eq!(t.text(key), word, "{tag} {key}");
        }
    }
}

#[test]
fn the_designs_main_words_read_as_the_owner_checked_them() {
    let table = [
        ("listening", ["Listening…", "Écoute…", "جارٍ الاستماع…"]),
        (
            "transcribing",
            ["Transcribing…", "Transcription…", "جارٍ تحويل الكلام إلى نص…"],
        ),
        ("stop", ["Stop", "Arrêter", "إيقاف"]),
        ("cancel", ["Cancel", "Annuler", "إلغاء"]),
        ("close", ["Close", "Fermer", "إغلاق"]),
        ("copy", ["Copy", "Copier", "نسخ"]),
        (
            "undo",
            ["Undo insert", "Annuler l’insertion", "تراجع عن الإدراج"],
        ),
        ("again", ["Try again", "Réessayer", "إعادة المحاولة"]),
        (
            "compare",
            ["Compare models", "Comparer les modèles", "مقارنة النماذج"],
        ),
        ("history", ["History", "Historique", "السجل"]),
        ("cpu", ["Processor", "Processeur", "المعالج"]),
    ];
    for (id, words) in table {
        for (tag, word) in ["en", "fr", "ar"].into_iter().zip(words) {
            assert_eq!(
                lang(tag).text(&format!("{VOICE_WORD}{id}")),
                word,
                "{tag} {id}"
            );
        }
    }
    let card = ["Graphics card", "Carte graphique", "بطاقة الرسوميات"];
    for (tag, word) in ["en", "fr", "ar"].into_iter().zip(card) {
        assert!(lang(tag).text("voice.card-line").starts_with(word), "{tag}");
    }
}

#[test]
fn a_note_names_the_language_in_its_own_words() {
    let no_model = |tag: &str, name: &str| {
        let hint = BarHint::NoModel {
            lang: named(name),
            bytes: 1,
        };
        lang(tag).voice_hint(&hint)
    };
    assert_eq!(no_model("en", "English"), "No model for English yet");
    assert_eq!(
        no_model("fr", "français"),
        "Aucun modèle en français pour l’instant"
    );
    assert_eq!(no_model("ar", "العربية"), "لا يوجد نموذج للغة العربية بعد");
    let hidden = lang("en").voice_hint(&BarHint::Language(named("Eng\u{202e}lish")));
    assert!(hidden.contains("English"), "hidden marks go: {hidden}");
}

// Words pinned to another language than the one in front name it by its code in their own words, French in lower case
// inside a sentence; a language they have no name for keeps the system's own name.
#[test]
fn a_note_names_the_language_in_front_by_its_code_in_the_words_language() {
    let no_model = |tag: &str, iso: &str, name: &str| {
        let mut front = named(name);
        iso.clone_into(&mut front.iso);
        lang(tag).voice_hint(&BarHint::NoModel {
            lang: front,
            bytes: 1,
        })
    };
    let fr = "Aucun modèle en anglais pour l’instant";
    assert_eq!(no_model("fr", "en", "English"), fr);
    assert_eq!(
        no_model("fr", "fr", "Français"),
        "Aucun modèle en français pour l’instant"
    );
    assert_eq!(
        no_model("fr", "ar", "العربية"),
        "Aucun modèle en arabe pour l’instant"
    );
    let ar = "لا يوجد نموذج للغة الإنجليزية بعد";
    assert_eq!(no_model("ar", "en", "English"), ar);
    assert_eq!(
        no_model("ar", "fr", "français"),
        "لا يوجد نموذج للغة الفرنسية بعد"
    );
    assert_eq!(no_model("en", "fr", "français"), "No model for French yet");
    assert_eq!(
        no_model("en", "de", "Deutsch"),
        "No model for Deutsch yet",
        "the system's name"
    );
}

//! Voice typing's texts in each language: every id the hub words has its text, the owner's and the mock-up's words,
//! sizes, memory, seconds and factors with the language's marks, Arabic's plural forms, and a note naming its language.
//! The port's newer notes are in `newer`.

use crate::{all, lang};
use kx_module_api::ports::{
    BarButton, BarHint, BarPlace, FetchButton, Hundredths, PanelButton, VoiceLang, VoiceTab,
};
use kx_ui::i18n::keys::{
    LANG_IN, LANG_NAME, TRAY_VOICE, VOICE_COL, VOICE_COLS, VOICE_CORES, VOICE_FASTER, VOICE_MARK,
    VOICE_MS, VOICE_NOTE, VOICE_PLACE, VOICE_SECONDS, VOICE_TEXTS, VOICE_TIMES, VOICE_TITLE,
    VOICE_WORD, VoiceMark, bar_button_id, bar_hint_id, fetch_button_id, mark_id, panel_button_id,
    place_id, voice_tab_id,
};

mod cloud;
mod icons;
mod newer;

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

#[test]
fn file_sizes_show_three_significant_digits_with_the_languages_mark() {
    let (en, fr, ar) = (lang("en"), lang("fr"), lang("ar"));
    assert_eq!(en.voice_size(739_508_576), "740 MB");
    assert_eq!(en.voice_size(1_161_143_008), "1.16 GB");
    assert_eq!(en.voice_size(2_410_655_232), "2.41 GB");
    assert_eq!(en.voice_size(21_300_000_000), "21.3 GB");
    assert_eq!(en.voice_size(212_000_000_000), "212 GB");
    assert_eq!(
        en.voice_size(999_600_000),
        "1 GB",
        "rounds up past a gigabyte"
    );
    assert_eq!(fr.voice_size(1_161_143_008), "1,16\u{a0}Go");
    assert_eq!(ar.voice_size(1_161_143_008), "1,16 غيغابايت");
    assert_eq!(ar.voice_size(193_749_056), "194 ميغابايت");
}

#[test]
fn memory_rooms_seconds_and_factors_drop_trailing_zeros() {
    let (en, fr, ar) = (lang("en"), lang("fr"), lang("ar"));
    assert_eq!(en.voice_memory(16_384), "16 GB");
    assert_eq!(en.voice_memory(9_626), "9.4 GB");
    assert_eq!(en.voice_memory(512), "0.5 GB");
    assert_eq!(en.voice_room(512), "512 MB");
    assert_eq!(en.voice_room(1_024), "1 GB");
    assert_eq!(en.voice_room(65_536), "64 GB");
    assert_eq!(en.voice_secs(2_100), "2.1 s");
    assert_eq!(
        en.voice_secs(4_000),
        "4.0 s",
        "a wait keeps its decimal, as the mock-up writes it"
    );
    assert_eq!(fr.voice_secs(2_100), "2,1\u{a0}s");
    assert_eq!(
        ar.voice_secs(4_000),
        "4,0 ثانية",
        "a decimal takes the singular"
    );
    assert_eq!(en.voice_times(VOICE_TIMES, Hundredths(150)), "1.5×");
    assert_eq!(en.voice_times(VOICE_TIMES, Hundredths(125)), "1.25×");
    assert_eq!(
        fr.voice_times(VOICE_FASTER, Hundredths(900)),
        "9× plus rapide"
    );
}

#[test]
fn arabic_factors_agree_in_number_and_case() {
    let ar = lang("ar");
    let times = |h: u32| ar.voice_times(VOICE_TIMES, Hundredths(h));
    let faster = |h: u32| ar.voice_times(VOICE_FASTER, Hundredths(h));
    assert_eq!(times(100), "مرة واحدة");
    assert_eq!(times(200), "مرتان", "a value stands as a subject");
    assert_eq!(times(300), "3 مرات");
    assert_eq!(times(1000), "10 مرات");
    assert_eq!(times(150), "1,5 مرة");
    assert_eq!(
        faster(200),
        "أسرع مرتين",
        "an adverb of times is accusative"
    );
    assert_eq!(faster(900), "أسرع 9 مرات");
    assert_eq!(faster(942), "أسرع 9,42 مرة");
}

#[test]
fn arabic_counts_texts_cores_and_seconds_in_every_form() {
    let ar = lang("ar");
    let cases = [
        (VOICE_TEXTS, 0, "لا نصوص"),
        (VOICE_TEXTS, 1, "نص واحد"),
        (VOICE_TEXTS, 2, "نصان"),
        (VOICE_TEXTS, 3, "3 نصوص"),
        (VOICE_TEXTS, 11, "11 نصًا"),
        (VOICE_TEXTS, 100, "100 نص"),
        (VOICE_TEXTS, 103, "103 نصوص"),
        (VOICE_CORES, 2, "نواتان"),
        (VOICE_CORES, 8, "8 أنوية"),
        (VOICE_CORES, 16, "16 نواة"),
        (VOICE_SECONDS, 1, "ثانية واحدة"),
        (VOICE_SECONDS, 9, "9 ثوانٍ"),
        (VOICE_SECONDS, 30, "30 ثانية"),
        (VOICE_MS, 5, "5 ميلي ثوانٍ"),
        (VOICE_MS, 20, "20 ميلي ثانية"),
        (VOICE_MS, 400, "400 ميلي ثانية"),
    ];
    for (key, n, want) in cases {
        assert_eq!(ar.counted(key, n), want, "{key} {n}");
    }
    let (en, fr) = (lang("en"), lang("fr"));
    assert_eq!(en.counted(VOICE_TEXTS, 1), "1 text");
    assert_eq!(en.counted(VOICE_CORES, 8), "8 cores");
    assert_eq!(fr.counted(VOICE_TEXTS, 0), "0 texte");
    assert_eq!(fr.counted(VOICE_CORES, 8), "8\u{a0}cœurs");
}

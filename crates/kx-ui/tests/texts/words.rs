//! The words each language uses: key names and labels, theme and product names, and its punctuation.

use crate::files::keys_of;
use crate::{all, lang};
use kx_module_api::keys::{SCROLL_FAILED, SCROLL_POINT, WORDS_DAMAGED, WORDS_LOCKED};
use kx_ui::i18n::keys::{
    CHIP_LOCKED, CHIP_MORE, CHIP_NONE, CHIP_UNDO, MODULE, THEME, WORDS_CANCEL, WORDS_FORGET,
    WORDS_FORGET_SHORT, WORDS_MENU, WORDS_OFF, WORDS_OFF_SHORT, WORDS_ON, WORDS_ON_SHORT,
    WORDS_PAUSE, WORDS_PAUSE_SHORT, WORDS_RESUME, WORDS_RESUME_SHORT,
};
use kx_ui::{Lang, find};

/// Arabic names of keys that print a Latin word: "مفتاح" and that word (owner, 2026-09-30).
const AR_LATIN_NAMES: [(&str, &str); 8] = [
    ("esc", "مفتاح Esc"),
    ("tab", "مفتاح Tab"),
    ("shift", "مفتاح Shift"),
    ("rshift", "مفتاح Shift الأيمن"),
    ("ctrl", "مفتاح Ctrl"),
    ("alt", "مفتاح Alt"),
    ("altgr", "مفتاح Alt Gr"),
    ("win", "مفتاح Windows"),
];
/// Arabic names that keep Microsoft's established term.
const AR_TERMS: [(&str, &str); 5] = [
    ("bksp", "مسافة للخلف"),
    ("del", "حذف"),
    ("enter", "إدخال"),
    ("caps", "تأمين الأحرف الكبيرة"),
    ("space", "مسافة"),
];
/// The word suggestion texts in Arabic, as the owner gave them (P7).
const AR_WORDS: [(&str, &str); 13] = [
    (CHIP_MORE, "عناوين بريد أخرى"),
    (CHIP_UNDO, "تراجع"),
    (CHIP_NONE, "لا توجد اقتراحات"),
    (CHIP_LOCKED, "خانة كلمة مرور: لا اقتراحات ولا تعلّم"),
    (WORDS_MENU, "قائمة الاقتراحات"),
    (WORDS_FORGET, "نسيان كلمة"),
    (WORDS_CANCEL, "إلغاء"),
    (WORDS_PAUSE, "إيقاف التعلّم مؤقتًا"),
    (WORDS_RESUME, "استئناف التعلّم"),
    (WORDS_OFF, "إيقاف الاقتراحات"),
    (WORDS_ON, "تشغيل الاقتراحات"),
    (
        WORDS_DAMAGED,
        "كان ملف الكلمات المتعلَّمة تالفًا، لذا يبدأ التعلّم من جديد.",
    ),
    (
        WORDS_LOCKED,
        "تعذّر فتح الكلمات المتعلَّمة، لذا لن يُحفَظ ما يُتعلَّم هذه المرة.",
    ),
];
/// The menu rows' short labels in English, French and Arabic, as the owner gave them (P7).
const SHORT: [(&str, [&str; 3]); 5] = [
    (WORDS_FORGET_SHORT, ["Forget", "Oublier", "نسيان"]),
    (WORDS_PAUSE_SHORT, ["Pause", "Pause", "إيقاف مؤقت"]),
    (WORDS_RESUME_SHORT, ["Resume", "Reprendre", "استئناف"]),
    (WORDS_OFF_SHORT, ["Off", "Désactiver", "إيقاف"]),
    (WORDS_ON_SHORT, ["On", "Activer", "تشغيل"]),
];
/// "Switch": Arabic keeps it for the mode button, never for a key's name.
const AR_SWITCH: &str = "التبديل";
/// The group of key names.
const KEY_GROUP: &str = "key.";
/// Characters French sets after a no-break space.
const HIGH_PUNCTUATION: [char; 4] = [':', ';', '!', '?'];
/// The spaces allowed before them and inside guillemets: no-break and narrow no-break.
const NO_BREAK: [char; 2] = ['\u{a0}', '\u{202f}'];
/// French guillemets, which hold a no-break space on their inner side.
const GUILLEMETS: [char; 2] = ['«', '»'];
/// Latin punctuation Arabic text writes as "،", "؛" and "؟".
const LATIN_ONLY: [char; 3] = [',', ';', '?'];
/// Windows' French says "pointez sur"; a bare "pointez" means tick off.
const FR_POINT: [&str; 2] = ["pointez", "pointez sur"];
/// A French word for a tap that Windows never uses; it says "appuyez sur".
const FR_NOT_TAP: &str = "touchez";

/// Lines that hold a number or time format, not prose, so their marks are not punctuation: `num.*`, `clip.time`.
const FORMATS: [&str; 3] = [".decimal =", ".group =", ".time ="];

/// The lines of `ftl` that hold text, not comments.
fn text_lines(ftl: &str) -> impl Iterator<Item = &str> {
    ftl.lines().filter(|l| !l.trim_start().starts_with('#'))
}

/// The prose lines of every file of language `l`: no comments and no formats.
fn prose(l: &Lang) -> impl Iterator<Item = &'static str> {
    let format = |line: &&str| FORMATS.iter().any(|f| line.trim_start().starts_with(f));
    l.files()
        .flat_map(text_lines)
        .filter(move |line| !format(line))
}

#[test]
fn theme_names_keep_model_names_and_translate_plain_words() {
    let names = |tag: &str| ["native", "et66", "dolch"].map(|id| lang(tag).name_of(THEME, id));
    assert_eq!(names("en"), ["Native Adaptive", "ET66", "Modern Dolch"]);
    assert_eq!(names("fr"), ["Natif adaptatif", "ET66", "Dolch moderne"]);
    assert_eq!(names("ar"), ["الأصلي المتكيّف", "ET66", "دولش الحديث"]);
}

#[test]
fn the_kernel_settings_carry_the_product_name() {
    for t in all() {
        assert_eq!(t.name_of(MODULE, "kernel"), "KeyXtend", "{}", t.lang().tag);
    }
}

#[test]
fn french_keys_print_french_words_and_arabic_keys_the_english_ones() {
    let labels = |tag: &str| {
        ["esc", "del", "caps", "altgr", "bksp", "shift"]
            .map(|k| lang(tag).text(&format!("label.{k}")))
    };
    let fr = [
        "Échap",
        "Suppr",
        "Verr. Maj",
        "Alt Gr",
        "Retour arrière",
        "Maj",
    ];
    assert_eq!(labels("fr"), fr);
    let p1 = ["Esc", "Del", "Caps", "AltGr", "Back", "Shift"];
    assert_eq!(labels("en"), p1, "the P1 test build's words");
    assert_eq!(labels("ar"), p1, "Arabic keyboards print the English words");
    assert_eq!(lang("fr").text("key.bksp"), "Retour arrière");
}

#[test]
fn arabic_names_latin_printed_keys_by_that_word_and_the_others_by_microsoft_terms() {
    let ar = lang("ar");
    for (id, name) in AR_LATIN_NAMES.iter().chain(&AR_TERMS) {
        assert_eq!(ar.text(&format!("{KEY_GROUP}{id}")), *name);
    }
    let names = keys_of(ar.lang().ftl)
        .into_iter()
        .filter(|k| k.starts_with(KEY_GROUP));
    for key in names {
        assert!(!ar.text(&key).contains(AR_SWITCH), "{key}");
    }
}

#[test]
fn the_word_suggestion_texts_read_in_arabic_as_the_owner_gave_them() {
    let ar = lang("ar");
    for (key, text) in AR_WORDS {
        assert_eq!(ar.text(key), text, "{key}");
    }
}

#[test]
fn the_menus_short_labels_read_as_the_owner_gave_them() {
    for (key, texts) in SHORT {
        for (tag, text) in ["en", "fr", "ar"].into_iter().zip(texts) {
            assert_eq!(lang(tag).text(key), text, "{tag} {key}");
        }
    }
}

#[test]
fn a_key_that_types_nothing_is_named_empty_in_each_language() {
    let empty = |tag: &str| lang(tag).text("key.empty");
    assert_eq!(empty("en"), "Empty key");
    assert_eq!(empty("fr"), "Touche vide");
    assert_eq!(
        empty("ar"),
        "مفتاح فارغ",
        "a key, then empty, both masculine"
    );
}

#[test]
fn french_spaces_high_punctuation_and_guillemets_with_no_break_spaces() {
    let fr = find("fr").expect("French");
    for line in prose(fr) {
        let chars: Vec<char> = line.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            let before = i.checked_sub(1).map(|b| chars[b]);
            let spaced_before = before.is_some_and(|b| NO_BREAK.contains(&b));
            let spaced_after = chars.get(i + 1).is_some_and(|a| NO_BREAK.contains(a));
            let needs_before = HIGH_PUNCTUATION.contains(c) || *c == GUILLEMETS[1];
            assert!(!needs_before || spaced_before, "{line}");
            assert!(*c != GUILLEMETS[0] || spaced_after, "{line}");
        }
    }
}

#[test]
fn french_points_sur_a_thing_and_taps_with_appuyez_sur() {
    let fr = lang("fr");
    let point = "Pointez sur ce que vous voulez faire défiler, puis appuyez sur une flèche.";
    assert_eq!(fr.text(SCROLL_POINT), point);
    assert!(fr.text(SCROLL_FAILED).contains("Appuyez sur une flèche"));
    for line in prose(fr.lang()).map(str::to_lowercase) {
        assert!(!line.contains(FR_NOT_TAP), "{line}");
        let [bare, with_sur] = FR_POINT.map(|w| line.matches(w).count());
        assert_eq!(bare, with_sur, "{line}");
    }
}

#[test]
fn french_names_switch_app_with_windows_own_term() {
    let fr = lang("fr").text("shortcut.switch-app");
    assert_eq!(fr, "Basculer entre les applications");
}

#[test]
fn arabic_uses_arabic_punctuation() {
    let ar = find("ar").expect("Arabic");
    for line in prose(ar) {
        assert!(!line.contains(LATIN_ONLY), "{line}");
    }
}

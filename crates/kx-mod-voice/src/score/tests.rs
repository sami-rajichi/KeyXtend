//! The scorer in English, French and Arabic: case, punctuation, apostrophes and digits fold. Arabic drops only its
//! marks and tatweel, so ه for ة, a lost hamza or ي for ى is a word wrong; a language with no rules gives no count.

use super::*;
use proptest::prelude::*;

/// Words wrong of the set text's words, for `lang`.
fn count(set: &str, heard: &str, lang: &str) -> (u16, u16) {
    let got = wrong(set, heard, lang).unwrap();
    (got.words, got.total)
}

#[test]
fn the_shipped_rules_load() {
    let rules = rules().unwrap();
    for lang in ["en", "fr", "ar"] {
        assert!(rules.has(lang), "{lang}");
    }
    assert!(!rules.has("de"));
}

#[test]
fn english_ignores_case_punctuation_apostrophe_forms_and_digits() {
    assert_eq!(
        count("The quick brown fox.", "the quick brown fox", "en"),
        (0, 4)
    );
    assert_eq!(count("Don't stop!", "don’t stop", "en"), (0, 2));
    assert_eq!(count("I have three cats.", "I have 3 cats", "en"), (0, 4));
    assert_eq!(count("‘Tis the season", "tis the season", "en"), (0, 3));
    assert_eq!(count("Hello, world!", "hello word", "en"), (1, 2));
}

#[test]
fn a_word_put_in_left_out_or_changed_is_one_word_wrong() {
    assert_eq!(count("one two three", "one three", "en"), (1, 3));
    assert_eq!(count("one two three", "one two two three", "en"), (1, 3));
    assert_eq!(count("one two three", "four five six", "en"), (3, 3));
    assert_eq!(count("one two three", "", "en"), (3, 3));
}

#[test]
fn french_keeps_accents_but_folds_case_punctuation_and_digits() {
    assert_eq!(count("« Bonjour ! »", "bonjour", "fr"), (0, 1));
    assert_eq!(count("C’est l’école.", "c'est l'école", "fr"), (0, 2));
    assert_eq!(count("Il a trois chats.", "il a 3 chats", "fr"), (0, 4));
    assert_eq!(count("J'ai une pomme.", "j'ai 1 pomme", "fr"), (0, 3));
    assert_eq!(
        count("Il est à côté.", "il est a côté", "fr"),
        (1, 4),
        "à and a are two words"
    );
    assert_eq!(count("Où es-tu ?", "ou es-tu", "fr"), (1, 3));
}

#[test]
fn arabic_tashkeel_and_tatweel_alone_are_no_difference() {
    let same = [
        ("مَدْرَسَةٌ جَمِيلَةٌ", "مدرسة جميلة", "harakat and tanwin"),
        (
            "بِسْمِ اللَّهِ الرَّحْمَٰنِ",
            "بسم الله الرحمن",
            "shadda and superscript alef",
        ),
        ("جميـــل", "جميل", "tatweel"),
        (
            "مرحبا، كيف حالك؟",
            "مرحبا كيف حالك",
            "Arabic comma and question mark",
        ),
    ];
    for (set, heard, why) in same {
        let words = u16::try_from(heard.split(' ').count()).unwrap();
        assert_eq!(count(set, heard, "ar"), (0, words), "{why}");
    }
}

#[test]
fn arabic_drops_exactly_the_harakat_the_superscript_alef_and_tatweel() {
    let rules = rules().unwrap();
    let gone: Vec<u32> = ('\u{0600}'..='\u{06FF}')
        .filter(|c| {
            let words = rules.tokens(&format!("ب{c}ب"), "ar");
            words.len() == 1 && words[0].as_str() == "بب"
        })
        .map(u32::from)
        .collect();
    let mut want: Vec<u32> = (0x064B..=0x0652).collect();
    want.extend([0x0640, 0x0670]);
    want.sort_unstable();
    assert_eq!(gone, want);
}

#[test]
fn arabic_counts_a_wrong_letter_form_as_a_word_wrong() {
    assert_eq!(
        count("ذهبت إلى المدرسة", "ذهبت إلى المدرسه", "ar"),
        (1, 3),
        "ه for ة"
    );
    let apart = [
        ("أحمد", "احمد", "hamza above alef"),
        ("إلى", "الى", "hamza below alef"),
        ("آمن", "امن", "madda"),
        ("سؤال", "سوال", "hamza on waw"),
        ("مسائل", "مسايل", "hamza on ya"),
        ("شيء", "شي", "lone hamza"),
        ("على", "علي", "alef maksura and ya"),
    ];
    for (set, heard, why) in apart {
        assert_eq!(count(set, heard, "ar"), (1, 1), "{why}");
    }
}

#[test]
fn arabic_joins_a_hamza_written_apart() {
    for (set, heard) in [
        ("سأل", "سا\u{0654}ل"),
        ("إلى", "ا\u{0655}لى"),
        ("آمن", "ا\u{0653}من"),
        ("سؤال", "سو\u{0654}ال"),
        ("مسائل", "مساي\u{0654}ل"),
        ("سَأَلَ", "سا\u{064E}\u{0654}ل"),
    ] {
        assert_eq!(count(set, heard, "ar"), (0, 1), "{heard}");
    }
}

#[test]
fn arabic_digits_count_as_their_number_word_in_either_gender() {
    assert_eq!(count("عندي ثلاثة كتب", "عندي ٣ كتب", "ar"), (0, 3));
    assert_eq!(count("عندي ثلاث سيارات", "عندي 3 سيارات", "ar"), (0, 3));
    assert_eq!(
        count("عشرة", "۱۰", "ar"),
        (0, 1),
        "Eastern Arabic-Indic digits"
    );
    assert_eq!(
        count("عندي ثلاث سيارات", "عندي ثلاثة سيارات", "ar"),
        (1, 3),
        "the wrong gender is a word wrong"
    );
    assert_eq!(
        count("أربعة", "اربعة", "ar"),
        (1, 1),
        "a number word keeps its hamza"
    );
}

#[test]
fn arabic_still_counts_a_real_mistake() {
    assert_eq!(
        count("ذهب الولد إلى المدرسة", "ذهب الولد إلى البيت", "ar"),
        (1, 4)
    );
    assert_eq!(count("كتب", "كتاب", "ar"), (1, 1));
}

#[test]
fn a_language_without_rules_gives_no_count_but_words_apart_still_work() {
    assert!(wrong("Guten Tag", "guten tag", "de").is_none());
    assert_eq!(apart("Guten Tag!", "guten Tag", "de"), 0);
    assert_eq!(apart("Guten Tag", "guten Abend", "de"), 1);
    assert_eq!(apart("ذهبَ الولدُ", "ذهب الولد", "ar"), 0);
    assert_eq!(apart("المدرسة", "المدرسه", "ar"), 1);
}

#[test]
fn a_broken_rules_table_is_refused() {
    let bad = [
        "[langs.\"fr-FR\"]\n",
        "[langs.fr]\nmap = { \"ab\" = \"c\" }\n",
        "[langs.fr]\nwords = { \"3\" = [\"trois chats\"] }\n",
        "[langs.fr]\nwords = { \"3 4\" = [\"trois\"] }\n",
        "[langs.fr]\nwords = { \"3\" = [] }\n",
        "[langs.fr]\nwords = { \"3\" = \"trois\" }\n",
        "[langs.fr]\nspeed = 2\n",
        "[langs.ar]\nmarks = { from = \"\\u06FF\", to = \"\\u0600\" }\n",
        "[langs.ar]\njoin = { \"\\u0627\" = \"\\u0623\" }\n",
        "[langs.ar]\njoin = { \"abc\" = \"d\" }\n",
        "[base]\napostrophes = 3\n",
    ];
    for text in bad {
        assert!(Rules::parse(text).is_err(), "{text}");
    }
    assert!(Rules::parse("[base]\n[langs.fr]\n").is_ok());
}

proptest! {
    #[test]
    fn any_text_scores_without_panic_and_matches_itself(text in "\\PC{0,80}", other in "\\PC{0,80}") {
        for lang in ["en", "fr", "ar", "de"] {
            prop_assert_eq!(apart(&text, &text, lang), 0);
            prop_assert_eq!(apart(&text, &other, lang), apart(&other, &text, lang));
            if let Some(got) = wrong(&text, &text, lang) {
                prop_assert_eq!(got.words, 0);
            }
        }
    }
}

//! Counts: "and N more" and a bare count in each plural form, CLDR's Arabic rule, Western digits and huge counts.

use crate::{all, items, lang};

/// CLDR's Arabic form for integer `n` (`n % 100` for few and many), as the text after a named list.
fn cldr_arabic_more(n: usize) -> String {
    match (n, n % 100) {
        (1, _) => "وإعداد آخر".to_owned(),
        (2, _) => "وإعدادان آخران".to_owned(),
        (_, 3..=10) => format!("و{n} إعدادات أخرى"),
        (_, 11..=99) => format!("و{n} إعدادًا آخر"),
        _ => format!("و{n} إعداد آخر"),
    }
}

/// CLDR's Arabic form for integer `n`, as a count with no list before it.
fn cldr_arabic_count(n: usize) -> String {
    match (n, n % 100) {
        (1, _) => "إعداد واحد".to_owned(),
        (2, _) => "إعدادان".to_owned(),
        (_, 3..=10) => format!("{n} إعدادات"),
        (_, 11..=99) => format!("{n} إعدادًا"),
        _ => format!("{n} إعداد"),
    }
}

#[test]
fn arabic_counts_the_settings_left_out_in_its_five_reachable_forms() {
    // CLDR's sixth form, zero, is never reached: a notice counts at least one setting.
    let ar = lang("ar");
    let forms = [
        (1, "وإعداد آخر"),
        (2, "وإعدادان آخران"),
        (3, "و3 إعدادات أخرى"),
        (10, "و10 إعدادات أخرى"),
        (11, "و11 إعدادًا آخر"),
        (99, "و99 إعدادًا آخر"),
        (100, "و100 إعداد آخر"),
        (102, "و102 إعداد آخر"),
        (103, "و103 إعدادات أخرى"),
        (111, "و111 إعدادًا آخر"),
    ];
    for (more, form) in forms {
        assert_eq!(ar.list(&items(&["a"]), more), format!("a {form}"));
    }
}

#[test]
fn arabic_and_french_forms_follow_cldr_for_every_count_up_to_1200() {
    let (ar, fr) = (lang("ar"), lang("fr"));
    for more in 1..=1200 {
        let named = format!("a {}", cldr_arabic_more(more));
        assert_eq!(ar.list(&items(&["a"]), more), named);
        assert_eq!(ar.list(&[], more), cldr_arabic_count(more));
        let word = if more == 1 { "autre" } else { "autres" };
        let french = format!("a et {more} {word}");
        assert_eq!(fr.list(&items(&["a"]), more), french);
    }
}

#[test]
fn french_and_english_count_the_rest() {
    let fr = lang("fr");
    assert_eq!(fr.list(&items(&["a"]), 1), "a et 1 autre");
    assert_eq!(fr.list(&items(&["a"]), 3), "a et 3 autres");
    assert_eq!(fr.list(&items(&["a"]), 103), "a et 103 autres");
    assert_eq!(lang("en").list(&items(&["a"]), 3), "a and 3 more");
}

#[test]
fn a_list_that_names_nothing_says_how_many() {
    let cases = [
        ("en", 1, "1 setting"),
        ("en", 3, "3 settings"),
        ("fr", 1, "1 paramètre"),
        ("fr", 3, "3 paramètres"),
        ("ar", 1, "إعداد واحد"),
        ("ar", 2, "إعدادان"),
        ("ar", 3, "3 إعدادات"),
        ("ar", 11, "11 إعدادًا"),
        ("ar", 103, "103 إعدادات"),
    ];
    for (tag, more, want) in cases {
        assert_eq!(lang(tag).list(&[], more), want, "{tag} {more}");
    }
    assert_eq!(lang("en").list(&[], 0), "");
}

#[test]
fn arabic_writes_western_digits() {
    let text = lang("ar").list(&items(&["a"]), 1234);
    assert!(text.contains("1234"), "{text}");
    assert!(!text.chars().any(|c| ('\u{660}'..='\u{669}').contains(&c)));
}

#[test]
fn a_huge_count_never_panics() {
    for t in all() {
        let text = t.list(&items(&["a"]), usize::MAX) + &t.list(&[], usize::MAX);
        assert!(text.chars().any(|c| c.is_ascii_digit()));
    }
}

/// CLDR's Arabic form for integer `n`, as the banner counts the notices still waiting.
fn cldr_arabic_notices(n: usize) -> String {
    match (n, n % 100) {
        (1, _) => "إشعار آخر".to_owned(),
        (2, _) => "إشعاران آخران".to_owned(),
        (_, 3..=10) => format!("{n} إشعارات أخرى"),
        (_, 11..=99) => format!("{n} إشعارًا آخر"),
        _ => format!("{n} إشعار آخر"),
    }
}

#[test]
fn the_banner_counts_the_notices_waiting_in_each_plural_form() {
    let (en, fr, ar) = (lang("en"), lang("fr"), lang("ar"));
    assert_eq!(en.more(1), "1 more notice");
    assert_eq!(fr.more(1), "1 autre notification");
    for n in 2..=1200 {
        assert_eq!(en.more(n), format!("{n} more notices"));
        assert_eq!(fr.more(n), format!("{n} autres notifications"));
    }
    for n in 1..=1200 {
        assert_eq!(ar.more(n), cldr_arabic_notices(n), "{n}");
    }
    assert!(all().all(|t| t.more(0).is_empty()), "nothing waits");
    assert!(all().all(|t| t.more(usize::MAX).chars().any(|c| c.is_ascii_digit())));
}

#[test]
fn a_page_number_names_its_page_and_the_count() {
    let cases = [("en", "2 of 3"), ("fr", "2 sur 3"), ("ar", "2 من 3")];
    for (tag, want) in cases {
        assert_eq!(lang(tag).page(2, 3), want, "{tag}");
    }
}

#[test]
fn the_layers_page_names_its_page_and_the_count_for_readers() {
    let cases = [
        ("en", "Page 2 of 3"),
        ("fr", "Page 2 sur 3"),
        ("ar", "الصفحة 2 من 3"),
    ];
    for (tag, want) in cases {
        assert_eq!(lang(tag).layer_page(2, 3), want, "{tag}");
    }
}

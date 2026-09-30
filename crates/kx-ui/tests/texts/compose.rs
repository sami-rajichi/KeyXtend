//! Texts built from parts: notices with their arguments, lists with "and N more", plurals, digits and the space bar.

use crate::{ISOLATE, all, lang, plain};
use kx_module_api::keys::{ALL_KEYS, MODULE_FAILED, SETTINGS_RESET};
use kx_module_api::{ModuleId, Notice, NoticeArg, args};

/// A notice `key` with every argument a notice may carry.
fn sample(key: &'static str) -> Notice {
    Notice {
        key,
        module: Some(ModuleId::new("keyboard")),
        args: vec![
            (args::MODULE, NoticeArg::Text("keyboard".into())),
            (args::KEYS, NoticeArg::List(vec!["size".into(), "x".into()])),
            (args::MORE, NoticeArg::Count(3)),
        ],
    }
}

/// `items` as a notice list.
fn items(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

#[test]
fn arabic_counts_the_settings_left_out_in_all_six_forms() {
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
        assert_eq!(plain(&ar.list(&items(&["a"]), more)), format!("a {form}"));
    }
}

/// CLDR's Arabic form for integer `n` (`n % 100` for few and many), as the text after the list.
fn cldr_arabic(n: usize) -> String {
    match (n, n % 100) {
        (1, _) => "وإعداد آخر".to_owned(),
        (2, _) => "وإعدادان آخران".to_owned(),
        (_, 3..=10) => format!("و{n} إعدادات أخرى"),
        (_, 11..=99) => format!("و{n} إعدادًا آخر"),
        _ => format!("و{n} إعداد آخر"),
    }
}

#[test]
fn arabic_forms_follow_cldr_for_every_count_up_to_1200() {
    let (ar, fr) = (lang("ar"), lang("fr"));
    for more in 1..=1200 {
        let want = format!("a {}", cldr_arabic(more));
        assert_eq!(plain(&ar.list(&items(&["a"]), more)), want);
        let word = if more == 1 { "autre" } else { "autres" };
        assert_eq!(
            plain(&fr.list(&items(&["a"]), more)),
            format!("a et {more} {word}")
        );
    }
}

#[test]
fn french_and_english_count_the_rest() {
    let fr = lang("fr");
    assert_eq!(plain(&fr.list(&items(&["a"]), 1)), "a et 1 autre");
    assert_eq!(plain(&fr.list(&items(&["a"]), 3)), "a et 3 autres");
    assert_eq!(plain(&fr.list(&items(&["a"]), 101)), "a et 101 autres");
    assert_eq!(plain(&fr.list(&items(&["a"]), 103)), "a et 103 autres");
    assert_eq!(plain(&lang("en").list(&items(&["a"]), 3)), "a and 3 more");
}

#[test]
fn each_language_joins_a_list_its_own_way() {
    let three = items(&["a", "b", "c"]);
    assert_eq!(lang("en").list(&three, 0), "a, b, c");
    assert_eq!(lang("fr").list(&three, 0), "a, b, c");
    assert_eq!(lang("ar").list(&three, 0), "a، b، c");
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
        assert!(
            t.list(&items(&["a"]), usize::MAX)
                .chars()
                .any(|c| c.is_ascii_digit())
        );
    }
}

#[test]
fn a_notice_shows_its_module_list_and_count() {
    let text = lang("en").notice(&sample(SETTINGS_RESET));
    let want = "Some Keyboard settings were not valid, so they went back to their defaults: size, x and 3 more.";
    assert_eq!(plain(&text), want);
}

#[test]
fn a_notice_about_an_unknown_module_shows_its_id() {
    let mut notice = sample(MODULE_FAILED);
    notice.args[0].1 = NoticeArg::Text("mystery".into());
    assert!(lang("fr").notice(&notice).contains("mystery"));
}

#[test]
fn every_notice_formats_with_its_arguments_in_every_language() {
    for t in all() {
        for key in ALL_KEYS {
            let text = t.notice(&sample(key));
            assert!(
                !text.contains(['{', '}']) && text != key,
                "{}: {text}",
                t.lang().tag
            );
        }
    }
}

#[test]
fn latin_words_inside_arabic_text_are_isolated() {
    let mut notice = sample(SETTINGS_RESET);
    notice.args.truncate(2);
    notice.args[1].1 = NoticeArg::List(items(&["log_level"]));
    let text = lang("ar").notice(&notice);
    let isolated: String = [
        ISOLATE[0], 'l', 'o', 'g', '_', 'l', 'e', 'v', 'e', 'l', ISOLATE[1],
    ]
    .iter()
    .collect();
    assert!(text.contains(&isolated), "{text:?}");
}

#[test]
fn the_space_bar_joins_the_language_and_the_layout() {
    let en = lang("en");
    assert_eq!(plain(&en.space(&items(&["English", "US"]))), "English · US");
    assert_eq!(en.space(&items(&["English"])), "English");
    assert_eq!(en.space(&[]), "");
}

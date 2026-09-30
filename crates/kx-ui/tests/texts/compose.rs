//! Texts built from parts: notices with their arguments, joined lists, hidden marks and the space bar.

use crate::{ISOLATE, all, items, lang, plain};
use kx_module_api::keys::{ALL_KEYS, MODULE_FAILED, SETTINGS_RESET, SETTINGS_UNSAVED};
use kx_module_api::{ModuleId, Notice, NoticeArg, args};
use kx_ui::i18n::keys::LIST_JOIN;

/// Every bidi control a quoted TOML key may hold, Fluent's own isolation marks included.
const CONTROLS: [char; 12] = [
    '\u{61c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}',
    '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
];
/// Line breaks a quoted TOML key may hold: LF, CR, NEL, and the line and paragraph separators.
const BREAKS: [char; 5] = ['\n', '\r', '\u{85}', '\u{2028}', '\u{2029}'];
/// A C0 control character (BEL).
const C0: char = '\u{7}';

/// A notice `key` with every argument a notice may carry.
fn sample(key: &'static str) -> Notice {
    Notice {
        key,
        module: Some(ModuleId::new("keyboard")),
        args: vec![
            (args::MODULE, NoticeArg::Text("keyboard".into())),
            (args::KEYS, NoticeArg::List(items(&["size", "x"]))),
            (args::MORE, NoticeArg::Count(3)),
        ],
    }
}

#[test]
fn a_notice_shows_its_module_list_and_count() {
    let text = lang("en").notice(&sample(SETTINGS_RESET));
    let want = "Some Keyboard settings were not valid, so they went back to their defaults: size, x and 3 more.";
    assert_eq!(plain(&text), want);
}

#[test]
fn a_reset_that_names_no_setting_says_how_many() {
    let mut notice = sample(SETTINGS_RESET);
    notice.args[1].1 = NoticeArg::List(Vec::new());
    let text = lang("en").notice(&notice);
    let want =
        "Some Keyboard settings were not valid, so they went back to their defaults: 3 settings.";
    assert_eq!(plain(&text), want);
}

#[test]
fn french_names_the_part_in_guillemets() {
    let fr = lang("fr");
    let text = plain(&fr.notice(&sample(MODULE_FAILED)));
    let want = "La partie «\u{a0}Clavier\u{a0}» s’est arrêtée après une erreur. Tout le reste fonctionne toujours.";
    assert_eq!(text, want);
    let unsaved = fr.notice(&sample(SETTINGS_UNSAVED));
    assert!(unsaved.contains("ne dureront que jusqu’à"), "{unsaved}");
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
            let tag = t.lang().tag;
            assert!(!text.contains(['{', '}']) && text != key, "{tag}: {text}");
        }
    }
}

#[test]
fn each_language_joins_a_list_its_own_way() {
    let three = items(&["a", "b", "c"]);
    assert_eq!(lang("en").list(&three, 0), "a, b, c");
    assert_eq!(lang("fr").list(&three, 0), "a, b, c");
    assert_eq!(lang("ar").list(&three, 0), "a، b، c");
}

#[test]
fn latin_words_inside_arabic_text_are_isolated() {
    let mut notice = sample(SETTINGS_RESET);
    notice.args.truncate(2);
    notice.args[1].1 = NoticeArg::List(items(&["log_level"]));
    let text = lang("ar").notice(&notice);
    let isolated = format!("{}log_level{}", ISOLATE[0], ISOLATE[1]);
    assert!(text.contains(&isolated), "{text:?}");
}

#[test]
fn bidi_controls_in_names_and_words_are_dropped() {
    let all_controls: String = CONTROLS.iter().collect();
    for t in all() {
        assert_eq!(t.list(&[format!("a{all_controls}b")], 0), "ab");
    }
    let only_controls = lang("en").list(std::slice::from_ref(&all_controls), 0);
    assert_eq!(plain(&only_controls), "1 setting", "counted, not named");
    let mut notice = sample(SETTINGS_RESET);
    notice.args[0].1 = NoticeArg::Text(format!("{all_controls}mystery"));
    notice.args[1].1 = NoticeArg::List(vec![format!("\u{202e}{all_controls}evil")]);
    let text = lang("ar").notice(&notice);
    let foreign = |c: &char| CONTROLS.contains(c) && !ISOLATE.contains(c);
    assert!(!text.chars().any(|c| foreign(&c)), "{text:?}");
    let count = |mark: char| text.chars().filter(|c| *c == mark).count();
    assert_eq!(count(ISOLATE[0]), count(ISOLATE[1]), "{text:?}");
}

#[test]
fn line_breaks_and_control_characters_in_names_are_dropped() {
    let names: Vec<String> = BREAKS.iter().map(|b| format!("a{b}b")).collect();
    let names = [names, vec![format!("c{C0}d")]].concat();
    let mut notice = sample(SETTINGS_RESET);
    notice.args[1].1 = NoticeArg::List(names.clone());
    for t in all() {
        let tag = t.lang().tag;
        let want = ["ab", "ab", "ab", "ab", "ab", "cd"].join(&t.text(LIST_JOIN));
        assert_eq!(t.list(&names, 0), want, "{tag}");
        let text = t.notice(&notice);
        let bad = text.chars().find(|c| c.is_control() || BREAKS.contains(c));
        assert_eq!(bad, None, "{tag}: {text:?}");
        assert!(plain(&text).contains(&want), "{tag}: {text:?}");
    }
}

#[test]
fn the_space_bar_joins_the_language_and_the_layout() {
    let en = lang("en");
    assert_eq!(plain(&en.space(&items(&["English", "US"]))), "English · US");
    assert_eq!(en.space(&items(&["English"])), "English");
    assert_eq!(en.space(&[]), "");
}

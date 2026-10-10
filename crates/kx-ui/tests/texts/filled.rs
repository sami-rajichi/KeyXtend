//! Every text the hub fills in for QML holds no bidi isolate, which would make Qt search every font (ADR-0022), and
//! the filled texts the Quick reading-order test draws.

use crate::files::keys_of;
use crate::keyboard_defaults;
use crate::qml::spaces;
use kx_module_api::keys::{ALL_KEYS, SETTINGS_RESET};
use kx_module_api::{Notice, NoticeArg, args};
use kx_ui::i18n::keys::MODULE;
use kx_ui::{I18n, LANGS};
use serde_json::{Value, json};
use std::ops::RangeInclusive;

/// Counts that reach every plural form: one, two, few and many, and 100 for Arabic's other.
const COUNTS: [usize; 5] = [1, 2, 3, 11, 100];
/// How many settings a notice's list leaves out: Arabic's few form.
const LEFT_OUT: usize = COUNTS[2];
/// A module id no text names, which shows as itself: a Latin word inside any language.
const UNKNOWN: &str = "mystery";
/// How many real `[keyboard]` setting names a list names.
const NAMED: usize = 2;
/// The bidi isolates: LRI, RLI, FSI and PDI.
const ISOLATES: RangeInclusive<char> = '\u{2066}'..='\u{2069}';
/// A language and a layout name as an OS may give them, wrapped in isolates.
const MARKED_OS_NAMES: [&str; 2] = ["\u{2068}العربية\u{2069}", "\u{2066}Arabic (102)\u{2069}"];

/// Every module id the texts name, then one they do not.
fn modules(t: &I18n) -> Vec<String> {
    let prefix = format!("{MODULE}.");
    let named = keys_of(t.lang().ftl).into_iter();
    let ids = named.filter_map(|k| k.strip_prefix(&prefix).map(str::to_owned));
    ids.chain([UNKNOWN.to_owned()]).collect()
}

/// Real setting names, as a reset notice lists them.
fn setting_names() -> Vec<String> {
    keyboard_defaults().keys().take(NAMED).cloned().collect()
}

/// Notice `key` about `module`, listing `names` and leaving `more` out.
fn notice(t: &I18n, key: &'static str, module: &str, names: &[String], more: usize) -> String {
    t.notice(&Notice {
        key,
        module: None,
        args: vec![
            (args::MODULE, NoticeArg::Text(module.to_owned())),
            (args::KEYS, NoticeArg::List(names.to_vec())),
            (args::MORE, NoticeArg::Count(more)),
        ],
    })
}

/// Lists, banner counts and page numbers at each plural form.
fn counted(t: &I18n) -> Vec<String> {
    let names = setting_names();
    let each = |n: usize| [t.list(&names, n), t.list(&[], n), t.more(n), t.page(1, n)];
    COUNTS.iter().flat_map(|n| each(*n)).collect()
}

/// Every notice with each module name, and with settings named alone, named with more, or only counted.
fn notices(t: &I18n) -> Vec<String> {
    let names = setting_names();
    let lists = [
        (names.clone(), 0),
        (names, LEFT_OUT),
        (Vec::new(), LEFT_OUT),
    ];
    let mut out = Vec::new();
    for key in ALL_KEYS {
        for module in modules(t) {
            out.extend(lists.iter().map(|(l, n)| notice(t, key, &module, l, *n)));
        }
    }
    out
}

/// Every text with arguments, and the space bar for the face's states and for marked OS names.
fn every_filled(t: &I18n) -> Vec<String> {
    let os_names: Vec<String> = MARKED_OS_NAMES.iter().map(|n| (*n).to_owned()).collect();
    let space = spaces().iter().map(|(_, parts)| t.space(parts)).collect();
    [counted(t), notices(t), space, vec![t.space(&os_names)]].concat()
}

/// The texts the Quick reading-order test draws, per language tag: each space-bar text with its parts and face state,
/// and a reset notice naming settings and leaving some out.
pub fn order() -> Value {
    let mut by_lang = serde_json::Map::new();
    for l in &LANGS {
        let t = I18n::new(l);
        let space: Vec<Value> = spaces()
            .iter()
            .map(|(state, parts)| json!({ "state": state, "parts": parts, "text": t.space(parts) }))
            .collect();
        let names = setting_names();
        let reset = notice(&t, SETTINGS_RESET, UNKNOWN, &names, LEFT_OUT);
        let reset = json!({ "names": names, "more": LEFT_OUT.to_string(), "text": reset });
        by_lang.insert(l.tag.to_owned(), json!({ "spaces": space, "reset": reset }));
    }
    Value::Object(by_lang)
}

#[test]
fn no_text_the_hub_gives_holds_a_bidi_isolate() {
    for l in &LANGS {
        let t = I18n::new(l);
        let keys = keys_of(l.ftl).into_iter().map(|k| t.text(&k));
        for text in keys.chain(every_filled(&t)) {
            let found = text.chars().find(|c| ISOLATES.contains(c));
            assert_eq!(found, None, "{}: {text:?}", l.tag);
        }
    }
}

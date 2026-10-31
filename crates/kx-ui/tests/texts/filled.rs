//! Every text the hub fills in for QML holds no bidi control, which could make Qt search every font (ADR-0022), QML
//! draws OS names only through the hub's cleaning, and the filled texts the Quick reading-order test draws.

use crate::combo;
use crate::files::keys_of;
use crate::markup::code;
use crate::qml::spaces;
use crate::{CONTROLS, keyboard_defaults, qml_files};
use kx_module_api::keys::{ALL_KEYS, SETTINGS_RESET};
use kx_module_api::{Notice, NoticeArg, args};
use kx_ui::i18n::keys::MODULE;
use kx_ui::{I18n, LANGS};
use serde_json::{Value, json};

/// Counts that reach every plural form: one, two, few and many, and 100 for Arabic's other.
const COUNTS: [usize; 5] = [1, 2, 3, 11, 100];
/// How many settings a notice's list leaves out: Arabic's few form.
const LEFT_OUT: usize = COUNTS[2];
/// A module id no text names, which shows as itself: a Latin word inside any language.
const UNKNOWN: &str = "mystery";
/// How many real `[keyboard]` setting names a list names.
const NAMED: usize = 2;
/// A language and a layout name as an OS gives them; the tests wrap them in every bidi control.
const OS_NAMES: [&str; 2] = ["العربية", "Arabic (102)"];
/// Where QML reads OS names from the face: a list row's names and the space bar's parts.
const OS_NAME_READS: [&str; 2] = ["row.name", "lang.space"];
/// The hub's call that cleans OS names, as QML makes it.
const SPACE_CALL: &str = "space(";

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

/// Notice `key` about `module`, listing `names` and leaving `more` out; its language is named as an OS names it, with
/// every bidi control around.
fn notice(t: &I18n, key: &'static str, module: &str, names: &[String], more: usize) -> String {
    let controls: String = CONTROLS.iter().collect();
    let lang = format!("{controls}{}{controls}", OS_NAMES[0]);
    t.notice(&Notice {
        key,
        module: None,
        args: vec![
            (args::MODULE, NoticeArg::Text(module.to_owned())),
            (args::KEYS, NoticeArg::List(names.to_vec())),
            (args::MORE, NoticeArg::Count(more)),
            (args::LANG, NoticeArg::Text(lang)),
        ],
    })
}

/// Lists, banner counts and page numbers at each plural form.
fn counted(t: &I18n) -> Vec<String> {
    let names = setting_names();
    let each = |n: usize| {
        let paged = [t.page(1, n), t.layer_page(1, n)];
        [t.list(&names, n), t.list(&[], n), t.more(n)]
            .into_iter()
            .chain(paged)
    };
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

/// The space bar for the face's states, and marked OS names together and one by one, as the space bar and the
/// language list draw them.
fn os_texts(t: &I18n) -> Vec<String> {
    let controls: String = CONTROLS.iter().collect();
    let marked: Vec<String> = OS_NAMES
        .iter()
        .map(|n| format!("{controls}{n}{controls}"))
        .collect();
    let alone = marked.iter().map(|n| t.space(std::slice::from_ref(n)));
    let states = spaces();
    let space = states.iter().map(|(_, parts)| t.space(parts));
    space.chain(alone).chain([t.space(&marked)]).collect()
}

/// The words a list of `names` gains when `more` are left out, such as "and 3 more", so a test finds the whole count
/// rather than its first digit.
fn count_words(t: &I18n, names: &[String], more: usize) -> String {
    let [listed, full] = [t.list(names, 0), t.list(names, more)];
    let words = full
        .strip_prefix(&listed)
        .expect("a longer list starts with its names");
    words.trim().to_owned()
}

/// Shortcut tips whose keys a right-to-left line could move, each with its name and keys apart.
fn tips(t: &I18n) -> Vec<Value> {
    let tip = |(key, steps): &(&str, &[&[&str]])| {
        let steps = combo::steps(steps);
        let (name, keys) = (t.text(key), t.keys(&steps));
        json!({ "name": name, "keys": keys, "text": t.keys_tip(key, &steps) })
    };
    combo::ORDERED.iter().map(tip).collect()
}

/// The texts the Quick reading-order test draws, per language tag: each space-bar text with its parts and face state,
/// a reset notice naming settings and leaving some out, and shortcut tips.
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
        let more = count_words(&t, &names, LEFT_OUT);
        let reset = json!({ "names": names, "more": more, "text": reset });
        let tips = tips(&t);
        by_lang.insert(
            l.tag.to_owned(),
            json!({ "spaces": space, "reset": reset, "tips": tips }),
        );
    }
    Value::Object(by_lang)
}

#[test]
fn no_text_the_hub_gives_holds_a_bidi_control() {
    for l in &LANGS {
        let t = I18n::new(l);
        let keys = l.files().flat_map(keys_of).map(|k| t.text(&k));
        let filled = [
            counted(&t),
            notices(&t),
            os_texts(&t),
            crate::clip::filled(&t),
        ]
        .concat();
        for text in keys.chain(filled) {
            let found = text.chars().find(|c| CONTROLS.contains(c));
            assert_eq!(found, None, "{}: {text:?}", l.tag);
        }
    }
}

/// The `OS_NAME_READS` entries code line `l` reads, each as a whole name such as `r.row.name` but not `arrow.name`.
fn os_name_reads(l: &str) -> Vec<&'static str> {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let whole = |i: usize| !l[..i].chars().next_back().is_some_and(word);
    let read = |r: &&str| l.match_indices(*r).any(|(i, _)| whole(i));
    OS_NAME_READS.into_iter().filter(read).collect()
}

#[test]
fn qml_draws_os_names_only_through_the_hubs_cleaning() {
    assert_eq!(os_name_reads("x: r.row.name[0]"), ["row.name"]);
    assert!(os_name_reads("x: arrow.name").is_empty());
    let mut seen = Vec::new();
    for (file, text) in qml_files() {
        for (n, line) in text.lines().map(code).enumerate() {
            let reads = os_name_reads(&line);
            let at = format!("{file}:{}: {line}", n + 1);
            assert!(reads.is_empty() || line.contains(SPACE_CALL), "{at}");
            seen.extend(reads);
        }
    }
    // Each read is found somewhere, so a renamed receiver cannot switch the scan off unseen.
    for r in OS_NAME_READS {
        assert!(seen.contains(&r), "no QML line reads {r}");
    }
}

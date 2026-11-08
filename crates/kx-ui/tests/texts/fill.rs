//! Quick-fill's texts: every key the hub names is in every language, none is left unused, each language takes the same
//! placeables, the vault's kinds and labels each have their text, and the counts, dates and notes read as they should.

mod arabic;
mod words;

use crate::files::keys_of;
use crate::{all, lang};
use kx_module_api::ports::{
    Date, EntryKind, FieldButton, FillButton, FillFilter, FillNote, NoteCounts, PasswordAsk, Pick,
    RowButton, When,
};
use kx_ui::I18n;
use kx_ui::LANGS;
use kx_ui::i18n::keys::{
    FILL_ADD, FILL_ASK, FILL_CHECKS, FILL_CONFIRM, FILL_DRIVES, FILL_EDIT_TITLE, FILL_EMPTY,
    FILL_FIELDS, FILL_FILTER, FILL_FOLDERS, FILL_FOREIGN, FILL_FOREIGN_UNDATED, FILL_FOUND,
    FILL_HIDE, FILL_JOIN, FILL_KIND, FILL_KINDS, FILL_LABEL, FILL_MASK, FILL_NEW_TITLE,
    FILL_NO_PLACES, FILL_NONE, FILL_NOTE, FILL_PICK, FILL_PLACE_WAITING, FILL_PLACES,
    FILL_REMOVE_PLACE, FILL_REPLACE, FILL_SEARCH, FILL_TITLE, FILL_UPDATED_DATE,
    FILL_UPDATED_TODAY, FILL_UPDATED_YESTERDAY, FILL_VALUES, FILL_WAITING, FILL_WORD, MODULE,
    ask_id, entry_kind_id, field_button_id, fill_button_id, fill_filter_id, fill_note_id, pick_id,
    row_button_id,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The vault module's tables, from this crate: kinds with their filters, and the labels.
const KINDS: &str = "../kx-mod-vault/kinds.toml";
const LABELS: &str = "../kx-mod-vault/labels.toml";
/// What a placeable that picks a plural form is followed by.
const SELECT: &str = "->";
/// A place's short name, as the places table gives one.
pub const PLACE: &str = "OneDrive";
/// The counts that reach every Arabic plural form.
const COUNTS: [usize; 7] = [0, 1, 2, 3, 11, 100, 103];

/// The Quick-fill keys the code names one by one.
const FIXED: [&str; 29] = [
    FILL_TITLE,
    FILL_SEARCH,
    FILL_EMPTY,
    FILL_NONE,
    FILL_MASK,
    FILL_JOIN,
    FILL_FIELDS,
    FILL_VALUES,
    FILL_WAITING,
    FILL_HIDE,
    FILL_PICK,
    FILL_ADD,
    FILL_NEW_TITLE,
    FILL_EDIT_TITLE,
    FILL_KINDS,
    FILL_CONFIRM,
    FILL_CHECKS,
    FILL_NO_PLACES,
    FILL_UPDATED_TODAY,
    FILL_UPDATED_YESTERDAY,
    FILL_UPDATED_DATE,
    FILL_PLACE_WAITING,
    FILL_REPLACE,
    FILL_REMOVE_PLACE,
    FILL_PLACES,
    FILL_DRIVES,
    FILL_FOUND,
    FILL_FOREIGN,
    FILL_FOREIGN_UNDATED,
];

/// Every kind of password page, `Open` both with and without this computer's own padlock.
pub const ASKS: [PasswordAsk; 6] = [
    PasswordAsk::New { min: 8 },
    PasswordAsk::Check { tries: 3 },
    PasswordAsk::Open { own: false },
    PasswordAsk::Open { own: true },
    PasswordAsk::Foreign,
    PasswordAsk::Keepass,
];

/// The table file `rel` under this crate.
fn table(rel: &str) -> toml::Table {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let text = std::fs::read_to_string(path).expect("a vault table reads");
    toml::from_str(&text).expect("a vault table parses")
}

/// Field `field` of every row of `list` in table `rel`.
fn ids(rel: &str, list: &str, field: &str) -> Vec<String> {
    let rows = table(rel)[list].as_array().expect("a list of rows").clone();
    let id = |r: &toml::Value| r[field].as_str().expect("a text id").to_owned();
    rows.iter().map(id).collect()
}

/// The vault's label ids, in table order.
pub fn label_ids() -> Vec<String> {
    ids(LABELS, "label", "id")
}

/// Every Quick-fill key the code names, the groups' ids included.
fn keys() -> BTreeSet<String> {
    let word = |id: &str| format!("{FILL_WORD}{id}");
    let buttons = RowButton::ALL.map(row_button_id).into_iter();
    let buttons = buttons.chain(FieldButton::ALL.map(field_button_id));
    let buttons = buttons.chain(FillButton::ALL.map(fill_button_id)).map(word);
    let kinds = EntryKind::ALL.map(|k| format!("{FILL_KIND}{}", entry_kind_id(k)));
    let filters = FillFilter::ALL.map(|f| format!("{FILL_FILTER}{}", fill_filter_id(f)));
    let notes = FillNote::ALL.map(|n| format!("{FILL_NOTE}{}", fill_note_id(n)));
    let folders = Pick::ALL.map(|p| format!("{FILL_FOLDERS}{}", pick_id(p)));
    let asks = ASKS.map(|a| format!("{FILL_ASK}{}", ask_id(a)));
    let labels = label_ids()
        .into_iter()
        .map(|id| format!("{FILL_LABEL}{id}"));
    let fixed = FIXED.iter().map(|k| (*k).to_owned());
    let groups = kinds.into_iter().chain(filters).chain(notes);
    let groups = groups.chain(folders).chain(asks);
    fixed.chain(buttons).chain(groups).chain(labels).collect()
}

/// Each language's Quick-fill file, the one that holds its title.
pub fn fill_file(tag: &str) -> &'static str {
    let l = LANGS
        .iter()
        .find(|l| l.tag == tag)
        .expect("a shipped language");
    let mut files = l.files();
    files
        .find(|f| keys_of(f).contains(FILL_TITLE))
        .expect("the Quick-fill file")
}

/// The variables each key of `ftl` shows, leaving out those that only pick a plural form.
fn shown_vars(ftl: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut group = String::new();
    let mut key = None;
    for line in ftl.lines().filter(|l| !l.starts_with('#')) {
        if let Some((head, _)) = line.split_once(" =") {
            match head.trim_start().strip_prefix('.') {
                Some(attr) => key = Some(format!("{group}.{attr}")),
                None if !line.starts_with(char::is_whitespace) => head.clone_into(&mut group),
                None => {}
            }
        }
        if let Some(k) = &key {
            out.entry(k.clone()).or_default().extend(vars(line));
        }
    }
    out
}

/// The variables line `l` shows, leaving out a selector.
fn vars(l: &str) -> Vec<String> {
    let name = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    let mut out = Vec::new();
    for (at, _) in l.match_indices('$') {
        let rest = &l[at + 1..];
        let end = rest.find(|c| !name(c)).unwrap_or(rest.len());
        if !rest[end..].trim_start().starts_with(SELECT) {
            out.push(rest[..end].to_owned());
        }
    }
    out
}

/// A place's box written `days_ago` days back at 10:42 on 1 November 2026.
pub fn when(days_ago: u32) -> When {
    When {
        days_ago,
        date: Date::new(2026, 11, 1).expect("a date"),
        hour: 10,
        minute: 42,
    }
}

/// Every Quick-fill text `t` fills in with sample values, by key.
pub fn filled(t: &I18n) -> Vec<(String, String)> {
    let plain = keys().into_iter().map(|k| {
        let text = t.text(&k);
        (k, text)
    });
    let mut out: Vec<(String, String)> = plain.collect();
    let kind = t.text(&format!("{FILL_KIND}{}", entry_kind_id(EntryKind::Person)));
    for n in COUNTS {
        out.push((FILL_FIELDS.into(), t.fill_fields(&kind, n)));
        out.push((FILL_VALUES.into(), t.counted(FILL_VALUES, n)));
    }
    let asks = ASKS.map(|a| format!("{FILL_ASK}{}", ask_id(a)));
    for key in asks.iter().filter(|k| t.has(k)) {
        out.extend(COUNTS.map(|n| (key.clone(), t.counted(key, n))));
    }
    let counts = |added, skipped| NoteCounts { added, skipped };
    let imported = format!("{FILL_NOTE}{}", fill_note_id(FillNote::Imported));
    for (added, skipped) in [(0, 0), (1, 2), (12, 3), (103, 1)] {
        out.push((imported.clone(), t.fill_imported(counts(added, skipped))));
    }
    out.extend([0, 1, 5].map(|d| (FILL_UPDATED_DATE.to_owned(), t.fill_updated(&when(d)))));
    out.push((FILL_WAITING.into(), t.fill_waiting(PLACE)));
    out.push((FILL_FOREIGN_UNDATED.into(), t.fill_foreign_undated(PLACE)));
    out.push((FILL_REMOVE_PLACE.into(), t.fill_remove_place(PLACE)));
    for key in [FILL_FOUND, FILL_FOREIGN] {
        out.push((key.into(), t.fill_found(key, PLACE, &when(3))));
    }
    out
}

#[test]
fn every_fill_key_is_in_every_language_and_none_is_unused() {
    let named = keys();
    let english = keys_of(fill_file("en"));
    let unused: Vec<&String> = english.difference(&named).collect();
    assert!(unused.is_empty(), "no code names {unused:?}");
    for t in all() {
        let missing: Vec<&String> = named.iter().filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
    }
    assert_eq!(lang("en").name_of(MODULE, "vault"), "Quick-fill");
}

#[test]
fn each_language_shows_the_same_placeables() {
    let english = shown_vars(fill_file("en"));
    assert!(english[FILL_FOUND].contains("place"), "the scan reads vars");
    for tag in ["fr", "ar"] {
        assert_eq!(shown_vars(fill_file(tag)), english, "{tag}");
    }
}

#[test]
fn every_kind_and_label_in_the_vaults_tables_has_its_text() {
    let kinds = ids(KINDS, "kind", "id");
    let ours: Vec<String> = EntryKind::ALL.map(|k| entry_kind_id(k).to_owned()).into();
    assert_eq!(kinds, ours, "one id per kind, in the table's order");
    let filters: BTreeSet<String> = ids(KINDS, "kind", "filter").into_iter().collect();
    let ours: BTreeSet<String> = FillFilter::ALL.map(|f| fill_filter_id(f).to_owned()).into();
    assert!(filters.is_subset(&ours), "{filters:?}");
    for t in all() {
        let labels = t.fill_labels();
        let tag = t.lang().tag;
        assert_eq!(
            labels.keys().cloned().collect::<Vec<_>>(),
            sorted(label_ids()),
            "{tag}"
        );
        for id in label_ids() {
            assert_ne!(labels[&id], id, "{tag}: {id} is worded");
        }
    }
}

/// `ids` sorted, as a map's keys come.
fn sorted(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids
}

#[test]
fn the_label_names_follow_the_language() {
    let email = |tag: &str| lang(tag).fill_labels()["email"].clone();
    assert_eq!(email("en"), "Email");
    assert_eq!(email("fr"), "E-mail");
    assert_eq!(email("ar"), "البريد الإلكتروني");
    assert_eq!(
        lang("fr").fill_label("mystery"),
        "mystery",
        "an unknown id shows itself"
    );
}

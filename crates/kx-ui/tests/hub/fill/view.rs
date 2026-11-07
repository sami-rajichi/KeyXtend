//! The panel as QML reads it: every word in the keyboard's language, the filters in click order, a row's hint, a
//! hidden value only while it is shown, the note with its counts, the waiting footer, and no name, value or search in a
//! log line or a debug print.

use super::{fill, heard, listed, login, page, redacted, secret};
use crate::Rig;
use crate::logs::lines_of;
use crate::words::speaking;
use kx_module_api::ports::{
    EntryKind, EntrySheet, FieldLine, FieldValue, FillFilter, FillNote, FillPage, FillView, Hint,
    Label, NoteCounts, PageNote, PlaceKind, Sheet, Spot,
};
use kx_module_api::{Redacted, caret};
use kx_ui::fill_acts::{BUTTON, ROW};
use serde_json::{Value, json};

/// A value that appears nowhere else.
const SECRET: &str = "hunter2-zebra";

/// The panel showing `page` in a keyboard of language `iso`.
pub fn read(iso: &str, page: FillPage) -> Value {
    let (rig, _host) = speaking(iso, 5);
    let heard = heard(&rig);
    heard.shown(&rig, page);
    fill(&rig)
}

/// The entry page of a login whose password, `SECRET`, is hidden and `shown` or not.
fn entry(shown: bool) -> FillPage {
    let line = |label: &str, text: &str, hidden: bool| FieldLine {
        label: Label::Table(label.to_owned()),
        value: FieldValue::new(secret(text), hidden, shown),
        make: false,
    };
    let sheet = EntrySheet {
        kind: EntryKind::Login,
        name: redacted("Bank"),
        fields: vec![line("user", "alex", false), line("password", SECRET, true)],
    };
    page(4, Sheet::Entry(sheet))
}

#[test]
fn the_panel_words_its_title_buttons_filters_and_search() {
    let v = read("en", listed(3));
    assert_eq!(v["round"], 3);
    assert_eq!(v["title"], "Quick-fill");
    assert_eq!(v["words"]["window"], "KeyXtend Quick-fill");
    assert_eq!(v["words"]["mask"], "••••••••");
    assert_eq!(
        (v["words"]["yes"].clone(), v["words"]["no"].clone()),
        (json!("Yes"), json!("No"))
    );
    assert_eq!(v["buttons"]["new"], "New");
    assert_eq!(v["buttons"]["back-up-now"], "Back up now");
    let ids: Vec<&str> = v["filters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "all", "people", "logins", "wifi", "cards", "notes", "labels"
        ]
    );
    assert_eq!(
        v["filters"][0],
        json!({"id": "all", "label": "All", "on": true})
    );
    assert_eq!(v["filters"][6]["label"], "By label");
    let search =
        json!({"text": "", "caret": 0, "hint": "Type to search", "rtl": false, "arabic": false});
    assert_eq!(v["search"], search);
    assert_eq!(v["page"], json!({"at": 0, "count": 1}));
    assert_eq!(v["highlight"], 0);
    for gone in ["note", "waiting", "empty"] {
        assert!(v[gone].is_null(), "{gone}");
    }
}

#[test]
fn a_list_row_names_its_entry_hint_and_buttons() {
    let v = read("en", listed(3));
    let row = &v["sheet"]["rows"][0];
    assert_eq!(v["sheet"]["is"], "list");
    assert_eq!(
        row["id"], "7",
        "a string, as JavaScript loses a large number"
    );
    assert_eq!(row["kind"], "login");
    assert_eq!(
        row["name"],
        json!({"text": "Bank", "rtl": false, "arabic": false})
    );
    assert_eq!(row["hint"]["text"], "Login · 3 fields");
    let buttons = json!([
        {"id": "user", "label": "User"},
        {"id": "password", "label": "Password"},
        {"id": "both", "label": "Both"},
    ]);
    assert_eq!(row["buttons"], buttons);
}

#[test]
fn a_hidden_value_shows_only_while_shown() {
    let masked = read("en", entry(false));
    let fields = &masked["sheet"]["fields"];
    assert_eq!(masked["sheet"]["is"], "entry");
    assert_eq!(fields[0]["value"]["text"], "alex");
    assert_eq!(
        fields[0]["buttons"],
        json!([{"id": "insert", "label": "Insert"}])
    );
    assert!(fields[1]["value"].is_null(), "the mask shows");
    assert!(fields[1]["count"].is_null(), "which tells no length");
    assert_eq!(fields[1]["label"]["text"], "Password");
    assert_eq!(
        (fields[1]["hidden"].clone(), fields[1]["shown"].clone()),
        (json!(true), json!(false))
    );
    let show = json!([{"id": "insert", "label": "Insert"}, {"id": "show", "label": "Show"}]);
    assert_eq!(fields[1]["buttons"], show);
    assert!(!masked.to_string().contains(SECRET), "no copy while masked");
    let shown = read("en", entry(true));
    let field = &shown["sheet"]["fields"][1];
    assert_eq!(field["value"]["text"], SECRET);
    assert_eq!(field["shown"], true);
    assert_eq!(field["buttons"][1], json!({"id": "show", "label": "Hide"}));
}

#[test]
fn no_name_value_or_search_reaches_a_log_line_or_a_debug_print() {
    let (rig, _host) = speaking("en", 5);
    let heard = heard(&rig);
    let mut shown = entry(true);
    shown.search = redacted(SECRET);
    let view = FillView::new(shown.clone());
    assert!(!format!("{view:?}").contains(SECRET), "{view:?}");
    let lines = lines_of(|| {
        heard.shown(&rig, shown);
        let _ = rig.hub.fill_json();
        rig.hub.fill_act(ROW, SECRET, 0, 4);
        rig.hub.fill_act(BUTTON, "close", 0, 4);
        rig.hub.fill_act(SECRET, SECRET, 0, 4);
    });
    assert!(lines.iter().all(|l| !l.contains(SECRET)), "{lines:?}");
}

#[test]
fn a_row_hint_glimpses_values_with_the_mask_or_counts_fields() {
    let mut person = login(8, "Alex");
    person.kind = EntryKind::Person;
    person.hint = Hint::Fields(0);
    let mut glimpsed = login(9, "Mail");
    let values = vec![
        FieldValue::new(secret("alex@example.com"), false, false),
        FieldValue::new(secret(SECRET), true, false),
    ];
    glimpsed.hint = Hint::Values(values);
    let mut one = login(10, "Gym");
    one.hint = Hint::Fields(1);
    let v = read("en", page(2, Sheet::List(vec![person, glimpsed, one])));
    let rows = &v["sheet"]["rows"];
    assert_eq!(rows[0]["hint"]["text"], "Person · no fields");
    assert_eq!(rows[1]["hint"]["text"], "alex@example.com · ••••••••");
    assert_eq!(rows[2]["hint"]["text"], "Login · 1 field");
    assert!(!v.to_string().contains(SECRET));
}

#[test]
fn an_arabic_keyboard_words_the_panel_and_each_name_reads_its_own_way() {
    let mut list = listed(1);
    if let Sheet::List(rows) = &mut list.sheet {
        rows.push(login(8, "مصرف"));
    }
    let v = read("ar", list);
    assert_eq!(v["title"], "التعبئة السريعة");
    assert_eq!(v["search"]["hint"], "اكتب للبحث");
    let rows = &v["sheet"]["rows"];
    assert_eq!(
        rows[0]["name"],
        json!({"text": "Bank", "rtl": false, "arabic": false})
    );
    assert_eq!(
        rows[1]["name"],
        json!({"text": "مصرف", "rtl": true, "arabic": true})
    );
    assert_eq!(rows[1]["hint"]["text"], "حساب · 3 حقول");
    assert_eq!(rows[1]["hint"]["rtl"], true);
}

#[test]
fn hidden_marks_leave_names_and_the_search_and_an_empty_name_shows_its_kind() {
    let mut list = page(
        1,
        Sheet::List(vec![login(7, "Ba\u{202e}nk"), login(8, "\u{200f}")]),
    );
    let search = "ba\u{2066}n";
    list.search = redacted(search);
    let before_n = search.len() - 1;
    list.caret = Redacted::new(caret::units(search, before_n));
    let v = read("en", list);
    assert_eq!(v["sheet"]["rows"][0]["name"]["text"], "Bank");
    assert_eq!(v["sheet"]["rows"][1]["name"]["text"], "Login");
    assert_eq!(v["search"]["text"], "ban");
    assert_eq!(
        v["search"]["caret"], 2,
        "the module's count lost the mark too"
    );
}

#[test]
fn a_note_is_worded_with_its_counts() {
    let mut typed = listed(1);
    typed.note = Some(PageNote::plain(FillNote::NotTyped));
    let v = read("en", typed);
    assert_eq!(v["note"], "The fill did not finish. Check what was typed.");
    let mut imported = listed(1);
    let counts = NoteCounts {
        added: 1200,
        skipped: 2,
    };
    imported.note = Some(PageNote {
        note: FillNote::Imported,
        counts,
    });
    assert_eq!(
        read("en", imported.clone())["note"],
        "1,200 entries came in. Entries or values left out: 2."
    );
    assert_eq!(
        read("fr", imported)["note"],
        "1\u{202f}200 éléments importés. Éléments ou valeurs ignorés\u{a0}: 2."
    );
    let mut busy = listed(1);
    busy.note = Some(PageNote::plain(FillNote::ClipBusy));
    assert_eq!(
        read("en", busy.clone())["note"],
        "The clipboard is busy; try again."
    );
    assert_eq!(read("ar", busy)["note"], "الحافظة مشغولة؛ حاول مرة أخرى.");
}

#[test]
fn the_footer_names_the_place_the_backup_waits_for() {
    let mut waits = listed(1);
    waits.waiting = Some(Spot {
        kind: PlaceKind::Removable,
        name: redacted("USB (E:)\u{202e}"),
    });
    let v = read("en", waits);
    assert_eq!(
        v["waiting"],
        json!({"kind": "removable", "text": "Backup waiting for USB (E:)"})
    );
}

#[test]
fn an_empty_notebook_invites_a_first_entry_and_an_empty_search_finds_none() {
    let mut empty = page(1, Sheet::List(Vec::new()));
    empty.empty = true;
    let v = read("en", empty);
    assert_eq!(
        v["empty"],
        "Nothing saved yet. Click New to save your first entry."
    );
    assert!(v["sheet"]["none"].is_null(), "the invitation says it");
    let mut none = page(1, Sheet::List(Vec::new()));
    none.filter = FillFilter::Cards;
    let v = read("en", none);
    assert!(v["empty"].is_null());
    assert_eq!(v["sheet"]["none"], "No entry matches.");
    assert_eq!(v["filters"][4]["on"], true);
    assert_eq!(v["filters"][0]["on"], false);
}

#[test]
fn with_no_view_the_panel_reads_null() {
    let rig = Rig::new();
    let _heard = heard(&rig);
    assert_eq!(rig.hub.fill_json(), "null");
}

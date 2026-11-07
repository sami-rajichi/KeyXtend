//! The notebook's pages as QML reads them: By label and its values, New and Edit with what takes the letters and how
//! many dots a masked value draws, and the labels table to add a field. Each row's Open is worded for what it does.

use super::view::read;
use super::{heard, page, redacted, secret};
use crate::logs::lines_of;
use crate::words::speaking;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    EditSheet, EntryKind, EntrySheet, FieldLine, FieldValue, FillView, Label, LabelCount,
    LabelValue, Sheet, Typing,
};
use serde_json::{Value, json};

/// The sheet of `sheet` in English.
pub fn sheet(sheet: Sheet) -> Value {
    read("en", page(1, sheet))["sheet"].clone()
}

/// One button, worded.
pub fn button(id: &str, label: &str) -> Value {
    json!([{"id": id, "label": label}])
}

#[test]
fn by_label_counts_values_and_opens_one_label_to_insert_its_values() {
    let labels = vec![
        LabelCount {
            label: Label::Table("email".to_owned()),
            count: 3,
        },
        LabelCount {
            label: Label::Own(redacted("Locker")),
            count: 1,
        },
    ];
    let v = sheet(Sheet::Labels(labels));
    assert_eq!(v["is"], "labels");
    assert_eq!(v["rows"][0]["label"]["text"], "Email");
    assert_eq!(v["rows"][0]["values"], "3 values");
    assert_eq!(v["rows"][1]["label"]["text"], "Locker");
    assert_eq!(v["rows"][1]["values"], "1 value");
    assert_eq!(v["rows"][0]["buttons"], button("open", "Open"));
    let rows = vec![LabelValue {
        name: redacted("Alex"),
        value: FieldValue::new(secret("a@b.c"), false, false),
    }];
    let v = sheet(Sheet::Values {
        label: Label::Table("email".to_owned()),
        rows,
    });
    assert_eq!(v["is"], "values");
    assert_eq!(v["label"]["text"], "Email");
    assert_eq!(v["rows"][0]["name"]["text"], "Alex");
    assert_eq!(v["rows"][0]["value"]["text"], "a@b.c");
    assert_eq!(v["rows"][0]["buttons"], button("insert", "Insert"));
}

/// The ids of field `f`'s buttons, in order.
fn ids(f: &Value) -> Vec<String> {
    let buttons = f["buttons"].as_array().unwrap().iter();
    buttons
        .map(|b| b["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn new_asks_the_kind_first() {
    let ask = EditSheet {
        new: true,
        kind: None,
        name: redacted(""),
        fields: Vec::new(),
        typing: None,
        caret: Redacted::new(0),
        confirm: false,
    };
    let v = sheet(Sheet::Edit(ask));
    assert_eq!(v["is"], "edit");
    assert_eq!(v["title"], "New entry");
    assert_eq!(v["ask"], "What do you want to save?");
    assert!(v["kind"].is_null());
    assert_eq!(v["kinds"].as_array().unwrap().len(), EntryKind::ALL.len());
    assert_eq!(v["kinds"][2], json!({"id": "wifi", "label": "Wi-Fi"}));
}

#[test]
fn edit_offers_each_fields_buttons_and_shows_what_takes_the_letters() {
    let password = FieldLine {
        label: Label::Table("password".to_owned()),
        value: FieldValue::edit(secret("x"), true, false),
        make: true,
    };
    let own = FieldLine {
        label: Label::Own(redacted("Locker")),
        value: FieldValue::edit(secret("12"), false, false),
        make: false,
    };
    let edit = EditSheet {
        new: false,
        kind: Some(EntryKind::Login),
        name: redacted("Bank"),
        fields: vec![password, own],
        typing: Some(Typing::Value(1)),
        caret: Redacted::new(1),
        confirm: true,
    };
    let v = sheet(Sheet::Edit(edit));
    assert_eq!(v["title"], "Edit entry");
    assert_eq!(v["kind"], "login");
    assert!(v["ask"].is_null() && v["kinds"].as_array().unwrap().is_empty());
    assert_eq!(v["name"]["text"], "Bank");
    assert_eq!(v["typing"], false, "the name takes no letters");
    assert_eq!(v["caret"], 1, "in the line that takes them");
    assert_eq!(ids(&v["fields"][0]), ["show", "hidden", "paste", "make"]);
    assert_eq!(v["fields"][0]["buttons"][3]["label"], "Make one");
    assert_eq!(ids(&v["fields"][1]), ["hidden", "paste"]);
    assert_eq!(
        (v["fields"][0]["own"].clone(), v["fields"][1]["own"].clone()),
        (json!(false), json!(true))
    );
    assert!(v["fields"][0]["typing"].is_null());
    assert_eq!(v["fields"][1]["typing"], "value");
    assert_eq!(v["confirm"], "Delete this entry? It cannot come back.");
}

#[test]
fn the_name_or_an_own_label_can_take_the_letters() {
    let own = FieldLine {
        label: Label::Own(redacted("")),
        value: FieldValue::edit(secret(""), false, false),
        make: false,
    };
    let mut edit = EditSheet {
        new: true,
        kind: Some(EntryKind::Other),
        name: redacted(""),
        fields: vec![own],
        typing: Some(Typing::Name),
        caret: Redacted::new(0),
        confirm: false,
    };
    assert_eq!(sheet(Sheet::Edit(edit.clone()))["typing"], true);
    edit.typing = Some(Typing::Label(0));
    let v = sheet(Sheet::Edit(edit));
    assert_eq!(v["typing"], false);
    assert_eq!(v["fields"][0]["typing"], "label");
    assert!(v["confirm"].is_null());
}

#[test]
fn a_field_is_added_from_the_labels_table() {
    let v = sheet(Sheet::AddField(vec![
        Label::Table("phone".to_owned()),
        Label::Table("pin".to_owned()),
    ]));
    assert_eq!(v["is"], "add-field");
    assert_eq!(v["rows"][1]["label"]["text"], "PIN");
    assert_eq!(v["rows"][0]["buttons"], button("open", "Add"));
}

#[test]
fn a_hidden_value_not_shown_is_masked_on_edit_and_under_its_label() {
    const SECRET: &str = "quartz-hunter2";
    let line = FieldLine {
        label: Label::Table("password".to_owned()),
        value: FieldValue::edit(secret(SECRET), true, false),
        make: true,
    };
    let edit = EditSheet {
        new: false,
        kind: Some(EntryKind::Login),
        name: redacted("Bank"),
        fields: vec![line],
        typing: None,
        caret: Redacted::new(0),
        confirm: false,
    };
    let v = sheet(Sheet::Edit(edit));
    assert!(v["fields"][0]["value"].is_null(), "the mask shows");
    assert_eq!(v["fields"][0]["hidden"], true);
    assert_eq!(
        v["fields"][0]["count"],
        SECRET.chars().count(),
        "one dot per letter"
    );
    assert!(!v.to_string().contains(SECRET), "no copy while masked");
    let rows = vec![LabelValue {
        name: redacted("Bank"),
        value: FieldValue::new(secret(SECRET), true, false),
    }];
    let label = Label::Table("password".to_owned());
    let v = sheet(Sheet::Values { label, rows });
    assert!(v["rows"][0]["value"].is_null(), "the mask shows");
    assert!(v["rows"][0]["count"].is_null(), "which tells no length");
    assert!(!v.to_string().contains(SECRET), "no copy while masked");
}

#[test]
fn a_masked_values_count_and_caret_reach_no_log_line_or_debug_print() {
    const COUNT: usize = 7919;
    const CARET: u32 = 7907;
    let (rig, _host) = speaking("en", 5);
    let heard = heard(&rig);
    let line = FieldLine {
        label: Label::Table("password".to_owned()),
        value: FieldValue::masked_edit(COUNT),
        make: false,
    };
    let form = page(
        4,
        Sheet::Edit(EditSheet {
            new: true,
            kind: Some(EntryKind::Login),
            name: redacted(""),
            fields: vec![line],
            typing: Some(Typing::Value(0)),
            caret: Redacted::new(CARET),
            confirm: false,
        }),
    );
    let view = FillView::new(form.clone());
    let told = [COUNT.to_string(), CARET.to_string()];
    let shown = format!("{view:?}");
    assert!(!told.iter().any(|t| shown.contains(t)), "{shown}");
    let lines = lines_of(|| {
        heard.shown(&rig, form);
        let v: Value = serde_json::from_str(&rig.hub.fill_json()).expect("panel JSON");
        assert_eq!(v["sheet"]["fields"][0]["count"], COUNT, "QML reads it");
        assert_eq!(v["sheet"]["caret"], CARET, "and the caret");
    });
    let clean = |l: &String| !told.iter().any(|t| l.contains(t));
    assert!(lines.iter().all(clean), "{lines:?}");
}

#[test]
fn an_empty_hidden_value_carries_its_empty_text_on_every_page() {
    let blank = |value| FieldLine {
        label: Label::Table("password".to_owned()),
        value,
        make: false,
    };
    let edit = EditSheet {
        new: true,
        kind: Some(EntryKind::Login),
        name: redacted(""),
        fields: vec![blank(FieldValue::edit(secret(""), true, false))],
        typing: None,
        caret: Redacted::new(0),
        confirm: false,
    };
    let entry = EntrySheet {
        kind: EntryKind::Login,
        name: redacted("Bank"),
        fields: vec![blank(FieldValue::new(secret(""), true, false))],
    };
    for fields in [
        sheet(Sheet::Edit(edit))["fields"].clone(),
        sheet(Sheet::Entry(entry))["fields"].clone(),
    ] {
        assert_eq!(fields[0]["value"]["text"], "", "no mask draws");
        assert!(fields[0]["count"].is_null(), "and no dot");
        assert_eq!(fields[0]["hidden"], true);
        assert_eq!(fields[0]["shown"], false, "it still offers Show");
        assert!(ids(&fields[0]).contains(&"show".to_owned()));
    }
    let rows = vec![LabelValue {
        name: redacted("Bank"),
        value: FieldValue::new(secret(""), true, false),
    }];
    let label = Label::Table("password".to_owned());
    let v = sheet(Sheet::Values { label, rows });
    assert_eq!(v["rows"][0]["value"]["text"], "", "no mask draws");
}

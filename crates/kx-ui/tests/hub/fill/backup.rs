//! The backup and import pages as QML reads them: each place's state and Remove place's question on its row, the
//! places and folders to pick, the password page by what it asks, and the boxes found or from another computer.

use super::sheets::{button, sheet};
use super::view::read;
use super::{page, redacted, secret};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    BackupAsk, BackupSheet, BoxState, Date, FolderSheet, FoundRow, PasswordAsk, PasswordSheet,
    Pick, PlaceKind, PlaceRow, Sheet, Spot, When,
};
use serde_json::{Value, json};

/// 10:42 on 1 November 2026, `days_ago` days back.
fn when(days_ago: u32) -> When {
    When {
        days_ago,
        date: Date::new(2026, 11, 1).unwrap(),
        hour: 10,
        minute: 42,
    }
}

/// `name`, a place of kind `kind`.
fn spot(kind: PlaceKind, name: &str) -> Spot {
    Spot {
        kind,
        name: redacted(name),
    }
}

#[test]
fn the_backup_page_shows_each_places_state_and_offers_remove() {
    let places = vec![
        PlaceRow {
            place: spot(PlaceKind::Cloud, "OneDrive"),
            state: BoxState::Updated(when(0)),
        },
        PlaceRow {
            place: spot(PlaceKind::Removable, "USB (E:)"),
            state: BoxState::Updated(when(1)),
        },
        PlaceRow {
            place: spot(PlaceKind::Drive, "D:"),
            state: BoxState::Waiting,
        },
    ];
    let v = sheet(Sheet::Backup(BackupSheet {
        places,
        checks: true,
        ask: None,
    }));
    assert_eq!(v["is"], "backup");
    assert_eq!(v["checks"], true);
    assert!(v["none"].is_null());
    let row = &v["places"][0];
    assert_eq!(
        (row["kind"].clone(), row["name"]["text"].clone()),
        (json!("cloud"), json!("OneDrive"))
    );
    assert_eq!(row["state"], "Updated today 10:42");
    assert_eq!(row["waiting"], false);
    assert_eq!(row["buttons"], button("remove", "Remove"));
    assert_eq!(v["places"][1]["state"], "Updated yesterday 10:42");
    assert_eq!(v["places"][2]["state"], "Waiting");
    assert_eq!(v["places"][2]["waiting"], true);
    assert!(v["confirm"].is_null(), "no question shows");
}

#[test]
fn a_backup_page_with_no_place_invites_one() {
    let none = sheet(Sheet::Backup(BackupSheet {
        places: Vec::new(),
        checks: false,
        ask: None,
    }));
    assert_eq!(
        none["none"],
        "No backup yet. Add a place to keep a locked copy of your entries."
    );
    assert!(none["confirm"].is_null());
}

#[test]
fn remove_asks_on_its_row_before_the_place_goes() {
    let row_of = |name: &str| PlaceRow {
        place: spot(PlaceKind::Removable, name),
        state: BoxState::Waiting,
    };
    let sheet_of = |row: u8| {
        Sheet::Backup(BackupSheet {
            places: vec![row_of("USB (E:)"), row_of("USB (F:)")],
            checks: true,
            ask: Some(BackupAsk::Remove(row)),
        })
    };
    let text = "Remove USB (F:)? The backup file there stays.";
    let v = sheet(sheet_of(1));
    assert_eq!(v["confirm"], json!({"row": 1, "text": text}));
    assert!(v["replace"].is_null(), "one question at a time");
    let asked_in = |iso| read(iso, page(1, sheet_of(0)))["sheet"]["confirm"]["text"].clone();
    let asked = "هل تريد إزالة USB (E:)؟ يبقى ملف النسخة الاحتياطية هناك.";
    assert_eq!(asked_in("ar"), asked);
    let asked = "Retirer USB (E:)\u{a0}? Le fichier de sauvegarde qui s’y trouve est conservé.";
    assert_eq!(asked_in("fr"), asked);
    assert!(
        sheet(sheet_of(2))["confirm"].is_null(),
        "a row the page does not show asks nothing"
    );
    let marked = sheet(Sheet::Backup(BackupSheet {
        places: vec![row_of("\u{202e}USB (G:)")],
        checks: true,
        ask: Some(BackupAsk::Remove(0)),
    }));
    assert_eq!(
        marked["confirm"]["text"], "Remove USB (G:)? The backup file there stays.",
        "without hidden marks"
    );
}

#[test]
fn replace_asks_for_the_whole_page_while_the_notebook_did_not_read() {
    let replace = |iso| {
        let asking = Sheet::Backup(BackupSheet {
            places: Vec::new(),
            checks: true,
            ask: Some(BackupAsk::Replace),
        });
        read(iso, page(1, asking))["sheet"].clone()
    };
    let text = "Replace the backup? It may hold the entries this computer could not read.";
    let v = replace("en");
    assert_eq!(v["replace"], json!({"text": text}));
    assert!(v["confirm"].is_null(), "one question at a time");
    let ar = "هل تريد استبدال النسخة الاحتياطية؟ قد تحتوي على العناصر التي تعذّرت قراءتها على هذا الكمبيوتر.";
    assert_eq!(replace("ar")["replace"]["text"], ar);
    let unasked = sheet(Sheet::Backup(BackupSheet {
        places: Vec::new(),
        checks: true,
        ask: None,
    }));
    assert!(unasked["replace"].is_null());
}

#[test]
fn places_and_folders_open_what_holds_more_and_pick_the_rest() {
    let v = sheet(Sheet::Places(vec![spot(PlaceKind::Cloud, "OneDrive")]));
    assert_eq!(v["is"], "places");
    assert_eq!(v["text"], "Choose where to keep your backup.");
    assert_eq!(v["rows"][0]["buttons"], button("open", "Choose"));
    let rows = vec![
        spot(PlaceKind::Drive, "C:"),
        spot(PlaceKind::File, "KeyXtend Backup.kxbox"),
    ];
    let v = sheet(Sheet::Folders(FolderSheet {
        pick: Pick::Backup,
        folder: None,
        rows,
    }));
    assert_eq!(v["is"], "folders");
    assert_eq!(v["pick"], "backup");
    assert_eq!(v["text"], "Find your backup file.");
    assert_eq!(v["drives"], true);
    assert_eq!(v["folder"]["text"], "This PC");
    assert_eq!(v["rows"][0]["buttons"], button("open", "Open"));
    assert_eq!(v["rows"][1]["buttons"], button("open", "Choose"));
    let inside = FolderSheet {
        pick: Pick::Folder,
        folder: Some(redacted("Backups")),
        rows: Vec::new(),
    };
    let v = sheet(Sheet::Folders(inside));
    assert_eq!(
        (v["drives"].clone(), v["folder"]["text"].clone()),
        (json!(false), json!("Backups"))
    );
}

/// What is typed so far on the password page.
const TYPED: &str = "typed so far";

/// Where the caret stands on the password page.
const CARET: u32 = 3;

/// The password page asking `ask`, with `TYPED` typed.
fn asked(ask: PasswordAsk) -> Value {
    sheet(Sheet::Password(PasswordSheet {
        ask,
        text: secret(TYPED),
        caret: Redacted::new(CARET),
    }))
}

#[test]
fn a_new_password_names_its_least_length_and_a_check_its_tries() {
    let new = asked(PasswordAsk::New { min: 8 });
    assert_eq!(new["caret"], CARET);
    assert_eq!(
        (new["is"].clone(), new["ask"].clone()),
        (json!("password"), json!("new"))
    );
    let least = "Make a backup password of at least 8 characters. If you forget it, the backup cannot be opened.";
    assert_eq!(new["text"], least);
    assert_eq!(new["typed"]["text"], TYPED, "it shows while typed");
    let check = asked(PasswordAsk::Check { tries: 1 });
    assert_eq!(check["ask"], "check");
    let tries = "Type your backup password to check you still remember it. 1 try left.";
    assert_eq!(check["text"], tries);
}

#[test]
fn each_other_password_page_says_what_it_opens() {
    let cases = [
        (
            PasswordAsk::Open { own: false },
            "open",
            "Type this backup's password to bring your entries back.",
        ),
        (
            PasswordAsk::Foreign,
            "foreign",
            "Type the password of the backup from the other computer.",
        ),
        (
            PasswordAsk::Keepass,
            "keepass",
            "Type the KeePass file's password.",
        ),
    ];
    for (ask, id, text) in cases {
        let v = asked(ask);
        assert_eq!(
            (v["ask"].clone(), v["text"].clone()),
            (json!(id), json!(text))
        );
    }
    assert_eq!(asked(PasswordAsk::Open { own: true })["ask"], "own");
}

#[test]
fn a_found_box_brings_back_and_a_foreign_one_asks() {
    let found = FoundRow {
        place: spot(PlaceKind::Cloud, "OneDrive"),
        when: when(3),
    };
    let v = sheet(Sheet::Found(vec![found.clone()]));
    assert_eq!(v["is"], "found");
    assert_eq!(v["rows"][0]["kind"], "cloud");
    assert_eq!(v["rows"][0]["name"]["text"], "OneDrive");
    assert!(
        v["rows"][0]["text"]
            .as_str()
            .unwrap()
            .ends_with("on OneDrive. Bring it back?")
    );
    assert_eq!(v["rows"][0]["buttons"], button("open", "Bring back"));
    let v = sheet(Sheet::Foreign(found));
    assert_eq!(v["is"], "foreign");
    assert_eq!(v["kind"], "cloud");
    assert!(
        v["text"]
            .as_str()
            .unwrap()
            .starts_with("A backup from another computer is in OneDrive, from ")
    );
    let v = sheet(Sheet::ForeignUndated(spot(PlaceKind::Cloud, "OneDrive")));
    assert_eq!(v["is"], "foreign");
    assert_eq!(v["kind"], "cloud");
    assert_eq!(
        v["text"],
        "A backup from another computer is in OneDrive. Bring its entries in?"
    );
}

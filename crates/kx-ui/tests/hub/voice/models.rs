//! The Models page and the folder list: each model's size or why it does not fit, its marks, a download's progress
//! with Pause or Resume and Cancel, the best model's note, the folder line, the first download's question, and the
//! drives and folders.

use super::{acting, button, buttons, heard, panel, panel_json};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    FetchState, FolderLine, FolderList, ModelRow, ModelState, ModelsSheet, PanelButton, VoiceSheet,
};
use serde_json::{Value, json};

/// A model row named `name` of `bytes` in `state`, not the default, unused and unticked.
fn row(name: &str, bytes: u64, state: ModelState) -> ModelRow {
    ModelRow {
        name: Redacted::new(name.to_owned()),
        bytes,
        default: false,
        state,
        used: false,
        tick: false,
        fetch: None,
    }
}

/// The models folder on D:.
fn folder() -> FolderLine {
    FolderLine {
        path: Redacted::new("D:\\KeyXtend\\models".to_owned()),
        free: 212_000_000_000,
    }
}

/// The page with `rows`, Cohere as the best, untested, the folder shown and no question.
fn sheet(rows: Vec<ModelRow>) -> ModelsSheet {
    ModelsSheet {
        rows,
        best: Some("Cohere Transcribe".to_owned()),
        untested: true,
        slow: false,
        folder: Some(folder()),
        offer: None,
    }
}

/// The `[id, text]` of each of `row`'s marks.
fn marks(row: &Value) -> Vec<String> {
    let all = row["marks"].as_array().cloned().unwrap_or_default();
    all.iter()
        .map(|m| m["id"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// One model of each state: in use and the default, ready and ticked, missing, too big, and an other file.
fn each_state() -> Vec<ModelRow> {
    let mut used = row("Whisper small", 487_601_967, ModelState::Ready);
    (used.default, used.used) = (true, true);
    let mut ready = row("Whisper medium", 1_533_763_059, ModelState::Ready);
    ready.tick = true;
    let unfit = ModelState::NotHere {
        need_mb: 3277,
        have_mb: 3072,
        free_mb: 1434,
    };
    vec![
        used,
        ready,
        row("Whisper large", 3_095_033_483, ModelState::Missing),
        row("Cohere Transcribe", 4_000_000_000, unfit),
        row("mine.bin", 1_000_000, ModelState::Other),
    ]
}

#[test]
fn each_model_shows_its_size_and_marks() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let rows = each_state();
    let p = panel_json(&rig, &heard, panel(VoiceSheet::Models(sheet(rows))));
    assert_eq!(
        (&p["page"], &p["sheet"]["is"]),
        (&json!("models"), &json!("models"))
    );
    assert_eq!(
        (&p["title"], &p["sub"]),
        (&json!("Models"), &json!("For English"))
    );
    let rows = &p["sheet"]["rows"];
    let first = json!({"name": "Whisper small", "line": "488 MB", "tick": false, "fetch": null,
        "marks": [{"id": "used", "text": "In use"}, {"id": "default", "text": "Default"}]});
    assert_eq!(rows[0], first);
    assert_eq!(
        (marks(&rows[1]), &rows[1]["tick"]),
        (vec!["ready".to_owned()], &json!(true))
    );
    assert_eq!(marks(&rows[2]), ["download"]);
    assert_eq!(rows[2]["line"], "3.1 GB");
    assert_eq!(
        rows[3]["line"], "Needs 3.2 GB of memory, 1.4 GB free",
        "the free memory, not the total"
    );
    assert_eq!(marks(&rows[3]), ["not-here"]);
    assert_eq!(
        (marks(&rows[4]), &rows[4]["line"]),
        (vec!["other".to_owned()], &json!("1 MB"))
    );
}

#[test]
fn in_use_shows_only_for_a_file_that_is_here() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut missing = row("Parakeet", 739_508_576, ModelState::Missing);
    (missing.default, missing.used) = (true, true);
    let unfit = ModelState::NotHere {
        need_mb: 3277,
        have_mb: 3072,
        free_mb: 1434,
    };
    let mut big = row("Cohere Transcribe", 4_000_000_000, unfit);
    (big.default, big.used) = (true, true);
    let mut mine = row("mine.bin", 1_000_000, ModelState::Other);
    mine.used = true;
    let p = panel_json(
        &rig,
        &heard,
        panel(VoiceSheet::Models(sheet(vec![missing, big, mine]))),
    );
    let rows = &p["sheet"]["rows"];
    assert_eq!(
        marks(&rows[0]),
        ["default", "download"],
        "not downloaded yet"
    );
    assert_eq!(marks(&rows[1]), ["default", "not-here"]);
    assert_eq!(marks(&rows[2]), ["used"], "an other file in use");
}

#[test]
fn a_download_shows_its_progress_with_its_own_buttons_and_no_mark() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut paused = row("Whisper large", 3_095_033_483, ModelState::Missing);
    paused.fetch = Some(FetchState {
        done: 1_000_000_000,
        total: 3_095_033_483,
        paused: true,
    });
    let mut running = paused.clone();
    running.fetch = Some(FetchState {
        done: 500_000_000,
        total: 1_000_000_000,
        paused: false,
    });
    let p = panel_json(
        &rig,
        &heard,
        panel(VoiceSheet::Models(sheet(vec![paused, running]))),
    );
    let rows = &p["sheet"]["rows"];
    assert_eq!(rows[0]["line"], "Paused · 1 GB of 3.1 GB");
    assert!(marks(&rows[0]).is_empty());
    let fetch = &rows[0]["fetch"];
    assert_eq!(fetch["paused"], true);
    let share = fetch["share"].as_f64().unwrap();
    assert!((share - 1e9 / 3_095_033_483.0).abs() < 1e-9, "{share}");
    let resume = json!({"kind": "fetch", "id": "resume", "arg": 0, "label": "Resume", "on": true, "tone": "plain"});
    assert_eq!(fetch["buttons"][0], resume);
    assert_eq!(rows[1]["line"], "500 MB of 1 GB");
    let pair = buttons(&rows[1]["fetch"]["buttons"]);
    let pause = button("fetch", "pause", "Pause", true, "plain");
    assert_eq!(
        pair,
        [pause, button("fetch", "cancel", "Cancel", true, "plain")]
    );
    assert_eq!(rows[1]["fetch"]["buttons"][1]["arg"], 1, "the row's place");
}

#[test]
fn the_page_names_the_best_model_the_folder_and_its_buttons() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let acts = [PanelButton::Change, PanelButton::Use];
    let p = panel_json(
        &rig,
        &heard,
        acting(VoiceSheet::Models(sheet(vec![])), &acts),
    );
    let best =
        json!({"id": "best", "text": "Recommended for English:", "name": "Cohere Transcribe"});
    assert_eq!(p["top"], best, "the name is drawn apart, bold");
    assert_eq!(
        p["sheet"]["untested"],
        "This order is not tested for English."
    );
    let room = &p["sheet"]["room"];
    assert_eq!(
        room["text"],
        "Models folder: D:\\KeyXtend\\models · 212 GB free"
    );
    let change = button("button", "change", "Change", true, "plain");
    assert_eq!(
        buttons(&room["buttons"]),
        [change, button("button", "open", "Open", false, "plain")]
    );
    let foot = [
        button("button", "delete", "Delete file", false, "danger"),
        button("button", "download", "Download", false, "plain"),
        button("button", "use", "Use for this language", true, "primary"),
    ];
    assert_eq!(buttons(&p["foot"]), foot);
    assert_eq!(p["line"], Value::Null);
    assert_eq!(p["sheet"]["slow"], Value::Null, "the default fits");
}

#[test]
fn a_default_not_for_this_pc_says_local_models_are_slow_here() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut slow = sheet(vec![]);
    slow.slow = true;
    let p = panel_json(&rig, &heard, panel(VoiceSheet::Models(slow)));
    assert_eq!(
        p["sheet"]["slow"],
        "Local models for English will be very slow on this PC."
    );
}

#[test]
fn the_first_download_asks_where_the_models_go() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut asking = sheet(vec![]);
    (asking.offer, asking.folder, asking.best, asking.untested) =
        (Some(folder()), None, None, false);
    let p = panel_json(&rig, &heard, panel(VoiceSheet::Models(asking)));
    assert_eq!(
        p["line"],
        "Keep models in D:\\KeyXtend\\models? 212 GB free."
    );
    let yes = json!({"kind": "confirm", "id": "", "arg": 1, "label": "Yes", "on": true, "tone": "primary"});
    assert_eq!(p["foot"][0], yes);
    assert_eq!(p["foot"][1]["arg"], 0);
    assert_eq!(p["foot"][1]["label"], "No");
    assert_eq!(
        (&p["top"], &p["sheet"]["room"]),
        (&Value::Null, &Value::Null)
    );
    assert_eq!(p["sheet"]["untested"], Value::Null);
}

#[test]
fn the_folder_list_shows_the_drives_then_a_folders_subfolders() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let names = |n: &[&str]| n.iter().map(|s| Redacted::new((*s).to_owned())).collect();
    let drives = FolderList {
        folder: None,
        rows: names(&["C:\\", "D:\\"]),
    };
    let p = panel_json(&rig, &heard, panel(VoiceSheet::Folders(drives)));
    assert_eq!(
        (&p["page"], &p["sheet"]["is"]),
        (&json!("folders"), &json!("folders"))
    );
    assert_eq!(
        (&p["title"], &p["sub"]),
        (&json!("Models folder"), &json!("Drives"))
    );
    assert_eq!(p["sheet"]["rows"], json!(["C:\\", "D:\\"]));
    let here = button("button", "here", "Use this folder", false, "primary");
    assert_eq!(buttons(&p["foot"]), [here]);
    assert_eq!(p["back"]["id"], "back", "up one folder");
    let inside = FolderList {
        folder: Some(Redacted::new("D:\\KeyXtend".to_owned())),
        rows: names(&["models"]),
    };
    let p = panel_json(&rig, &heard, panel(VoiceSheet::Folders(inside)));
    assert_eq!(p["sub"], "D:\\KeyXtend");
}

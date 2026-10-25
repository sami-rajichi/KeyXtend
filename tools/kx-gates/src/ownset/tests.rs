//! Tests of `ownset`.

use kx_test_support::tempdir::TempDir;

use super::*;

/// An owner's file: a module switched off, a log level, a version, a predict choice and another module's section.
const OWNER: &str = "[kernel]\nversion = 1\ndisabled = [\"voice\"]\nlog_level = \"debug\"\n\n[predict]\non = false\n\
                     chips = 4\n\n[keyboard]\nrelabel_ms = 700\n";

fn folder() -> (TempDir, Files) {
    let dir = TempDir::new("gates-ownset").expect("a temp folder");
    let files = Files::new(dir.path());
    (dir, files)
}

fn owner() -> Table {
    OWNER.parse().expect("TOML")
}

fn disabled(t: &Table) -> Vec<String> {
    let list = t["kernel"]["disabled"].as_array().expect("a list");
    list.iter()
        .map(|v| v.as_str().expect("an id").to_string())
        .collect()
}

#[test]
fn both_files_come_back_byte_for_byte_after_the_gate_wrote_them() {
    let (_dir, files) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    fs::write(&files.previous, b"old\r\n").expect("written");
    let kept = Kept::take(&files).expect("kept");
    assert_eq!(kept.table(), owner());
    kept.write(&predict(&kept.table(), false, &[]).expect("edits"))
        .expect("written");
    fs::write(&files.previous, OWNER).expect("the product's save");
    assert_ne!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert_eq!(kept.put_back(), Ok(()));
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert_eq!(fs::read(&files.previous).expect("reads"), b"old\r\n");
}

#[test]
fn a_file_that_was_missing_is_removed_again() {
    let (_dir, files) = folder();
    let kept = Kept::take(&files).expect("kept");
    assert!(kept.table().is_empty(), "no file");
    kept.write(&predict(&Table::new(), true, &[]).expect("edits"))
        .expect("written");
    assert!(files.settings.exists());
    assert_eq!(kept.put_back(), Ok(()));
    assert!(!files.settings.exists() && !files.previous.exists());
}

#[test]
fn a_dropped_keep_puts_the_owners_file_back() {
    let (_dir, files) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    {
        let kept = Kept::take(&files).expect("kept");
        kept.write(&Table::new()).expect("written");
    }
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
}

#[test]
fn a_file_past_the_stores_cap_is_refused_and_left_alone() {
    let (_dir, files) = folder();
    let big = vec![b'#'; usize::try_from(MAX_FILE_BYTES).expect("fits") + 1];
    fs::write(&files.settings, &big).expect("written");
    let Err(e) = Kept::take(&files) else {
        panic!("too big, yet kept")
    };
    assert!(e.contains(&files.settings.display().to_string()), "{e}");
    assert_eq!(fs::read(&files.settings).expect("reads"), big);
}

#[test]
fn an_on_run_takes_predict_off_the_list_shows_chips_and_pauses_learning() {
    let got = predict(&owner(), true, &[]).expect("edits");
    assert_eq!(disabled(&got), ["voice"], "the owner's own choice stays");
    assert_eq!(got["kernel"]["log_level"].as_str(), Some("debug"));
    assert_eq!(got["kernel"]["version"].as_integer(), Some(1));
    let p = &got["predict"];
    assert_eq!(
        (p[ON_KEY].as_bool(), p[LEARN_KEY].as_bool()),
        (Some(true), Some(false))
    );
    assert_eq!(
        p["chips"].as_integer(),
        Some(4),
        "the owner's other predict choices stay"
    );
    assert_eq!(got["keyboard"], owner()["keyboard"]);
}

#[test]
fn an_off_run_lists_predict_once_and_leaves_its_section() {
    let got = predict(&owner(), false, &[]).expect("edits");
    assert_eq!(disabled(&got), ["voice", "predict"]);
    assert_eq!(got["predict"], owner()["predict"]);
    let twice = predict(&got, false, &[]).expect("edits");
    assert_eq!(disabled(&twice), ["voice", "predict"], "never twice");
    assert_eq!(
        disabled(&predict(&twice, true, &[]).expect("edits")),
        ["voice"]
    );
}

#[test]
fn modules_switched_off_for_a_run_join_the_owners_list_once_and_learning_stays_paused() {
    let mouse = kx_mod_mouse::ID.as_str();
    let got = predict(&owner(), true, &[kx_mod_mouse::ID]).expect("edits");
    assert_eq!(disabled(&got), ["voice", mouse]);
    assert_eq!(got["predict"][LEARN_KEY].as_bool(), Some(false));
    let again = predict(&got, true, &[kx_mod_mouse::ID]).expect("edits");
    assert_eq!(disabled(&again), ["voice", mouse], "never twice");
}

#[test]
fn a_file_without_a_kernel_section_gets_the_kernels_defaults() {
    let got = predict(&Table::new(), false, &[]).expect("edits");
    let defaults = KernelSettings::from_defaults().expect("defaults");
    assert_eq!(disabled(&got), ["predict"]);
    assert_eq!(
        got["kernel"]["log_max_mb"].as_integer(),
        Some(i64::from(defaults.log_max_mb))
    );
    assert!(
        got["kernel"].get(VERSION_KEY).is_none(),
        "a missing version reads as current"
    );
}

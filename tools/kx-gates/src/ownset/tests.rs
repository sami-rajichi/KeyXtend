//! Tests of `ownset`.

use kx_test_support::tempdir::TempDir;

use kx_settings::MAX_FILE_BYTES;

use super::*;

/// An owner's file: a module switched off, a log level, a version, a predict choice and another module's section.
const OWNER: &str = "[kernel]\nversion = 1\ndisabled = [\"voice\"]\nlog_level = \"debug\"\n\n[predict]\non = false\n\
                     chips = 4\n\n[keyboard]\nrelabel_ms = 700\n";

/// A temp folder holding the owner's data folder and, beside it, the sidecar's place.
fn folder() -> (TempDir, Files, PathBuf) {
    let dir = TempDir::new("gates-ownset").expect("a temp folder");
    let data = dir.path().join("data");
    fs::create_dir(&data).expect("the data folder");
    let side = dir.path().join("side.json");
    (dir, Files::new(&data), side)
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
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    fs::write(&files.previous, b"old\r\n").expect("written");
    let kept = Kept::take(&files, &side).expect("kept").0;
    assert_eq!(kept.table(), owner());
    kept.write(&predict(&kept.table(), false, &[]).expect("edits"))
        .expect("written");
    fs::write(&files.previous, OWNER).expect("the product's save");
    assert_ne!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert_eq!(kept.put_back(false), Ok(()));
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert_eq!(fs::read(&files.previous).expect("reads"), b"old\r\n");
}

#[test]
fn a_file_that_was_missing_is_removed_again() {
    let (_dir, files, side) = folder();
    let kept = Kept::take(&files, &side).expect("kept").0;
    assert!(kept.table().is_empty(), "no file");
    kept.write(&predict(&Table::new(), true, &[]).expect("edits"))
        .expect("written");
    assert!(files.settings.exists());
    assert_eq!(kept.put_back(false), Ok(()));
    assert!(!files.settings.exists() && !files.previous.exists());
}

#[test]
fn a_dropped_keep_puts_the_owners_file_back_and_keeps_the_sidecar() {
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    {
        let kept = Kept::take(&files, &side).expect("kept").0;
        kept.write(&Table::new()).expect("written");
    }
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert!(side.exists(), "a drop cannot know that every copy ended");
}

#[test]
fn the_sidecar_stays_while_a_copy_may_run() {
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    let (kept, _) = Kept::take(&files, &side).expect("kept");
    kept.write(&Table::new()).expect("written");
    let e = kept
        .put_back(true)
        .expect_err("not final while a copy runs");
    assert!(e.contains(&side.display().to_string()), "{e}");
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert!(side.exists(), "the next open restores again");
}

#[test]
fn close_restores_a_left_sidecar_only_once_no_copy_runs() {
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    killed(&files, &side);
    let e = recover(&files, &side, || true).expect_err("a copy runs");
    assert!(e.contains(&side.display().to_string()), "{e}");
    assert_ne!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert!(side.exists(), "kept while a copy runs");
    assert_eq!(recover(&files, &side, || false), Ok(true));
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert!(!side.exists(), "gone after a verified put-back");
    assert_eq!(recover(&files, &side, || false), Ok(false), "none left");
}

#[test]
fn close_keeps_a_sidecar_it_cannot_put_back_or_that_names_another_folder() {
    let (dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    killed(&files, &side);
    let other = Files::new(&dir.path().join("other"));
    assert!(recover(&other, &side, || false).is_err());
    let data = files.settings.parent().expect("a folder").to_path_buf();
    fs::remove_dir_all(&data).expect("the data folder goes");
    assert!(recover(&files, &side, || false).is_err());
    assert!(side.exists(), "the owner's bytes are still kept");
}

#[test]
fn a_sidecar_holding_a_file_past_the_stores_cap_is_refused_before_any_write() {
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    let big = vec![b'#'; usize::try_from(MAX_FILE_BYTES).expect("fits") + 1];
    let saved = |path: &Path, bytes| saved::Saved {
        path: path.to_path_buf(),
        bytes,
    };
    let pair = saved::Pair {
        settings: saved(&files.settings, Some(big)),
        previous: saved(&files.previous, None),
    };
    fs::write(&side, serde_json::to_vec(&pair).expect("JSON")).expect("written");
    let e = Kept::take(&files, &side).expect_err("over the cap");
    assert!(e.contains(&MAX_FILE_BYTES.to_string()), "{e}");
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert!(side.exists(), "left for the owner to check");
}

#[test]
fn a_file_past_the_stores_cap_is_refused_and_left_alone() {
    let (_dir, files, side) = folder();
    let big = vec![b'#'; usize::try_from(MAX_FILE_BYTES).expect("fits") + 1];
    fs::write(&files.settings, &big).expect("written");
    let Err(e) = Kept::take(&files, &side) else {
        panic!("too big, yet kept")
    };
    assert!(e.contains(&files.settings.display().to_string()), "{e}");
    assert_eq!(fs::read(&files.settings).expect("reads"), big);
    assert!(!side.exists(), "nothing kept, so no sidecar");
}

/// A run that wrote the gate's settings, then was killed before it put the owner's back.
fn killed(files: &Files, side: &Path) {
    let (kept, _) = Kept::take(files, side).expect("kept");
    kept.write(&Table::new()).expect("written");
    std::mem::forget(kept);
}

#[test]
fn a_sidecar_left_by_a_killed_run_is_restored_at_the_next_open() {
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    killed(&files, &side);
    assert!(side.exists(), "written before the gate's first write");
    assert_ne!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    let (kept, recovered) = Kept::take(&files, &side).expect("kept");
    assert!(recovered, "it says so");
    assert_eq!(fs::read_to_string(&files.settings).expect("reads"), OWNER);
    assert!(
        !files.previous.exists(),
        "a file that was missing is missing again"
    );
    assert_eq!(
        kept.table(),
        owner(),
        "never the gate's file as the owner's"
    );
    assert_eq!(kept.put_back(false), Ok(()));
    assert!(!side.exists(), "gone after a verified put-back");
}

#[test]
fn a_second_open_never_overwrites_the_sidecar() {
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    killed(&files, &side);
    let first = fs::read(&side).expect("the sidecar");
    killed(&files, &side);
    assert_eq!(fs::read(&side).expect("the sidecar"), first);
    let (kept, _) = Kept::take(&files, &side).expect("kept");
    assert_eq!(kept.table(), owner());
}

#[test]
fn the_sidecar_stays_when_the_put_back_is_not_verified() {
    let (_dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    let (kept, _) = Kept::take(&files, &side).expect("kept");
    let data = files.settings.parent().expect("a folder").to_path_buf();
    fs::remove_dir_all(&data).expect("the data folder goes");
    assert!(kept.put_back(false).is_err());
    assert!(side.exists(), "the owner's bytes are still kept");
}

#[test]
fn a_sidecar_for_another_data_folder_is_refused_and_left() {
    let (dir, files, side) = folder();
    fs::write(&files.settings, OWNER).expect("written");
    killed(&files, &side);
    let other = dir.path().join("other");
    fs::create_dir(&other).expect("another folder");
    let Err(e) = Kept::take(&Files::new(&other), &side) else {
        panic!("another folder's sidecar, yet kept")
    };
    assert!(e.contains(&side.display().to_string()), "{e}");
    assert!(side.exists() && !Files::new(&other).settings.exists());
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

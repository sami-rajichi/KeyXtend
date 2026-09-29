//! Tests of the whole-file layer, on temp folders only.

use super::*;
use kx_test_support::tempdir::TempDir;

const GOOD: &str = "[keyboard]\nversion = 1\nsize = 3\n";
const OLDER: &str = "[keyboard]\nversion = 1\nsize = 2\n";
const BAD_TOML: &[u8] = b"[keyboard\nsize = = 3\n";
const NOT_UTF8: &[u8] = &[0xff, 0xfe, 0xfd, b'=', 0x80];

fn setup(label: &str) -> (TempDir, Files) {
    let dir = TempDir::new(label).unwrap();
    let files = Files::new(dir.path());
    (dir, files)
}

fn table(text: &str) -> toml::Table {
    text.parse().unwrap()
}

#[test]
fn a_fresh_folder_reads_as_fresh_and_empty() {
    let (_dir, files) = setup("fresh");
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Fresh);
    assert!(got.table.is_empty());
    assert!(got.notices.is_empty());
}

#[test]
fn save_then_load_gives_main_and_the_same_table() {
    let (_dir, files) = setup("roundtrip");
    save(&files, GOOD).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Main);
    assert_eq!(got.table, table(GOOD));
    assert!(got.notices.is_empty());
}

#[test]
fn a_first_save_makes_no_previous_copy() {
    let (_dir, files) = setup("first");
    save(&files, GOOD).unwrap();
    assert!(!files.previous.exists());
    assert!(!files.temp.exists());
}

#[test]
fn a_second_save_moves_the_first_text_to_the_previous_copy() {
    let (_dir, files) = setup("second");
    save(&files, OLDER).unwrap();
    save(&files, GOOD).unwrap();
    assert_eq!(fs::read_to_string(&files.previous).unwrap(), OLDER);
    assert_eq!(fs::read_to_string(&files.settings).unwrap(), GOOD);
    assert!(!files.temp.exists());
}

#[test]
fn a_save_creates_the_data_folder() {
    let (dir, _) = setup("mkdir");
    let files = Files::new(&dir.path().join("a").join("b"));
    save(&files, GOOD).unwrap();
    assert_eq!(fs::read_to_string(&files.settings).unwrap(), GOOD);
}

#[test]
fn a_failed_save_leaves_the_old_file_byte_for_byte() {
    let (_dir, files) = setup("failed");
    save(&files, GOOD).unwrap();
    fs::create_dir(&files.temp).unwrap();
    let err = save(&files, OLDER).unwrap_err();
    assert!(matches!(&err, SaveError::WriteTemp { path, .. } if *path == files.temp));
    assert_eq!(fs::read(&files.settings).unwrap(), GOOD.as_bytes());
    assert!(!files.previous.exists());
}

#[test]
fn a_failed_move_to_previous_removes_the_temp_and_keeps_the_old_file() {
    let (_dir, files) = setup("keepfail");
    save(&files, GOOD).unwrap();
    fs::create_dir(&files.previous).unwrap();
    fs::write(files.previous.join("blocker"), "x").unwrap();
    let err = save(&files, OLDER).unwrap_err();
    assert!(matches!(&err, SaveError::KeepPrevious { path, .. } if *path == files.previous));
    assert_eq!(fs::read(&files.settings).unwrap(), GOOD.as_bytes());
    assert!(!files.temp.exists());
}

#[test]
fn bad_toml_restores_the_previous_copy_and_keeps_the_bad_bytes() {
    let (_dir, files) = setup("badtoml");
    fs::write(&files.previous, GOOD).unwrap();
    fs::write(&files.settings, BAD_TOML).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Restored);
    assert_eq!(got.table, table(GOOD));
    assert_eq!(got.notices, vec![notice(keys::SETTINGS_RESTORED)]);
    assert_eq!(fs::read(&files.broken).unwrap(), BAD_TOML);
    assert!(!files.settings.exists());
}

#[test]
fn a_main_and_a_previous_that_are_both_bad_give_defaults() {
    let (_dir, files) = setup("bothbad");
    fs::write(&files.previous, NOT_UTF8).unwrap();
    fs::write(&files.settings, BAD_TOML).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Defaults);
    assert!(got.table.is_empty());
    assert_eq!(got.notices, vec![notice(keys::SETTINGS_DEFAULTS)]);
    assert_eq!(fs::read(&files.broken).unwrap(), BAD_TOML);
}

#[test]
fn a_damaged_main_with_no_previous_gives_defaults() {
    let (_dir, files) = setup("noprev");
    fs::write(&files.settings, BAD_TOML).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Defaults);
    assert_eq!(got.notices, vec![notice(keys::SETTINGS_DEFAULTS)]);
    assert_eq!(fs::read(&files.broken).unwrap(), BAD_TOML);
}

#[test]
fn an_oversized_file_is_damaged_by_its_size_alone() {
    let (_dir, files) = setup("big");
    fs::write(&files.previous, GOOD).unwrap();
    let mut big = b"# ".to_vec();
    big.resize(usize::try_from(MAX_FILE_BYTES).unwrap() + 1, b'x');
    assert!(table(std::str::from_utf8(&big).unwrap()).is_empty());
    fs::write(&files.settings, &big).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Restored);
    assert_eq!(
        fs::metadata(&files.broken).unwrap().len(),
        MAX_FILE_BYTES + 1
    );
}

#[test]
fn a_file_of_exactly_the_limit_is_read() {
    let (_dir, files) = setup("limit");
    let mut text = b"# ".to_vec();
    text.resize(usize::try_from(MAX_FILE_BYTES).unwrap(), b'x');
    fs::write(&files.settings, &text).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Main);
    assert!(!files.broken.exists());
}

#[test]
fn a_missing_main_with_a_good_previous_is_restored() {
    let (_dir, files) = setup("lost");
    fs::write(&files.previous, GOOD).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Restored);
    assert_eq!(got.table, table(GOOD));
    assert_eq!(got.notices, vec![notice(keys::SETTINGS_RESTORED)]);
    assert!(!files.broken.exists());
}

#[test]
fn a_missing_main_with_a_bad_previous_starts_fresh() {
    let (_dir, files) = setup("lostbad");
    fs::write(&files.previous, BAD_TOML).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Fresh);
    assert!(got.notices.is_empty());
}

#[test]
fn bytes_that_are_not_utf8_count_as_damaged() {
    let (_dir, files) = setup("utf8");
    fs::write(&files.previous, GOOD).unwrap();
    fs::write(&files.settings, NOT_UTF8).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Restored);
    assert_eq!(fs::read(&files.broken).unwrap(), NOT_UTF8);
}

#[test]
fn a_new_broken_copy_replaces_an_older_one() {
    let (_dir, files) = setup("replace");
    fs::write(&files.broken, "old damage").unwrap();
    fs::write(&files.settings, BAD_TOML).unwrap();
    let _ = load(&files);
    assert_eq!(fs::read(&files.broken).unwrap(), BAD_TOML);
}

#[test]
fn a_save_after_a_repair_keeps_the_good_previous_copy() {
    let (_dir, files) = setup("after");
    fs::write(&files.previous, OLDER).unwrap();
    fs::write(&files.settings, BAD_TOML).unwrap();
    let _ = load(&files);
    save(&files, GOOD).unwrap();
    assert_eq!(fs::read_to_string(&files.previous).unwrap(), OLDER);
    assert_eq!(fs::read_to_string(&files.settings).unwrap(), GOOD);
}

#[test]
fn a_settings_folder_in_place_of_the_file_is_damaged_not_a_panic() {
    let (_dir, files) = setup("isdir");
    fs::create_dir(&files.settings).unwrap();
    let got = load(&files);
    assert_eq!(got.outcome, Outcome::Defaults);
}

#[test]
fn a_damaged_main_never_replaces_the_previous_copy() {
    let (_dir, files) = setup("damagedmain");
    fs::write(&files.previous, GOOD).unwrap();
    fs::write(&files.settings, BAD_TOML).unwrap();
    save(&files, OLDER).unwrap();
    assert_eq!(fs::read_to_string(&files.previous).unwrap(), GOOD);
    assert_eq!(fs::read_to_string(&files.settings).unwrap(), OLDER);
}

#[test]
fn a_main_that_turned_damaged_after_the_load_is_kept_as_the_broken_copy() {
    let (_dir, files) = setup("turned");
    fs::write(&files.previous, GOOD).unwrap();
    fs::write(&files.settings, BAD_TOML).unwrap();
    save(&files, OLDER).unwrap();
    assert_eq!(fs::read(&files.broken).unwrap(), BAD_TOML);
    assert_eq!(fs::read_to_string(&files.settings).unwrap(), OLDER);
}

#[test]
fn a_failed_replace_after_the_move_puts_the_old_file_back() {
    let (_dir, mut files) = setup("roll");
    // With one path for both, the last rename has no source and must fail.
    files.temp = files.settings.clone();
    let err = save(&files, GOOD).unwrap_err();
    assert!(matches!(err, SaveError::Replace { .. }));
    assert!(!files.previous.exists());
}

#[test]
fn a_failed_replace_that_moved_nothing_leaves_the_previous_copy() {
    let (_dir, files) = setup("noroll");
    fs::write(&files.previous, GOOD).unwrap();
    for blocked in [&files.settings, &files.broken] {
        fs::create_dir(blocked).unwrap();
        fs::write(blocked.join("blocker"), "x").unwrap();
    }
    let err = save(&files, OLDER).unwrap_err();
    assert!(matches!(&err, SaveError::Replace { path, .. } if *path == files.settings));
    assert_eq!(fs::read_to_string(&files.previous).unwrap(), GOOD);
    assert!(files.settings.join("blocker").exists());
    assert!(!files.temp.exists());
}

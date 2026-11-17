//! The kernel's repair of `[voice]`: a bad pick stays for the module to drop alone (plan edge case 19), while a bad
//! `place`, `device` (edge case 35) or `history_join` goes back to its default with the settings notice; the other values
//! stay.

use std::fs;

use kx_mod_voice::models::{Pick, table};
use kx_mod_voice::{ID, SPEC, VoiceSettings};
use kx_module_api::ports::{BarPlace, DevicePick};
use kx_module_api::{keys, parse_as};
use kx_settings::{Files, Store};
use kx_test_support::tempdir::TempDir;

/// The `[voice]` section read from a file holding `lines`, and the keys of the notices it raised.
fn load(lines: &str) -> (VoiceSettings, Vec<&'static str>) {
    let dir = TempDir::new("voice-repair").unwrap();
    let files = Files::new(dir.path());
    fs::write(&files.settings, format!("[voice]\nversion = 1\n{lines}")).unwrap();
    let (store, report) = Store::load(files, &[(ID, &SPEC)]);
    let notices = report.notices.iter().map(|n| n.key).collect();
    (parse_as(store.get(ID).unwrap()).unwrap(), notices)
}

/// The id of the row `s` uses for `lang`.
fn used(s: &VoiceSettings, lang: &str) -> String {
    match table().unwrap().used(lang, &s.picks) {
        Some(Pick::Row(row)) => row.id.clone(),
        other => panic!("{lang}: {other:?}"),
    }
}

#[test]
fn a_pick_of_a_row_no_longer_in_the_table_falls_back_alone() {
    let (mut s, notices) =
        load("picks = { fr = \"whisper-medium-q5\", ar = \"cohere-q8\" }\nthreads = 4\n");
    assert!(
        notices.is_empty(),
        "the kernel keeps the picks whole: {notices:?}"
    );
    assert_eq!((s.picks.len(), s.threads), (2, 4));
    assert_eq!(s.drop_bad(), ["picks.fr"]);
    assert_eq!(used(&s, "fr"), "parakeet-v3-q8", "the default");
    assert_eq!(
        used(&s, "ar"),
        "cohere-q8",
        "the other language's pick stays"
    );
}

#[test]
fn a_pick_of_a_row_that_does_not_know_the_language_falls_back_alone() {
    let (mut s, _) = load("picks = { ar = \"canary-v2-q8\", fr = \"canary-v2-q8\" }\n");
    assert_eq!(s.drop_bad(), ["picks.ar"]);
    assert_eq!(used(&s, "ar"), "cohere-arabic-q8");
    assert_eq!(used(&s, "fr"), "canary-v2-q8");
}

#[test]
fn a_pick_that_is_not_text_resets_the_whole_table_with_the_notice() {
    let (s, notices) = load("picks = { fr = 3, ar = \"cohere-q8\" }\nsounds = false\n");
    assert_eq!(notices, [keys::SETTINGS_RESET]);
    assert!(s.picks.is_empty() && !s.sounds, "the other values stay");
}

#[test]
fn a_bad_place_goes_back_to_above_with_the_notice() {
    for bad in ["\"left\"", "\"\"", "3", "\"Top\""] {
        let (s, notices) = load(&format!("place = {bad}\nsounds = false\n"));
        assert_eq!(notices, [keys::SETTINGS_RESET], "{bad}");
        assert_eq!(s.bar_place(), BarPlace::Above, "{bad}");
        assert!(!s.sounds, "the other values stay");
    }
}

#[test]
fn a_bad_device_goes_back_to_automatic_with_the_notice() {
    for bad in ["\"\"", "\"Vulkan 0\"", "\"المعالج\"", "false"] {
        let (s, notices) = load(&format!("device = {bad}\nhistory_max = 80\n"));
        assert_eq!(notices, [keys::SETTINGS_RESET], "{bad}");
        assert_eq!((s.device(), s.history_max), (DevicePick::Auto, 80), "{bad}");
    }
}

// History's join (P11 task 25): too long, or a mark that does not show, goes back to a space with the notice.
#[test]
fn a_bad_history_join_goes_back_to_a_space_with_the_notice() {
    for bad in ["\"123456789\"", "\"\\n\"", "\"\\u200B\"", "3"] {
        let (s, notices) = load(&format!("history_join = {bad}\nhistory_max = 80\n"));
        assert_eq!(notices, [keys::SETTINGS_RESET], "{bad}");
        assert_eq!((s.history_join.as_str(), s.history_max), (" ", 80), "{bad}");
    }
    let (s, notices) = load("history_join = \" · \"\n");
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!(s.history_join, " · ");
}

#[test]
fn a_good_section_raises_no_notice() {
    let (mut s, notices) = load(
        "place = \"top\"\ndevice = \"Vulkan1\"\npicks = { ar = \"cohere-q8\", de = \"my.gguf\" }\n",
    );
    assert!(notices.is_empty(), "{notices:?}");
    assert_eq!(s.bar_place(), BarPlace::Top);
    assert_eq!(s.device().name(), "Vulkan1");
    assert!(s.drop_bad().is_empty());
    assert_eq!(s.picks.len(), 2);
}

//! The cloud line's pure parts: a borrowed key copied byte for byte and nothing else, the plan of services, the copy's
//! settings with one service on as the product's own rules read them, the requests it sends, and a hidden copy.

use std::fs;
use std::path::Path;

use kx_crypto::{KEY_FILE, SEALED_EXT};
use kx_mod_voice::cloud::{Cloud, pick_text};
use kx_mod_voice::keys::KEYS;
use kx_mod_voice::services::table;
use kx_mod_voice::{ID, SPEC, VoiceSettings};
use kx_module_api::ports::{CloudLimits, Date, SAMPLE_BYTES, SOUND_HZ, SpeechAsk};
use kx_module_api::{MS_PER_S, Redacted};
use kx_platform::VOICE_DIR;
use kx_platform::ui::{HiddenError, Probe};
use kx_settings::{Files, peek_section};
use kx_test_support::tempdir::TempDir;

use super::super::cloud::{hidden_why, typed as daily_typed};
use super::super::cloudset::{self, Plan, Planned, Typed};
use super::super::{borrow, table_of};
use crate::config::load;

/// Bytes standing in for a sealed blob and a wrapped data key; never a real key.
const BLOB: &[u8] = b"KXSL-a-stand-in-blob";
const WRAPPED: &[u8] = b"a-stand-in-wrapped-key";
/// The rows the tests plan, and the clip languages.
const GROQ: &str = "groq";
const CUSTOM: &str = "custom";
const OPENROUTER: &str = "openrouter";
const LANGS: [&str; 3] = ["en", "fr", "ar"];
/// A typed Custom address and model on a test host.
const TYPED: &str = "https://stt.example.test/v1/audio/transcriptions";
const TYPED_MODEL: &str = "whisper-test";

/// A daily folder holding the stand-in data key, a stand-in sealed key for each of `ids`, and files that must stay.
fn daily(ids: &[&str]) -> TempDir {
    let dir = TempDir::new("g22-daily").expect("a folder");
    let voice = dir.path().join(VOICE_DIR);
    fs::create_dir_all(voice.join(KEYS)).expect("the keys folder");
    fs::write(voice.join(KEY_FILE), WRAPPED).expect("the data key");
    for id in ids {
        fs::write(voice.join(KEYS).join(format!("{id}.{SEALED_EXT}")), BLOB).expect("a blob");
    }
    fs::create_dir_all(voice.join("texts")).expect("history");
    fs::write(voice.join("texts").join("t.sealed"), b"history").expect("a text");
    fs::write(dir.path().join("settings.toml"), b"[voice]\n").expect("settings");
    dir
}

/// How many files lie under `dir`.
fn files(dir: &Path) -> usize {
    let entries = fs::read_dir(dir)
        .expect("a folder")
        .map(|e| e.expect("an entry").path());
    entries
        .map(|p| if p.is_dir() { files(&p) } else { 1 })
        .sum()
}

#[test]
fn a_key_is_there_only_with_its_blob_and_the_data_key_by_their_names() {
    let d = daily(&[GROQ]);
    assert!(borrow::has(d.path(), GROQ));
    assert!(!borrow::has(d.path(), "lemonfox"), "no blob");
    assert!(!borrow::has(d.path(), "a/b"), "no blob name");
    fs::remove_file(d.path().join(VOICE_DIR).join(KEY_FILE)).expect("removed");
    assert!(!borrow::has(d.path(), GROQ), "no data key");
}

#[test]
fn a_borrow_copies_that_blob_and_the_data_key_byte_for_byte_and_nothing_else() {
    let d = daily(&[GROQ, "lemonfox"]);
    let copy = TempDir::new("g22-copy").expect("a folder");
    assert_eq!(borrow::copy(d.path(), copy.path(), GROQ), Ok(()));
    let voice = copy.path().join(VOICE_DIR);
    assert_eq!(fs::read(voice.join(KEY_FILE)).expect("copied"), WRAPPED);
    let blob = voice.join(KEYS).join(format!("{GROQ}.{SEALED_EXT}"));
    assert_eq!(fs::read(blob).expect("copied"), BLOB);
    assert_eq!(files(copy.path()), 2, "only those two");
}

#[test]
fn a_failed_borrow_names_no_path_and_no_folder_is_copied_onto_itself() {
    let d = daily(&[GROQ]);
    let copy = TempDir::new("g22-copy").expect("a folder");
    let e = borrow::copy(d.path(), copy.path(), "lemonfox").expect_err("no blob");
    for path in [d.path(), copy.path()] {
        assert!(!e.contains(&path.display().to_string()), "{e}");
    }
    assert!(
        borrow::copy(d.path(), d.path(), GROQ).is_err(),
        "its own folder"
    );
    assert_eq!(files(d.path()), 4, "the daily folder is as it was");
}

/// The plans of every offered row with `has` keys, for the clip languages, with `typed` lines.
fn plans(has: fn(&str) -> bool, typed: fn(&str) -> Option<Typed>) -> Vec<Plan<'static>> {
    cloudset::plan(table().expect("the table"), has, &LANGS, typed)
}

/// Custom's typed lines at `address`.
fn typed(address: &str) -> Typed {
    Typed {
        address: Redacted::new(address.to_owned()),
        model: Redacted::new(TYPED_MODEL.to_owned()),
    }
}

#[test]
fn every_offered_row_is_planned_in_order_and_only_one_with_a_key_runs() {
    let got = plans(|id| id == GROQ, |_| None);
    let offered: Vec<&str> = table().unwrap().offered().map(|r| r.id.as_str()).collect();
    let planned: Vec<&str> = got.iter().map(Plan::id).collect();
    assert_eq!(planned, offered, "a hidden row never runs");
    let runs: Vec<&str> = got
        .iter()
        .filter_map(Plan::run)
        .map(|p| p.row.id.as_str())
        .collect();
    assert_eq!(runs, [GROQ]);
}

#[test]
fn a_listed_row_runs_its_first_model_on_the_clip_languages_it_knows() {
    let got = plans(|_| true, |_| None);
    for p in got.iter().filter_map(Plan::run) {
        let first = p.row.models.first().expect("a listed row");
        assert_eq!(p.model.expose(), &first.id);
        assert!(!p.langs.is_empty() && p.langs.iter().all(|l| first.code(l).is_some()));
    }
}

#[test]
fn custom_runs_only_with_the_daily_address_and_model() {
    let custom = |got: &[Plan]| {
        got.iter()
            .find(|p| p.id() == CUSTOM)
            .map(|p| p.run().is_some())
    };
    assert_eq!(custom(&plans(|id| id == CUSTOM, |_| None)), Some(false));
    let plain = plans(
        |id| id == CUSTOM,
        |_| Some(typed("http://stt.example.test/v1")),
    );
    assert_eq!(custom(&plain), Some(false), "not https");
    let good = plans(|id| id == CUSTOM, |_| Some(typed(TYPED)));
    let p = good.iter().find_map(Plan::run).expect("custom runs");
    assert_eq!(
        (p.row.id.as_str(), p.model.expose().as_str()),
        (CUSTOM, TYPED_MODEL)
    );
    assert_eq!(p.langs, LANGS, "a typed id takes each code as it is");
}

/// The copy's settings for `p`, read back from its settings file as a later copy reads them.
fn written(p: &Planned) -> (VoiceSettings, VoiceSettings) {
    let s = cloudset::voice(p, Date::new(2026, 11, 20).expect("a day")).expect("settings");
    let file = table_of(&s, &load().expect("the settings load").g22_product).expect("a table");
    let text: toml::Table = file.to_string().parse().expect("TOML");
    (s, peek_section(Some(&text), ID, &SPEC).expect("read back"))
}

#[test]
fn the_copys_settings_hold_one_service_confirmed_with_its_reminder_seen_as_the_product_reads_them()
{
    let got = plans(|_| true, |_| Some(typed(TYPED)));
    let services = table().expect("the table");
    for p in got.iter().filter_map(Plan::run) {
        let (s, back) = written(p);
        assert_eq!(back, s, "the product keeps every value");
        assert!(s.cloud);
        assert_eq!(s.services.keys().collect::<Vec<_>>(), [&p.row.id]);
        let on = &s.services[&p.row.id];
        assert_eq!((on.terms, on.warned), (p.row.terms, p.row.terms));
        for lang in &p.langs {
            assert_eq!(s.picks[lang], pick_text(&p.row.id, p.model.expose()));
            let c = Cloud::new(services, false)
                .choice(&s, lang)
                .expect("a cloud choice");
            assert_eq!(c.service.id, p.row.id);
            assert!(
                !c.ask && c.warn.is_none() && c.unavailable.is_none(),
                "{lang}"
            );
        }
        assert!(
            Cloud::new(services, true).choice(&s, &p.langs[0]).is_none(),
            "hidden"
        );
    }
}

#[test]
fn only_a_typed_rows_two_lines_are_read_from_the_daily_settings() {
    let d = daily(&[CUSTOM]);
    assert!(daily_typed(d.path())(CUSTOM).is_none(), "no service entry");
    let got = plans(|id| id == CUSTOM, |_| Some(typed(TYPED)));
    let (s, _) = written(got.iter().find_map(Plan::run).expect("custom runs"));
    let file = table_of(&s, &load().expect("the settings load").g22_product).expect("a table");
    fs::write(Files::new(d.path()).settings, file.to_string()).expect("the daily settings");
    let read = daily_typed(d.path());
    let t = read(CUSTOM).expect("both lines");
    let lines = (t.address.expose().as_str(), t.model.expose().as_str());
    assert_eq!(lines, (TYPED, TYPED_MODEL));
    assert!(read(GROQ).is_none(), "a row not on");
}

/// The whole seconds of sound one piece within `limits` holds, before the run's own piece range.
fn cap_s(limits: &CloudLimits) -> u32 {
    u32::try_from(limits.piece_bytes() / (SOUND_HZ * SAMPLE_BYTES)).unwrap()
}

#[test]
fn a_clip_within_a_piece_is_one_request_and_a_longer_one_more_as_the_product_cuts_it() {
    let services = table().unwrap();
    let groq = services.get(GROQ).unwrap().limits;
    let top_s = u32::from(*SpeechAsk::PIECE_S.end());
    assert!(
        cap_s(&groq) > top_s,
        "groq's cap is above the run's longest piece"
    );
    assert_eq!(cloudset::pieces(6000, &groq), 1);
    assert_eq!(cloudset::pieces(0, &groq), 1, "at least one");
    assert_eq!(cloudset::pieces((top_s + 1) * MS_PER_S, &groq), 2);
    let short = services.get(OPENROUTER).unwrap().limits;
    assert!(cap_s(&short) < top_s, "openrouter's cap is below it");
    assert_eq!(cloudset::pieces((cap_s(&short) + 1) * MS_PER_S, &short), 2);
}

#[test]
fn the_heads_up_counts_each_runs_requests_for_the_clips_it_dictates() {
    let got = plans(|id| id == GROQ || id == CUSTOM, |_| Some(typed(TYPED)));
    let long = (u32::from(*SpeechAsk::PIECE_S.end()) + 1) * MS_PER_S;
    let clips = [("en", 6000), ("fr", 6000), ("ar", long), ("de", 6000)];
    assert_eq!(
        cloudset::requests(&got, &clips),
        2 * 4,
        "no clip in German is planned"
    );
}

#[test]
fn a_copy_that_would_be_a_hidden_run_fails_without_naming_the_app() {
    assert_eq!(hidden_why(&Ok(Probe::Normal)), Ok(()));
    let hidden = Probe::Hidden {
        package: "Pkg_abc".into(),
        app: "Shell".into(),
    };
    let e = hidden_why(&Ok(hidden)).expect_err("hidden");
    assert!(
        e.contains("hidden run") && !e.contains("Shell") && !e.contains("Pkg"),
        "{e}"
    );
    assert!(hidden_why(&Err(HiddenError::Unreadable)).is_err());
}

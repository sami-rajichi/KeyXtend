//! G22's product and staged settings: each bad key is refused, the shipped files pass, each section sits in one file,
//! and the names they share with the product, the stage and kx-ui stay the same.

use std::path::Path;

use kx_module_api::ports::DevicePick;

use super::*;
use crate::cfgfile::OWN;
use crate::config::{GatesConfig, load};

/// kx-ui's bar names, whose group G22 clicks Stop by.
const VOICE_NAMES: &str = include_str!("../../../../crates/kx-ui/qml/VoiceNames.qml");
/// kx-ui's bar button ids.
const BAR_IDS: &str = include_str!("../../../../crates/kx-ui/src/i18n/keys/voice/ids.rs");
/// The app's run, which logs the voice gate build's warning.
const APP_RUN: &str = include_str!("../../../../apps/keyxtend/src/run.rs");
/// The voice gate build's stage lines.
const APP_STAGE: &str = include_str!("../../../../apps/keyxtend/src/played/stage.rs");
/// The voice module's typing, which logs each text sent to the keyboard.
const MOD_TYPING: &str = include_str!("../../../../crates/kx-mod-voice/src/actor/typing.rs");
/// The root manifest, whose stage settings G22's staged edges read.
const ROOT_MANIFEST: &str = include_str!("../../../../Cargo.toml");

mod cloud;

fn shipped() -> GatesConfig {
    load().expect("the settings load")
}

/// The root manifest's `[workspace.metadata.stage]`.
fn stage() -> toml::Value {
    let root: toml::Table = ROOT_MANIFEST.parse().expect("the root manifest is TOML");
    root["workspace"]["metadata"]["stage"].clone()
}

#[test]
fn a_plain_name_is_one_folder_name_and_a_clip_must_be_one() {
    assert!(plain_name("check-en.wav"));
    for bad in ["", ".", "..", "a/b", r"a\b", "C:x", r"C:\x"] {
        assert!(!plain_name(bad), "{bad:?}");
    }
    let voice = Path::new("voice");
    assert_eq!(
        clip_in(voice, "check-en.wav"),
        Ok(voice.join("check-en.wav"))
    );
    for outside in [r"..\secret.wav", "sub/clip.wav", r"D:\clip.wav", ""] {
        assert!(clip_in(voice, outside).is_err(), "{outside:?}");
    }
}

#[test]
fn the_shipped_product_run_passes_its_check_and_each_bad_key_is_refused() {
    let c = shipped();
    assert_eq!(c.g22_product.check(&c.g22_bench), Ok(()));
    let bad: [fn(&mut G22Product); 21] = [
        |g| g.card_wait_ms = 0,
        |g| g.program = "../../target/p11-gates/release/other.exe".into(),
        |g| g.program = "../../target/gate-input/release/keyxtend.exe".into(),
        |g| drop(g.layouts.pop_first()),
        |g| drop(g.layouts.insert("de".into(), "0407".into())),
        |g| drop(g.layouts.insert("en".into(), "409".into())),
        |g| g.device = String::new(),
        |g| g.ready_ms = 0,
        |g| g.bar_wait_ms = 0,
        |g| g.warn_line = " ".into(),
        |g| g.group.clear(),
        |g| g.group = "voice.bar".into(),
        |g| g.stop.clear(),
        |g| g.device = "card with space".into(),
        |g| g.open_ms = 0,
        |g| g.settle_ms = 0,
        |g| g.layout_wait_ms = 0,
        |g| g.log_level = "loud".into(),
        |g| g.why_marks.clear(),
        |g| g.why_marks.push(" ".into()),
        |g| g.log_level = "info".into(),
    ];
    for (i, spoil) in bad.iter().enumerate() {
        let mut g = c.g22_product.clone();
        spoil(&mut g);
        assert!(g.check(&c.g22_bench).is_err(), "case {i} passed");
    }
}

#[test]
fn the_product_run_takes_each_clips_layout_and_stops_by_the_bars_gate_id() {
    let g = shipped().g22_product;
    assert_eq!(g.layout("fr"), Some(0x040C));
    assert_eq!(g.layout("de"), None);
    assert_eq!(g.stop_id(), format!("{}.{}", g.group, g.stop));
}

#[test]
fn the_product_runs_names_are_kx_uis_and_the_apps() {
    let g = shipped().g22_product;
    let group = format!("readonly property string group: \"{}\"", g.group);
    assert!(VOICE_NAMES.contains(&group), "{group}");
    let stop = format!("BarButton::Stop => \"{}\"", g.stop);
    assert!(BAR_IDS.contains(&stop), "{stop}");
    assert!(
        APP_RUN.contains(&format!("\"{}\"", g.warn_line)),
        "the app's warning"
    );
}

#[test]
fn the_timing_lines_are_the_apps_and_the_voice_modules_at_the_level_the_run_keeps() {
    let g = shipped().g22_product;
    let level = G22Product::LINES_LEVEL;
    assert!(APP_STAGE.contains(&format!("tracing::{level}!({{ STAGE }}")));
    let typed = format!("tracing::{level}!(\"{{TYPED_LINE}}\")");
    assert!(MOD_TYPING.contains(&typed), "{typed}");
    let mut trace = g;
    trace.log_level = "trace".into();
    let c = shipped();
    assert_eq!(trace.check(&c.g22_bench), Ok(()), "trace keeps debug lines");
}

#[test]
fn only_a_card_in_device_waits_after_the_layout_before_dictation() {
    let mut g = shipped().g22_product;
    assert_eq!(g.pick(), Some(DevicePick::Cpu));
    assert_eq!(g.dictate_wait_ms(), 0, "the processor");
    g.device = DevicePick::AUTO.into();
    assert_eq!(
        g.dictate_wait_ms(),
        0,
        "Automatic checks no card on a fresh folder"
    );
    g.device = "Vulkan1".into();
    assert_eq!(g.dictate_wait_ms(), g.card_wait_ms, "a card");
    g.device = "card with space".into();
    assert_eq!((g.pick(), g.dictate_wait_ms()), (None, 0), "no pick");
}

#[test]
fn a_device_flags_pick_replaces_the_files_for_the_run_and_none_keeps_it() {
    let mut g = shipped().g22_product;
    let kept = g.clone();
    g.set_pick(None);
    assert_eq!(
        (g.device.as_str(), g.pick()),
        (kept.device.as_str(), kept.pick())
    );
    let card = DevicePick::parse("Vulkan1");
    g.set_pick(card.as_ref());
    assert_eq!(g.pick(), card, "the flag wins");
    assert_eq!(
        g.dictate_wait_ms(),
        g.card_wait_ms,
        "a card's wait follows it"
    );
    g.set_pick(Some(&DevicePick::Cpu));
    assert_eq!((g.pick(), g.dictate_wait_ms()), (Some(DevicePick::Cpu), 0));
}

#[test]
fn the_shipped_staged_edges_pass_their_check_and_each_bad_key_is_refused() {
    let c = shipped();
    assert_eq!(c.g22_staged.check(&c.g22_bench), Ok(()));
    let bad: [fn(&mut G22Staged); 11] = [
        |g| g.stage.clear(),
        |g| g.stage = r"D:\stage".into(),
        |g| g.copy = "..".into(),
        |g| g.card_part = r"..\ggml-vulkan.dll".into(),
        |g| g.card_part = "ggml-cpu-x64.dll".into(),
        |g| g.floor_part = "ggml-vulkan.dll".into(),
        |g| g.plant_as = "transcribe.dll".into(),
        |g| g.plant = "a/b".into(),
        |g| g.lang = "de".into(),
        |g| g.floor_part = "ggml-cpu-".into(),
        |g| g.poll_ms = 0,
    ];
    for (i, spoil) in bad.iter().enumerate() {
        let mut g = c.g22_staged.clone();
        spoil(&mut g);
        assert!(g.check(&c.g22_bench).is_err(), "case {i} passed");
    }
}

#[test]
fn the_staged_edges_read_the_stage_the_root_manifest_names() {
    let (g, s) = (shipped().g22_staged, stage());
    let want = format!(
        "../../{}/{}",
        s["out_dir"].as_str().unwrap(),
        s["app"].as_str().unwrap()
    );
    assert_eq!(g.stage, want);
    let engine = &s["engine"];
    assert_eq!(g.floor_part, engine["floor"].as_str().unwrap());
    let by = engine["driver"]["by"].as_array().unwrap();
    assert!(
        by.iter().any(|v| v.as_str() == Some(&g.card_part)),
        "the card part"
    );
    let loaded = engine["loaded"].as_array().unwrap();
    assert!(
        loaded.iter().any(|v| v.as_str() == Some(&g.plant_as)),
        "a part's name"
    );
}

#[test]
fn each_g22_section_sits_in_one_file_and_the_spike_run_is_gone() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tables: Vec<toml::Table> = OWN
        .iter()
        .map(|f| {
            std::fs::read_to_string(dir.join(f))
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    for section in [
        "g22_bench",
        "g22_product",
        "g22_cloud",
        "g22_staged",
        "boxed",
    ] {
        let files = tables.iter().filter(|t| t.contains_key(section)).count();
        assert_eq!(files, 1, "[{section}]");
    }
    assert!(
        tables.iter().all(|t| !t.contains_key("g22")),
        "the spike's [g22]"
    );
}

/// Section `name` of the shipped settings file that holds it, with `stray` set inside the table at `at` in it.
fn with_stray(name: &str, at: &[&str]) -> (toml::Value, toml::Value) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |f: &&str| std::fs::read_to_string(dir.join(f)).expect("a settings file");
    let mut tables = OWN
        .iter()
        .map(read)
        .map(|t| t.parse::<toml::Table>().expect("TOML"));
    let good = tables.find_map(|mut t| t.remove(name)).expect(name);
    let mut bad = good.clone();
    let inner = at.iter().fold(&mut bad, |v, k| &mut v[*k]);
    inner
        .as_table_mut()
        .expect("a table")
        .insert("stray".into(), 1.into());
    (good, bad)
}

#[test]
fn each_g22_and_box_section_refuses_an_unknown_key() {
    fn refused<T: serde::de::DeserializeOwned>((good, bad): (toml::Value, toml::Value)) -> bool {
        good.try_into::<T>().is_ok() && bad.try_into::<T>().is_err()
    }
    assert!(
        refused::<G22Bench>(with_stray("g22_bench", &[])),
        "[g22_bench]"
    );
    let clip = with_stray("g22_bench", &["clips", "en"]);
    assert!(refused::<G22Bench>(clip), "[g22_bench.clips.en]");
    assert!(
        refused::<G22Product>(with_stray("g22_product", &[])),
        "[g22_product]"
    );
    assert!(
        refused::<G22Staged>(with_stray("g22_staged", &[])),
        "[g22_staged]"
    );
    let cloud = with_stray("g22_cloud", &[]);
    assert!(refused::<G22Cloud>(cloud), "[g22_cloud]");
    let boxed = with_stray("boxed", &[]);
    assert!(refused::<crate::boxed::BoxProofs>(boxed), "[boxed]");
}

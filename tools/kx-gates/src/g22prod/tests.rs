//! G22 on the product's pure parts: the copy's settings, the bar's look, the foreground's count, the typed text's
//! settling, the device that wrote each text and each language's verdict, which prints numbers only.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use kx_mod_voice::score::Wrong;
use kx_mod_voice::{SPEC, VoiceSettings};
use kx_module_api::ports::{CardId, DevicePick};
use kx_settings::VERSION_KEY;
use serde_json::{Value, json};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{WS_EX_NOACTIVATE, WS_EX_TOPMOST};

use super::bar::BarLook;
use super::front::{FrontCount, Fronts};
use super::settle::Settle;
use super::wrote::{self, DEVICE};
use super::{Said, table};
use crate::budget::stamp_us;
use crate::config::load;

/// The voice gate build's speech, which logs each written dictation's device.
const APP_TOLD: &str = include_str!("../../../../apps/keyxtend/src/played/told.rs");

#[test]
fn the_copy_reads_the_bench_models_on_the_runs_device_and_keeps_every_other_default() {
    let mut c = load().expect("the settings load");
    c.g22_bench.models_dir = r"E:\models".into();
    let t = table(&c.g22_product, &c.g22_bench).expect("a table");
    let mut section = t[kx_mod_voice::ID.as_str()]
        .as_table()
        .expect("[voice]")
        .clone();
    assert_eq!(
        section.remove(VERSION_KEY),
        Some(i64::from(SPEC.version).into())
    );
    let got: VoiceSettings = toml::Value::Table(section)
        .try_into()
        .expect("the section reads");
    let mut want = VoiceSettings::from_defaults().expect("the defaults");
    (want.models_dir, want.device) = (c.g22_bench.models_dir.clone(), c.g22_product.device.clone());
    assert_eq!(got, want);
    assert_eq!(t.len(), 1, "only [voice]");
    c.g22_bench.models_dir.clear();
    assert!(
        table(&c.g22_product, &c.g22_bench).is_err(),
        "no models folder"
    );
}

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

#[test]
fn the_bar_must_be_no_activate_topmost_uncovered_above_the_keyboard_and_beside_it() {
    let ex = WS_EX_NOACTIVATE.0 | WS_EX_TOPMOST.0;
    let (bar, kb) = (rect(0, 0, 100, 40), rect(0, 50, 100, 300));
    let order = [7, 3, 9];
    let good = BarLook::of(ex, &order, (7, bar), (3, kb), Some(7));
    assert!(good.ok(), "{good:?}");
    assert!(
        !BarLook::of(WS_EX_TOPMOST.0, &order, (7, bar), (3, kb), Some(7)).ok(),
        "it can take focus"
    );
    assert!(
        !BarLook::of(WS_EX_NOACTIVATE.0, &order, (7, bar), (3, kb), Some(7)).ok(),
        "not on top"
    );
    assert!(
        !BarLook::of(ex, &[3, 7], (7, bar), (3, kb), Some(7)).ok(),
        "under the keyboard"
    );
    assert!(
        !BarLook::of(ex, &[9, 3], (7, bar), (3, kb), Some(7)).ok(),
        "not listed"
    );
    assert!(
        !BarLook::of(ex, &order, (7, rect(0, 0, 100, 60)), (3, kb), Some(7)).ok(),
        "over the keys"
    );
    assert!(
        !BarLook::of(ex, &order, (7, bar), (3, kb), Some(9)).ok(),
        "another window over it"
    );
}

#[test]
fn the_target_keeps_the_foreground_only_when_no_other_window_ever_took_it() {
    let (target, bar) = (5, 8);
    let kept = Fronts {
        samples: 4,
        seen: BTreeSet::new(),
    };
    assert_eq!(
        kept.count(target, bar),
        FrontCount {
            samples: 4,
            away: 0,
            bar: false
        }
    );
    assert!(kept.count(target, bar).kept());
    let lost = Fronts {
        samples: 4,
        seen: [bar, 2].into(),
    };
    let n = lost.count(target, bar);
    assert_eq!((n.away, n.bar, n.kept()), (2, true, false));
    let own = Fronts {
        samples: 4,
        seen: [target].into(),
    };
    assert!(own.count(target, bar).kept(), "the target itself");
    assert!(
        !Fronts::default().count(target, bar).kept(),
        "never sampled"
    );
}

#[test]
fn the_text_settles_once_it_shows_and_stays_the_same_for_the_wait() {
    let (t0, ms) = (Instant::now(), Duration::from_millis);
    let mut s = Settle::new(ms(500));
    assert!(!s.feed("", t0));
    assert!(!s.feed("", t0 + ms(900)), "nothing typed yet");
    assert!(!s.feed("Today", t0 + ms(1000)));
    assert!(!s.feed("Today the", t0 + ms(1400)), "still typing");
    assert!(!s.feed("Today the", t0 + ms(1800)));
    assert!(s.feed("Today the", t0 + ms(1900)));
    assert_eq!(s.first(), Some(t0 + ms(1000)));
}

fn said(words: Option<u16>) -> Said {
    Said {
        lang: "en".into(),
        clip_ms: 6000,
        first_ms: Some(900),
        wrong: words.map(|words| Wrong { words, total: 14 }),
        look: BarLook {
            no_activate: true,
            topmost: true,
            above: true,
            apart: true,
            uncovered: true,
        },
        front: FrontCount {
            samples: 30,
            away: 0,
            bar: false,
        },
    }
}

#[test]
fn a_language_passes_within_card_diff_with_the_bar_and_the_foreground_right() {
    assert!(said(Some(1)).pass(1));
    assert!(!said(Some(2)).pass(1), "too many words wrong");
    assert!(!said(None).pass(1), "nothing judged");
    let mut s = said(Some(0));
    s.look.no_activate = false;
    assert!(!s.pass(1), "the bar could take focus");
    let mut s = said(Some(0));
    s.front.away = 1;
    assert!(!s.pass(1), "the target lost the foreground");
}

#[test]
fn a_languages_line_holds_numbers_never_text() {
    let v = said(Some(1)).json(1);
    assert_eq!(v["wrong"], 1);
    assert_eq!(v["of"], 14);
    assert_eq!(v["pass"], true);
    let strings: Vec<&str> = v
        .as_object()
        .unwrap()
        .values()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(strings, ["en"], "only the language's code is text");
}

/// The written-dictation line the shipped settings name.
fn marker() -> String {
    load().expect("the settings load").g22_product.wrote_line
}

fn card(id: &str) -> DevicePick {
    DevicePick::Card(CardId::new(id).expect("an id"))
}

/// A line of the copy's log stamped at second `s` past 10:00 that holds `rest`.
fn logged(s: u8, rest: &str) -> String {
    format!("2026-11-16T10:00:{s:02}.000000Z  INFO keyxtend::played::told: {rest}")
}

#[test]
fn the_copys_written_lines_since_its_start_give_each_device_by_id() {
    let m = marker();
    let log = [
        logged(0, &format!("{m} device=Vulkan9")),
        logged(2, "another line device=Vulkan8"),
        logged(3, &format!("{m} device=cpu")),
        logged(4, &format!("{m} device=Vulkan0")),
    ]
    .join("\n");
    let since = stamp_us("2026-11-16T10:00:01.000000Z").expect("a stamp");
    let got = wrote::read(&log, since, &m);
    assert_eq!(got, Ok(vec![DevicePick::Cpu, card("Vulkan0")]));
    assert_eq!(wrote::read("", since, &m), Ok(Vec::new()), "none written");
}

#[test]
fn a_written_line_without_a_device_id_is_refused_and_its_value_never_printed() {
    let m = marker();
    for bad in [
        "device=auto",
        "device=a/b",
        "device=\"Vulkan0\"",
        "devices=cpu",
        "",
    ] {
        let log = logged(3, &format!("{m} {bad}"));
        let e = wrote::read(&log, 0, &m).expect_err(bad);
        assert!(e.contains(DEVICE), "{e}");
        assert!(bad.is_empty() || !e.contains(bad), "{e}");
    }
}

#[test]
fn with_a_card_picked_only_that_card_may_write_and_some_device_must() {
    let (vk, cpu, auto) = (card("Vulkan0"), DevicePick::Cpu, DevicePick::Auto);
    let fits = |read: &[&DevicePick], pick: &DevicePick| {
        let read = read.iter().map(|&d| d.clone()).collect();
        wrote::judge(&Ok(read), pick).1
    };
    assert!(fits(&[&vk], &vk));
    assert!(!fits(&[&cpu], &vk), "the processor wrote it");
    assert!(!fits(&[&card("Vulkan1")], &vk), "another card");
    assert!(!fits(&[&vk, &cpu], &vk), "one on the processor");
    assert!(!fits(&[], &vk), "no reading");
    assert!(fits(&[&cpu], &cpu) && !fits(&[&vk], &cpu), "the processor");
    assert!(
        fits(&[&cpu], &auto) && fits(&[&vk], &auto),
        "Automatic takes either"
    );
    assert!(!fits(&[], &auto), "but needs one");
    let (v, ok) = wrote::judge(&Err("no log".into()), &vk);
    assert!(!ok && v["device_error"] == "no log", "{v}");
}

#[test]
fn a_languages_device_fields_are_its_id_and_numbers_never_text() {
    let vk = card("Vulkan0");
    let (v, ok) = wrote::judge(&Ok(vec![DevicePick::Cpu, vk.clone()]), &DevicePick::Auto);
    assert!(ok);
    assert_eq!(v["device"], "Vulkan0", "the last one");
    assert_eq!(v["written"], 2);
    assert_eq!(v["device_match"], true);
    let strings: Vec<&str> = v
        .as_object()
        .unwrap()
        .values()
        .filter_map(|x| x.as_str())
        .collect();
    assert_eq!(strings, ["Vulkan0"], "only the device's id is text");
    let (none, _) = wrote::judge(&Ok(Vec::new()), &vk);
    let fields = [&none["device"], &none["written"], &none["device_match"]];
    assert_eq!(fields, [&Value::Null, &json!(0), &json!(false)]);
}

#[test]
fn the_device_field_is_the_one_the_app_logs() {
    assert!(
        APP_TOLD.contains(&format!("{DEVICE} = %")),
        "the app's field"
    );
}

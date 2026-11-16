//! G22 on the product's pure parts: the copy's settings, the bar's look, the foreground's count, the typed text's
//! settling and each language's verdict, which prints numbers only.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use kx_mod_voice::score::Wrong;
use kx_mod_voice::{SPEC, VoiceSettings};
use kx_settings::VERSION_KEY;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{WS_EX_NOACTIVATE, WS_EX_TOPMOST};

use super::bar::BarLook;
use super::front::{FrontCount, Fronts};
use super::settle::Settle;
use super::{Said, table};
use crate::config::load;

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

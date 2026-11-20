//! G22's cloud line settings: the shipped ones pass with the daily folder unset, each bad key is refused, the text wait
//! stays within the product's own `net_wait_s`, and the failed line is the app's.

use std::path::Path;

use kx_mod_voice::VoiceSettings;
use kx_module_api::MS_PER_S;

use super::super::G22Cloud;
use crate::cfgfile::{LOCAL_FILE, LOCAL_KEYS};
use crate::config::load;

/// The voice gate build's cloud speech, which logs each failed cloud dictation.
const APP_SENT: &str = include_str!("../../../../../apps/keyxtend/src/played/sent.rs");
/// A daily folder as the local file would set it; never a real one.
const DAILY: &str = r"E:\daily";

fn shipped() -> G22Cloud {
    load().expect("the settings load").g22_cloud
}

/// The product's own wait for one cloud answer, in ms.
fn net_wait_ms() -> u64 {
    let s = VoiceSettings::from_defaults().expect("the voice defaults");
    u64::from(s.net_wait_s) * u64::from(MS_PER_S)
}

#[test]
fn the_shipped_cloud_line_passes_its_check_and_each_bad_key_is_refused() {
    let c = shipped();
    assert_eq!(c.check(), Ok(()));
    let bad: [fn(&mut G22Cloud); 6] = [
        |c| c.daily_dir = "daily".into(),
        |c| c.daily_dir = r"\daily".into(),
        |c| c.open_ms = 0,
        |c| c.text_wait_ms = 0,
        |c| c.text_wait_ms = net_wait_ms() + 1,
        |c| c.failed_line = " ".into(),
    ];
    for (i, spoil) in bad.iter().enumerate() {
        let mut g = c.clone();
        spoil(&mut g);
        assert!(g.check().is_err(), "case {i} passed");
    }
    let mut good = c;
    (good.daily_dir, good.text_wait_ms) = (DAILY.into(), net_wait_ms());
    assert_eq!(good.check(), Ok(()), "a set folder, the product's own wait");
}

#[test]
fn an_unset_daily_folder_skips_the_line_naming_the_local_file() {
    let c = shipped();
    assert_eq!(c.daily(), None, "never in a tracked file");
    let why = G22Cloud::unset();
    assert!(
        why.contains(LOCAL_FILE) && why.contains("daily_dir"),
        "{why}"
    );
    assert!(LOCAL_KEYS.contains(&("g22_cloud", "daily_dir")));
    let mut set = c;
    set.daily_dir = DAILY.into();
    assert_eq!(set.daily(), Some(Path::new(DAILY)));
}

#[test]
fn a_bad_daily_folder_is_refused_without_printing_it() {
    let mut c = shipped();
    c.daily_dir = "owner-folder".into();
    let e = c.check().expect_err("not absolute");
    assert!(!e.contains("owner-folder"), "{e}");
}

#[test]
fn the_failed_line_is_the_apps() {
    let line = format!("\"{}\"", shipped().failed_line);
    assert!(APP_SENT.contains(&line), "{line}");
}

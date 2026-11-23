//! G22's cloud line settings: the tracked file names no daily folder, the loaded ones pass with or without this PC's,
//! each bad key is refused, and the text wait stays within the product's own `net_wait_s`.

use std::path::Path;

use kx_mod_voice::VoiceSettings;
use kx_module_api::MS_PER_S;

use super::super::G22Cloud;
use crate::cfgfile::{LOCAL_FILE, LOCAL_KEYS};
use crate::config::load;

/// A daily folder as the local file would set it; never a real one.
const DAILY: &str = r"E:\daily";
/// The tracked file that holds `[g22_cloud]`, read without this PC's local file.
const P11: &str = include_str!("../../../kx-gates-p11.toml");

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
    let bad: [fn(&mut G22Cloud); 5] = [
        |c| c.daily_dir = "daily".into(),
        |c| c.daily_dir = r"\daily".into(),
        |c| c.open_ms = 0,
        |c| c.text_wait_ms = 0,
        |c| c.text_wait_ms = net_wait_ms() + 1,
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
fn the_tracked_settings_name_no_daily_folder() {
    let p11: toml::Table = P11.parse().expect("the P11 file parses");
    assert!(p11["g22_cloud"].get("daily_dir").is_none());
}

#[test]
fn an_unset_daily_folder_skips_the_line_naming_the_local_file() {
    let mut c = shipped();
    c.daily_dir.clear();
    assert_eq!(c.daily(), None, "unset, as without the local file");
    let why = G22Cloud::unset();
    assert!(
        why.contains(LOCAL_FILE) && why.contains("daily_dir"),
        "{why}"
    );
    assert!(LOCAL_KEYS.contains(&("g22_cloud", "daily_dir")));
    c.daily_dir = DAILY.into();
    assert_eq!(c.daily(), Some(Path::new(DAILY)));
}

#[test]
fn a_bad_daily_folder_is_refused_without_printing_it() {
    let mut c = shipped();
    c.daily_dir = "owner-folder".into();
    let e = c.check().expect_err("not absolute");
    assert!(!e.contains("owner-folder"), "{e}");
}

//! G22's cloud line (`[g22_cloud]`): the owner's daily data folder, which only this PC's local file sets, and the
//! copy's listening and text waits.
#![cfg(windows)]

use std::path::Path;

use kx_mod_voice::VoiceSettings;
use kx_module_api::MS_PER_S;
use kx_module_api::ports::path_ok;
use serde::Deserialize;

use crate::cfgfile::{FILE, LOCAL_FILE};

/// The key of the owner's daily data folder, as the help and the errors name it.
pub const DAILY: &str = "g22_cloud.daily_dir";

/// G22's cloud line on the product.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G22Cloud {
    /// The owner's daily data folder, an absolute drive path that only `LOCAL_FILE` sets; empty while unset.
    #[serde(default)]
    pub daily_dir: String,
    /// The wait after the bar shows, beside the clip and the bench's tail, before Stop, in ms; no model loads.
    pub open_ms: u64,
    /// The longest wait for a cloud dictation's text after Stop, in ms; at most `[voice] net_wait_s`.
    pub text_wait_ms: u64,
}

impl G22Cloud {
    /// The daily data folder; `None` while unset, so the cloud line is skipped.
    pub fn daily(&self) -> Option<&Path> {
        (!self.daily_dir.is_empty()).then(|| Path::new(&self.daily_dir))
    }

    /// Why the cloud line is skipped while the daily folder is unset, naming the local file that sets it.
    #[cfg_attr(
        not(feature = "voice"),
        allow(dead_code, reason = "Only the voice gates call it.")
    )]
    pub fn unset() -> String {
        format!("set {DAILY} to the daily data folder in {LOCAL_FILE} beside {FILE}")
    }

    /// Refuses settings the cloud line could not run or judge by, naming the first bad key but never the folder.
    pub fn check(&self) -> Result<(), String> {
        let s = VoiceSettings::from_defaults().map_err(|e| format!("[voice] defaults: {e}"))?;
        if self.daily().is_some_and(|d| !path_ok(d)) {
            return Err(format!("{DAILY} must be an absolute drive path"));
        }
        let most = u64::from(s.net_wait_s) * u64::from(MS_PER_S);
        if self.open_ms == 0 || self.text_wait_ms == 0 || self.text_wait_ms > most {
            return Err(
                "g22_cloud's waits must be above 0, text_wait_ms within [voice] net_wait_s".into(),
            );
        }
        Ok(())
    }
}

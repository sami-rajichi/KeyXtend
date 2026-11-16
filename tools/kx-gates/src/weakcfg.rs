//! The roadmap's weak PC (`[weak]` in `kx-gates.toml`), one section for every weak-mode run: G8's weak runs cap the
//! product's copy to it, and G22's bench caps its worker and reads its memory by the product's fit rule.
#![cfg(windows)]

use kx_mod_voice::VoiceSettings;
use kx_module_api::ports::SpeechAsk;
use kx_platform_windows::worker::CpuCap;
use serde::Deserialize;

use crate::weakjob::Caps;

/// The section's name.
pub const SECTION: &str = "weak";

/// The weak PC, as weak mode copies it here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weak {
    /// Cores a weak copy may use, which the speech host also counts.
    pub cores: u8,
    /// Each core's speed, in percent of one core here.
    pub core_pct: u8,
    /// Its memory in MB of 2²⁰ bytes, as the product counts: the job's cap and what the fit rule reads.
    pub mem_mb: u32,
}

impl Weak {
    /// Refuses a core count the speech host does not take, a speed outside 1 to 100 % of a core, memory no larger
    /// than `[voice] job_mb`, or caps no job takes.
    pub fn check(self) -> Result<(), String> {
        let settings =
            VoiceSettings::from_defaults().map_err(|e| format!("[voice] defaults: {e}"))?;
        if !(1..=*SpeechAsk::THREADS.end()).contains(&self.cores) {
            return Err(format!(
                "{SECTION}.cores must be a core count the speech host takes"
            ));
        }
        if !(1..=CpuCap::FULL_PCT).contains(&self.core_pct) {
            return Err(format!(
                "{SECTION}.core_pct must be 1 to {}",
                CpuCap::FULL_PCT
            ));
        }
        if self.mem_mb <= settings.job_mb {
            return Err(format!("{SECTION}.mem_mb must be above [voice] job_mb"));
        }
        Caps::from(self, 1)
            .map(drop)
            .map_err(|e| format!("{SECTION}: {e}"))
    }
}

#[cfg(test)]
mod tests;

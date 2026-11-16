//! The staged worker's edges (`[g22_staged]`): where `cargo xtask stage` puts the install, the gate's copy of it, the
//! engine parts each edge keeps or plants, and the language whose clip and default model the checks read.
#![cfg(windows)]

use serde::Deserialize;

use super::{G22Bench, is_part, plain_name};

/// The staged worker's edges 28, 29 and 39 (`g22 staged`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G22Staged {
    /// The staged app's folder, relative to the settings folder; it is only read.
    pub stage: String,
    /// The gate's copy of the staged worker's files, a plain folder name under TEMP.
    pub copy: String,
    /// The graphics-card part, which edge 28 leaves out.
    pub card_part: String,
    /// The plainest processor part, the only one edge 29 keeps.
    pub floor_part: String,
    /// The models folder edge 39 makes, a plain folder name in `g22_bench.models_dir`.
    pub plant: String,
    /// The part name the planted copy of `floor_part` takes there.
    pub plant_as: String,
    /// The clip's language, whose default model edges 29 and 39 check.
    pub lang: String,
    /// How often edges 29 and 39 list the worker's modules while its check runs, in ms.
    pub poll_ms: u64,
}

impl G22Staged {
    /// Refuses settings the edges could not run by, naming the first bad key; `bench` holds the clips and parts.
    pub fn check(&self, bench: &G22Bench) -> Result<(), String> {
        let stage = std::path::Path::new(&self.stage);
        if self.stage.is_empty() || !stage.is_relative() {
            return Err("g22_staged.stage must be the stage's folder, relative".into());
        }
        let plain = [&self.copy, &self.card_part, &self.plant];
        if !plain.iter().all(|n| plain_name(n)) || is_part(&self.card_part, &bench.part_prefix) {
            return Err("g22_staged.copy, card_part and plant must be plain names, the card no processor part".into());
        }
        let part = |n: &String| {
            plain_name(n) && n.len() > bench.part_prefix.len() && is_part(n, &bench.part_prefix)
        };
        if !part(&self.floor_part) || !part(&self.plant_as) {
            return Err("g22_staged.floor_part and plant_as must be processor parts".into());
        }
        if !bench.clips.contains_key(&self.lang) {
            return Err("g22_staged.lang must be one of g22_bench.clips".into());
        }
        if self.poll_ms == 0 {
            return Err("g22_staged.poll_ms must be above 0".into());
        }
        Ok(())
    }
}

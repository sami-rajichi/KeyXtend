//! G22's bench settings (`[g22_bench]` in `kx-gates-p11.toml`): the gate worker and its weak copy, the models folder,
//! the shipped voice folder with its detector and clips, weak mode and the pass rule. The dictation's numbers, the
//! words a text may differ by and each language's default model come from kx-mod-voice, so they are written once.
#![cfg(windows)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use kx_mod_voice::VoiceSettings;
use kx_mod_voice::models::{Table, table};
use kx_mod_voice::score::{self, Wrong};
use kx_module_api::ports::{SpeechAsk, path_ok};
use kx_platform_windows::worker::{CpuCap, WORKER_EXE};
use serde::Deserialize;

use super::plain_name;

/// One core at its full speed, in percent; weak mode's speed is a share of it.
const FULL_PCT: u8 = CpuCap::FULL_PCT;

/// True when `name` is one of the engine's processor parts: it starts with `prefix`, in any case.
pub fn is_part(name: &str, prefix: &str) -> bool {
    name.get(..prefix.len())
        .is_some_and(|s| s.eq_ignore_ascii_case(prefix))
}

/// One shipped check clip: its file and the sentence it reads, which only the words-wrong count reads.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    /// The WAV file's plain name in the voice folder.
    pub file: String,
    /// The sentence read in it, as `assets/voice/README.md` credits it.
    pub sentence: String,
    /// Other readings of the same words the count accepts, where the credited spelling is a quirk.
    pub also: Vec<String>,
}

impl Clip {
    /// Words wrong in `heard` against the closest of its readings, by `lang`'s rules; `None` for a language without.
    pub fn wrong(&self, heard: &str, lang: &str) -> Option<Wrong> {
        fewest(&self.sentence, &self.also, heard, lang)
    }
}

/// Words wrong in `heard` against the closest of `sentence` and its other readings `also`, by `lang`'s rules: the
/// one count the bench, the product run and the staged edges judge by.
pub fn fewest(sentence: &str, also: &[String], heard: &str, lang: &str) -> Option<Wrong> {
    let readings = std::iter::once(sentence).chain(also.iter().map(String::as_str));
    readings
        .filter_map(|r| score::wrong(r, heard, lang))
        .min_by_key(|w| w.words)
}

/// G22's bench (`g22 bench`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G22Bench {
    /// The worker built with `gate`, relative to the settings folder; never a staged one.
    pub worker: String,
    /// Weak mode's copy of it, in a folder whose processor parts are only `weak_parts`.
    pub weak_worker: String,
    /// What every processor part's file name starts with.
    pub part_prefix: String,
    /// The processor parts a weak PC's processor loads, which has no AVX.
    pub weak_parts: Vec<String>,
    /// The folder holding the model files, an absolute drive path.
    pub models_dir: String,
    /// The shipped voice folder, relative to the settings folder; the detector and clips are plain names in it.
    pub voice_dir: String,
    /// The shipped voice detector's plain name.
    pub detector: String,
    /// Weak mode's cores, which the host counts and the job's cap allows.
    pub weak_cores: u8,
    /// Weak mode's speed of each core, in percent.
    pub weak_core_pct: u8,
    /// Weak mode's memory in MB, which the product's fit rule reads.
    pub weak_mb: u32,
    /// How long each take listens past its clip's end, in ms.
    pub tail_ms: u64,
    /// The highest warm wait over clip length that passes in normal mode.
    pub max_rtf: f64,
    /// The weak rate past which a row reads slow; weak rows are readings, never a fail.
    pub max_weak_rtf: f64,
    /// The longest wait for a worker the bench or the staged edges let go to end, after the host's goodbye, in ms.
    pub end_wait_ms: u64,
    /// The check clips, by language.
    pub clips: BTreeMap<String, Clip>,
}

impl G22Bench {
    /// Refuses settings the bench could not run or judge by, naming the first bad key.
    pub fn check(&self) -> Result<(), String> {
        let settings =
            VoiceSettings::from_defaults().map_err(|e| format!("[voice] defaults: {e}"))?;
        let table = table().map_err(|e| format!("the models table: {e}"))?;
        self.check_workers()?;
        self.check_files()?;
        self.check_weak(&settings)?;
        self.check_rule(&settings)?;
        self.check_clips(table)?;
        self.check_asks(&settings, table)
    }

    /// Both workers, in folders of their own, and weak mode's processor parts.
    fn check_workers(&self) -> Result<(), String> {
        let (normal, weak) = (Path::new(&self.worker), Path::new(&self.weak_worker));
        for (key, exe) in [("worker", normal), ("weak_worker", weak)] {
            if exe
                .file_name()
                .is_none_or(|n| !n.eq_ignore_ascii_case(WORKER_EXE))
            {
                return Err(format!("g22_bench.{key} must name {WORKER_EXE}"));
            }
        }
        if normal.parent() == weak.parent() {
            return Err("g22_bench.weak_worker must sit in a folder of its own".into());
        }
        let prefix = self.part_prefix.as_str();
        let part = |p: &String| plain_name(p) && p.len() > prefix.len() && is_part(p, prefix);
        if prefix.is_empty() || self.weak_parts.is_empty() || !self.weak_parts.iter().all(part) {
            return Err(
                "g22_bench.weak_parts must be plain names that start with part_prefix".into(),
            );
        }
        Ok(())
    }

    /// The models folder and the shipped voice files.
    fn check_files(&self) -> Result<(), String> {
        if !path_ok(Path::new(&self.models_dir)) {
            return Err("g22_bench.models_dir must be an absolute drive path".into());
        }
        if self.voice_dir.is_empty() || !Path::new(&self.voice_dir).is_relative() {
            return Err("g22_bench.voice_dir must be the shipped folder, relative".into());
        }
        if !plain_name(&self.detector) {
            return Err("g22_bench.detector must be a plain name in voice_dir".into());
        }
        Ok(())
    }

    /// Weak mode's cores, speed and memory.
    fn check_weak(&self, settings: &VoiceSettings) -> Result<(), String> {
        if !(1..=*SpeechAsk::THREADS.end()).contains(&self.weak_cores) {
            return Err("g22_bench.weak_cores must be a core count the host takes".into());
        }
        if !(1..=FULL_PCT).contains(&self.weak_core_pct) {
            return Err(format!("g22_bench.weak_core_pct must be 1 to {FULL_PCT}"));
        }
        if self.weak_mb <= settings.job_mb {
            return Err("g22_bench.weak_mb must be above [voice] job_mb".into());
        }
        Ok(())
    }

    /// The tail, the pass rule and the end wait.
    fn check_rule(&self, settings: &VoiceSettings) -> Result<(), String> {
        if self.tail_ms < u64::from(settings.pad_ms) {
            return Err("g22_bench.tail_ms must be at least [voice] pad_ms".into());
        }
        let limit = |r: f64| r.is_finite() && r > 0.0;
        if !limit(self.max_rtf) || !limit(self.max_weak_rtf) || self.max_weak_rtf < self.max_rtf {
            return Err(
                "g22_bench.max_rtf and max_weak_rtf must be above 0, weak's no lower".into(),
            );
        }
        // A rate past this lets one piece outlast [voice] piece_wait_s, the wait the product allows it.
        let ceiling = f64::from(settings.piece_wait_s) / f64::from(settings.piece_s);
        if self.max_weak_rtf > ceiling {
            return Err(
                "g22_bench.max_weak_rtf must keep [voice] piece_s within piece_wait_s".into(),
            );
        }
        if self.end_wait_ms == 0 {
            return Err("g22_bench.end_wait_ms must be above 0".into());
        }
        Ok(())
    }

    /// One clip for each language the table has a tested order for, each a plain name with words its rules count.
    fn check_clips(&self, table: &Table) -> Result<(), String> {
        let want: BTreeSet<&str> = table.tested().collect();
        if self
            .clips
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != want
        {
            return Err(format!("g22_bench.clips must be exactly {want:?}"));
        }
        for (lang, clip) in &self.clips {
            let counted = |s: &str| score::wrong(s, s, lang).is_some_and(|w| w.total > 0);
            let differs =
                |s: &str| score::wrong(&clip.sentence, s, lang).is_some_and(|w| w.words > 0);
            let other = |s: &String| counted(s) && differs(s);
            if !plain_name(&clip.file) || !counted(&clip.sentence) || !clip.also.iter().all(other) {
                return Err(format!(
                    "g22_bench.clips.{lang} needs a plain file name, a counted sentence and other readings unlike it"
                ));
            }
        }
        Ok(())
    }

    /// Each language's default model, and the product's ask for it, which the speech port takes.
    fn check_asks(&self, settings: &VoiceSettings, table: &Table) -> Result<(), String> {
        let dir = Path::new(&self.models_dir);
        for lang in self.clips.keys() {
            let model = table.default_for(lang).and_then(|r| r.model(dir, lang));
            let model = model.ok_or_else(|| format!("no default model knows {lang}"))?;
            let ask = settings.ask(Some(model), None);
            ask.check()
                .map_err(|e| format!("g22_bench's ask for {lang}: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

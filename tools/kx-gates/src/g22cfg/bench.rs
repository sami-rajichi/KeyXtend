//! G22's bench settings (`[g22_bench]` in `kx-gates-p11.toml`): the gate worker and its weak copy, the models folder
//! from this PC's local file, the shipped voice folder with its detector and clips, and the pass rule; weak mode is
//! `[weak]`. The dictation's numbers, the words a text may differ by and each default come from kx-mod-voice.
#![cfg(windows)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use kx_mod_voice::VoiceSettings;
use kx_mod_voice::models::{Table, table};
use kx_mod_voice::score::{self, Wrong};
use kx_module_api::ports::path_ok;
use kx_platform_windows::worker::WORKER_EXE;
use serde::Deserialize;

use super::plain_name;
use crate::cfgfile::{FILE, LOCAL_FILE};

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

    /// Whether a row of `credits`, the credits file's table, starts with this clip's file and ends with its sentence.
    fn credited(&self, credits: &str) -> bool {
        let name = format!("`{}`", self.file);
        let row = |line: &str| {
            let cells: Vec<&str> = line
                .split('|')
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .collect();
            cells.first() == Some(&name.as_str()) && cells.last() == Some(&self.sentence.as_str())
        };
        credits.lines().any(row)
    }
}

/// The voice folder's credits file, whose table names each clip's sentence.
const CREDITS: &str = "README.md";

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
    /// The folder holding the model files, an absolute drive path that only `LOCAL_FILE` sets; empty while unset.
    #[serde(default)]
    pub models_dir: String,
    /// The shipped voice folder, relative to the settings folder; the detector and clips are plain names in it.
    pub voice_dir: String,
    /// The shipped voice detector's plain name.
    pub detector: String,
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
    /// Refuses settings the bench could not run or judge by, naming the first bad key; an unset models folder loads,
    /// so every other gate runs, and only the runs that need it refuse it (`models`).
    pub fn check(&self) -> Result<(), String> {
        let settings =
            VoiceSettings::from_defaults().map_err(|e| format!("[voice] defaults: {e}"))?;
        let table = table().map_err(|e| format!("the models table: {e}"))?;
        self.check_workers()?;
        self.check_files()?;
        self.check_rule(&settings)?;
        self.check_clips(table)?;
        self.check_asks(&settings, table)
    }

    /// Refuses a clip whose sentence is not the one the voice folder's credits give its file, the folder found from
    /// the settings folder `dir`; every G22 run judges by these sentences.
    pub fn credited(&self, dir: &Path) -> Result<(), String> {
        let file = dir.join(&self.voice_dir).join(CREDITS);
        let credits = std::fs::read_to_string(file)
            .map_err(|e| format!("the voice folder's {CREDITS} could not be read: {e}"))?;
        self.check_credits(&credits)
    }

    /// Refuses a clip whose sentence is not the one `credits`, the credits file's text, gives its file, naming only
    /// its language.
    pub fn check_credits(&self, credits: &str) -> Result<(), String> {
        match self.clips.iter().find(|(_, c)| !c.credited(credits)) {
            Some((lang, _)) => Err(format!(
                "the {lang} clip's sentence is not the one the voice folder's {CREDITS} credits, so the settings file \
                 may have been re-encoded"
            )),
            None => Ok(()),
        }
    }

    /// The models folder.
    ///
    /// # Errors
    /// While it is unset, naming the local file that sets it.
    pub fn models(&self) -> Result<&str, String> {
        if self.models_dir.is_empty() {
            return Err(format!(
                "set g22_bench.models_dir to this PC's models folder in {LOCAL_FILE} beside {FILE}"
            ));
        }
        Ok(&self.models_dir)
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

    /// The models folder, once set, and the shipped voice files.
    fn check_files(&self) -> Result<(), String> {
        let set = !self.models_dir.is_empty();
        if set && !path_ok(Path::new(&self.models_dir)) {
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

    /// Each language's default model, and the product's ask for it in the models folder once set, which the speech
    /// port takes.
    fn check_asks(&self, settings: &VoiceSettings, table: &Table) -> Result<(), String> {
        let Ok(dir) = self.models().map(Path::new) else {
            return Ok(());
        };
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

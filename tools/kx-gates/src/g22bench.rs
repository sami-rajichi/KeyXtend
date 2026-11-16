//! G22's bench (`g22 bench`, risk 7): each check clip's default model reads that clip through the gate worker and the
//! product's speech host, normal then as the roadmap's weak PC, and one table of numbers prints, never the text. Weak
//! mode is `[weak]`'s few slow cores (the job's cap), no AVX (a worker folder with only the plain processor parts) and
//! its memory (the product's own fit rule); the bench starts no screen and moves nothing.
#![cfg(windows)]

pub(crate) mod clip;
mod spawn;
mod table;
mod take;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use kx_mod_voice::VoiceSettings;
use kx_mod_voice::models::{Row as ModelRow, table as models};
use kx_module_api::MS_PER_S;
use kx_module_api::ports::{Memory, ModelState, SpeechModel};
use kx_platform::speech::{Assets, Hardware, WorkerSpeech};
use kx_platform_windows::worker::CpuCap;
use kx_platform_windows::{QpcClock, WinHardware, WinSpawner};
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::g22::{self, BENCH};
use crate::g22cfg::{Clip, G22Bench};
use crate::out;
use crate::weakcfg::Weak;
use spawn::{Cores, GateWorker, lock};
use table::{Miss, Row, Rule, Timed, mode};
use take::{Script, take};

/// Why a row has no numbers, beside a take's own fault.
const NO_DEFAULT: &str = "no default model";
const NOT_ENDED: &str = "the worker did not end in time";
const RELOADED: &str = "the warm take loaded the model again";

/// One clip in one mode: its default model, its plain path and length, and the cores the host counts.
struct Case<'c> {
    lang: &'c str,
    clip: &'c Clip,
    model: SpeechModel,
    path: PathBuf,
    clip_ms: u32,
    weak: bool,
    cores: usize,
}

/// What a run reads once: the resolved files, the product's `[voice]` defaults and the processor's real cores.
struct Bench<'a> {
    g: &'a G22Bench,
    weak: Weak,
    worker: PathBuf,
    weak_worker: PathBuf,
    voice: PathBuf,
    detector: PathBuf,
    models: PathBuf,
    settings: VoiceSettings,
    real: usize,
}

/// Runs the bench: both modes of each clip, then the table on stdout and the numbers in the result line.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let bench = Bench::of(&ctx.cfg.g22_bench, ctx.cfg.weak, &ctx.cfg.dir)?;
    let rule = bench.rule();
    let mut rows = Vec::new();
    for (lang, clip) in &bench.g.clips {
        for weak in [false, true] {
            eprintln!("g22 bench: {lang}, {}", mode(weak));
            rows.push(bench.row(lang, clip, weak));
        }
    }
    let mut text = vec![bench.lead(&rule)];
    text.extend(table::lines(&rows, &rule));
    for line in &text {
        out::say!("{line}");
    }
    let pass = table::pass(&rows, &rule);
    let rows: Vec<Value> = rows.iter().map(|r| r.json(&rule)).collect();
    Ok(json!({ "gate": "G22", "name": BENCH, "pass": pass, "rows": rows, "table": text }))
}

impl<'a> Bench<'a> {
    /// The bench `g` describes on the weak PC `weak`, its paths from the settings folder `dir`; a missing file is an
    /// error naming it.
    fn of(g: &'a G22Bench, weak: Weak, dir: &Path) -> Result<Self, String> {
        let file = |rel: &str| out::plain(&dir.join(rel));
        let worker = file(&g.worker).map_err(|e| format!("{e}; build it with --features gate"))?;
        let weak_worker = file(&g.weak_worker).map_err(|e| {
            format!(
                "{e}; copy the gate worker's folder there with only {:?}",
                g.weak_parts
            )
        })?;
        spawn::plain_parts(&weak_worker, &g.part_prefix, &g.weak_parts)?;
        let models = PathBuf::from(g.models()?);
        if !models.is_dir() {
            return Err(format!(
                "g22_bench.models_dir {} is no folder",
                models.display()
            ));
        }
        let voice = file(&g.voice_dir)?;
        Ok(Self {
            g,
            weak,
            worker,
            weak_worker,
            detector: out::plain(&voice.join(&g.detector))?,
            voice,
            models,
            settings: VoiceSettings::from_defaults().map_err(|e| e.to_string())?,
            real: WinHardware::new().cores(),
        })
    }

    /// The pass rule: the settings' rates (weak's only marks a row slow), and `[voice] card_diff` words wrong.
    fn rule(&self) -> Rule {
        Rule {
            rtf: self.g.max_rtf,
            weak_rtf: self.g.max_weak_rtf,
            wrong: u16::from(self.settings.card_diff),
        }
    }

    /// The table's first line: weak mode and the pass rule.
    fn lead(&self, rule: &Rule) -> String {
        let w = self.weak;
        format!(
            "G22 bench: weak mode is {} cores at {} % each, {} MB and only {:?}; RTF is the warm wait over the \
             clip; {}",
            w.cores,
            w.core_pct,
            w.mem_mb,
            self.g.weak_parts,
            rule.text()
        )
    }

    /// The row of `lang`'s clip in one mode.
    fn row(&self, lang: &str, clip: &Clip, weak: bool) -> Row {
        let cores = if weak {
            usize::from(self.weak.cores)
        } else {
            self.real
        };
        let default = models().ok().and_then(|t| t.default_for(lang));
        let mut row = Row {
            lang: lang.to_owned(),
            model: default.map(|r| r.name.clone()).unwrap_or_default(),
            weak,
            cores,
            cap: None,
            clip_ms: 0,
            got: Err(Miss::Fault(NO_DEFAULT.into())),
        };
        if let Some(default) = default {
            match self.case(lang, clip, default, weak, cores) {
                Ok(case) => (row.clip_ms, (row.cap, row.got)) = (case.clip_ms, self.timed(&case)),
                Err(miss) => row.got = Err(miss),
            }
        }
        row
    }

    /// The case of `lang`'s clip: its default model, which must fit the mode's PC and be in the folder, and the clip.
    fn case<'c>(
        &self,
        lang: &'c str,
        clip: &'c Clip,
        default: &ModelRow,
        weak: bool,
        cores: usize,
    ) -> Result<Case<'c>, Miss> {
        self.fits(default, weak)?;
        let fault = |e: &str| Miss::Fault(e.to_owned());
        let model = default
            .model(&self.models, lang)
            .ok_or_else(|| fault("the default does not know it"))?;
        if !model.file.expose().is_file() {
            return Err(fault(&format!(
                "{} is not in the models folder",
                default.file
            )));
        }
        let path = out::plain(&self.voice.join(&clip.file)).map_err(Miss::Fault)?;
        let clip_ms = clip::read_ms(&path).map_err(Miss::Fault)?;
        let room = u64::from(self.settings.record_max_s) * u64::from(MS_PER_S);
        if u64::from(clip_ms) + self.g.tail_ms >= room {
            return Err(fault("the clip and its tail pass [voice] record_max_s"));
        }
        Ok(Case {
            lang,
            clip,
            model,
            path,
            clip_ms,
            weak,
            cores,
        })
    }

    /// In weak mode, the product's own rule on the weak PC's memory: `NotHere` when it would mark the default so.
    fn fits(&self, default: &ModelRow, weak: bool) -> Result<(), Miss> {
        if !weak {
            return Ok(());
        }
        let total_mb = self.weak.mem_mb;
        let memory = Memory {
            total_mb,
            free_mb: total_mb,
        };
        match default.not_here(&memory, self.settings.job_mb) {
            Some(ModelState::NotHere {
                need_mb, have_mb, ..
            }) => Err(Miss::NotHere { need_mb, have_mb }),
            _ => Ok(()),
        }
    }

    /// Both takes of `case` in one fresh job, then its end: the cap read back from the job and what was measured.
    fn timed(&self, case: &Case) -> (Option<u32>, Result<Timed, Miss>) {
        let Some(clip) = case.path.to_str() else {
            return (
                None,
                Err(Miss::Fault("the clip's path is not UTF-8".into())),
            );
        };
        let extra = vec![g22::sound_flag(), clip.to_owned()];
        let spawner = if case.weak {
            let cap = CpuCap {
                cores: case.cores,
                pct: self.weak.core_pct,
            };
            WinSpawner::at(self.weak_worker.clone()).capped(cap)
        } else {
            WinSpawner::at(self.worker.clone())
        };
        let gate = GateWorker::new(spawner, extra, Duration::from_millis(self.g.end_wait_ms));
        let (cap, alive) = (gate.cap(), gate.alive());
        let got = self.takes(gate, case);
        let ended = alive.none_within(self.let_go());
        let got = got.and_then(|t| if ended { Ok(t) } else { Err(NOT_ENDED.into()) });
        (*lock(&cap), got.map_err(Miss::Fault))
    }

    /// The cold take, which loads the model, then the warm take on the loaded model; the job ends with `speech`.
    fn takes(&self, gate: GateWorker, case: &Case) -> Result<Timed, String> {
        let clock = Arc::new(QpcClock::new().map_err(|e| e.to_string())?);
        let assets = Assets {
            detector: self.detector.clone(),
            ..Assets::default()
        };
        let speech = WorkerSpeech::new(gate, clock, assets, Arc::new(Cores(case.cores)));
        let ask = self.settings.ask(Some(case.model.clone()), None);
        let sound_ms = u64::from(case.clip_ms).saturating_add(self.g.tail_ms);
        let levels = sound_ms.saturating_mul(u64::from(self.settings.level_hz));
        let script = Script {
            levels: u32::try_from(levels.div_ceil(u64::from(MS_PER_S))).unwrap_or(u32::MAX),
            within: within(&self.settings),
            sentence: &case.clip.sentence,
            also: &case.clip.also,
            lang: case.lang,
        };
        let cold = take(&speech, ask.clone(), &script).map_err(|e| format!("cold: {e}"))?;
        let warm = take(&speech, ask, &script).map_err(|e| format!("warm: {e}"))?;
        if warm.load_ms > 0 {
            return Err(RELOADED.into());
        }
        Ok(Timed {
            start_ms: cold.open_ms,
            load_ms: cold.load_ms,
            first_use_ms: cold.total_ms,
            first_ms: warm.first_ms,
            total_ms: warm.total_ms,
            peak_mb: cold.memory_mb.max(warm.memory_mb),
            wrong: warm.wrong,
        })
    }

    /// The longest wait for a worker let go to end: the host's goodbye within one answer, then the end's own wait.
    fn let_go(&self) -> Duration {
        Duration::from_millis(u64::from(self.settings.ask_ms).saturating_add(self.g.end_wait_ms))
    }
}

/// The longest wait for a clip to be heard, or for text after Stop: the host's own for a load, a warm-up and a piece,
/// and one answer, from the product's settings `s`.
pub(crate) fn within(s: &VoiceSettings) -> Duration {
    let secs = u64::from(s.load_s) + u64::from(s.warm_s) + u64::from(s.piece_wait_s);
    Duration::from_secs(secs) + Duration::from_millis(u64::from(s.ask_ms))
}

#[cfg(test)]
mod tests;

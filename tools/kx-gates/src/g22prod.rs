//! G22 on the product (`g22 keyxtend`): a copy of the voice gate build plays each check clip in place of the microphone
//! while the gate dictates it into target-window, and a language passes within `[voice] card_diff`, with the foreground
//! kept, the bar on top and never activated, and the right writer in the copy's log. It prints numbers and ids, never
//! text: a passing language's timing, or a failed one's marked log lines, windows in front and bar button ids.
#![cfg(windows)]

mod bar;
mod borrow;
mod cloud;
mod cloudset;
mod front;
mod line;
mod say;
mod settle;
mod stages;
mod why;
mod wrote;

use std::path::Path;

use kx_kernel::{KERNEL, KernelSettings, SPEC as KERNEL_SPEC};
use kx_mod_voice::score::Wrong;
use kx_mod_voice::{SPEC, VoiceSettings};
use kx_module_api::ports::DevicePick;
use kx_settings::VERSION_KEY;
use serde::Serialize;
use serde_json::{Value, json};
use toml::Table;

use crate::apps::Ctx;
use crate::g22::{app_build, worker_build};
use crate::g22bench::clip::read_ms;
use crate::g22cfg::{Clip, G22Bench, G22Product, clip_in};
use crate::out::{self, say};
use crate::product::NAME;
use bar::BarLook;
use front::FrontCount;
use line::Line;

/// The gate's name in its result lines.
const GATE: &str = "G22";

/// One clip to dictate: its language, its readings, its file as the copy's flag takes it, and its length.
struct Case<'a> {
    lang: String,
    clip: &'a Clip,
    path: String,
    clip_ms: u32,
}

/// One language's reading: the words wrong, the bar's look and the foreground's count, numbers and one class name.
#[derive(Debug, Clone)]
struct Said {
    lang: String,
    clip_ms: u32,
    /// Stop to the first typed text, in ms.
    first_ms: Option<u64>,
    wrong: Option<Wrong>,
    look: BarLook,
    front: FrontCount,
    /// The class of the window over the bar's box, only when one covers it.
    top_class: Option<String>,
}

impl Said {
    /// Within `diff` words wrong, with the bar's look right and target-window in front all through.
    fn pass(&self, diff: u8) -> bool {
        let words = self.wrong.is_some_and(|w| w.words <= u16::from(diff));
        words && self.look.ok() && self.front.kept()
    }

    /// Its result line, which names the language and holds numbers, with the covering window's class when one covers
    /// the bar.
    fn json(&self, diff: u8) -> Value {
        let (l, f) = (&self.look, &self.front);
        let mut line = json!({
            "lang": self.lang, "clip_ms": self.clip_ms, "first_ms": self.first_ms,
            "wrong": self.wrong.map(|w| w.words), "of": self.wrong.map(|w| w.total),
            "no_activate": l.no_activate, "topmost": l.topmost, "above": l.above, "apart": l.apart,
            "uncovered": l.uncovered,
            "samples": f.samples, "away": f.away, "bar_front": f.bar, "pass": self.pass(diff),
        });
        if let Some(class) = &self.top_class {
            line["top_class"] = json!(class);
        }
        line
    }
}

/// The local copy's settings file: `[voice]`'s defaults with the bench's models folder and the run's device, and
/// `[kernel]`'s with the run's log level.
fn table(g: &G22Product, bench: &G22Bench) -> Result<Table, String> {
    let mut s = VoiceSettings::from_defaults().map_err(|e| format!("the voice defaults: {e}"))?;
    (s.models_dir, s.device) = (bench.models()?.to_owned(), g.device.clone());
    table_of(&s, g)
}

/// A copy's settings file: `[voice]` as `s`, and `[kernel]`'s defaults with the run's log level.
fn table_of(s: &VoiceSettings, g: &G22Product) -> Result<Table, String> {
    let mut k = KernelSettings::from_defaults().map_err(|e| format!("the kernel defaults: {e}"))?;
    k.log_level.clone_from(&g.log_level);
    let mut table = Table::new();
    table.insert(
        kx_mod_voice::ID.as_str().into(),
        section(s, SPEC.version)?.into(),
    );
    table.insert(
        KERNEL.as_str().into(),
        section(&k, KERNEL_SPEC.version)?.into(),
    );
    Ok(table)
}

/// Settings `s` as a section of version `version`; it goes through the file's text, so a day stays a plain date.
fn section(s: &impl Serialize, version: u32) -> Result<Table, String> {
    let fail = |e: &dyn std::fmt::Display| format!("the copy's settings: {e}");
    let text = toml::to_string(s).map_err(|e| fail(&e))?;
    let mut t: Table = toml::from_str(&text).map_err(|e| fail(&e))?;
    t.insert(VERSION_KEY.into(), i64::from(version).into());
    Ok(t)
}

/// Each clip of the bench, its file in the voice folder `voice` and its length read.
fn cases<'a>(bench: &'a G22Bench, voice: &Path) -> Result<Vec<Case<'a>>, String> {
    let case = |(lang, clip): (&String, &'a Clip)| {
        let file = clip_in(voice, &clip.file)?;
        let path = file
            .to_str()
            .ok_or("the clip's path is not UTF-8")?
            .to_owned();
        Ok(Case {
            lang: lang.clone(),
            clip,
            clip_ms: read_ms(&file)?,
            path,
        })
    };
    bench.clips.iter().map(case).collect()
}

/// Fine unless `pick` is a card and `has_part` finds no card part beside the gate worker, a run the gate cannot judge.
fn card_ready(pick: Option<&DevicePick>, has_part: impl FnOnce() -> bool) -> Result<(), String> {
    if matches!(pick, Some(DevicePick::Card(_))) && !has_part() {
        return Err(format!(
            "the gate worker has no card part, so build it with {}",
            worker_build()
        ));
    }
    Ok(())
}

/// Runs G22 on the product: the local line, one copy per clip on a data folder that then goes, and the cloud line.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let (g, b) = (&ctx.cfg.g22_product, &ctx.cfg.g22_bench);
    let program = out::plain(&ctx.cfg.dir.join(&g.program))
        .map_err(|e| format!("{e}; build it with {}", app_build()))?;
    // The copy's worker is the gate worker beside the program.
    let part = || {
        program
            .with_file_name(&ctx.cfg.g22_staged.card_part)
            .is_file()
    };
    card_ready(g.pick().as_ref(), part)?;
    let cases = cases(b, &out::plain(&ctx.cfg.dir.join(&b.voice_dir))?)?;
    let settings = VoiceSettings::from_defaults().map_err(|e| e.to_string())?;
    let (file, local) = (table(g, b)?, Line::local(ctx, &settings)?);
    let all: Vec<&Case> = cases.iter().collect();
    let run = (all.as_slice(), &settings, &local);
    let mut out = line::on_copy(ctx, &program, run, |f| f.write(&file))?;
    let cloud = cloud::run(ctx, &program, &cases, &settings);
    let pass = out["pass"] == true && cloud["pass"] != false;
    say!("{GATE} {NAME}: {pass}");
    if let Some(o) = out.as_object_mut() {
        o.insert("gate".into(), json!(GATE));
        o.insert("name".into(), json!(NAME));
        o.insert("cloud".into(), cloud);
        o.insert("pass".into(), json!(pass));
    }
    Ok(out)
}

#[cfg(test)]
mod tests;

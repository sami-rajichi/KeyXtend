//! G22 on the product (`g22 keyxtend`): for each check clip, a copy of the voice gate build on a data folder the gate
//! makes plays that clip through the gate worker in place of the microphone, while the gate dictates it by clicks into
//! target-window in the clip's layout. The words typed must be within `[voice] card_diff`, target-window must keep the
//! foreground, and the bar must sit on top above the keyboard, never activated. It prints numbers, never text.
#![cfg(windows)]

mod bar;
mod front;
mod say;
mod settle;

use std::path::Path;

use kx_mod_voice::score::Wrong;
use kx_mod_voice::{SPEC, VoiceSettings};
use kx_settings::VERSION_KEY;
use serde_json::{Value, json};
use toml::Table;

use crate::apps::Ctx;
use crate::copyend::{self, Ended};
use crate::freshcopy::Fresh;
use crate::g22;
use crate::g22bench::clip::read_ms;
use crate::g22cfg::{Clip, G22Bench, G22Product, clip_in};
use crate::out::{self, say};
use crate::product::NAME;
use crate::typebox;
use bar::BarLook;
use front::FrontCount;

/// The gate's name in its result lines.
const GATE: &str = "G22";

/// One clip to dictate: its language, its readings, its file as the copy's flag takes it, and its length.
struct Case<'a> {
    lang: String,
    clip: &'a Clip,
    path: String,
    clip_ms: u32,
}

/// One language's reading: the words wrong, the bar's look and the foreground's count, numbers only.
#[derive(Debug, Clone)]
struct Said {
    lang: String,
    clip_ms: u32,
    /// Stop to the first typed text, in ms.
    first_ms: Option<u64>,
    wrong: Option<Wrong>,
    look: BarLook,
    front: FrontCount,
}

impl Said {
    /// Within `diff` words wrong, with the bar's look right and target-window in front all through.
    fn pass(&self, diff: u8) -> bool {
        let words = self.wrong.is_some_and(|w| w.words <= u16::from(diff));
        words && self.look.ok() && self.front.kept()
    }

    /// Its result line, which names the language and holds numbers only.
    fn json(&self, diff: u8) -> Value {
        let (l, f) = (&self.look, &self.front);
        json!({
            "lang": self.lang, "clip_ms": self.clip_ms, "first_ms": self.first_ms,
            "wrong": self.wrong.map(|w| w.words), "of": self.wrong.map(|w| w.total),
            "no_activate": l.no_activate, "topmost": l.topmost, "above": l.above, "apart": l.apart,
            "uncovered": l.uncovered,
            "samples": f.samples, "away": f.away, "bar_front": f.bar, "pass": self.pass(diff),
        })
    }
}

/// The copy's settings file: `[voice]`'s defaults with the bench's models folder and the run's device.
fn table(g: &G22Product, bench: &G22Bench) -> Result<Table, String> {
    let mut s = VoiceSettings::from_defaults().map_err(|e| format!("the voice defaults: {e}"))?;
    (s.models_dir, s.device) = (bench.models()?.to_owned(), g.device.clone());
    let mut section = Table::try_from(&s).map_err(|e| format!("the copy's voice settings: {e}"))?;
    section.insert(VERSION_KEY.into(), i64::from(SPEC.version).into());
    let mut table = Table::new();
    table.insert(kx_mod_voice::ID.as_str().into(), section.into());
    Ok(table)
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

/// Runs G22 on the product: one copy per clip, then the data folder goes.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let (g, b) = (&ctx.cfg.g22_product, &ctx.cfg.g22_bench);
    let program = out::plain(&ctx.cfg.dir.join(&g.program))
        .map_err(|e| format!("{e}; build it with --features gate-voice"))?;
    let cases = cases(b, &out::plain(&ctx.cfg.dir.join(&b.voice_dir))?)?;
    let (settings, file) = (
        VoiceSettings::from_defaults().map_err(|e| e.to_string())?,
        table(g, b)?,
    );
    let fresh = Fresh::open_at(ctx, GATE, program)?;
    let (lines, ended) = match fresh.write(&file) {
        Ok(()) => dictate_all(ctx, &fresh, &cases, &settings),
        Err(e) => (vec![json!({ "error": e, "pass": false })], None),
    };
    let removed = fresh.close(ended.as_ref());
    let pass = !lines.is_empty() && lines.iter().all(|l| l["pass"] == true) && removed.is_ok();
    say!("{GATE} {NAME}: {pass}");
    Ok(json!({
        "gate": GATE, "name": NAME, "langs": lines, "copy": ended.as_ref().map(Ended::line),
        "data_removed": removed.is_ok(), "data_error": removed.err(), "pass": pass,
    }))
}

/// Each case on a copy of its own, while each copy before it ended: the lines, and how the last copy went.
fn dictate_all(
    ctx: &Ctx,
    fresh: &Fresh,
    cases: &[Case],
    s: &VoiceSettings,
) -> (Vec<Value>, Option<Ended>) {
    let (g, diff) = (&ctx.cfg.g22_product, s.card_diff);
    let (mut lines, mut last) = (Vec::new(), None);
    for case in cases {
        eprintln!("g22 {NAME}: {}", case.lang);
        let args = [g22::sound_flag(), case.path.clone()];
        let drive = |pid| one(ctx, pid, case, s);
        let (got, ended) = fresh.with_args(&args, g.ready_ms, Some(&g.warn_line), drive);
        let mut line = got.map_or_else(
            |e| json!({ "lang": case.lang, "error": e, "pass": false }),
            |l| l.json_with(diff),
        );
        let quit = ended.as_ref().is_some_and(Ended::ok);
        line["pass"] = json!(line["pass"] == true && quit);
        line["copy_quit"] = json!(quit);
        lines.push(line);
        let over = copyend::may_follow(ended.as_ref().map(|e| e.gone));
        last = ended;
        if !over {
            break;
        }
    }
    (lines, last)
}

/// One case in a fresh target-window on the copy `pid`: its line, and whether the box's first layout came back.
fn one(ctx: &Ctx, pid: u32, case: &Case, s: &VoiceSettings) -> Result<Done, String> {
    let boxed = typebox::in_target(ctx, &[], |t| say::dictate(ctx, pid, t, case, s));
    Ok(Done {
        said: boxed.got?,
        back: boxed.back,
        left: boxed.left,
    })
}

/// A case's end: its reading, whether the layout came back and what clean-up let go.
struct Done {
    said: Said,
    back: bool,
    left: Vec<String>,
}

impl Done {
    /// Its line: the reading's, with the layout's return and the clean-up, which the pass needs too.
    fn json_with(&self, diff: u8) -> Value {
        let mut v = self.said.json(diff);
        v["pass"] = json!(v["pass"] == true && self.back);
        v["layout_back"] = json!(self.back);
        v["clean_up"] = json!(self.left);
        v
    }
}

#[cfg(test)]
mod tests;

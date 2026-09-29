//! G22 bench: the kept clips through the local engine, the local engine in weak mode, and the cloud engine when allowed.
//! Only times and word error rates are reported, never the words; the clips are deleted once every run worked.
#![cfg(windows)]

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use spike_core::voicecfg::{Engine, VoiceConfig};
use spike_core::voiceproto::{Cmd, Event, clip_lang};
use spike_core::voiceworker::Worker;

use crate::apps::Ctx;
use crate::g22cfg::G22;
use crate::{wer, win};

/// Bench name that also runs the cloud engine; the key must be in kx-gates' environment.
pub const WITH_CLOUD: &str = "bench-cloud";
/// Shown when the cloud run was left out.
const CLOUD_SKIPPED: &str = "skipped: cloud is off";

/// A worker the bench started, and its events.
struct Bench {
    worker: Worker,
    events: Receiver<Event>,
}

impl Bench {
    fn start(exe: &Path, args: &[String], max_line: usize) -> Result<Bench, String> {
        let (tx, events) = mpsc::channel();
        let worker = Worker::spawn(exe, args, max_line, move |e| drop(tx.send(e)))?;
        Ok(Bench { worker, events })
    }

    /// The next event, waiting up to `wait_ms`.
    fn next(&self, wait_ms: u64) -> Option<Event> {
        self.events
            .recv_timeout(Duration::from_millis(wait_ms))
            .ok()
    }

    /// The next event that is not a note, waiting up to `wait_ms` in all.
    fn answer(&self, wait_ms: u64) -> Option<Event> {
        let end = Instant::now() + Duration::from_millis(wait_ms);
        loop {
            let left = end.saturating_duration_since(Instant::now());
            match self.events.recv_timeout(left).ok()? {
                Event::Note { .. } => {}
                e => return Some(e),
            }
        }
    }
}

/// The kept clips and their languages, sorted by name: by language, then time.
pub fn clips(dir: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let list = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut all: Vec<_> = list
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let lang = clip_lang(&name)?.to_string();
            Some((e.path(), lang))
        })
        .collect();
    all.sort();
    Ok(all)
}

/// Deletes the listed clips, then the folder, which fails if anything else is in it.
fn delete(dir: &Path, clips: &[(PathBuf, String)]) -> Result<(), String> {
    for (p, _) in clips {
        std::fs::remove_file(p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    std::fs::remove_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))
}

/// Deletes the clips once every run passed: true when gone, false when kept, else the error.
fn clean_up(pass: bool, dir: &Path, clips: &[(PathBuf, String)]) -> Value {
    if !pass {
        return json!(false);
    }
    match delete(dir, clips) {
        Ok(()) => json!(true),
        Err(e) => json!({ "error": e }),
    }
}

/// The sentence read in `lang`: the one whose layout the voice settings map to it; the settings allow one per language.
fn sentence<'a>(g: &'a G22, v: &VoiceConfig, lang: &str) -> Option<&'a str> {
    g.sentences
        .iter()
        .find(|s| s.layout_id().is_some_and(|id| v.language(id) == lang))
        .map(|s| s.text.as_str())
}

/// Time to answer over clip length; below 1 is faster than real time.
#[allow(
    clippy::cast_precision_loss,
    reason = "Clip times in ms are far below 2^53."
)]
fn rtf(ms: u64, audio_ms: u64) -> Option<f64> {
    (audio_ms > 0).then(|| ms as f64 / audio_ms as f64)
}

/// Mean of the numbers at `key` in `rows`.
#[allow(clippy::cast_precision_loss, reason = "Row counts are far below 2^53.")]
fn mean(rows: &[Value], key: &str) -> Option<f64> {
    let v: Vec<f64> = rows.iter().filter_map(|r| r[key].as_f64()).collect();
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// One clip through `engine`: its times and word error rate.
fn one(ctx: &Ctx, b: &mut Bench, engine: Engine, (path, lang): &(PathBuf, String)) -> Value {
    let path = path.to_string_lossy().into_owned();
    let cmd = Cmd::File {
        path,
        lang: lang.clone(),
        engine,
    };
    if let Err(e) = b.worker.send(&cmd) {
        return json!({ "lang": lang, "error": e });
    }
    let want = sentence(&ctx.cfg.g22, &ctx.spike.voice, lang);
    match b.answer(ctx.cfg.g22.answer_wait_ms) {
        Some(Event::Text {
            text, ms, audio_ms, ..
        }) => json!({
            "lang": lang, "audio_ms": audio_ms, "ms": ms, "rtf": rtf(ms, audio_ms),
            "wer": want.and_then(|s| wer::wer(s, &text)),
        }),
        Some(Event::Error { error }) => json!({ "lang": lang, "error": error }),
        _ => json!({ "lang": lang, "error": "no answer in time" }),
    }
}

/// True once nothing listens on the local server's port, so the next worker can start its own.
fn port_free(ctx: &Ctx) -> bool {
    let (l, t) = (&ctx.spike.voice.local, &ctx.cfg.timing);
    let addr = format!("{}:{}", l.host, l.port);
    win::poll_until(t.close_wait_ms, t.poll_ms, || {
        TcpStream::connect(&addr).is_err().then_some(())
    })
    .is_some()
}

/// One worker set-up: start (the model loads once), every clip, quit.
fn run_one(
    ctx: &Ctx,
    exe: &Path,
    run: &(&str, Vec<String>, Engine),
    clips: &[(PathBuf, String)],
) -> Value {
    let (name, args, engine) = run;
    if !port_free(ctx) {
        return json!({ "run": name, "error": "the local server port is still busy" });
    }
    let mut b = match Bench::start(exe, args, ctx.spike.voice.line_max_bytes) {
        Ok(b) => b,
        Err(e) => return json!({ "run": name, "error": e }),
    };
    let load = match b.next(ctx.cfg.g22.answer_wait_ms) {
        Some(Event::Ready { load_ms }) => json!(load_ms),
        Some(Event::Note { note: error } | Event::Error { error }) => json!({ "error": error }),
        _ => Value::Null,
    };
    let rows: Vec<Value> = clips.iter().map(|c| one(ctx, &mut b, *engine, c)).collect();
    b.worker
        .quit(ctx.cfg.timing.close_wait_ms, ctx.cfg.timing.poll_ms);
    let errors = rows.iter().filter(|r| r.get("error").is_some()).count();
    let (rtf, wer) = (mean(&rows, "rtf"), mean(&rows, "wer"));
    json!({ "run": name, "load_ms": load, "errors": errors, "mean_rtf": rtf, "mean_wer": wer, "clips": rows })
}

/// Runs the kept clips through each engine set-up for bench `name`; the clips are deleted only when every run worked.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let (g, v) = (&ctx.cfg.g22, &ctx.spike.voice);
    let dir = g.clips_dir();
    let clips = clips(&dir)?;
    if clips.is_empty() {
        return Err(format!("no clips in {}", dir.display()));
    }
    let exe = ctx.cfg.dir.join(&g.worker_dir).join(&v.worker);
    if !exe.is_file() {
        return Err(format!("worker not found: {}", exe.display()));
    }
    let cloud = name == WITH_CLOUD || v.cloud_on;
    let mut runs = vec![
        ("local", vec![], Engine::Local),
        ("local-weak", vec![v.weak_arg.clone()], Engine::Local),
    ];
    if cloud {
        runs.push(("cloud", vec![v.cloud_arg.clone()], Engine::Cloud));
    }
    let results: Vec<Value> = runs.iter().map(|r| run_one(ctx, &exe, r, &clips)).collect();
    let pass = results
        .iter()
        .all(|r| r["errors"] == 0 && r["mean_rtf"].is_number());
    let deleted = clean_up(pass, &dir, &clips);
    Ok(json!({
        "gate": "G22", "face": name, "pass": pass, "clips": clips.len(),
        "cloud": if cloud { "run" } else { CLOUD_SKIPPED }, "runs": results, "clips_deleted": deleted,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh folder in TEMP holding `files`.
    fn folder(name: &str, files: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("creates");
        for f in files {
            std::fs::write(dir.join(f), b"x").expect("writes");
        }
        dir
    }

    #[test]
    fn only_named_clips_are_listed_with_their_language() {
        let dir = folder(
            "kx-bench-list-test",
            &["ar-1.wav", "notes.txt", "nolang.wav"],
        );
        let got = clips(&dir).expect("lists");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].1, "ar");
        std::fs::remove_dir_all(&dir).expect("cleans");
    }

    #[test]
    fn only_the_listed_clips_are_deleted() {
        let dir = folder("kx-bench-delete-test", &["en-1.wav", "keep.txt"]);
        let listed = clips(&dir).expect("lists");
        assert!(
            delete(&dir, &listed).is_err(),
            "the folder still holds another file"
        );
        assert!(dir.join("keep.txt").is_file() && !dir.join("en-1.wav").exists());
        std::fs::remove_dir_all(&dir).expect("cleans");
        let dir = folder("kx-bench-delete-test", &["fr-2.wav"]);
        assert_eq!(delete(&dir, &clips(&dir).expect("lists")), Ok(()));
        assert!(!dir.exists());
    }

    #[test]
    fn clean_up_reports_kept_deleted_or_the_error() {
        let dir = folder("kx-bench-clean-test", &["en-1.wav", "keep.txt"]);
        let listed = clips(&dir).expect("lists");
        assert_eq!(clean_up(false, &dir, &listed), json!(false));
        assert!(
            dir.join("en-1.wav").is_file(),
            "a failed bench keeps the clips"
        );
        assert!(clean_up(true, &dir, &listed)["error"].is_string());
        std::fs::remove_dir_all(&dir).expect("cleans");
        let dir = folder("kx-bench-clean-test", &["fr-2.wav"]);
        let listed = clips(&dir).expect("lists");
        assert_eq!(clean_up(true, &dir, &listed), json!(true));
    }

    #[test]
    fn each_clip_is_scored_against_the_sentence_in_its_language() {
        let g = crate::config::load().expect("kx-gates.toml loads").g22;
        let v = spike_core::config::load().expect("spike.toml loads").voice;
        for s in &g.sentences {
            let lang = v.language(s.layout_id().expect("a layout"));
            let want = Some(s.text.as_str());
            assert_eq!(sentence(&g, &v, lang), want, "{}", s.layout);
        }
        assert_eq!(sentence(&g, &v, "zz"), None, "no sentence is read in zz");
    }

    #[test]
    fn speed_and_means_are_plain_ratios() {
        assert_eq!(rtf(500, 2000), Some(0.25));
        assert_eq!(rtf(5, 0), None);
        let rows = [
            json!({ "rtf": 0.5 }),
            json!({ "rtf": 1.5 }),
            json!({ "error": "x" }),
        ];
        assert_eq!(mean(&rows, "rtf"), Some(1.0));
        assert_eq!(mean(&rows[2..], "rtf"), None);
    }
}

//! The worker's state: one recording at a time, each clip sent to the chosen engine, every event through a sink.

use std::io::Read;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use spike_core::voicecfg::{Engine, VoiceConfig};
use spike_core::voiceproto::{Cmd, Event, clip_name, valid_lang};

use crate::args::Args;
use crate::engines::Engines;
use crate::mic::{Clip, Mic, OnStop, Stop};
use crate::{ms_since, wav};

/// Why a language is refused; it names the kept clip, so it must be a short plain code.
const BAD_LANG: &str = "bad language code";
/// Why a recording gave nothing to send.
const NO_SOUND: &str = "no sound was recorded";
/// Why a saved clip is refused.
const TOO_LONG: &str = "larger than the longest recording";
/// Starts the note sent when a clip cannot be kept.
const KEEP_FAILED: &str = "keeping the clip";

/// Where events go: one JSON line on stdout, or a list in tests.
pub type Sink = Box<dyn Fn(&Event) + Send>;

/// What the main loop acts on.
pub enum Msg {
    /// A command from the face or the bench.
    Cmd(Cmd),
    /// Recording number `.0` stopped by itself.
    Stop(u64, Stop),
    /// Stdin closed: the face is gone.
    End,
}

/// A recording in progress, its number and its language.
struct Rec {
    id: u64,
    mic: Mic,
    lang: String,
}

/// The worker's state.
pub struct Worker {
    cfg: VoiceConfig,
    args: Args,
    engines: Engines,
    rec: Option<Rec>,
    /// Number of the latest recording.
    last_id: u64,
    tx: Sender<Msg>,
    /// Where events go.
    out: Sink,
}

impl Worker {
    /// A worker with nothing recording; `tx` feeds its own loop and `out` takes its events.
    pub fn new(
        cfg: VoiceConfig,
        args: Args,
        engines: Engines,
        tx: Sender<Msg>,
        out: Sink,
    ) -> Worker {
        Worker {
            cfg,
            args,
            engines,
            rec: None,
            last_id: 0,
            tx,
            out,
        }
    }

    /// Acts on one message; false ends the worker.
    pub fn handle(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::Cmd(Cmd::Record { lang }) => self.record(lang),
            Msg::Cmd(Cmd::Stop) => self.stop(),
            Msg::Stop(id, why) => self.stopped(id, why),
            Msg::Cmd(Cmd::File { path, lang, engine }) => self.file(&path, &lang, engine),
            Msg::Cmd(Cmd::Quit) | Msg::End => return false,
        }
        true
    }

    fn record(&mut self, lang: String) {
        if self.rec.is_some() {
            return self.note("already recording");
        }
        if !valid_lang(&lang) {
            return self.fail(BAD_LANG);
        }
        self.last_id += 1;
        let (tx, id) = (self.tx.clone(), self.last_id);
        let on_stop: OnStop = Arc::new(move |s| drop(tx.send(Msg::Stop(id, s))));
        match Mic::start(self.cfg.record_max_s, on_stop) {
            Ok(mic) => {
                self.rec = Some(Rec { id, mic, lang });
                self.emit(&Event::Listening);
            }
            Err(error) => self.fail(error),
        }
    }

    /// Recording `id` filled up or lost its mic; a stop from an older recording is ignored.
    fn stopped(&mut self, id: u64, why: Stop) {
        if !is_current(id, self.rec.as_ref().map(|r| r.id)) {
            return;
        }
        if let Stop::Lost(e) = why {
            self.note(&e);
        }
        self.stop();
    }

    /// Ends the recording and sends its sound to the chosen engine.
    fn stop(&mut self) {
        let Some(Rec { mic, lang, .. }) = self.rec.take() else {
            return;
        };
        self.transcribe(&mic.stop(), &lang);
    }

    /// Sends a finished clip to the chosen engine; an empty clip is an error.
    fn transcribe(&self, clip: &Clip, lang: &str) {
        if clip.is_empty() {
            return self.fail(NO_SOUND);
        }
        let wav = clip.wav(self.cfg.rate_hz);
        let audio_ms = wav::audio_ms(&wav).unwrap_or_default();
        self.emit(&Event::Transcribing { audio_ms });
        self.keep(&wav, lang);
        self.run(&wav, lang, self.cfg.engine, audio_ms);
    }

    /// A saved clip through `engine` (bench only).
    fn file(&self, path: &str, lang: &str, engine: Engine) {
        if !valid_lang(lang) {
            return self.fail(BAD_LANG);
        }
        let max = wav::max_len(self.cfg.record_max_s, self.cfg.rate_hz);
        match read_clip(path, max).and_then(|w| wav::audio_ms(&w).map(|ms| (w, ms))) {
            Ok((wav, ms)) => self.run(&wav, lang, engine, ms),
            Err(error) => self.fail(error),
        }
    }

    fn run(&self, wav: &[u8], lang: &str, engine: Engine, audio_ms: u64) {
        let t = Instant::now();
        match self.engines.pick(engine).and_then(|p| p.text(wav, lang)) {
            Ok(text) => self.emit(&Event::Text {
                text,
                engine,
                ms: ms_since(t),
                audio_ms,
            }),
            Err(error) => self.fail(error),
        }
    }

    /// Saves the clip for the bench when the keep switch was given; a failure is only a note.
    fn keep(&self, wav: &[u8], lang: &str) {
        let Some(dir) = &self.args.keep else { return };
        let file = dir.join(clip_name(lang, now_ms()));
        if let Err(e) = std::fs::create_dir_all(dir).and_then(|_| std::fs::write(file, wav)) {
            self.note(&format!("{KEEP_FAILED}: {e}"));
        }
    }

    /// Sends `e` to the face.
    fn emit(&self, e: &Event) {
        (self.out)(e);
    }

    /// A note: shown by the face, but it neither ends nor starts anything.
    fn note(&self, n: &str) {
        self.emit(&Event::Note { note: n.into() });
    }

    /// An error that ends this cycle.
    fn fail(&self, error: impl Into<String>) {
        self.emit(&Event::Error {
            error: error.into(),
        });
    }
}

/// A stop from recording `id` counts only while `id` is the one recording now.
fn is_current(id: u64, now: Option<u64>) -> bool {
    now == Some(id)
}

/// A clip file, refused when larger than `max` bytes; it is never read past `max + 1`.
fn read_clip(path: &str, max: u64) -> Result<Vec<u8>, String> {
    let why = |e: std::io::Error| format!("{path}: {e}");
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(why)?
        .take(max.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(why)?;
    if bytes.len() as u64 > max {
        return Err(format!("{path}: {TOO_LONG}"));
    }
    Ok(bytes)
}

/// Time since 1970 in ms, which names each kept clip.
fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::mpsc::{self, Receiver};

    /// Why the local engine is missing in these tests.
    const NO_LOCAL: &str = "no local server";

    /// A worker with no engine; its events land in the receiver.
    fn worker(keep: Option<PathBuf>) -> (Worker, Receiver<Event>) {
        let cfg = spike_core::config::load().expect("spike.toml loads").voice;
        let args = Args {
            keep,
            ..Args::default()
        };
        let engines = Engines {
            local: Err(NO_LOCAL.into()),
            cloud: Err("no cloud".into()),
        };
        let (out, events) = mpsc::channel();
        let sink: Sink = Box::new(move |e: &Event| drop(out.send(e.clone())));
        (
            Worker::new(cfg, args, engines, mpsc::channel().0, sink),
            events,
        )
    }

    /// The events sent so far.
    fn seen(rx: &Receiver<Event>) -> Vec<Event> {
        rx.try_iter().collect()
    }

    /// `bytes` in a temp file called `name`.
    fn temp(name: &str, bytes: &[u8]) -> PathBuf {
        let p = std::env::temp_dir().join(name);
        std::fs::write(&p, bytes).expect("writes");
        p
    }

    /// The error event for `e`.
    fn failed(e: &str) -> Event {
        Event::Error { error: e.into() }
    }

    #[test]
    fn a_clip_over_the_cap_is_refused() {
        let p = std::env::temp_dir().join("kx-worker-cap-test.wav");
        std::fs::write(&p, [0u8; 10]).expect("writes");
        let path = p.to_string_lossy();
        assert!(read_clip(&path, 9).is_err());
        assert_eq!(read_clip(&path, 10).map(|b| b.len()), Ok(10));
        std::fs::remove_file(&p).expect("removes");
    }

    #[test]
    fn a_stop_from_an_old_recording_is_ignored() {
        assert!(is_current(3, Some(3)));
        assert!(!is_current(2, Some(3)));
        assert!(!is_current(3, None));
    }

    #[test]
    fn a_saved_clip_in_a_bad_language_is_refused() {
        let (w, rx) = worker(None);
        w.file("clip.wav", "../x", Engine::Local);
        assert_eq!(seen(&rx), [failed(BAD_LANG)]);
    }

    #[test]
    fn a_saved_clip_longer_than_a_recording_is_refused() {
        let (mut w, rx) = worker(None);
        w.cfg.record_max_s = 0;
        let p = temp("kx-worker-long.wav", &wav::encode(&[0; 1], w.cfg.rate_hz));
        let path = p.to_string_lossy().into_owned();
        w.file(&path, "en", Engine::Local);
        std::fs::remove_file(&p).expect("removes");
        assert_eq!(seen(&rx), [failed(&format!("{path}: {TOO_LONG}"))]);
    }

    #[test]
    fn a_saved_clip_goes_to_its_engine_or_says_why_it_cannot() {
        let (w, rx) = worker(None);
        let p = temp("kx-worker-ok.wav", &wav::encode(&[0; 160], w.cfg.rate_hz));
        w.file(&p.to_string_lossy(), "en", Engine::Local);
        std::fs::remove_file(&p).expect("removes");
        assert_eq!(seen(&rx), [failed(NO_LOCAL)]);
    }

    #[test]
    fn a_clip_that_cannot_be_kept_is_only_a_note() {
        let blocker = temp(
            "kx-worker-keep-blocker",
            b"a file where the folder should be",
        );
        let (w, rx) = worker(Some(blocker.clone()));
        w.keep(b"clip", "en");
        std::fs::remove_file(&blocker).expect("removes");
        let got = seen(&rx);
        assert!(
            matches!(got.as_slice(), [Event::Note { note }] if note.starts_with(KEEP_FAILED)),
            "{got:?}"
        );
    }

    #[test]
    fn a_recording_with_no_sound_is_an_error() {
        let (w, rx) = worker(None);
        w.transcribe(&Clip::default(), "en");
        assert_eq!(seen(&rx), [failed(NO_SOUND)]);
    }
}

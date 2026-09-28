//! P1 spike worker: the only process that opens the mic; each clip goes to the local or the opt-in cloud engine.

mod args;
mod cloud;
mod engines;
mod http;
mod local;
mod mic;
mod multipart;
mod resample;
mod wav;
mod worker;

use std::io::Write;
use std::sync::mpsc::{self, Sender};
use std::time::Instant;

use spike_core::lines;
use spike_core::voicecfg::VoiceConfig;
use spike_core::voiceproto::{self, Cmd, Event};
use spike_core::weak::Caps;

use args::Args;
use cloud::Cloud;
use engines::Engines;
use local::Local;
use worker::{Msg, Worker};

fn main() {
    let (cfg, args) = match setup() {
        Ok(s) => s,
        Err(error) => return emit(&Event::Error { error }),
    };
    let engines = start(&cfg, &args);
    let (tx, rx) = mpsc::channel();
    read_stdin(tx.clone(), cfg.line_max_bytes);
    let mut w = Worker::new(cfg, args, engines, tx, Box::new(emit));
    for msg in rx {
        if !w.handle(msg) {
            break;
        }
    }
}

/// The voice settings and the switches.
fn setup() -> Result<(VoiceConfig, Args), String> {
    let cfg = spike_core::config::load()?.voice;
    let args = Args::parse(&cfg, std::env::args().skip(1))?;
    Ok((cfg, args))
}

/// Starts the local server (the model loads once, here) and the cloud engine when allowed.
fn start(cfg: &VoiceConfig, args: &Args) -> Engines {
    let t0 = Instant::now();
    let cpus = std::thread::available_parallelism().map_or(1, usize::from);
    let caps = args.weak.then(|| Caps::from(&cfg.weak, cpus)).transpose();
    let local = caps.and_then(|c| Local::start(&cfg.local, c.as_ref()));
    match &local {
        Ok(_) => emit(&Event::Ready {
            load_ms: ms_since(t0),
        }),
        Err(e) => note(e),
    }
    let cloud = Cloud::from_env(&cfg.cloud, args.cloud);
    Engines { local, cloud }
}

/// Reads commands on a thread; a bad or overlong line is reported and skipped.
fn read_stdin(tx: Sender<Msg>, max_line: usize) {
    std::thread::spawn(move || {
        let mut r = std::io::stdin().lock();
        while let Some(line) = lines::next_line(&mut r, max_line) {
            let cmd = match line {
                Ok(l) if l.is_empty() => continue,
                Ok(l) => voiceproto::parse::<Cmd>(&l),
                Err(e) => Err(e),
            };
            match cmd {
                Ok(c) => {
                    if tx.send(Msg::Cmd(c)).is_err() {
                        return;
                    }
                }
                Err(e) => note(&format!("bad command: {e}")),
            }
        }
        let _ = tx.send(Msg::End);
    });
}

/// One event as one line on stdout; the worker's sink in production.
fn emit(e: &Event) {
    if let Ok(line) = voiceproto::to_line(e) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{line}").and_then(|_| out.flush());
    }
}

/// A note: shown by the face, but it neither ends nor starts anything.
fn note(n: &str) {
    emit(&Event::Note { note: n.into() });
}

/// Milliseconds since `t`.
fn ms_since(t: Instant) -> u64 {
    u64::try_from(t.elapsed().as_millis()).unwrap_or(u64::MAX)
}

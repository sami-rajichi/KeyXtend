//! The voice worker as the face sees it: started next to the face, fed JSON commands, read on its own thread.

use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::config::SpikeConfig;
use crate::lines;
use crate::voicecfg::VoiceConfig;
use crate::voiceproto::{Cmd, Event, parse, to_line};

/// Shown when the worker's output ends or it cannot take a command.
pub const WORKER_GONE: &str = "the voice worker stopped";

/// Worker switches: cloud when settings turn it on, and the keep folder when the face was given one.
pub fn worker_args(v: &VoiceConfig, face_args: &[String]) -> Vec<String> {
    let mut out: Vec<String> = v
        .cloud_on
        .then(|| v.cloud_arg.clone())
        .into_iter()
        .collect();
    if let Some(i) = face_args.iter().position(|a| *a == v.keep_arg)
        && let Some(dir) = face_args.get(i + 1)
    {
        out.extend([v.keep_arg.clone(), dir.clone()]);
    }
    out
}

/// The running worker.
pub struct Worker {
    child: Child,
    stdin: ChildStdin,
}

impl Worker {
    /// Starts the worker next to this exe; each event goes to `on_event` on a reader thread.
    pub fn start(
        cfg: &SpikeConfig,
        face_args: &[String],
        on_event: impl FnMut(Event) + Send + 'static,
    ) -> Result<Worker, String> {
        let exe = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .with_file_name(&cfg.voice.worker);
        let args = worker_args(&cfg.voice, face_args);
        Worker::spawn(&exe, &args, cfg.voice.line_max_bytes, on_event)
    }

    /// Starts the worker `exe` with `args`; each event line, up to `max_line` bytes, goes to `on_event` on a reader thread.
    pub fn spawn(
        exe: &Path,
        args: &[String],
        max_line: usize,
        mut on_event: impl FnMut(Event) + Send + 'static,
    ) -> Result<Worker, String> {
        let mut child = Command::new(exe)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW.0)
            .spawn()
            .map_err(|e| format!("cannot start {}: {e}", exe.display()))?;
        let (Some(stdin), Some(out)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(WORKER_GONE.to_string());
        };
        std::thread::spawn(move || read_events(BufReader::new(out), max_line, &mut on_event));
        Ok(Worker { child, stdin })
    }

    /// Asks the worker to quit and waits up to `wait_ms` for it; then it is ended.
    pub fn quit(mut self, wait_ms: u64, poll_ms: u64) {
        let _ = self.send(&Cmd::Quit);
        let t0 = Instant::now();
        while t0.elapsed() < Duration::from_millis(wait_ms) {
            if let Ok(Some(_)) = self.child.try_wait() {
                return;
            }
            std::thread::sleep(Duration::from_millis(poll_ms));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// Sends one command.
    pub fn send(&mut self, c: &Cmd) -> Result<(), String> {
        let line = to_line(c)?;
        writeln!(self.stdin, "{line}")
            .and_then(|_| self.stdin.flush())
            .map_err(|e| format!("{WORKER_GONE}: {e}"))
    }
}

/// Hands each event line to `on_event`; a bad or overlong line becomes a note, and the end says the worker is gone.
fn read_events(mut r: impl BufRead, max_line: usize, on_event: &mut impl FnMut(Event)) {
    while let Some(line) = lines::next_line(&mut r, max_line) {
        let event = match line {
            Ok(l) if l.is_empty() => continue,
            Ok(l) => parse(&l).unwrap_or_else(|note| Event::Note { note }),
            Err(note) => Event::Note { note },
        };
        on_event(event);
    }
    on_event(Event::Error {
        error: WORKER_GONE.into(),
    });
}

impl Drop for Worker {
    /// Asks the worker to quit; it closes its local server with it.
    fn drop(&mut self) {
        let _ = self.send(&Cmd::Quit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_switches_follow_the_settings_and_the_face() {
        let mut v = crate::config::load().expect("spike.toml loads").voice;
        assert!(worker_args(&v, &[]).is_empty(), "cloud stays off");
        v.cloud_on = true;
        assert_eq!(worker_args(&v, &[]), vec![v.cloud_arg.clone()]);
        v.cloud_on = false;
        let face = [v.keep_arg.clone(), "D:/clips".into()];
        assert_eq!(worker_args(&v, &face), face.to_vec());
        let alone = worker_args(&v, &face[..1]);
        assert!(alone.is_empty(), "keep without a folder is dropped");
    }

    #[test]
    fn bad_and_long_lines_become_notes_and_the_end_says_gone() {
        let good = to_line(&Event::Listening).expect("encodes");
        let input = format!("{good}\n\nnot json\n{}\n", "x".repeat(64));
        let mut seen = Vec::new();
        read_events(std::io::Cursor::new(input), 32, &mut |e| seen.push(e));
        assert_eq!(seen.len(), 4, "{seen:?}");
        assert_eq!(seen[0], Event::Listening);
        assert!(matches!(seen[1], Event::Note { .. }), "bad JSON: {seen:?}");
        assert!(matches!(seen[2], Event::Note { .. }), "too long: {seen:?}");
        let gone = Event::Error {
            error: WORKER_GONE.into(),
        };
        assert_eq!(seen[3], gone);
    }

    #[test]
    fn a_missing_worker_is_named() {
        let mut cfg = crate::config::load().expect("spike.toml loads");
        cfg.voice.worker = "kx-no-such-worker.exe".into();
        let r = Worker::start(&cfg, &[], |_| {});
        assert!(r.is_err_and(|e| e.contains(&cfg.voice.worker)));
    }
}

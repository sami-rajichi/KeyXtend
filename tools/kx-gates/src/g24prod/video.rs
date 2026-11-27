//! G24's recordings: target-window for `[g24] record_s`, whose copied MP4 must last that long within `slack_ms`, and
//! the whole screen for `long_s`, copied at `speed` from the strip while the worker stays under `[snip] record_mb`.
#![cfg(windows)]

use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use kx_module_api::MS_PER_S;
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;

use super::clock::Read;
use super::judge::within;
use super::scene::Scene;
use super::shots::{corners, covered};
use super::snip::Snip;
use super::watch::{Watch, Watched};
use super::{clip, copyset, mp4, wins};
use crate::budget::Part;
use crate::proc::{MB, Process};
use crate::win::{self, sleep_ms};
use crate::workertok;

/// Bytes in a MiB, as the worker's job counts its cap.
const MIB: f64 = 1_048_576.0;

/// Waits until the record bar holding Stop at `stop` reads `secs` or a little later on its clock, which holds at 0:00
/// until the first recorded frame: up to `secs` and `[g24] wait_ms` more; an error says whether it was ever read.
fn counted(s: &Snip, stop: POINT, secs: u64) -> Result<(), String> {
    let ms = u64::from(MS_PER_S);
    let late = secs.saturating_add(s.g.wait_ms / ms);
    let (bar, most) = (
        win::root_at(stop),
        secs.saturating_mul(ms).saturating_add(s.g.wait_ms),
    );
    let mut best = Read::Never;
    let reached = win::poll_until(most, s.poll(), || {
        best = best.max(s.clock.read(&s.names(bar), secs, late));
        (best == Read::Reached).then_some(())
    });
    reached.ok_or_else(|| match best {
        Read::Never => format!("the record bar's clock was never read within {most} ms"),
        _ => format!("the record bar's clock did not reach {secs} s within {most} ms: the worker recorded late"),
    })
}

/// Snip, way `way`, clicks at `spots`, Record, then Stop at its kept spot once the clock reads `secs`, until the
/// video's strip shows Play; the watch runs throughout.
fn record(
    s: &Snip,
    sc: &Scene,
    (way, spots): (&str, &[POINT]),
    secs: u64,
) -> Result<Watched, String> {
    let ids = &s.g.ids;
    let watch = Watch::start(s);
    let done = (|| {
        covered(s, &sc.frame)?;
        s.click(way)?;
        spots.iter().try_for_each(|&p| s.on_frozen(p))?;
        s.click(&ids.record)?;
        let stop = s.find(&ids.stop)?;
        counted(s, stop, secs)?;
        s.click_at(stop)?;
        s.until(|| s.has(&ids.play).then_some(()))
            .ok_or("the video's strip did not come back")?;
        Ok::<(), String>(())
    })();
    let watched = watch.stop();
    done.map(|()| watched)
}

/// A recording of target-window for `[g24] warm_s` on the clock, closed unsaved, so the countdown, record bar, edge,
/// strip and question have each shown before idle memory is read.
pub fn warm(s: &Snip, sc: &Scene) -> Result<Value, String> {
    let g = s.g;
    let frame = wins::frame(s.app.hwnd)?;
    let watched = record(s, sc, (&g.ids.video_region, &corners(&frame)), g.warm_s)?;
    s.click(&g.ids.close)?;
    s.wait_for(&s.titles.ask, true, "question")?;
    s.click(&g.ids.drop)?;
    s.ended()?;
    let ok = watched.ok(s.way);
    Ok(json!({ "asked": true, "watch": watched, "ok": ok }))
}

/// The length of the MP4 at `file`, in ms.
fn length(file: &Path) -> Result<u64, String> {
    let mut f = File::open(file).map_err(|e| format!("the video: {}", e.kind()))?;
    mp4::length_ms(&mut f)
}

/// The strip's Close after a Copy kept the video, which ends the snip unasked. A Close that lands before the Copy told
/// its end is ignored, so it clicks again, up to `[g24] close_tries` times.
fn close_kept(s: &Snip) -> Result<(), String> {
    let (g, close) = (s.g, &s.g.ids.close);
    let gone = || win::poll_until(g.close_ms, s.poll(), || (!s.has(close)).then_some(())).is_some();
    close_until(g.close_tries, || s.click(close), || s.has(close), gone)?;
    s.ended()
}

/// Clicks up to `tries` times until `gone` says the strip went: the clicks it took, or an error naming the tries. A
/// retry that finds no Close (`shows`) takes the strip as gone, as it went just after the last wait.
fn close_until(
    tries: usize,
    mut click: impl FnMut() -> Result<(), String>,
    mut shows: impl FnMut() -> bool,
    mut gone: impl FnMut() -> bool,
) -> Result<usize, String> {
    for n in 1..=tries {
        if n > 1 && !shows() {
            return Ok(n - 1);
        }
        click()?;
        if gone() {
            return Ok(n);
        }
    }
    Err(format!("the strip stayed after {tries} clicks on Close"))
}

/// A recording of target-window's frame, copied from the strip: its length against `[g24] record_s`.
pub fn short(s: &Snip, sc: &Scene, data: &Path) -> Result<Value, String> {
    let g = s.g;
    let frame = wins::frame(s.app.hwnd)?;
    let want = g.record_s * u64::from(MS_PER_S);
    let watched = record(s, sc, (&g.ids.video_region, &corners(&frame)), g.record_s)?;
    let since = clip::mark();
    s.click(&g.ids.copy)?;
    let got = length(&clip::video(s.ctx, (since, g.wait_ms), data)?)?;
    close_kept(s)?;
    let ok = within(got, want, g.slack_ms) && watched.ok(s.way);
    Ok(
        json!({ "length_ms": got, "want_ms": want, "slack_ms": g.slack_ms, "watch": watched, "ok": ok }),
    )
}

/// The most private commit any worker beside the test build held, sampled every `poll_ms` until stopped, in MB.
struct Peak {
    stop: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<Option<f64>>,
}

impl Peak {
    /// Starts sampling the workers in folder `dir`.
    fn start(dir: &Path, poll_ms: u64) -> Self {
        let (stop, dir) = (Arc::new(AtomicBool::new(false)), dir.to_path_buf());
        let flag = Arc::clone(&stop);
        let thread = std::thread::spawn(move || {
            let mut peak = None::<f64>;
            while !flag.load(Ordering::Relaxed) {
                let found = workertok::processes();
                for pid in workertok::beside(&found, &dir) {
                    let mb = Process::to_measure(pid)
                        .and_then(|p| p.memory())
                        .map(|m| m.commit_mb);
                    peak = mb.ok().map_or(peak, |v| Some(peak.map_or(v, |p| p.max(v))));
                }
                sleep_ms(poll_ms);
            }
            peak
        });
        Self { stop, thread }
    }

    /// Stops sampling: the peak, none when no worker was seen.
    fn stop(self) -> Option<f64> {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.join().ok().flatten()
    }
}

/// The whole screen for `[g24] long_s`, then the strip's Speed up to `speed` and its Copy: the sped video's length, the
/// copy's wait against `speed_wait_ms`, and the workers' peak memory against the job's cap.
pub fn sped(s: &Snip, sc: &Scene, data: &Path, workers: &Path) -> Result<Value, String> {
    let (g, long_ms) = (s.g, s.g.long_s * u64::from(MS_PER_S));
    let watched = record(
        s,
        sc,
        (&g.ids.video_full, &[wins::centre(&sc.frame)]),
        g.long_s,
    )?;
    (0..g.speed_steps()).try_for_each(|_| s.click(&g.ids.speed))?;
    let (since, started) = (clip::mark(), Instant::now());
    let peak = Peak::start(workers, s.poll());
    let copied = s
        .click(&g.ids.copy)
        .and_then(|()| clip::video(s.ctx, (since, g.speed_wait_ms), data));
    let copy_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let peak = peak.stop();
    let got = length(&copied?)?;
    close_kept(s)?;
    let want = long_ms / u64::from(g.speed);
    let record_mb = copyset::snip(s.way)?.record_mb;
    let cap = Part::at_most("worker_commit_mb", peak, f64::from(record_mb) * MIB / MB);
    let ok = within(got, want, g.slack_ms) && cap.pass && watched.ok(s.way);
    Ok(json!({
        "length_ms": got, "want_ms": want, "slack_ms": g.slack_ms, "speed": g.speed, "memory": cap,
        "copy_ms": copy_ms, "wait_ms": g.speed_wait_ms, "watch": watched, "ok": ok,
    }))
}

#[cfg(test)]
mod tests;

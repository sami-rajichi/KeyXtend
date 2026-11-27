//! The microphone and the PC's sound on one timeline: each source's packets go into its lane where they were heard,
//! and the lanes are summed into 16-bit stereo, written once, in order. A source switched off is cut at that moment
//! but heard one cushion longer, so sound already captured still lands and nothing after the cut does.

use std::time::Instant;

use super::lane::Lane;
use super::{CHANNELS, Ear, RATE, Switch, queue_frames};
use crate::table::Table;

/// Milliseconds and nanoseconds in a second.
const MS_S: i64 = 1_000;
const NS_S: u128 = 1_000_000_000;

/// One source: its stream while open, the cut it is closing at, and its lane.
struct Source {
    which: Switch,
    ear: Option<Box<dyn Ear>>,
    closing: Option<i64>,
    lane: Lane,
}

/// Both sources of a run.
pub(super) struct Mixer {
    sources: [Source; 2],
    /// The next sound frame to write.
    written: i64,
    /// How far writing stays behind now, in frames.
    cushion: i64,
    /// The most frames one mix gives.
    most: i64,
    /// The timeline's start, once the first frame came; until then packets wait in their streams.
    t0: Option<Instant>,
}

/// `ms` in sound frames.
pub(super) fn frames(ms: u32) -> i64 {
    i64::from(ms) * i64::from(RATE) / MS_S
}

/// The place of `at` on the timeline that starts at `t0`, in sound frames; before it, below 0.
pub(super) fn place(at: Instant, t0: Instant) -> i64 {
    let ns = |d: std::time::Duration| {
        i64::try_from(d.as_nanos() * u128::from(RATE) / NS_S).unwrap_or(i64::MAX)
    };
    match at.checked_duration_since(t0) {
        Some(after) => ns(after),
        None => -ns(t0 - at),
    }
}

impl Mixer {
    /// Both sources silent, with `table`'s drift, queue and cushion; one mix gives a queue at most.
    pub(super) fn new(table: &Table) -> Self {
        let (s, queue) = (&table.sound, queue_frames(table));
        let source = |which| Source {
            which,
            ear: None,
            closing: None,
            lane: Lane::new(frames(s.drift_ms), queue),
        };
        Self {
            sources: [source(Switch::Mic), source(Switch::Sound)],
            written: 0,
            cushion: frames(s.cushion_ms),
            most: i64::try_from(queue).unwrap_or(i64::MAX),
            t0: None,
        }
    }

    /// Starts the timeline at `t0`.
    pub(super) fn begin(&mut self, t0: Instant) {
        self.t0 = Some(t0);
    }

    /// How far writing stays behind now, in frames.
    pub(super) fn cushion(&self) -> i64 {
        self.cushion
    }

    fn source(&mut self, which: Switch) -> &mut Source {
        let n = usize::from(which == Switch::Sound);
        &mut self.sources[n]
    }

    /// Whether `which` is switched on.
    pub(super) fn on(&self, which: Switch) -> bool {
        let s = &self.sources[usize::from(which == Switch::Sound)];
        s.ear.is_some() && s.closing.is_none()
    }

    /// Switches `which` on with `ear`; the old stream's last sound, if any, lands first.
    pub(super) fn open(&mut self, which: Switch, ear: Box<dyn Ear>) {
        let t0 = self.t0;
        let s = self.source(which);
        s.pull(t0);
        (s.ear, s.closing) = (Some(ear), None);
        s.lane.resume();
    }

    /// Switches `which` off from place `at`.
    pub(super) fn close(&mut self, which: Switch, at: i64) {
        let s = self.source(which);
        s.lane.cut(at);
        s.closing = Some(at);
    }

    /// The sources whose stream stopped for good while switched on.
    pub(super) fn lost(&self) -> Vec<Switch> {
        let failed =
            |s: &&Source| s.closing.is_none() && s.ear.as_ref().is_some_and(|e| e.failed());
        self.sources
            .iter()
            .filter(failed)
            .map(|s| s.which)
            .collect()
    }

    /// Puts what each source heard into its lane, and lets a closing stream go once its cut is a cushion behind `now`.
    pub(super) fn pull(&mut self, now: i64) {
        let (cushion, t0) = (self.cushion, self.t0);
        for s in &mut self.sources {
            s.pull(t0);
            if s.closing.is_some_and(|c| now - cushion >= c) {
                (s.ear, s.closing) = (None, None);
            }
        }
    }

    /// The 16-bit stereo sound from the last write toward place `upto`, a queue at most, and where it starts; `None`
    /// once written up to it.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "A clamped sample times i16::MAX fits an i16."
    )]
    pub(super) fn mix(&mut self, upto: i64) -> Option<(u64, Vec<i16>)> {
        let upto = upto.min(self.written.saturating_add(self.most));
        let count = usize::try_from(upto - self.written)
            .ok()
            .filter(|n| *n > 0)?;
        let from = self.written;
        let mut sum = vec![0.0_f32; count * usize::from(CHANNELS)];
        for s in &mut self.sources {
            for (o, v) in sum.iter_mut().zip(s.lane.take(from, count)) {
                *o += v;
            }
        }
        self.written = upto;
        let full = f32::from(i16::MAX);
        let pcm = sum
            .iter()
            .map(|s| (s.clamp(-1.0, 1.0) * full).round() as i16);
        Some((u64::try_from(from).ok()?, pcm.collect()))
    }
}

impl Source {
    /// Puts what the stream heard into the lane.
    fn pull(&mut self, t0: Option<Instant>) {
        let (Some(ear), Some(t0)) = (self.ear.as_mut(), t0) else {
            return;
        };
        for packet in ear.heard() {
            self.lane.push(place(packet.at, t0), &packet.samples);
        }
    }
}

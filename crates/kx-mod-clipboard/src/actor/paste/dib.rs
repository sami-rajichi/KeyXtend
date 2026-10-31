//! The DIB an inserted PNG goes with, so apps that read only `CF_DIB` get the picture: the media worker makes it within
//! `media_ms` and `max_entry_mb`, and the step writes both. A worker down, late or refusing, or a DIB past the cap,
//! leaves the PNG alone with one line logged; an entry that has a DIB goes as it is.

use std::fmt::Display;
use std::mem;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipPart, DibAsk, ImageCodec, PictureError};

use super::super::Actor;
use super::Paste;
use super::step::{Ready, Run};
use crate::media::{DibJob, Made, Task};

impl Actor {
    /// Asks the media thread for the DIB of `ready`'s PNG, which waits in `run` meanwhile; a stopped thread leaves the
    /// PNG alone.
    pub(super) fn make_dib(&mut self, mut run: Run, ready: Ready) {
        let Some(png) = png_of(&ready.parts) else {
            return self.write_ready(run, ready);
        };
        self.paste.dib = self.paste.dib.wrapping_add(1);
        let n = self.paste.dib;
        let max_bytes = self.limits.max_entry_bytes;
        let ask = DibAsk { png, max_bytes };
        if self.ports.media.send(Task::Dib(DibJob { n, ask })).is_err() {
            alone(&PictureError::Gone);
            return self.write_ready(run, ready);
        }
        run.ready = Some(ready);
        let since = self.ports.clock.now();
        self.paste.state = Paste::Making { run, n, since };
    }

    /// Takes the DIB of ask `n`: one within the cap goes right after its PNG, else the PNG goes alone; a late or stray
    /// answer is dropped.
    pub(in crate::actor) fn made(&mut self, made: Made) {
        let waits = matches!(self.paste.state, Paste::Making { n, .. } if n == made.n);
        if !waits {
            return;
        }
        let Paste::Making { mut run, .. } = mem::take(&mut self.paste.state) else {
            return;
        };
        let Some(mut ready) = run.ready.take() else {
            return self.step(run);
        };
        let cap = self.limits.max_entry_bytes;
        match made.dib {
            Ok(dib) if u64::try_from(dib.expose().len()).is_ok_and(|n| n <= cap) => {
                add_dib(&mut ready.parts, dib);
            }
            Ok(_) => alone(&PictureError::TooBig),
            Err(error) => alone(&error),
        }
        self.write_ready(run, ready);
    }

    /// No DIB came within `media_ms`: the PNG goes alone.
    pub(super) fn dib_late(&mut self, mut run: Run) {
        alone(&PictureError::Timeout);
        match run.ready.take() {
            Some(ready) => self.write_ready(run, ready),
            None => self.step(run),
        }
    }

    /// Writes the entry `ready` holds.
    fn write_ready(&mut self, run: Run, ready: Ready) {
        self.write(run, &ready.parts, Some(ready.step));
    }
}

/// True when `parts` hold a PNG and no DIB, so the insert asks for one.
pub(super) fn wants_dib(parts: &[ClipPart]) -> bool {
    let has = |want| parts.iter().any(|p| codec_of(p) == Some(want));
    has(ImageCodec::Png) && !has(ImageCodec::Dib)
}

/// The image codec of `part`, if it is an image.
fn codec_of(part: &ClipPart) -> Option<ImageCodec> {
    match part {
        ClipPart::Image { codec, .. } => Some(*codec),
        _ => None,
    }
}

/// A copy of the PNG's bytes in `parts`, for the worker; the parts keep theirs for the write.
fn png_of(parts: &[ClipPart]) -> Option<Redacted<Vec<u8>>> {
    parts.iter().find_map(|p| match p {
        ClipPart::Image {
            codec: ImageCodec::Png,
            bytes,
        } => Some(bytes.clone()),
        _ => None,
    })
}

/// Puts `dib` right after the PNG in `parts`, so the writer sets both.
fn add_dib(parts: &mut Vec<ClipPart>, dib: Redacted<Vec<u8>>) {
    let png = parts
        .iter()
        .position(|p| codec_of(p) == Some(ImageCodec::Png));
    let at = png.map_or(parts.len(), |i| i + 1);
    let codec = ImageCodec::Dib;
    parts.insert(at, ClipPart::Image { codec, bytes: dib });
}

/// Logs why an inserted image went as PNG alone, never its bytes.
fn alone(why: &dyn Display) {
    tracing::warn!(%why, "an inserted image went as PNG alone");
}

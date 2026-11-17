//! A listening run: the source's chunks become 16 kHz mono samples, recorded up to `record_max_s`, measured into
//! levels at `level_hz` and cut at pauses into pieces for the engine's writer, or packed for a listen-only run's Stop.
//! It stops itself at the limit, or after `silence_stop_s` of silence once speech was heard. Again cuts a held
//! recording the same way, a stretch at a time.

use super::Notes;
use super::cut::{Cut, Cutter, Piece};
use super::engine::Writer;
use super::hand::PIECE_SAMPLES;
use super::job::Fail;
use super::level::Meter;
use super::mic::Opened;
use super::pack::Packer;
use super::resample::{Mixer, Resampler, pcm16};
use super::span;
use super::wipe::Buf;
use kx_ipc::{SOUND_HZ, Setup, Stage, VoiceFault, VoiceNote};
use std::any::Any;

/// What an ended run leaves: its recording, its writer when the engine still writes, and a listen-only run's speech.
pub type Ended = (Buf<i16>, Option<Box<dyn Writer>>, Option<Packer>);

/// The held sound Again cuts in one turn of the job's loop: the detector judges it in tens of milliseconds.
const REPLAY_S: u16 = 5;

/// One run while its source is open.
pub struct Run {
    /// The run's id, which its notes repeat.
    pub id: u32,
    /// The opening its chunks belong to.
    pub epoch: u64,
    live: Option<Box<dyn Any>>,
    mix: Mixer,
    rate: Resampler,
    meter: Meter,
    cut: Option<Cutter>,
    writer: Option<Box<dyn Writer>>,
    /// A listen-only run's speech, packed within `piece_s` and one frame for Stop.
    pack: Option<Packer>,
    /// The recording, at most `cap` samples.
    sound: Buf<i16>,
    cap: usize,
    /// The silence after speech that stops the run, in samples.
    quiet: Option<usize>,
    pad: usize,
}

impl Run {
    /// Run `id` over opening `epoch` of the source, as `setup` says, cut by `cut`, its pieces going to `writer`.
    ///
    /// # Errors
    /// `Internal` for a source with no rate or no channel.
    pub fn new(
        id: u32,
        epoch: u64,
        opened: Opened,
        setup: &Setup<'_>,
        cut: Option<Cutter>,
        writer: Option<Box<dyn Writer>>,
    ) -> Result<Self, VoiceFault> {
        let format = opened.format;
        if format.rate == 0 || format.channels == 0 {
            return Err(VoiceFault::Internal);
        }
        let cap = span::secs(setup.record_max_s);
        let most = span::secs(setup.piece_s.into()).min(PIECE_SAMPLES);
        Ok(Self {
            id,
            epoch,
            live: Some(opened.live),
            mix: Mixer::new(format.channels),
            rate: Resampler::new(format.rate, SOUND_HZ),
            meter: Meter::new(setup.level_hz),
            cut,
            pack: writer.is_none().then(|| Packer::new(most)),
            writer,
            sound: Buf::with_limit(cap),
            cap,
            quiet: setup.silence_stop_s.map(|s| span::secs(s.into())),
            pad: span::millis(setup.pad_ms),
        })
    }

    /// Takes one chunk of interleaved samples; `Limit` or `Silence` when the run stops itself after it.
    ///
    /// # Errors
    /// `Out` when the pipe fails, `Memory` when the recording cannot grow.
    pub fn take(&mut self, chunk: &[f32], out: &mut dyn Notes) -> Result<Option<Stage>, Fail> {
        let mono = self.mix.take(chunk)?;
        let even = self.rate.push(&mono)?;
        self.feed(&even, out)?;
        Ok(self.halt())
    }

    /// True while its pieces go to a writer.
    pub fn writes(&self) -> bool {
        self.writer.is_some()
    }

    /// Drops the writer and the packed speech, so no more text or speech comes: a cancel or a fault.
    pub fn mute(&mut self) {
        (self.writer, self.pack) = (None, None);
    }

    /// Closes the source and takes the last samples and the last piece; gives back the recording, the writer and the
    /// packed speech, which is the whole recording when no detector cut it.
    ///
    /// # Errors
    /// As `take`.
    pub fn end(mut self, out: &mut dyn Notes) -> Result<Ended, Fail> {
        self.live = None;
        let tail = self.rate.flush()?;
        self.feed(&tail, out)?;
        let mut cuts = Vec::new();
        if let Some(cut) = self.cut.as_mut() {
            cut.finish(&mut cuts);
        }
        self.act(cuts, out)?;
        if let (None, Some(pack)) = (&self.cut, self.pack.as_mut()) {
            pack.add(&self.sound, 0..self.sound.len());
        }
        let sound = std::mem::take(&mut self.sound);
        Ok((sound, self.writer.take(), self.pack.take()))
    }

    /// Records 16 kHz samples up to the cap, tells their levels and acts on their cuts.
    fn feed(&mut self, even: &[f32], out: &mut dyn Notes) -> Result<(), Fail> {
        let pcm = pcm16(even)?;
        let room = self.cap.saturating_sub(self.sound.len());
        let pcm = &pcm[..pcm.len().min(room)];
        self.sound.extend(pcm)?;
        let mut levels = Vec::new();
        self.meter.take(pcm, &mut levels);
        for level in levels {
            out.tell(VoiceNote::Level { id: self.id, level })?;
        }
        let mut cuts = Vec::new();
        if let Some(cut) = self.cut.as_mut() {
            cut.take(pcm, &mut cuts)?;
        }
        self.act(cuts, out)
    }

    /// Tells speech heard or lost, and gives each piece to the writer.
    fn act(&mut self, cuts: Vec<Cut>, out: &mut dyn Notes) -> Result<(), Fail> {
        for cut in cuts {
            match cut {
                Cut::Heard(speech) => out.tell(VoiceNote::Heard {
                    id: self.id,
                    speech,
                })?,
                Cut::Piece(piece) => self.give(&piece)?,
            }
        }
        Ok(())
    }

    /// Gives the writer a copy of `piece` trimmed to its speech and pad, or packs that stretch for a listen-only run; a
    /// silent piece is skipped.
    fn give(&mut self, piece: &Piece) -> Result<(), VoiceFault> {
        if let Some(writer) = self.writer.as_mut() {
            return give(writer.as_mut(), &self.sound, piece, self.pad);
        }
        if let (Some(pack), Some(span)) = (self.pack.as_mut(), piece.trim(self.pad)) {
            pack.add(&self.sound, span);
        }
        Ok(())
    }

    /// Why the run stops itself now, if it does.
    fn halt(&self) -> Option<Stage> {
        if self.sound.len() >= self.cap {
            return Some(Stage::Limit);
        }
        let quiet = self.cut.as_ref().and_then(Cutter::quiet)?;
        (quiet >= self.quiet?).then_some(Stage::Silence)
    }
}

/// `sound` copied into a fresh wiped buffer.
///
/// # Errors
/// `Memory` when the buffer cannot be had.
pub fn copy(sound: &[i16]) -> Result<Buf<i16>, VoiceFault> {
    let mut buf = Buf::new();
    buf.extend(sound)?;
    Ok(buf)
}

/// Gives `writer` a copy of `piece` of `sound` trimmed to its speech and `pad`; a silent piece is skipped.
fn give(
    writer: &mut dyn Writer,
    sound: &[i16],
    piece: &Piece,
    pad: usize,
) -> Result<(), VoiceFault> {
    if let Some(span) = piece.trim(pad) {
        writer.piece(copy(&sound[span])?);
    }
    Ok(())
}

/// Again cutting a held recording, one stretch a turn of the job's loop, so an ask waits for one stretch at most.
pub struct Replay {
    cut: Cutter,
    pad: usize,
    /// The samples cut so far.
    at: usize,
}

impl Replay {
    /// Cuts with `cut`, giving the engine each piece's speech with `pad` samples around it.
    pub fn new(cut: Cutter, pad: usize) -> Self {
        Self { cut, pad, at: 0 }
    }

    /// Cuts the next `REPLAY_S` of `sound`, giving `writer` each piece with speech, and ends it after the last; true
    /// once all is given.
    ///
    /// # Errors
    /// `Memory` when a buffer cannot be had.
    pub fn step(&mut self, sound: &[i16], writer: &mut dyn Writer) -> Result<bool, VoiceFault> {
        let from = self.at.min(sound.len());
        self.at = from.saturating_add(span::secs(REPLAY_S)).min(sound.len());
        let mut cuts = Vec::new();
        self.cut.take(&sound[from..self.at], &mut cuts)?;
        let done = self.at == sound.len();
        if done {
            self.cut.finish(&mut cuts);
        }
        for cut in cuts {
            if let Cut::Piece(piece) = cut {
                give(writer, sound, &piece, self.pad)?;
            }
        }
        if done {
            writer.end(span::heard_ms(sound.len()));
        }
        Ok(done)
    }
}

//! A fake video to re-time and a fake copy: the video gives its frames by number at their slots, each followed by its
//! share of sound, and the copy logs what it kept, on disk from its opening.

use std::fs;
use std::path::{Path, PathBuf};

use super::FakeStudio;
use super::film::ENDED;
use crate::record::{CHANNELS, Piece, RecordFault, Reel, Shape, Sink};

/// The video a fake copy reads.
#[derive(Clone, Debug)]
pub struct Tape {
    /// Its shape.
    pub shape: Shape,
    /// Its frames.
    pub frames: u64,
    /// The slots from one frame to the next: 1, or more for a run that fell behind.
    pub step: u64,
    /// Its stereo sound frames, spread over the frames.
    pub sound: u64,
    /// The sample its sound holds throughout.
    pub level: i16,
    /// The length it tells.
    pub length_ms: u32,
}

/// Reading the tape: the next frame and the sound already given.
struct FakeReel {
    tape: Tape,
    next: u64,
    heard: u64,
    sound_due: bool,
}

/// Opens the tape, as the script holds it, when `from` is a file.
pub(super) fn open(studio: &FakeStudio, from: &Path) -> Result<Box<dyn Reel<u64>>, RecordFault> {
    if !from.is_file() {
        return Err(RecordFault::Failed);
    }
    let tape = studio.world().script.tape.clone();
    Ok(Box::new(FakeReel {
        tape,
        next: 0,
        heard: 0,
        sound_due: false,
    }))
}

impl Reel<u64> for FakeReel {
    fn shape(&self) -> Shape {
        self.tape.shape
    }

    fn length_ms(&self) -> u32 {
        self.tape.length_ms
    }

    fn next(&mut self) -> Result<Option<Piece<u64>>, RecordFault> {
        if self.sound_due {
            self.sound_due = false;
            let upto = self.next * self.tape.sound / self.tape.frames.max(1);
            let count = usize::try_from(upto - self.heard).unwrap_or(0) * usize::from(CHANNELS);
            self.heard = upto;
            return Ok(Some(Piece::Sound(vec![self.tape.level; count])));
        }
        if self.next == self.tape.frames {
            return Ok(None);
        }
        let frame = self.next;
        self.next += 1;
        self.sound_due = true;
        Ok(Some(Piece::Frame(frame * self.tape.step, frame)))
    }
}

/// The fake copy.
struct FakeCut {
    studio: FakeStudio,
    file: PathBuf,
}

/// Opens a fake copy at `to`, made on disk, as the script's film answers.
pub(super) fn cut(
    studio: &FakeStudio,
    to: &Path,
    _: Shape,
) -> Result<Box<dyn Sink<u64>>, RecordFault> {
    studio.world().script.film?;
    fs::write(to, []).map_err(|_| RecordFault::Failed)?;
    Ok(Box::new(FakeCut {
        studio: studio.clone(),
        file: to.to_path_buf(),
    }))
}

impl Sink<u64> for FakeCut {
    fn frame(&mut self, slot: u64, frame: &u64) -> Result<(), RecordFault> {
        self.studio.world().log.kept.push((slot, *frame));
        Ok(())
    }

    fn sound(&mut self, first: u64, pcm: &[i16]) -> Result<(), RecordFault> {
        self.studio.world().log.wrote(first, pcm);
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<(), RecordFault> {
        fs::write(&self.file, ENDED).map_err(|_| RecordFault::Failed)?;
        self.studio.world().log.finished = true;
        Ok(())
    }
}

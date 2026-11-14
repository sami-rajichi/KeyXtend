//! A file's facts and a card's self-test (ADR-0044 decisions 5 to 7). The check takes the fit check, loads the model on
//! the card, warms it up on the shipped clip with progress, then times the clip on the card and on the processor and
//! hands both texts and times back; the module scores them (`card_diff`, `card_gain`).

use super::Tell;
use super::cards;
use super::library::{Failed, Library, Loaded, Place, Stop, ms_since};
use super::resample::{Mixer, Resampler};
use super::split;
use super::wav;
use super::wipe::Buf;
use kx_ipc::{MAX_LANGS, MAX_PERCENT, Model, SOUND_HZ, Stage, VoiceFault, VoiceNote};
use std::path::{Path, PathBuf};
use std::time::Instant;
use zeroize::Zeroizing;

/// The warm-up's progress before and after: the engine reports none in between.
const WARM_START: u8 = 0;

/// A card's check, as the job's thread hands it over.
pub struct Exam {
    /// The ask.
    pub id: u32,
    /// The model, aimed at the card to check.
    pub model: Model<'static>,
    /// The shipped clip, in a language the model knows.
    pub clip: PathBuf,
    /// Where its notes go.
    pub tell: Tell,
}

/// The facts of the model file at `path`: the runner and the language codes the frames take, or why not.
pub fn facts<L: Library>(lib: &L, id: u32, path: &Path) -> VoiceNote {
    match lib.langs(path) {
        Ok(langs) => VoiceNote::Facts {
            id,
            runner: L::RUNNER.to_owned(),
            langs: langs
                .into_iter()
                .filter(|l| cards::tag(l))
                .take(MAX_LANGS)
                .collect(),
        },
        Err(why) => VoiceNote::Fault {
            id,
            fault: why.read(),
        },
    }
}

/// The check's answer: both texts and times, or the fault that stopped it.
pub fn check<L: Library>(lib: &L, cores: usize, exam: &Exam) -> VoiceNote {
    let id = exam.id;
    exam_card(lib, cores, exam).unwrap_or_else(|fault| VoiceNote::Fault { id, fault })
}

/// Runs the check; a card fault ends it, as the card is what it checks.
fn exam_card<L: Library>(lib: &L, cores: usize, exam: &Exam) -> Result<VoiceNote, VoiceFault> {
    let (id, model) = (exam.id, &exam.model);
    let pcm = clip(&exam.clip)?;
    let (place, refused) = cards::place(&model.target, &lib.devices());
    if let Some(fault) = refused {
        return Err(fault);
    }
    let threads = cards::threads(model.threads, cores);
    let path = Path::new(&*model.path);
    exam.tell.note(VoiceNote::Stage {
        id,
        stage: Stage::Loading,
    });
    let mut card = lib.load(path, &place, threads).map_err(Failed::on_card)?;
    if !card.knows(&model.lang) {
        return Err(VoiceFault::Language);
    }
    let warm = |percent| exam.tell.note(VoiceNote::Warm { id, percent });
    warm(WARM_START);
    timed(&mut card, &pcm, &model.lang).map_err(Failed::on_card)?;
    warm(MAX_PERCENT);
    let (card_text, card_ms) = timed(&mut card, &pcm, &model.lang).map_err(Failed::on_card)?;
    drop(card);
    let mut cpu = lib
        .load(path, &Place::Cpu, threads)
        .map_err(Failed::fault)?;
    let (cpu_text, cpu_ms) = timed(&mut cpu, &pcm, &model.lang).map_err(Failed::fault)?;
    Ok(VoiceNote::Checked {
        id,
        card_ms,
        cpu_ms,
        card_text,
        cpu_text,
    })
}

/// The text of `pcm` on `model`, and how long it took.
fn timed<M: Loaded>(
    model: &mut M,
    pcm: &[f32],
    lang: &str,
) -> Result<(Zeroizing<String>, u32), Failed> {
    let start = Instant::now();
    let text = split::text(model, pcm, lang, &Stop::default())?;
    Ok((text, ms_since(start)))
}

/// The clip at `path` as 16 kHz mono floats.
///
/// # Errors
/// `NoFile` or `BadFile` from the reading, `Memory` when a buffer cannot be had.
pub fn clip(path: &Path) -> Result<Buf<f32>, VoiceFault> {
    let sound = wav::read(path)?;
    let mono = Mixer::new(sound.format.channels).take(&sound.samples)?;
    let mut rate = Resampler::new(sound.format.rate, SOUND_HZ);
    let mut even = rate.push(&mono)?;
    even.extend(&rate.flush()?)?;
    Ok(even)
}

#[cfg(test)]
mod tests;

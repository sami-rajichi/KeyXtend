//! The voice job's frames built from the port's asks: a dictation's setup and a model with its target, the threads
//! resolved to the real cores first, and the shipped files' paths.

use super::{Assets, threads};
use kx_ipc::{Model, Setup, Target};
use kx_module_api::ports::{CardUse, SpeechAsk, SpeechError, SpeechModel};
use std::borrow::Cow;
use std::path::Path;

/// `ask`'s setup, with the shipped detector and its threads resolved against `cores`.
pub(super) fn setup(
    ask: &SpeechAsk,
    assets: &Assets,
    cores: usize,
) -> Result<Setup<'static>, SpeechError> {
    let threads = threads(ask.threads, cores);
    let model = ask.model.as_ref();
    let model = model.map(|m| model_of(m, ask.card.as_ref(), threads));
    Ok(Setup {
        model: model.transpose()?,
        pad_ms: ask.pad_ms,
        piece_s: ask.piece_s,
        record_max_s: ask.record_max_s,
        level_hz: ask.level_hz,
        silence_stop_s: ask.silence_stop_s,
        detector: path(&assets.detector)?,
    })
}

/// `model` on `card`, or on the processor without one, with `threads` already resolved.
pub(super) fn model_of(
    model: &SpeechModel,
    card: Option<&CardUse>,
    threads: u8,
) -> Result<Model<'static>, SpeechError> {
    let target = card.map_or(Target::Cpu, |card| Target::Card {
        id: Cow::Owned(card.id.as_str().to_owned()),
        need_mb: card.need_mb.saturating_add(card.margin_mb),
    });
    Ok(Model {
        path: path(model.file.expose())?,
        lang: Cow::Owned(model.lang.clone()),
        runner: Cow::Owned(model.runner.clone()),
        target,
        threads,
    })
}

/// `file` as the frames carry it; `NoFile` for one that is not UTF-8, which no frame can name.
pub(super) fn path(file: &Path) -> Result<Cow<'static, str>, SpeechError> {
    let text = file.to_str().ok_or(SpeechError::NoFile)?;
    Ok(Cow::Owned(text.to_owned()))
}

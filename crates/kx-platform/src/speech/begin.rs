//! Asks that start a run's work on the dictation job: a fresh id mapped to its run before any note of it can come, and
//! a moved recording handed in as wiped copies before Again.

use super::lock;
use super::note::unsent;
use super::router::Router;
use crate::worker::{Spawner, Split, Stream, VoiceFrames, Wait};
use kx_ipc::{Setup, SoundPiece, VoiceAsk, Wiped};
use kx_module_api::SecretBytes;
use kx_module_api::ports::SpeechError;
use std::sync::{Arc, Mutex};

/// Begins the ask `make` builds on `stream`, its fresh id given to `run` before any note of it can come.
pub(super) fn begin<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    router: &Arc<Mutex<Router>>,
    stream: &Stream<S, VoiceFrames>,
    run: u64,
    make: impl FnOnce(u32) -> VoiceAsk<'static> + Send + 'static,
    wait: Wait,
) -> Result<u32, SpeechError> {
    let (given, mapped) = (Arc::new(Mutex::new(None)), Arc::clone(router));
    let seen = Arc::clone(&given);
    let mapping = move |id| {
        lock(&mapped).map(run, id);
        *lock(&seen) = Some(id);
        make(id)
    };
    stream.begin(mapping, wait).map_err(|e| {
        if let Some(id) = lock(&given).take() {
            lock(router).unmap(id);
        }
        unsent(e)
    })
}

/// Hands `pieces` to a new job as `run`'s, each piece a wiped copy sent unwatched, then asks it to write them.
pub(super) fn hand_in<S: Spawner<VoiceFrames, Worker: Split<VoiceFrames>>>(
    router: &Arc<Mutex<Router>>,
    stream: &Stream<S, VoiceFrames>,
    run: u64,
    pieces: &[SecretBytes],
    setup: Setup<'static>,
    wait: Wait,
) -> Result<u32, SpeechError> {
    let piece = |index: usize| -> Result<SoundPiece<'static>, SpeechError> {
        let bytes = pieces.get(index).ok_or(SpeechError::NoSound)?;
        Ok(SoundPiece {
            index: u16::try_from(index).map_err(|_| SpeechError::Internal)?,
            last: index + 1 == pieces.len(),
            bytes: Wiped::Kept(bytes.expose().clone()),
        })
    };
    let head = piece(0)?;
    let id = begin(router, stream, run, move |id| sound(id, head), Wait::Off)?;
    for index in 1..pieces.len() {
        stream
            .ask(sound(id, piece(index)?), Wait::Off)
            .map_err(unsent)?;
    }
    let again = VoiceAsk::Again { id, setup };
    stream.ask(again, wait).map_err(unsent)?;
    Ok(id)
}

/// A piece of a recording handed in as run `id`'s.
const fn sound(id: u32, piece: SoundPiece<'static>) -> VoiceAsk<'static> {
    VoiceAsk::Sound { id, piece }
}

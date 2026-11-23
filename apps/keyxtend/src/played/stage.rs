//! The voice gate build only: one debug line per stage of a dictation, by name, so G22 reads where its time goes from
//! the copy's log. A piece's line holds its index and, when known, its sound's length; never text, sound or a key.

pub use kx_ipc::gate::{
    INDEX, LOAD, OPEN, PIECE, SEND, SOUND_MS, STAGE, STAGE_LINE, START, STOP, TOOK_MS,
};

/// Logs that a dictation reached `name`; its Display form keeps the name one bare word on the line.
pub fn stage(name: &str) {
    tracing::debug!({ STAGE } = %name, "{STAGE_LINE}");
}

/// Logs that piece `index`'s text came, with its sound's length when known.
pub fn piece(index: u16, sound_ms: Option<u32>) {
    if let Some(sound_ms) = sound_ms {
        tracing::debug!({ STAGE } = %PIECE, { INDEX } = index, { SOUND_MS } = sound_ms, "{STAGE_LINE}");
    } else {
        tracing::debug!({ STAGE } = %PIECE, { INDEX } = index, "{STAGE_LINE}");
    }
}

/// Logs that the model's load took `took_ms`.
pub fn load(took_ms: u32) {
    tracing::debug!({ STAGE } = %LOAD, { TOOK_MS } = took_ms, "{STAGE_LINE}");
}

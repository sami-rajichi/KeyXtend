//! The voice gate build only: one debug line per stage of a dictation, by name, so G22 reads where its time goes from
//! the copy's log. A piece's line holds its index and, when known, its sound's length; never text, sound or a key.

/// The message of each stage's line; kx-gates reads it back.
pub const STAGE_LINE: &str = "voice gate build: a stage";
/// The field that names the stage, as the lines below write it.
#[cfg(test)]
pub const STAGE: &str = "stage";
/// The field that holds a piece's index.
#[cfg(test)]
pub const INDEX: &str = "index";
/// The field that holds a piece's sound length, in ms.
#[cfg(test)]
pub const SOUND_MS: &str = "sound_ms";
/// The field that holds the model's load time, in ms.
#[cfg(test)]
pub const TOOK_MS: &str = "took_ms";

/// The run was asked to start.
pub const START: &str = "start";
/// The microphone opened.
pub const OPEN: &str = "open";
/// Stop reached the run.
pub const STOP: &str = "stop";
/// The model's load ended.
pub const LOAD: &str = "load";
/// A piece's text came.
pub const PIECE: &str = "piece";
/// A cloud dictation's pieces went to its service.
pub const SEND: &str = "send";

/// Logs that a dictation reached `name`; its Display form keeps the name one bare word on the line.
pub fn stage(name: &str) {
    tracing::debug!(stage = %name, "{STAGE_LINE}");
}

/// Logs that piece `index`'s text came, with its sound's length when known.
pub fn piece(index: u16, sound_ms: Option<u32>) {
    if let Some(sound_ms) = sound_ms {
        tracing::debug!(stage = %PIECE, index, sound_ms, "{STAGE_LINE}");
    } else {
        tracing::debug!(stage = %PIECE, index, "{STAGE_LINE}");
    }
}

/// Logs that the model's load took `took_ms`.
pub fn load(took_ms: u32) {
    tracing::debug!(stage = %LOAD, took_ms, "{STAGE_LINE}");
}

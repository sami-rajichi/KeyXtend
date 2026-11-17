//! `Debug` for the voice frames: ids, codes, numbers, lengths, tags and device names, never a text, a sound or a path.

use super::ask::VoiceAsk;
use super::note::VoiceNote;
use super::{Model, Setup, SoundPiece};
use std::fmt;

impl fmt::Debug for Setup<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Setup")
            .field("model", &self.model)
            .field("pad_ms", &self.pad_ms)
            .field("piece_s", &self.piece_s)
            .field("record_max_s", &self.record_max_s)
            .field("level_hz", &self.level_hz)
            .field("silence_stop_s", &self.silence_stop_s)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Model<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Model")
            .field("lang", &self.lang)
            .field("runner", &self.runner)
            .field("target", &self.target)
            .field("threads", &self.threads)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for SoundPiece<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoundPiece")
            .field("index", &self.index)
            .field("last", &self.last)
            .field("len", &self.bytes.len())
            .finish()
    }
}

impl fmt::Debug for VoiceAsk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if matches!(self, Self::Bye) {
            return f.write_str("Bye");
        }
        let mut s = f.debug_struct(ask_name(self));
        s.field("id", &self.id());
        match self {
            Self::Start { setup, .. } | Self::Again { setup, .. } => {
                s.field("setup", setup).finish()
            }
            Self::Facts { .. } => s.finish_non_exhaustive(),
            Self::Check { model, .. } => s.field("model", model).finish_non_exhaustive(),
            Self::Sound { piece, .. } => s.field("piece", piece).finish(),
            Self::Bye
            | Self::Stop { .. }
            | Self::Cancel { .. }
            | Self::Keep { .. }
            | Self::Close { .. }
            | Self::Devices { .. } => s.finish(),
        }
    }
}

/// The variant's name, as `Debug` starts it.
const fn ask_name(ask: &VoiceAsk<'_>) -> &'static str {
    match ask {
        VoiceAsk::Start { .. } => "Start",
        VoiceAsk::Stop { .. } => "Stop",
        VoiceAsk::Cancel { .. } => "Cancel",
        VoiceAsk::Again { .. } => "Again",
        VoiceAsk::Keep { .. } => "Keep",
        VoiceAsk::Close { .. } => "Close",
        VoiceAsk::Devices { .. } => "Devices",
        VoiceAsk::Facts { .. } => "Facts",
        VoiceAsk::Check { .. } => "Check",
        VoiceAsk::Sound { .. } => "Sound",
        VoiceAsk::Bye => "Bye",
    }
}

impl fmt::Debug for VoiceNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct(note_name(self));
        s.field("id", &self.id());
        match self {
            Self::Level { level, .. } => s.field("level", level),
            Self::Heard { speech, .. } => s.field("speech", speech),
            Self::Stage { stage, .. } => s.field("stage", stage),
            Self::Warm { percent, .. } => s.field("percent", percent),
            Self::Piece { index, text, .. } => s.field("index", index).field("len", &text.len()),
            Self::Done {
                pieces,
                sound_ms,
                wait_ms,
                load_ms,
                memory_mb,
                card,
                ..
            } => s
                .field("pieces", pieces)
                .field("sound_ms", sound_ms)
                .field("wait_ms", wait_ms)
                .field("load_ms", load_ms)
                .field("memory_mb", memory_mb)
                .field("card", card),
            Self::Fault { fault, .. } => s.field("fault", fault),
            Self::Devices { list, .. } => s.field("list", list),
            Self::Facts { runner, langs, .. } => s.field("runner", runner).field("langs", langs),
            Self::Checked {
                card_ms, cpu_ms, ..
            } => s.field("card_ms", card_ms).field("cpu_ms", cpu_ms),
            Self::Sound { piece, .. } => s.field("piece", piece),
        }
        .finish()
    }
}

/// The variant's name, as `Debug` starts it.
const fn note_name(note: &VoiceNote) -> &'static str {
    match note {
        VoiceNote::Level { .. } => "Level",
        VoiceNote::Heard { .. } => "Heard",
        VoiceNote::Stage { .. } => "Stage",
        VoiceNote::Warm { .. } => "Warm",
        VoiceNote::Piece { .. } => "Piece",
        VoiceNote::Done { .. } => "Done",
        VoiceNote::Fault { .. } => "Fault",
        VoiceNote::Devices { .. } => "Devices",
        VoiceNote::Facts { .. } => "Facts",
        VoiceNote::Checked { .. } => "Checked",
        VoiceNote::Sound { .. } => "Sound",
    }
}

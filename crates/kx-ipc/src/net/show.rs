//! `Debug` for the net frames: ids, shapes, header names, the language, limits, counts and lengths, never a key, a
//! sound, an address, a model or a text.

use super::ask::NetAsk;
use super::note::NetNote;
use super::{NetCall, NetRun};
use std::fmt;

impl fmt::Debug for NetCall<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetCall")
            .field("shape", &self.shape)
            .field("header", &self.header)
            .field("wait_s", &self.wait_s)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for NetRun<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetRun")
            .field("lang", &self.lang)
            .field("limits", &self.limits)
            .field("tries", &self.tries)
            .field("pieces", &self.sizes.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for NetAsk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send { id, call, run } => f
                .debug_struct("Send")
                .field("id", id)
                .field("call", call)
                .field("run", run)
                .finish(),
            Self::Sound { id, piece, bytes } => f
                .debug_struct("Sound")
                .field("id", id)
                .field("piece", piece)
                .field("len", &bytes.len())
                .finish(),
            Self::Test { id, call } => f
                .debug_struct("Test")
                .field("id", id)
                .field("call", call)
                .finish(),
            Self::Cancel { id } => f.debug_struct("Cancel").field("id", id).finish(),
            Self::Bye => f.write_str("Bye"),
        }
    }
}

impl fmt::Debug for NetNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text { id, index, text } => f
                .debug_struct("Text")
                .field("id", id)
                .field("index", index)
                .field("len", &text.len())
                .finish(),
            Self::Done { id } => f.debug_struct("Done").field("id", id).finish(),
            Self::Fault { id, fault } => f
                .debug_struct("Fault")
                .field("id", id)
                .field("fault", fault)
                .finish(),
        }
    }
}

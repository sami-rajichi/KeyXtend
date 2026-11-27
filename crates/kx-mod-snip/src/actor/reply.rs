//! A job's answer, sent once: a job that panics or never runs still answers, so the flow and the panel never wait on
//! it forever.

use std::sync::mpsc::Sender;

use super::{Back, Msg};

/// The answer a job owes the actor; dropped unsent, it sends its fallback.
pub(super) struct Reply {
    back: Sender<Msg>,
    fallback: Option<Back>,
}

impl Reply {
    /// An answer owed on `back`, which is `fallback` unless the job sends another.
    pub(super) fn new(back: Sender<Msg>, fallback: Back) -> Self {
        Self {
            back,
            fallback: Some(fallback),
        }
    }

    /// Sends `answer` instead of the fallback; a stopped actor no longer hears it.
    pub(super) fn send(mut self, answer: Back) {
        self.fallback = None;
        let _ = self.back.send(Msg::Back(answer));
    }
}

impl Drop for Reply {
    fn drop(&mut self) {
        if let Some(fallback) = self.fallback.take() {
            let _ = self.back.send(Msg::Back(fallback));
        }
    }
}

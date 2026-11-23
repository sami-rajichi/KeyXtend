//! A cloud dictation's upload (ADR-0047): at Stop its run hands the speech over, the key is read off the actor just
//! before `send` and moved into the ask, and the service's notes go back to the flow. A run is dropped once it ends,
//! or at Halt, which cancels its requests.

use std::collections::HashMap;

use kx_module_api::SecretBytes;
use kx_module_api::ports::{CloudFault, CloudNote, SpeechError};

use super::ask::{Head, Pending, Wired, head_of, shut};
use super::lanes::Owner;
use super::trial::Trial;
use super::{Actor, Back};
use crate::flow::{In, Sent, Upload, renewal};
use crate::keys::Keys;

/// One dictation's upload; dropped at Halt, it sends nothing more.
struct Up {
    round: u32,
    pending: Pending,
}

/// The actor's cloud side: the keys, each dictation's upload and Compare's.
pub(super) struct Uploads {
    /// The keys on the voice shelf; a hidden run's has none.
    pub keys: Keys,
    ups: HashMap<u32, Up>,
    /// Compare's cloud models' uploads, by run and model.
    pub trials: HashMap<(u32, usize), Trial>,
}

impl Uploads {
    /// No upload yet, with `keys`.
    pub(super) fn new(keys: Keys) -> Self {
        Self {
            keys,
            ups: HashMap::new(),
            trials: HashMap::new(),
        }
    }

    /// Drops every upload, Compare's too, which cancels their runs.
    pub(super) fn clear(&mut self) {
        self.ups.clear();
        self.trials.clear();
    }

    /// No upload runs or waits for its speech.
    #[cfg(test)]
    pub(super) fn idle(&self) -> bool {
        self.ups.is_empty() && self.trials.is_empty()
    }
}

impl Actor {
    /// Sends `up`'s speech: its run hands it over first; a request that cannot be built ends as the cloud fault.
    pub(super) fn send(&mut self, up: &Upload) {
        let Some(head) = self.head(up) else {
            return self.sent(
                up.take,
                up.round,
                Sent::Note(CloudNote::Fault(CloudFault::Gone)),
            );
        };
        let entry = Up {
            round: up.round,
            pending: Pending::new(head),
        };
        self.net.ups.insert(up.take, entry);
        self.hand((Owner::Flow, up.take), up.round);
    }

    /// Drops dictation `take`'s upload: nothing more is sent, and its run's requests are cancelled.
    pub(super) fn halt(&mut self, take: u32) {
        self.net.ups.remove(&take);
    }

    /// The run handed dictation `take`'s speech over for upload `round`: none sends nothing, and the speech of an
    /// upload halted since is wiped as it drops.
    pub(super) fn spoken(
        &mut self,
        take: u32,
        round: u32,
        pieces: Result<Vec<SecretBytes>, SpeechError>,
    ) {
        let Some(up) = self.net.ups.get_mut(&take).filter(|u| u.round == round) else {
            return;
        };
        let (pieces, job) = match (pieces, up.pending.start()) {
            (Ok(pieces), Some(job)) if !pieces.is_empty() => (pieces, job),
            (pieces, _) => {
                self.net.ups.remove(&take);
                let sent = match pieces {
                    Ok(_) => Sent::Silent,
                    Err(error) => {
                        tracing::warn!(%error, "a cloud dictation's speech could not be had");
                        Sent::Speech(error)
                    }
                };
                return self.sent(take, round, sent);
            }
        };
        let note = move |note| Back::Net { take, round, note };
        if !self.post(job, pieces, note, move |run| Back::Sending {
            take,
            round,
            run,
        }) {
            self.net_note(take, round, CloudNote::Fault(CloudFault::Gone));
        }
    }

    /// A note from dictation `take`'s upload: its end drops the run, and its text comes after a look at the box.
    pub(super) fn net_note(&mut self, take: u32, round: u32, note: CloudNote) {
        if self.net.ups.get(&take).is_none_or(|u| u.round != round) {
            return;
        }
        if matches!(note, CloudNote::Done | CloudNote::Fault(_)) {
            self.net.ups.remove(&take);
        }
        if note == CloudNote::Done {
            self.look();
        }
        self.sent(take, round, Sent::Note(note));
    }

    /// The service's run opened: it is held until it ends, or dropped at once when it ended or was halted already.
    pub(super) fn opened_net(&mut self, take: u32, round: u32, run: Wired) {
        match self.net.ups.get_mut(&take).filter(|u| u.round == round) {
            Some(up) => up.pending.run = Some(run),
            None => shut(run),
        }
    }

    /// Tells the flow what dictation `take`'s upload of `round` told, with when a used allowance renews.
    fn sent(&mut self, take: u32, round: u32, sent: Sent) {
        let renews = renewal(&*self.ports.calendar, sent.fault());
        self.step(In::Sent {
            take,
            round,
            sent,
            renews,
        });
    }

    /// The head of `up`'s ask from the language's pick, while it still names `up`'s service.
    fn head(&self, up: &Upload) -> Option<Head> {
        let s = &self.held.settings;
        let choice = self.held.cloud()?.choice(s, &up.lang)?;
        (choice.service.id == up.service)
            .then(|| head_of(s, &choice))
            .flatten()
    }
}

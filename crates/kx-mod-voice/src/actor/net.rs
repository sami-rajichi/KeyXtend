//! A cloud dictation's upload (ADR-0047): at Stop its run hands the speech over, the key is read off the actor just
//! before `send` and moved into the ask, and the service's notes go back to the flow. A run is dropped once it ends,
//! or at Halt, which cancels its requests.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kx_module_api::SecretBytes;
use kx_module_api::ports::{CloudFault, CloudNote, SpeechError};

use super::ask::{Head, Tell, Wired, deliver, head_of, shut};
use super::lanes::Owner;
use super::trial::Trial;
use super::{Actor, Back, Job, Msg};
use crate::flow::{In, Sent, Upload};
use crate::keys::Keys;

/// One dictation's upload.
struct Up {
    round: u32,
    /// The ask's head, until the speech comes.
    head: Option<Head>,
    /// Set at Halt, so a job between the key and `send` sends nothing.
    stop: Arc<AtomicBool>,
    /// The service's run, once `send` gave it.
    run: Option<Wired>,
}

impl Drop for Up {
    /// A dropped upload sends nothing more, and its run's requests are cancelled.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(run) = self.run.take() {
            shut(run);
        }
    }
}

/// The actor's cloud side: the keys, each dictation's upload and Compare's.
pub(super) struct Uploads {
    /// The run's saves would be hidden (ADR-0037), so cloud is off and no key is read.
    pub hidden: bool,
    /// The keys on the voice shelf; a hidden run's has none.
    pub keys: Keys,
    ups: HashMap<u32, Up>,
    /// Compare's cloud models' uploads, by run and model.
    pub trials: HashMap<(u32, usize), Trial>,
}

impl Uploads {
    /// No upload yet, with `keys`, in a run that is `hidden` or not.
    pub(super) fn new(keys: Keys, hidden: bool) -> Self {
        Self {
            hidden,
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
        let stop = Arc::new(AtomicBool::new(false));
        let entry = Up {
            round: up.round,
            head: Some(head),
            stop,
            run: None,
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
        let (pieces, head) = match (pieces, up.head.take()) {
            (Ok(pieces), Some(head)) if !pieces.is_empty() => (pieces, head),
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
        let stop = Arc::clone(&up.stop);
        self.post(take, round, head, pieces, stop);
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
            Some(up) => up.run = Some(run),
            None => shut(run),
        }
    }

    /// Tells the flow what dictation `take`'s upload of `round` told, and when on the wall clock.
    fn sent(&mut self, take: u32, round: u32, sent: Sent) {
        let at = self.ports.calendar.now();
        self.step(In::Sent {
            take,
            round,
            sent,
            at,
        });
    }

    /// Reads the key and sends, in a job off the actor; its notes and its run come back through the inbox.
    fn post(
        &mut self,
        take: u32,
        round: u32,
        head: Head,
        pieces: Vec<SecretBytes>,
        stop: Arc<AtomicBool>,
    ) {
        let (back, cloud) = (self.ports.back.clone(), self.ports.cloud.clone());
        let keys = Keys::new(Arc::clone(&self.ports.shelf), self.net.hidden);
        let job: Job = Box::new(move || {
            let notes = back.clone();
            let tell: Tell = Box::new(move |note| {
                drop(notes.send(Msg::Back(Back::Net { take, round, note })));
            });
            let back_of = match deliver(cloud.as_deref(), &keys, head, pieces, &stop, tell) {
                Ok(Some(run)) => Back::Sending { take, round, run },
                Ok(None) => return,
                Err(fault) => Back::Net {
                    take,
                    round,
                    note: CloudNote::Fault(fault),
                },
            };
            // Only a stopped actor misses it, and the run then drops, which cancels it.
            let _ = back.send(Msg::Back(back_of));
        });
        if !(self.ports.spawn)(job) {
            self.net_note(take, round, CloudNote::Fault(CloudFault::Gone));
        }
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

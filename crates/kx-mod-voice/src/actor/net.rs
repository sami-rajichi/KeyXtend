//! A cloud dictation's upload (ADR-0047): at Stop its run hands the speech over, the key is read off the actor just
//! before `send` and moved into the ask, and the service's notes go back to the flow. A run is dropped once it ends,
//! or at Halt, which cancels its requests.

use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudLimits, CloudNote, CloudRun, CloudShape, CloudSpeech, HttpsUrl,
    JobLimits, KeyHeader, SpeechError,
};
use kx_module_api::{Redacted, Secret, SecretBytes};

use super::{Actor, Back, Job, Msg};
use crate::cloud::{Chosen, Cloud};
use crate::flow::{In, Sent, Upload};
use crate::keys::Keys;
use crate::services;

/// The service's run, which prints nothing of itself; dropping it cancels its requests.
pub(crate) struct Wired {
    /// Held only to be dropped.
    _run: Box<dyn CloudRun>,
}

/// What a run's notes are told to.
type Tell = Box<dyn Fn(CloudNote) + Send>;

/// A cloud ask without its key and speech, built on the actor from the row and `[voice]`.
struct Head {
    /// The service's id, which names its key's blob.
    service: String,
    job: JobLimits,
    shape: CloudShape,
    url: HttpsUrl,
    header: KeyHeader,
    model: Option<Redacted<String>>,
    lang: String,
    limits: CloudLimits,
    wait_s: u16,
    tries: u8,
}

impl Head {
    /// The ask with `key` and `pieces` moved in.
    fn ask(self, key: Secret, pieces: Vec<SecretBytes>) -> CloudAsk {
        CloudAsk {
            job: self.job,
            shape: self.shape,
            url: self.url,
            header: self.header,
            key,
            model: self.model,
            lang: self.lang,
            limits: self.limits,
            wait_s: self.wait_s,
            tries: self.tries,
            pieces,
        }
    }
}

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

/// The actor's cloud side: the keys, and each dictation's upload.
pub(super) struct Uploads {
    /// The run's saves would be hidden (ADR-0037), so cloud is off and no key is read.
    pub hidden: bool,
    /// The keys on the voice shelf; a hidden run's has none.
    pub keys: Keys,
    ups: HashMap<u32, Up>,
}

impl Uploads {
    /// No upload yet, with `keys`, in a run that is `hidden` or not.
    pub(super) fn new(keys: Keys, hidden: bool) -> Self {
        Self {
            hidden,
            keys,
            ups: HashMap::new(),
        }
    }

    /// Drops every upload, which cancels their runs.
    pub(super) fn clear(&mut self) {
        self.ups.clear();
    }

    /// No upload runs or waits for its speech.
    #[cfg(test)]
    pub(super) fn idle(&self) -> bool {
        self.ups.is_empty()
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
        self.hand(up.take, up.round);
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

    /// Tells the flow what dictation `take`'s upload of `round` told.
    fn sent(&mut self, take: u32, round: u32, sent: Sent) {
        self.step(In::Sent { take, round, sent });
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
        let table = services::table().ok()?;
        let choice = Cloud::new(table, self.net.hidden).choice(s, &up.lang)?;
        let (row, entry) = (choice.service, s.services.get(&up.service)?);
        if row.id != up.service {
            return None;
        }
        let (listed, model) = match choice.model {
            Chosen::Listed(m) => (Some(m), (!m.unnamed).then(|| m.id.clone())),
            Chosen::Typed(id) => (None, Some(id.to_owned())),
        };
        Some(Head {
            service: row.id.clone(),
            job: s.job(),
            shape: row.shape_for(listed, entry.keep),
            url: row.url(&entry.address)?,
            header: row.header.clone(),
            model: model.map(Redacted::new),
            lang: choice.code,
            limits: row.limits,
            wait_s: s.net_wait_s,
            tries: s.net_tries,
        })
    }
}

/// Drops `run`, which cancels its requests; a port that breaks as it drops is caught.
fn shut(run: Wired) {
    let _ = catch_unwind(AssertUnwindSafe(|| drop(run)));
}

/// Reads the key just before `send` and moves it into the ask: no key, or one that cannot be read, sends nothing;
/// neither does a halted upload, nor a run with no cloud port. A panic in the port counts as `Gone`.
fn deliver(
    cloud: Option<&dyn CloudSpeech>,
    keys: &Keys,
    head: Head,
    pieces: Vec<SecretBytes>,
    stop: &AtomicBool,
    tell: Tell,
) -> Result<Option<Wired>, CloudFault> {
    let cloud = cloud.ok_or(CloudFault::Gone)?;
    let key = match keys.get(&head.service) {
        Ok(Some(key)) => key,
        Ok(None) => return Err(CloudFault::Key),
        Err(error) => {
            tracing::warn!(%error, "a cloud key could not be read");
            return Err(CloudFault::Key);
        }
    };
    if stop.load(Ordering::SeqCst) {
        return Ok(None);
    }
    let ask = head.ask(key, pieces);
    let sent = catch_unwind(AssertUnwindSafe(|| cloud.send(ask, tell)));
    sent.unwrap_or(Err(CloudFault::Gone))
        .map(|run| Some(Wired { _run: run }))
}

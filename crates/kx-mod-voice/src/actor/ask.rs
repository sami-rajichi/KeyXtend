//! A cloud ask's parts the dictation's upload and Compare's share (ADR-0047): its head, built on the actor from the
//! row and `[voice]`, and its delivery in a job, which reads the key just before `send`.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudLimits, CloudNote, CloudRun, CloudShape, CloudSpeech, HttpsUrl,
    JobLimits, KeyHeader,
};
use kx_module_api::{Redacted, Secret, SecretBytes};

use super::{Actor, Back, Job, Msg};
use crate::VoiceSettings;
use crate::cloud::{Choice, Chosen};
use crate::keys::{KEY_UNREAD, Keys};

/// The service's run, which prints nothing of itself; dropping it cancels its requests.
pub(crate) struct Wired {
    /// Held only to be dropped.
    _run: Box<dyn CloudRun>,
}

/// What a run's notes are told to.
pub(super) type Tell = Box<dyn Fn(CloudNote) + Send>;

/// A cloud ask without its key and speech, built on the actor from the row and `[voice]`.
pub(super) struct Head {
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

/// The head of an ask to `choice`'s service and model, from its row and `[voice]`; `None` for a model ruled out now or
/// an address the row no longer takes.
pub(super) fn head_of(s: &VoiceSettings, choice: &Choice<'_>) -> Option<Head> {
    let (row, entry) = (choice.service, s.services.get(&choice.service.id)?);
    if choice.unavailable.is_some() {
        return None;
    }
    let listed = match choice.model {
        Chosen::Listed(m) => Some(m),
        Chosen::Typed(_) => None,
    };
    let model = choice.model.sent().map(str::to_owned);
    Some(Head {
        service: row.id.clone(),
        job: s.job(),
        shape: row.shape_for(listed, entry.keep),
        url: row.url(&entry.address)?,
        header: row.header.clone(),
        model: model.map(Redacted::new),
        lang: choice.code.clone(),
        limits: row.limits,
        wait_s: s.net_wait_s,
        tries: s.net_tries,
    })
}

/// Drops `run`, which cancels its requests or closes its job; a port that breaks as it drops is caught.
pub(super) fn shut<T>(run: T) {
    let _ = catch_unwind(AssertUnwindSafe(|| drop(run)));
}

/// Reads the key just before `send` and moves it into the ask: no key, or one that cannot be read, sends nothing;
/// neither does a halted upload, nor a run with no cloud port. A panic in the port counts as `Gone`.
pub(super) fn deliver(
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
            tracing::warn!(%error, "{KEY_UNREAD}");
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

/// One upload's holder, a dictation's or a Compare model's: the ask's head until the speech comes, the flag a job
/// between the key and `send` reads, and the service's run once `send` gave it.
pub(super) struct Pending {
    head: Option<Head>,
    stop: Arc<AtomicBool>,
    /// The service's run, held until it ends.
    pub(super) run: Option<Wired>,
}

impl Pending {
    /// An upload of `head`, its speech still to come.
    pub(super) fn new(head: Head) -> Self {
        Self {
            head: Some(head),
            stop: Arc::new(AtomicBool::new(false)),
            run: None,
        }
    }

    /// The head and the stop flag for its job, once the speech came; `None` after the first time.
    pub(super) fn start(&mut self) -> Option<(Head, Arc<AtomicBool>)> {
        Some((self.head.take()?, Arc::clone(&self.stop)))
    }
}

impl Drop for Pending {
    /// A dropped upload sends nothing more, and its run's requests are cancelled.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(run) = self.run.take() {
            shut(run);
        }
    }
}

impl Actor {
    /// Reads the key and sends `job`'s head with `pieces` off the actor; `note` and `opened` build what goes back to
    /// the inbox. False when the job could not start.
    pub(super) fn post<N, O>(
        &self,
        (head, stop): (Head, Arc<AtomicBool>),
        pieces: Vec<SecretBytes>,
        note: N,
        opened: O,
    ) -> bool
    where
        N: Fn(CloudNote) -> Back + Clone + Send + 'static,
        O: FnOnce(Wired) -> Back + Send + 'static,
    {
        let (back, cloud, keys) = (
            self.ports.back.clone(),
            self.ports.cloud.clone(),
            self.keys(),
        );
        let job: Job = Box::new(move || {
            let (notes, told) = (back.clone(), note.clone());
            let tell: Tell = Box::new(move |n| drop(notes.send(Msg::Back(told(n)))));
            let back_of = match deliver(cloud.as_deref(), &keys, head, pieces, &stop, tell) {
                Ok(Some(run)) => opened(run),
                Ok(None) => return,
                Err(fault) => note(CloudNote::Fault(fault)),
            };
            // Only a stopped actor misses it, and the run then drops, which cancels it.
            let _ = back.send(Msg::Back(back_of));
        });
        (self.ports.spawn)(job)
    }
}

//! Compare's cloud models on the actor (edge 12): once Compare's run hands its recording over, a copy goes to each
//! ticked model through `CloudSpeech`, with the dictation's rules: its service still on and ready for the language, the
//! key read in the job just before `send`, none in a hidden run. Cancel, Back, Close or a newer run drops the uploads.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kx_module_api::ports::{CloudFault, CloudNote, SpeechError};
use kx_module_api::{Mono, SecretBytes};

use super::ask::{Head, Tell, Wired, deliver, head_of, shut};
use super::lanes::{Lane, Owner};
use super::{Actor, Back, Job, Msg};
use crate::cloud::Cloud;
use crate::compare::Remote;
use crate::keys::Keys;
use crate::services;

/// The round Compare's hand-over is asked for, as its run hands its recording over once.
const ONCE: u32 = 0;

/// One cloud model's upload in a Compare run.
pub(super) struct Trial {
    /// The ask's head, until the speech comes.
    head: Option<Head>,
    /// Set as it drops, so a job between the key and `send` sends nothing.
    stop: Arc<AtomicBool>,
    /// The service's run, once `send` gave it.
    run: Option<Wired>,
    /// When the speech went, for the model's wait.
    since: Mono,
}

impl Drop for Trial {
    /// A dropped upload sends nothing more, and its run's requests are cancelled.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(run) = self.run.take() {
            shut(run);
        }
    }
}

impl Actor {
    /// Compare's run `take` will hand its recording over for `to`, in `lang`; a model whose service is not ready now
    /// fails at once, and an older run's uploads go.
    pub(super) fn trial(&mut self, take: u32, lang: &str, to: &[Remote]) {
        self.net.trials.clear();
        let since = self.ports.clock.now();
        for (at, remote) in to.iter().enumerate() {
            let Some(head) = self.trial_head(lang, remote) else {
                self.tell_compare(take, at, CloudNote::Fault(CloudFault::Gone), 0);
                continue;
            };
            let stop = Arc::new(AtomicBool::new(false));
            let head = Some(head);
            let trial = Trial {
                head,
                stop,
                run: None,
                since,
            };
            self.net.trials.insert((take, at), trial);
        }
        if !self.net.trials.is_empty() {
            self.hand((Owner::Compare, take), ONCE);
        }
    }

    /// The speech run `lane` handed over: a dictation's upload, or Compare's cloud models.
    pub(super) fn handed(
        &mut self,
        (owner, take): Lane,
        round: u32,
        pieces: Result<Vec<SecretBytes>, SpeechError>,
    ) {
        match owner {
            Owner::Flow => self.spoken(take, round, pieces),
            Owner::Compare => self.trial_spoken(take, pieces),
        }
    }

    /// Compare's run `take` handed its recording over: each waiting model's job reads its key and sends a copy; no
    /// speech fails them all.
    pub(super) fn trial_spoken(
        &mut self,
        take: u32,
        pieces: Result<Vec<SecretBytes>, SpeechError>,
    ) {
        let pieces = pieces.unwrap_or_default();
        let mut ats: Vec<usize> = (self.net.trials.keys())
            .filter(|(t, _)| *t == take)
            .map(|(_, at)| *at)
            .collect();
        ats.sort_unstable();
        let since = self.ports.clock.now();
        for at in ats {
            let Some(trial) = self.net.trials.get_mut(&(take, at)) else {
                continue;
            };
            trial.since = since;
            let (head, stop) = (trial.head.take(), Arc::clone(&trial.stop));
            match head.filter(|_| !pieces.is_empty()) {
                Some(head) => self.post_trial((take, at), head, pieces.clone(), stop),
                None => self.trial_note(take, at, CloudNote::Fault(CloudFault::Gone)),
            }
        }
    }

    /// A note from model `at` of Compare's run `take`, with its wait; its end drops the upload.
    pub(super) fn trial_note(&mut self, take: u32, at: usize, note: CloudNote) {
        let Some(trial) = self.net.trials.get(&(take, at)) else {
            return;
        };
        let waited = self
            .ports
            .clock
            .now()
            .saturating_since(trial.since)
            .as_millis();
        if matches!(note, CloudNote::Done | CloudNote::Fault(_)) {
            self.net.trials.remove(&(take, at));
        }
        self.tell_compare(take, at, note, u32::try_from(waited).unwrap_or(u32::MAX));
    }

    /// Model `at`'s service run opened: held until it ends, or dropped at once when its upload went.
    pub(super) fn tried(&mut self, take: u32, at: usize, run: Wired) {
        match self.net.trials.get_mut(&(take, at)) {
            Some(trial) => trial.run = Some(run),
            None => shut(run),
        }
    }

    /// Drops Compare's run `take`'s uploads, which cancels their requests.
    pub(super) fn halt_trials(&mut self, take: u32) {
        self.net.trials.retain(|(t, _), _| *t != take);
    }

    /// Tells the panel model `at`'s note.
    fn tell_compare(&mut self, take: u32, at: usize, note: CloudNote, wait_ms: u32) {
        let dos = self.panel.compare_net(take, at, note, wait_ms);
        self.dos(dos);
    }

    /// Reads the key and sends, in a job off the actor; its notes and its run come back through the inbox.
    fn post_trial(
        &mut self,
        (take, at): (u32, usize),
        head: Head,
        pieces: Vec<SecretBytes>,
        stop: Arc<AtomicBool>,
    ) {
        let (back, cloud) = (self.ports.back.clone(), self.ports.cloud.clone());
        let keys = Keys::new(Arc::clone(&self.ports.shelf), self.net.hidden);
        let job: Job = Box::new(move || {
            let notes = back.clone();
            let tell: Tell =
                Box::new(move |note| drop(notes.send(Msg::Back(Back::Trial { take, at, note }))));
            let back_of = match deliver(cloud.as_deref(), &keys, head, pieces, &stop, tell) {
                Ok(Some(run)) => Back::Tried { take, at, run },
                Ok(None) => return,
                Err(fault) => Back::Trial {
                    take,
                    at,
                    note: CloudNote::Fault(fault),
                },
            };
            // Only a stopped actor misses it, and the run then drops, which cancels it.
            let _ = back.send(Msg::Back(back_of));
        });
        if !(self.ports.spawn)(job) {
            self.trial_note(take, at, CloudNote::Fault(CloudFault::Gone));
        }
    }

    /// The head of model `remote`'s ask for `lang`, while its service is on, its terms confirmed and the model usable.
    fn trial_head(&self, lang: &str, remote: &Remote) -> Option<Head> {
        let s = &self.held.settings;
        let rules = Cloud::new(services::table().ok()?, self.net.hidden);
        let choice = rules.model_for(s, lang, &remote.service, remote.model.expose())?;
        head_of(s, &choice).filter(|_| !choice.ask)
    }
}

//! Compare's cloud models on the actor (edge 12): once Compare's run hands its recording over, a copy goes to each
//! ticked model through `CloudSpeech`, with the dictation's rules: its service still on and ready for the language, the
//! key read in the job just before `send`, none in a hidden run. Cancel, Back, Close or a newer run drops the uploads.

use kx_module_api::ports::{CloudFault, CloudNote, SpeechError};
use kx_module_api::{Mono, SecretBytes};

use super::ask::{Head, Pending, Wired, head_of, shut};
use super::lanes::{Lane, Owner};
use super::{Actor, Back};
use crate::compare::Remote;

/// The round Compare's hand-over is asked for, as its run hands its recording over once.
const ONCE: u32 = 0;

/// One cloud model's upload in a Compare run; dropped, it sends nothing more.
pub(super) struct Trial {
    pending: Pending,
    /// When the speech went, for the model's wait.
    since: Mono,
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
            let pending = Pending::new(head);
            let trial = Trial { pending, since };
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
            let job = trial.pending.start().filter(|_| !pieces.is_empty());
            let note = move |note| Back::Trial { take, at, note };
            let opened = move |run| Back::Tried { take, at, run };
            if !job.is_some_and(|job| self.post(job, pieces.clone(), note, opened)) {
                self.trial_note(take, at, CloudNote::Fault(CloudFault::Gone));
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
            Some(trial) => trial.pending.run = Some(run),
            None => shut(run),
        }
    }

    /// Drops Compare's run `take`'s uploads, which cancels their requests.
    pub(super) fn halt_trials(&mut self, take: u32) {
        self.net.trials.retain(|(t, _), _| *t != take);
    }

    /// Tells the panel model `at`'s note, with when it came on the wall clock.
    fn tell_compare(&mut self, take: u32, at: usize, note: CloudNote, wait_ms: u32) {
        let now = self.ports.calendar.now();
        let dos = self.panel.compare_net(take, at, note, wait_ms, now);
        self.dos(dos);
    }

    /// The head of model `remote`'s ask for `lang`, while its service is on, its terms confirmed and the model usable.
    fn trial_head(&self, lang: &str, remote: &Remote) -> Option<Head> {
        let s = &self.held.settings;
        let rules = self.held.cloud()?;
        let choice = rules.model_for(s, lang, &remote.service, remote.model.expose())?;
        head_of(s, &choice).filter(|_| !choice.ask)
    }
}

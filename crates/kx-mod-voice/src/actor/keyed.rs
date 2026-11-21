//! The key page's effects (ADR-0047): a key sealed or removed, which the flow hears so a cloud dictation under way
//! checks its own; the key's length for its dots; its test off the actor; Paste through the clipboard history's
//! `ClipTake`, as Quick-fill's; and the table's own link in the browser. No key is logged or kept beyond its blob.

use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_module_api::Secret;
use kx_module_api::ports::{
    ClipTake, ClipTaken, CloudFault, CloudSpeech, CloudTest, Link, MAX_KEY_BYTES, WebLink,
};

use super::{Actor, Back, Job, Msg};
use crate::flow::In;
use crate::keys::{Keys, dots_of};
use crate::panel::Do;

impl Actor {
    /// Carries out one of the key page's effects.
    pub(super) fn key_do(&mut self, done: Do) {
        match done {
            Do::SetKey { service, key } => self.seal(&service, key),
            Do::RemoveKey(service) => self.unseal(&service),
            Do::Dots(service) => {
                let dots = self.keys().dots(&service);
                let dots = dots.inspect_err(
                    |error| tracing::warn!(%error, "a cloud key's length was not read"),
                );
                self.panel.dots(&service, dots.unwrap_or(0));
            }
            Do::Test { service, ticket } => self.test_key(&service, ticket),
            Do::Take(round) => self.take_clip(round),
            Do::Link(link) => self.open_link(link),
            _ => {}
        }
    }

    /// The actor's one set of keys on the voice shelf, none in a hidden run, which `Offer` reads too.
    pub(super) fn keys(&self) -> Keys {
        self.net.keys.clone()
    }

    /// Reads which services have a key, from the blobs' names alone, as the panel opens; a failed listing keeps the
    /// last good one, so a passing fault never reads as a missing key.
    pub(super) fn read_keyed(&mut self) {
        match self.keys().listed() {
            Ok(listed) => self.held.keyed = listed,
            Err(error) => tracing::warn!(%error, "the cloud keys could not be listed"),
        }
    }

    /// Deletes the keys of rows no longer offered, as an update hid them; none with no table. A hidden run's keys list
    /// nothing, so it deletes none: `keys` is the one hidden-run guard.
    pub(super) fn sweep_keys(&mut self) {
        let Some(table) = self.held.table else {
            return;
        };
        let keys = self.keys();
        let listed = keys.listed();
        let listed = listed
            .inspect_err(|error| tracing::warn!(%error, "the cloud keys could not be listed"));
        let gone = |id: &&String| table.get(id).is_none_or(|row| !row.offered);
        for id in listed.unwrap_or_default().iter().filter(gone) {
            if let Err(error) = keys.remove(id) {
                tracing::warn!(%error, "a hidden row's cloud key was not removed");
            }
        }
    }

    /// Seals `key` as `service`'s; the panel shows its dots, and the flow hears the keys changed.
    fn seal(&mut self, service: &str, key: Secret) {
        let dots = dots_of(&key);
        let set = self.keys().set(service, key).map(|()| dots);
        if let Err(error) = set {
            tracing::warn!(%error, "a cloud key was not kept");
        } else {
            self.held.keyed.insert(service.to_owned());
        }
        self.panel.key_set(service, set);
        self.step(In::Keys);
    }

    /// Removes `service`'s key; the flow hears it, so an upload under way that needs it stops. A refused removal keeps
    /// the key listed and in use, and the panel says so.
    fn unseal(&mut self, service: &str) {
        let removed = self.keys().remove(service);
        match &removed {
            Ok(()) => drop(self.held.keyed.remove(service)),
            Err(error) => tracing::warn!(%error, "a cloud key was not removed"),
        }
        let dos = self.panel.key_removed(service, removed);
        self.dos(dos);
        self.step(In::Keys);
    }

    /// Tests `service`'s key in a job off the actor: the key is read there, just before the call (carry 3).
    fn test_key(&mut self, service: &str, ticket: u32) {
        let (Some(head), Some(cloud)) = (self.test_head(service), self.ports.cloud.clone()) else {
            return self.panel.tested(ticket, Err(CloudFault::Gone), None);
        };
        let (keys, back, service) = (self.keys(), self.ports.back.clone(), service.to_owned());
        let job: Job = Box::new(move || {
            let answer = test_with(cloud.as_ref(), &keys, &service, head);
            // Only a stopped actor misses it.
            let _ = back.send(Msg::Back(Back::Tested { ticket, answer }));
        });
        if !(self.ports.spawn)(job) {
            self.panel.tested(ticket, Err(CloudFault::Gone), None);
        }
    }

    /// The test's ask without its key, from the row and `[voice]`; `None` for a row with no test or a refused address.
    fn test_head(&self, service: &str) -> Option<CloudTest> {
        let s = &self.held.settings;
        let row = self.held.cloud()?.row(service)?;
        let entry = s.services.get(service)?;
        Some(CloudTest {
            job: s.job(),
            shape: row.shape_for(None, entry.keep),
            url: row.test_url(&entry.address)?,
            header: row.header.clone(),
            key: Secret::default(),
            wait_s: s.net_wait_s,
        })
    }

    /// Asks the clipboard history for its text as Paste's `round`, waited for up to `take_ms`; an answer comes as
    /// `ClipTaken`.
    fn take_clip(&mut self, round: u32) {
        self.pasting.take = Some((round, self.ports.clock.now()));
        let take = ClipTake::new(crate::ID, round, MAX_KEY_BYTES);
        if let Err(error) = self.ports.bus.publish(&take) {
            tracing::warn!(%error, "the clipboard's text could not be asked for");
        }
    }

    /// The clipboard history's answer to a Paste, which ends its wait.
    pub(super) fn clip_taken(&mut self, taken: ClipTaken) {
        if self
            .pasting
            .take
            .is_some_and(|(round, _)| round == taken.round)
        {
            self.pasting.take = None;
        }
        let dos = self.panel.pasted(taken.round, taken.text);
        self.dos(dos);
    }

    /// Opens a link from the services table in a job, as the shell may wait.
    fn open_link(&self, link: Link) {
        let Some(web) = self.ports.link.clone() else {
            return tracing::warn!("no browser link service");
        };
        let job: Job = Box::new(move || open_with(web.as_ref(), &link));
        if !(self.ports.spawn)(job) {
            tracing::warn!("a link's job did not start");
        }
    }
}

/// Reads `service`'s key and tests it with `head`; no key, or one that cannot be read and is logged by kind, is the key
/// fault.
fn test_with(
    cloud: &dyn CloudSpeech,
    keys: &Keys,
    service: &str,
    mut head: CloudTest,
) -> Result<(), CloudFault> {
    head.key = match keys.get(service) {
        Ok(Some(key)) => key,
        Ok(None) => return Err(CloudFault::Key),
        Err(error) => {
            tracing::warn!(%error, "a cloud key was not read for its test");
            return Err(CloudFault::Key);
        }
    };
    let tested = catch_unwind(AssertUnwindSafe(|| cloud.test(head)));
    tested.unwrap_or(Err(CloudFault::Gone))
}

/// Opens `link`; a refusal or a panic in the port is logged by kind.
fn open_with(web: &dyn WebLink, link: &Link) {
    match catch_unwind(AssertUnwindSafe(|| web.open(link))) {
        Ok(Ok(())) => {}
        Ok(Err(error)) => tracing::warn!(%error, "a link did not open"),
        Err(_) => tracing::warn!("the link service broke"),
    }
}

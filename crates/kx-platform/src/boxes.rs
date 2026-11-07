//! `Boxes` over the vault worker (ADR-0004, ADR-0038 decision 6): each session's `kx-boxes` thread runs the neutral
//! worker host on the vault job, one call at a time within `deadline_ms`. Passwords and parts cross in wiped copies,
//! and a late, dead or lying worker is a `BoxError`, never a hang.

mod ask;
mod host;
mod reply;

use crate::worker::{Host, Launch, Session, Spawner, Times, VaultFrames};
use kx_ipc::{VAULT_JOB, VAULT_MEMORY_MB};
use kx_module_api::ports::{
    BoxConfig, BoxError, BoxPart, BoxRun, Boxes, KeepassImport, Opened, Padlock,
};
use kx_module_api::{Clock, Redacted, Secret};
use std::sync::Arc;

/// The session thread's name.
const THREAD: &str = "kx-boxes";

impl Launch {
    /// The vault job: `vault` alone, under `VAULT_MEMORY_MB`, what scrypt at the top work factor and its boxes need.
    #[must_use]
    pub fn vault() -> Self {
        Self {
            args: vec![VAULT_JOB.to_owned()],
            memory_mb: VAULT_MEMORY_MB,
        }
    }
}

/// The host's times from `cfg`.
const fn times(cfg: &BoxConfig) -> Times {
    Times {
        deadline_ms: cfg.deadline_ms,
        idle_s: cfg.idle_s,
        restart_ms: cfg.restart_ms,
        restart_max_ms: cfg.restart_max_ms,
    }
}

/// Boxes sealed and opened by the worker `S` starts, one `kx-boxes` thread per session.
pub struct WorkerBoxes<S> {
    spawner: Arc<S>,
    clock: Arc<dyn Clock>,
}

impl<S: Spawner<VaultFrames>> WorkerBoxes<S> {
    /// Boxes over `spawner`, whose sessions read the time from `clock`.
    #[must_use]
    pub fn new(spawner: S, clock: Arc<dyn Clock>) -> Self {
        Self {
            spawner: Arc::new(spawner),
            clock,
        }
    }
}

impl<S: Spawner<VaultFrames>> Boxes for WorkerBoxes<S> {
    fn start(&self, cfg: BoxConfig) -> Box<dyn BoxRun> {
        let (spawner, clock) = (Arc::clone(&self.spawner), Arc::clone(&self.clock));
        let host = Host::new(spawner, clock, times(&cfg), Launch::vault());
        Box::new(Run(Session::open(THREAD, host)))
    }
}

/// A session; dropping it ends its thread, which stops the worker. Without the thread every call is `Failed`.
struct Run<S: Spawner<VaultFrames>>(Session<S, VaultFrames>);

impl<S: Spawner<VaultFrames>> Run<S> {
    /// Runs `call` on the session thread, `Failed` without it.
    fn run<T: Send + 'static>(
        &self,
        call: impl FnOnce(&mut Host<S, VaultFrames>) -> Result<T, BoxError> + Send + 'static,
    ) -> Result<T, BoxError> {
        self.0.run(call).unwrap_or(Err(BoxError::Failed))
    }
}

/// Each call copies its inputs into owned buffers, wiped ones for passwords and parts, to move them to the thread.
impl<S: Spawner<VaultFrames>> BoxRun for Run<S> {
    fn padlock(&mut self, password: &Secret, work: u8) -> Result<Padlock, BoxError> {
        let password = password.clone();
        self.run(move |host| host.padlock(password, work))
    }

    fn seal(
        &mut self,
        padlock: &Padlock,
        parts: &[BoxPart],
    ) -> Result<Redacted<Vec<u8>>, BoxError> {
        let (padlock, parts) = (padlock.clone(), parts.to_vec());
        self.run(move |host| host.seal(padlock, parts))
    }

    fn open(&mut self, bytes: &[u8], password: &Secret) -> Result<Opened, BoxError> {
        let (bytes, password) = (bytes.to_vec(), password.clone());
        self.run(move |host| host.open(bytes, password))
    }

    fn check(&mut self, padlock: &Padlock, password: &Secret) -> Result<bool, BoxError> {
        let (padlock, password) = (padlock.clone(), password.clone());
        self.run(move |host| host.check(padlock, password))
    }

    fn keepass(&mut self, bytes: &[u8], password: &Secret) -> Result<KeepassImport, BoxError> {
        let (bytes, password) = (bytes.to_vec(), password.clone());
        self.run(move |host| host.keepass(bytes, password))
    }
}

#[cfg(test)]
mod tests;

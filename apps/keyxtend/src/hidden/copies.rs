//! The hidden copies a normal run finds (ADR-0037 decision 4): one question for each copy the user did not keep. Yes
//! deletes it on a thread of its own; No adds its package to `[kernel] kept_copies`, so it is never asked about again.

use super::about_app;
use kx_kernel::{KEPT_KEY, KERNEL, Kernel};
use kx_module_api::{Notice, NoticeAsk, keys};
use kx_platform::ui::{Hidden, HiddenCopy};
use kx_ui::Hub;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use toml::Value;

/// The thread that deletes one copy, as debuggers show it.
const DELETE_THREAD: &str = "kx-copy-delete";

/// The copies asked about and not yet answered, each by the notice the app pushed, the port that deletes them, the hub
/// that asks, and the deletes in flight.
pub struct Asked {
    port: Arc<dyn Hidden>,
    hub: Arc<Hub>,
    open: Vec<(Notice, HiddenCopy)>,
    deleting: Deletes,
}

/// The deletes Yes started, each on a thread of its own; its clones share them, so the app can wait as it ends.
#[derive(Clone, Default)]
pub struct Deletes(Arc<Mutex<Vec<JoinHandle<()>>>>);

impl Deletes {
    /// Waits for every delete started so far; one that panicked is logged.
    pub fn settle(&self) {
        let started = std::mem::take(&mut *self.lock());
        for delete in started {
            if delete.join().is_err() {
                tracing::error!("a hidden copy's delete ended in a panic");
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<JoinHandle<()>>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Asked {
    /// Lists the copies through `port` and asks once about each one `kernel`'s `kept_copies` does not hold. A failed
    /// list is logged and asks nothing.
    pub fn ask(port: Arc<dyn Hidden>, kernel: &Kernel, hub: &Arc<Hub>) -> Self {
        let kept = kept(kernel);
        let listed = port.copies().unwrap_or_else(|e| {
            tracing::warn!(error = %e, "the hidden copies could not be listed, so none is asked about");
            Vec::new()
        });
        let fresh = listed
            .into_iter()
            .filter(|c| !kept.iter().any(|k| k == c.package()));
        let open: Vec<_> = fresh.map(|c| (question(&c), c)).collect();
        // Two copies of one app ask the same question, which the hub shows once; `answer` asks the next.
        for (notice, _) in &open {
            hub.notice(notice.clone());
        }
        let hub = Arc::clone(hub);
        Self {
            port,
            hub,
            open,
            deleting: Deletes::default(),
        }
    }

    /// The deletes its Yes answers start, for the app to wait on once the session is free.
    #[must_use]
    pub fn deletes(&self) -> Deletes {
        self.deleting.clone()
    }

    /// Takes the answer to `notice`: Yes deletes its copy off the kernel's thread, so Try again and Quit never wait for
    /// it; No keeps it for good. An answer to any other notice changes nothing.
    pub fn answer(&mut self, kernel: &mut Kernel, notice: &Notice, yes: bool) {
        let Some(at) = self.open.iter().position(|(n, _)| n == notice) else {
            return;
        };
        let (_, copy) = self.open.remove(at);
        if yes {
            self.remove(copy);
        } else {
            keep(kernel, &copy);
        }
        if self.open.iter().any(|(n, _)| n == notice) {
            self.hub.notice(notice.clone());
        }
    }

    /// Deletes `copy` on a thread of its own; a failure is logged and shown, and the next run asks again.
    fn remove(&mut self, copy: HiddenCopy) {
        let failed = about_app(keys::HIDDEN_COPY_FAILED, copy.app());
        let (port, hub, told) = (
            Arc::clone(&self.port),
            Arc::clone(&self.hub),
            failed.clone(),
        );
        let delete = move || {
            if let Err(e) = port.remove(&copy) {
                tracing::warn!(error = %e, "a hidden copy was not deleted");
                hub.notice(told);
            }
        };
        match thread::Builder::new()
            .name(DELETE_THREAD.to_owned())
            .spawn(delete)
        {
            Ok(handle) => self.deleting.lock().push(handle),
            Err(e) => {
                tracing::warn!(error = %e, "a hidden copy was not deleted, as no thread started");
                self.hub.notice(failed);
            }
        }
    }
}

/// The question about `copy`, naming its app, with Yes and No.
fn question(copy: &HiddenCopy) -> Notice {
    Notice {
        ask: Some(&NoticeAsk::YES_NO),
        ..about_app(keys::HIDDEN_COPY, copy.app())
    }
}

/// The package folders whose copies the user kept.
fn kept(kernel: &Kernel) -> Vec<String> {
    let settings = kernel.kernel_settings();
    settings.map(|s| s.kept_copies).unwrap_or_default()
}

/// Adds `copy`'s package to `kept_copies` through the kernel, which saves it; a package kept already stays once.
fn keep(kernel: &mut Kernel, copy: &HiddenCopy) {
    let mut list = kept(kernel);
    if list.iter().any(|k| k == copy.package()) {
        return;
    }
    list.push(copy.package().to_owned());
    let value = Value::Array(list.into_iter().map(Value::String).collect());
    if let Err(e) = kernel.set_setting(KERNEL, KEPT_KEY, value) {
        tracing::warn!(
            error = e.kind(),
            "a kept hidden copy was not noted, so it is asked about again"
        );
    }
}

#[cfg(test)]
mod tests;

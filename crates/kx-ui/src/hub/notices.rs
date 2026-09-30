//! The notices waiting for the banner and the ones it shows, each shown once until closed.

use crate::I18n;
use kx_module_api::{ModuleId, Notice, keys};
use serde::Serialize;

/// A notice as the banner shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Shown {
    /// Names it for Try again or OK.
    pub id: u64,
    /// Its text in the UI language.
    pub text: String,
    /// It offers Try again, as a failed module does.
    pub retry: bool,
}

/// Notices not yet taken, and the ones the banner shows until the user closes them.
#[derive(Debug, Default)]
pub(crate) struct Notices {
    next: u64,
    waiting: Vec<Notice>,
    open: Vec<(u64, Notice)>,
}

impl Notices {
    /// Queues `notice` unless the same one waits or shows; true when it was queued.
    pub fn push(&mut self, notice: Notice) -> bool {
        let seen = self.waiting.contains(&notice) || self.open.iter().any(|(_, n)| *n == notice);
        if !seen {
            self.waiting.push(notice);
        }
        !seen
    }

    /// The waiting notices as texts in `i18n`, oldest first; they show from now on.
    pub fn take(&mut self, i18n: &I18n) -> Vec<Shown> {
        let mut shown = Vec::new();
        for notice in std::mem::take(&mut self.waiting) {
            self.next += 1;
            shown.push(Shown {
                id: self.next,
                text: i18n.notice(&notice),
                retry: retry_of(&notice).is_some(),
            });
            self.open.push((self.next, notice));
        }
        shown
    }

    /// Closes shown notice `id`, and names its module when it offers Try again.
    pub fn close(&mut self, id: u64) -> Option<ModuleId> {
        let at = self.open.iter().position(|(i, _)| *i == id)?;
        retry_of(&self.open.remove(at).1)
    }
}

/// The module Try again restarts: only a failed module's notice offers it.
fn retry_of(notice: &Notice) -> Option<ModuleId> {
    notice.module.filter(|_| notice.key == keys::MODULE_FAILED)
}

#[cfg(test)]
mod tests;

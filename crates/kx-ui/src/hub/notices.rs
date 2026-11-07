//! The notices waiting for the banner and the ones it shows, each shown once until closed or answered.

use crate::I18n;
use kx_module_api::{ModuleId, Notice, NoticeAsk, keys};
use serde::Serialize;

/// A notice as the banner shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Shown {
    /// Names it for Try again, OK, Yes or No.
    pub id: u64,
    /// Its text in the keyboard's language.
    pub text: String,
    /// It offers Try again, as a failed module does.
    pub retry: bool,
    /// Its Yes and No texts when it asks.
    pub ask: Option<Answers>,
}

/// An asking notice's two answers, in the keyboard's language.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Answers {
    /// The main button's text.
    pub yes: String,
    /// The other button's text.
    pub no: String,
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
            shown.push(show(self.next, &notice, i18n));
            self.open.push((self.next, notice));
        }
        shown
    }

    /// The notices shown and not yet closed, as texts in `i18n`, oldest first.
    pub fn shown(&self, i18n: &I18n) -> Vec<Shown> {
        self.open.iter().map(|(id, n)| show(*id, n, i18n)).collect()
    }

    /// Closes shown notice `id` and gives it back, so the hub answers it as it asks; none when it does not show.
    pub fn close(&mut self, id: u64) -> Option<Notice> {
        let at = self.open.iter().position(|(i, _)| *i == id)?;
        Some(self.open.remove(at).1)
    }
}

/// Notice `notice`, shown as `id`, with its texts in `i18n`.
fn show(id: u64, notice: &Notice, i18n: &I18n) -> Shown {
    let answers = |ask: &NoticeAsk| Answers {
        yes: i18n.text(ask.yes),
        no: i18n.text(ask.no),
    };
    Shown {
        id,
        text: i18n.notice(notice),
        retry: retry_of(notice).is_some(),
        ask: notice.ask.map(answers),
    }
}

/// The module Try again restarts: only a failed module's notice offers it.
pub(super) fn retry_of(notice: &Notice) -> Option<ModuleId> {
    notice.module.filter(|_| notice.key == keys::MODULE_FAILED)
}

#[cfg(test)]
mod tests;

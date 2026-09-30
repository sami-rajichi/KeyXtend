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
mod tests {
    use super::*;
    use crate::LANGS;
    use kx_module_api::{NoticeArg, args};

    const KEYBOARD: ModuleId = ModuleId::new("keyboard");

    fn failed() -> Notice {
        Notice {
            key: keys::MODULE_FAILED,
            module: Some(KEYBOARD),
            args: vec![(args::MODULE, NoticeArg::Text(KEYBOARD.to_string()))],
        }
    }

    fn plain(key: &'static str) -> Notice {
        Notice {
            key,
            module: None,
            args: Vec::new(),
        }
    }

    #[test]
    fn notices_show_once_in_order_with_their_text() {
        let mut n = Notices::default();
        assert!(n.push(failed()));
        assert!(n.push(plain(keys::KEYS_NOT_SENT)));
        let shown = n.take(&I18n::new(&LANGS[0]));
        assert_eq!(shown.len(), 2);
        assert!(shown[0].text.contains("Keyboard") && shown[0].retry);
        assert!(!shown[1].retry);
        assert_ne!(shown[0].id, shown[1].id);
        assert!(n.take(&I18n::new(&LANGS[0])).is_empty(), "taken once");
    }

    #[test]
    fn the_same_notice_waits_or_shows_only_once() {
        let mut n = Notices::default();
        assert!(n.push(failed()));
        assert!(!n.push(failed()), "already waiting");
        let id = n.take(&I18n::new(&LANGS[0]))[0].id;
        assert!(!n.push(failed()), "already shown");
        assert_eq!(n.close(id), Some(KEYBOARD));
        assert!(n.push(failed()), "closed, so it may show again");
    }

    #[test]
    fn only_a_failed_module_offers_try_again() {
        let mut n = Notices::default();
        let mut settings = plain(keys::SETTINGS_RESET);
        settings.module = Some(KEYBOARD);
        n.push(settings);
        n.push(plain(keys::KEYS_NOT_SENT));
        let ids: Vec<u64> = n.take(&I18n::new(&LANGS[0])).iter().map(|s| s.id).collect();
        assert_eq!(n.close(ids[0]), None);
        assert_eq!(n.close(ids[1]), None);
        assert_eq!(n.close(ids[1]), None, "closed already");
        assert_eq!(n.close(u64::MAX), None, "never shown");
    }

    #[test]
    fn texts_follow_the_language_when_taken() {
        let mut n = Notices::default();
        n.push(failed());
        let ar = crate::find("ar").unwrap();
        let shown = n.take(&I18n::new(ar));
        assert_eq!(shown[0].text, I18n::new(ar).notice(&failed()));
    }
}

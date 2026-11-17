//! A service's key page (ADR-0047, design "Keys"): the key set from its line or Paste, sealed and shown only as dots,
//! its Test off the actor, Remove, the typed address and Other model kept in `services`, and the keep-nothing switch.
//! A Paste's or a test's answer is dropped once the page or the key changed.

use kx_module_api::ports::{
    AddressFault, BarHint, ClipText, CloudButton, CloudFault, HttpsUrl, KeyCheck, LINK_SCHEME,
};
use kx_module_api::{Redacted, Secret, Zeroizing};

use super::cloud::edit;
use super::{Do, Page, Panel};
use crate::keys::KeyError;

use crate::view::Facts;

impl Panel {
    /// The key page's Paste, Test or Remove on service `id`.
    pub(super) fn key_button(
        &mut self,
        id: &str,
        button: CloudButton,
        facts: &Facts<'_>,
    ) -> Vec<Do> {
        let round = self.cloud.next();
        match button {
            CloudButton::Paste => {
                self.cloud.take = Some(round);
                vec![Do::Take(round)]
            }
            CloudButton::Test if facts.keyed.contains(id) => {
                (self.cloud.test, self.cloud.check) = (Some(round), KeyCheck::Testing);
                vec![Do::Test {
                    service: id.to_owned(),
                    ticket: round,
                }]
            }
            CloudButton::Remove => {
                (self.cloud.dots, self.cloud.check) = (0, KeyCheck::Untested);
                vec![Do::RemoveKey(id.to_owned())]
            }
            _ => Vec::new(),
        }
    }

    /// The key line's text, set as the shown service's key; a test or Paste waited for is dropped.
    pub(super) fn set_key(&mut self, key: &Secret) -> Vec<Do> {
        let Some(id) = self.cloud.shown.clone() else {
            return Vec::new();
        };
        self.cloud.next();
        vec![Do::SetKey {
            service: id,
            key: trimmed(key),
        }]
    }

    /// The key was sealed with `dots` characters, or why not; the actor tells the flow the keys changed.
    pub fn key_set(&mut self, id: &str, set: Result<u32, KeyError>) {
        if self.cloud.shown.as_deref() != Some(id) {
            return;
        }
        match set {
            Ok(dots) => (self.cloud.dots, self.cloud.check) = (dots, KeyCheck::Untested),
            Err(KeyError::Unfit) => self.cloud.check = KeyCheck::Unfit,
            Err(_) => self.note = Some(BarHint::Fault),
        }
    }

    /// The clipboard's text a Paste of `round` took: set as the key while the page and round are still the same.
    pub fn pasted(&mut self, round: u32, text: ClipText) -> Vec<Do> {
        let waited = self.cloud.take == Some(round) && self.page == Some(Page::Lines);
        if !waited {
            return Vec::new();
        }
        self.cloud.take = None;
        match text {
            ClipText::Text(key) => self.set_key(&key),
            ClipText::Empty | ClipText::TooLong => {
                self.cloud.check = KeyCheck::Unfit;
                Vec::new()
            }
            ClipText::Busy => {
                self.note = Some(BarHint::Fault);
                Vec::new()
            }
        }
    }

    /// The test of `ticket` answered; dropped once the user left the page or changed the key.
    pub fn tested(&mut self, ticket: u32, answer: Result<(), CloudFault>) {
        if self.cloud.test != Some(ticket) || self.page != Some(Page::Lines) {
            return;
        }
        self.cloud.test = None;
        self.cloud.check = match answer {
            Ok(()) => KeyCheck::Works,
            Err(CloudFault::Key) => KeyCheck::Refused,
            Err(_) => KeyCheck::Failed,
        };
    }

    /// Service `id`'s key holds `dots` characters, as read when its page opened.
    pub fn dots(&mut self, id: &str, dots: u32) {
        if self.cloud.shown.as_deref() == Some(id) && self.page == Some(Page::Lines) {
            self.cloud.dots = dots;
        }
    }

    /// The address line's text, its scheme and host lowered: kept in `services` when HTTPS on a host name the row
    /// takes, else shown with why (edge 13). A changed address keeps the picks, and a cleared one sends them to the
    /// local default.
    pub(super) fn type_address(&mut self, text: &Redacted<String>, facts: &Facts<'_>) -> Vec<Do> {
        let Some(id) = self.lines_id() else {
            return Vec::new();
        };
        let typed = HttpsUrl::lowered(text.expose());
        let out = edit(facts, |c, s| c.type_address(s, &id, &typed));
        let same = facts
            .settings
            .services
            .get(&id)
            .is_some_and(|e| e.address == typed);
        let row = facts.cloud.and_then(|c| c.row(&id));
        self.cloud.refused = match row {
            Some(_) if out.is_empty() && !same => Some((text.clone(), fault(&typed))),
            _ => None,
        };
        out
    }

    /// The Other model line's id, kept in `services`; a new id drops the picks of the one before.
    pub(super) fn type_model(&mut self, text: &Redacted<String>, facts: &Facts<'_>) -> Vec<Do> {
        let Some(id) = self.lines_id() else {
            return Vec::new();
        };
        edit(facts, |c, s| c.type_model(s, &id, text.expose()))
    }

    /// `OpenRouter`'s keep-nothing switch on the key page.
    pub(super) fn keep_nothing(&mut self, on: bool, facts: &Facts<'_>) -> Vec<Do> {
        let Some(id) = self.lines_id() else {
            return Vec::new();
        };
        edit(facts, |c, s| c.keep(s, &id, on))
    }

    /// The key page's service, while it shows.
    fn lines_id(&self) -> Option<String> {
        self.cloud
            .shown
            .clone()
            .filter(|_| self.page == Some(Page::Lines))
    }
}

/// Why a typed `text`, its scheme and host lowered, was refused: no `https://`, too long, no host name, or a host
/// outside the row's domain.
fn fault(text: &str) -> AddressFault {
    if !text.starts_with(LINK_SCHEME) {
        AddressFault::Https
    } else if text.len() > HttpsUrl::MAX_BYTES {
        AddressFault::Long
    } else if HttpsUrl::new(text).is_none() {
        AddressFault::Host
    } else {
        AddressFault::Domain
    }
}

/// `key` without the spaces and line breaks a copy may carry at its ends, in a wiped buffer.
fn trimmed(key: &Secret) -> Secret {
    Redacted::new(Zeroizing::new(key.expose().trim().to_owned()))
}

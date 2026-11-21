//! A service's key page (ADR-0047, design "Keys"): the key set from its line or Paste, sealed and shown only as dots,
//! its Test off the actor, Remove, the typed address and Other model kept in `services`, each line's Paste, and the
//! keep-nothing switch. A Paste's or a test's answer is dropped once the page or the key changed.

use kx_module_api::ports::{
    AddressFault, BarHint, ClipText, CloudButton, CloudFault, CloudLine, HttpsUrl, KeyCheck,
    LINK_SCHEME, Renewal,
};
use kx_module_api::{Redacted, Secret, Zeroizing};

use super::cloud::edit;
use super::typing::line_max;
use super::{Do, Page, Panel};
use crate::keys::KeyError;
use crate::view::Facts;

impl Panel {
    /// The key page's Test or Remove on service `id`.
    pub(super) fn key_button(
        &mut self,
        id: &str,
        button: CloudButton,
        facts: &Facts<'_>,
    ) -> Vec<Do> {
        let round = self.cloud.next();
        match button {
            CloudButton::Test if facts.keyed.contains(id) => {
                (self.cloud.test, self.cloud.check) = (Some(round), KeyCheck::Testing);
                vec![Do::Test {
                    service: id.to_owned(),
                    ticket: round,
                }]
            }
            CloudButton::Remove => {
                self.cloud.typed.key = None;
                (self.cloud.dots, self.cloud.check) = (0, KeyCheck::Untested);
                vec![Do::RemoveKey(id.to_owned())]
            }
            _ => Vec::new(),
        }
    }

    /// Paste on the shown `line`: asks the clipboard history for at most what the line holds; a key typed is wiped.
    pub(super) fn paste(&mut self, line: CloudLine, facts: &Facts<'_>) -> Vec<Do> {
        if !self.shows_line(line, facts) {
            return Vec::new();
        }
        let round = self.cloud.next();
        if line == CloudLine::Key {
            self.cloud.typed.key = None;
        }
        self.cloud.take = Some((round, line));
        vec![Do::Take {
            round,
            max: line_max(line),
        }]
    }

    /// The key line's text, set as the shown service's key; a test or Paste waited for, and a key typed, are dropped.
    pub(super) fn set_key(&mut self, key: &Secret) -> Vec<Do> {
        let Some(id) = self.cloud.shown.clone() else {
            return Vec::new();
        };
        self.cloud.next();
        self.cloud.typed.key = None;
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
            Err(KeyError::Unfit) => self.cloud.unkept = Some(KeyCheck::Unfit),
            Err(_) => self.note = Some(BarHint::Fault),
        }
    }

    /// Service `id`'s key was removed, or why not: a refused removal shows the fault band and reads its dots again.
    pub fn key_removed(&mut self, id: &str, removed: Result<(), KeyError>) -> Vec<Do> {
        if removed.is_ok() {
            return Vec::new();
        }
        self.note = Some(BarHint::Fault);
        vec![Do::Dots(id.to_owned())]
    }

    /// The clipboard's text a Paste of `round` took, while the page and round are still the same: set as the key, or
    /// put on its address or model line to save; a busy clipboard or a text that cannot go shows why.
    pub fn pasted(&mut self, round: u32, text: ClipText, facts: &Facts<'_>) -> Vec<Do> {
        let line = match self.cloud.take {
            Some((r, line)) if r == round && self.page == Some(Page::Lines) => line,
            _ => return Vec::new(),
        };
        self.cloud.take = None;
        match (line, text) {
            (_, ClipText::Busy) => self.cloud.unkept = Some(KeyCheck::Busy),
            (CloudLine::Key, ClipText::Text(key)) => return self.set_key(&key),
            (CloudLine::Key, _) => self.cloud.unkept = Some(KeyCheck::Unfit),
            (line, ClipText::Text(text)) => self.put_line(line, text.expose().trim(), facts),
            (line, ClipText::Empty) => self.unfit(line, true, facts),
            (line, ClipText::TooLong) => self.unfit(line, false, facts),
        }
        Vec::new()
    }

    /// Puts a pasted `text` on the address or model `line`, to save, and aims the letters at it; an empty or too long
    /// one shows why instead.
    fn put_line(&mut self, line: CloudLine, text: &str, facts: &Facts<'_>) {
        if text.is_empty() || text.len() > line_max(line) {
            return self.unfit(line, text.is_empty(), facts);
        }
        self.cloud.typed.put(line, text);
        if line == CloudLine::Address {
            self.cloud.refused = None;
        } else {
            self.cloud.refused_model = None;
        }
    }

    /// An address or model a Paste could not put on `line`, `empty` or too long: the line keeps its text and says why,
    /// an empty clipboard on the address line by the HTTPS it needs.
    fn unfit(&mut self, line: CloudLine, empty: bool, facts: &Facts<'_>) {
        let shown = Redacted::new(self.line_text(line, facts));
        if line == CloudLine::Model {
            self.cloud.refused_model = Some(shown);
            return;
        }
        let fault = if empty {
            AddressFault::Https
        } else {
            AddressFault::Long
        };
        self.cloud.refused = Some((shown, fault));
    }

    /// The test of `ticket` answered, a refusal by its reason and a used allowance with `renews`; dropped once the user
    /// left the page or changed the key.
    pub fn tested(&mut self, ticket: u32, answer: Result<(), CloudFault>, renews: Option<Renewal>) {
        if self.cloud.test != Some(ticket) || self.page != Some(Page::Lines) {
            return;
        }
        self.cloud.test = None;
        (self.cloud.check, self.cloud.renews_at) = answered(answer, renews);
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

    /// The Other model line's id, kept in `services`; a new id drops the picks of the one before. An id the row takes
    /// but the frames refuse stays on the line with why, until the page changes.
    pub(super) fn type_model(&mut self, text: &Redacted<String>, facts: &Facts<'_>) -> Vec<Do> {
        let Some(id) = self.lines_id() else {
            return Vec::new();
        };
        let out = edit(facts, |c, s| c.type_model(s, &id, text.expose()));
        let same = facts
            .settings
            .services
            .get(&id)
            .is_some_and(|e| &e.model == text.expose());
        let other = facts
            .cloud
            .and_then(|c| c.row(&id))
            .is_some_and(|r| r.other);
        self.cloud.refused_model = (other && out.is_empty() && !same).then(|| text.clone());
        out
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

/// The key page's answer to a test, by its reason, with a used allowance's `renews`; only no answer is `Failed`.
fn answered(
    answer: Result<(), CloudFault>,
    renews: Option<Renewal>,
) -> (KeyCheck, Option<Renewal>) {
    let check = match answer {
        Ok(()) => KeyCheck::Works,
        Err(CloudFault::Key) => KeyCheck::Refused,
        Err(CloudFault::Credit) => KeyCheck::Credit,
        Err(CloudFault::Account) => KeyCheck::Account,
        Err(CloudFault::Region) => KeyCheck::Region,
        Err(CloudFault::Allowance { .. }) => return (KeyCheck::Allowance, renews),
        Err(CloudFault::Rate) => KeyCheck::Rate,
        Err(CloudFault::Strange) => KeyCheck::Strange,
        // No answer; a key test sends no sound, so TooBig, Language and Empty cannot come and read the same.
        Err(
            CloudFault::Network
            | CloudFault::Timeout
            | CloudFault::Gone
            | CloudFault::Range(_)
            | CloudFault::TooBig
            | CloudFault::Language
            | CloudFault::Empty,
        ) => KeyCheck::Failed,
    };
    (check, None)
}

/// Why a typed `text`, its scheme and host lowered, was refused: no `https://`, too long, a bad port on a good host,
/// no host name, a bad path or query, or a host outside the row's domain.
fn fault(text: &str) -> AddressFault {
    if !text.starts_with(LINK_SCHEME) {
        AddressFault::Https
    } else if text.len() > HttpsUrl::MAX_BYTES {
        AddressFault::Long
    } else if HttpsUrl::bad_port(text) {
        AddressFault::Port
    } else if !HttpsUrl::has_host(text) {
        AddressFault::Host
    } else if HttpsUrl::new(text).is_none() {
        AddressFault::Path
    } else {
        AddressFault::Domain
    }
}

/// `key` without the spaces and line breaks a copy may carry at its ends, in a wiped buffer.
fn trimmed(key: &Secret) -> Secret {
    Redacted::new(Zeroizing::new(key.expose().trim().to_owned()))
}

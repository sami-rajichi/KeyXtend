//! A service's key page (design "Keys"): the key shown only as dots, its test, the typed address and Other model lines,
//! the keep-nothing switch, the site where keys are made, and the buttons that act now.

use kx_module_api::Redacted;
use kx_module_api::ports::{CloudButton, KeyCheck, Pages, ServiceSheet, VoiceSheet};

use super::Facts;
use crate::panel::Panel;
use crate::services::{Address, KeyTest, Service};

impl Panel {
    /// The shown service's key page; the Cloud page once cloud no longer offers it.
    pub(super) fn lines_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let pages = self.single();
        let (Some(cloud), Some(row)) = (facts.cloud, self.shown_row(facts)) else {
            return self.cloud_sheet(facts);
        };
        let s = facts.settings;
        let entry = s.services.get(&row.id);
        let typed = matches!(row.address, Address::Typed { .. });
        let kept = entry.map(|e| Redacted::new(e.address.clone()));
        let refused = self.cloud.refused.as_ref();
        let address = refused
            .map(|(text, _)| text.clone())
            .or(kept)
            .filter(|_| typed);
        let sheet = ServiceSheet {
            service: row.shown(),
            mark: cloud.mark(s, row, facts.keyed.contains(&row.id)),
            key: Redacted::new(self.cloud.dots),
            check: self.key_check(row),
            address,
            address_fault: refused.map(|(_, fault)| *fault),
            model: Redacted::new(entry.map(|e| e.model.clone()).unwrap_or_default()),
            model_na: false,
            keep: row.keep.as_ref().map(|_| entry.is_some_and(|e| e.keep)),
            site: row.keys.as_ref().map(|url| url.host().to_owned()),
            acts: self.cloud_acts(facts),
        };
        (VoiceSheet::Service(sheet), pages)
    }

    /// The key page's buttons that act now: Paste always, Test and Remove with a key, the link where keys are made.
    pub(super) fn lines_acts(&self, facts: &Facts<'_>) -> Vec<CloudButton> {
        let Some(row) = self.shown_row(facts) else {
            return Vec::new();
        };
        let keyed = facts.keyed.contains(&row.id);
        let address = facts
            .settings
            .services
            .get(&row.id)
            .map(|e| e.address.as_str());
        let testable = row.test_url(address.unwrap_or_default()).is_some();
        let idle = self.cloud.check != KeyCheck::Testing;
        let mut acts = vec![CloudButton::Paste];
        acts.extend((keyed && testable && idle).then_some(CloudButton::Test));
        acts.extend(keyed.then_some(CloudButton::Remove));
        acts.extend(row.keys.as_ref().map(|_| CloudButton::Link));
        acts.extend([CloudButton::Facts, CloudButton::Off]);
        acts
    }

    /// How the shown key's test stands; a row with no test says so while the key is untested.
    fn key_check(&self, row: &Service) -> KeyCheck {
        match self.cloud.check {
            KeyCheck::Untested if row.test == KeyTest::None => KeyCheck::NoTest,
            check => check,
        }
    }
}

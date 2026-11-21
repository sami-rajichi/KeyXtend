//! A service's key page (design "Keys"): the key shown only as dots, its test, the typed address and Other model lines,
//! the keep-nothing switch, the site where keys are made, and the buttons that act now.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    CloudButton, CloudLine, KeyCheck, LinesPart, LocalTime, Pages, ServiceSheet, VoiceSheet,
};

use super::Facts;
use crate::VoiceSettings;
use crate::cloud::{Chosen, Cloud};
use crate::panel::Panel;
use crate::services::{Address, KeyTest, Service};

impl Panel {
    /// The shown service's key page, on two pages for a typed address (ADR-0008); the Cloud page once cloud no longer
    /// offers it.
    pub(super) fn lines_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let (Some(cloud), Some(row)) = (facts.cloud, self.shown_row(facts)) else {
            return self.cloud_sheet(facts);
        };
        let s = facts.settings;
        let entry = s.services.get(&row.id);
        let typed = matches!(row.address, Address::Typed { .. });
        let (part, pages) = if typed {
            self.parted(&LinesPart::ALL)
        } else {
            (None, self.single())
        };
        let lines = self.typed_lines(facts).unwrap_or_default();
        let typing = self.cloud.typed.at(lines).unwrap_or(CloudLine::Key);
        let text = |line| Redacted::new(self.line_text(line, facts));
        let (check, renews_at) = self.key_check(row);
        let sheet = ServiceSheet {
            service: row.shown(),
            mark: cloud.mark(s, row, facts.keyed.contains(&row.id)),
            key: Redacted::new(self.cloud.dots),
            typed: Redacted::new(self.cloud.typed.key_dots()),
            check,
            renews_at,
            address: typed.then(|| text(CloudLine::Address)),
            address_fault: self.cloud.refused.as_ref().map(|(_, fault)| *fault),
            model: text(CloudLine::Model),
            model_na: model_na(cloud, s, row),
            model_refused: self.cloud.refused_model.is_some(),
            keep: row.keep.as_ref().map(|_| entry.is_some_and(|e| e.keep)),
            site: row.keys.as_ref().map(|url| url.host().to_owned()),
            acts: self.cloud_acts(facts),
            part,
            typing,
            caret: Redacted::new(self.line_caret(typing, facts)),
            saves: (lines.iter().copied())
                .filter(|&l| !self.line_empty(l, facts))
                .collect(),
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

    /// Why a Paste or typed key kept nothing, else how the shown key's test stands, with a used allowance's renewal
    /// while that answer shows; a row with no test says so while the key is untested.
    fn key_check(&self, row: &Service) -> (KeyCheck, Option<LocalTime>) {
        if let Some(unkept) = self.cloud.unkept {
            return (unkept, None);
        }
        match self.cloud.check {
            KeyCheck::Untested if row.test == KeyTest::None => (KeyCheck::NoTest, None),
            KeyCheck::Allowance => (KeyCheck::Allowance, self.cloud.renews_at),
            check => (check, None),
        }
    }
}

/// True when the Other model line's kept id names a model no host of which keeps nothing, while `row`'s keep-nothing
/// switch is on, so the line says why it cannot go (edge 17).
fn model_na(cloud: Cloud<'_>, s: &VoiceSettings, row: &Service) -> bool {
    let typed = s.services.get(&row.id).map(|e| e.model.as_str());
    let typed = typed.filter(|id| !id.is_empty());
    typed.is_some_and(|id| cloud.unavailable(s, row, Chosen::Typed(id)).is_some())
}

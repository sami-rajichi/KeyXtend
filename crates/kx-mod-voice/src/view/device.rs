//! The Device page: Automatic, the processor with its cores and memory, and each card with its memory and its check for
//! the model in use; the latest failed check in the note band; and the device label the home list and Compare show.

use kx_module_api::ports::{
    CardCheck, CardId, Device, DeviceItem, DeviceKind, DeviceLabel, DeviceName, DevicePick,
    DeviceRow, DeviceSheet, FailedCheck, Fingerprint, Pages, PanelButton, Verdict, VoiceSheet,
};

use super::Facts;
use crate::flow::{Ready, Supply};
use crate::panel::{Key, Panel};

impl Panel {
    /// The Device page's shown rows.
    pub(super) fn device_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let print = self.used_print(facts);
        let pick = facts.settings.device();
        let auto = (DevicePick::Auto, DeviceItem::Auto);
        let items = facts
            .devices
            .iter()
            .map(|d| (pick_of(d), self.item(d, facts, print.as_ref())));
        let list = std::iter::once(auto).chain(items).map(|(p, item)| {
            let key = Key::Device(p.clone());
            let row = DeviceRow {
                item,
                used: p == pick,
                tick: self.ticks.contains(&key),
            };
            (key, row)
        });
        let list: Vec<(Key, DeviceRow)> = list.collect();
        let failed = list.iter().find_map(|(_, r)| failed(&r.item));
        let (rows, pages) = self.paged(list);
        (VoiceSheet::Device(DeviceSheet { rows, failed }), pages)
    }

    /// The Device page's buttons that act now: Use for a device not picked, Check for a card while none runs.
    pub(crate) fn device_acts(&self, facts: &Facts<'_>) -> Vec<PanelButton> {
        let Some(Key::Device(pick)) = self.ticked() else {
            return Vec::new();
        };
        let mut acts = Vec::new();
        acts.extend((*pick != facts.settings.device()).then_some(PanelButton::Use));
        let card = matches!(pick, DevicePick::Card(_));
        let free = self.checking.is_none() && self.used_print(facts).is_some();
        acts.extend((card && free).then_some(PanelButton::Check));
        acts
    }

    /// The fingerprint of the model used for the language in front, which the cards' checks go by, once its file was
    /// read.
    pub(crate) fn used_print(&self, facts: &Facts<'_>) -> Option<Fingerprint> {
        let print = self.stock(facts).supply(facts.iso()).ok().map(|r| r.print);
        print.filter(|p| self.folder.known(p))
    }

    /// What device `d`'s row shows.
    fn item(&self, d: &Device, facts: &Facts<'_>, print: Option<&Fingerprint>) -> DeviceItem {
        let DeviceKind::Card(id) = &d.kind else {
            return DeviceItem::Cpu {
                cores: d.cores,
                memory: d.memory,
            };
        };
        let running = self.checking.as_ref().filter(|(c, _)| c == id);
        let verdict = print.and_then(|p| facts.checks.verdict(id, &d.driver, p));
        let check = match (running, verdict) {
            (Some((_, percent)), _) => CardCheck::Running { percent: *percent },
            (None, Some(v)) => CardCheck::Done(v),
            (None, None) => CardCheck::Unchecked,
        };
        DeviceItem::Card {
            name: d.name.clone(),
            built_in: d.built_in,
            memory: d.memory,
            check,
        }
    }
}

/// The pick a device's row names.
fn pick_of(device: &Device) -> DevicePick {
    match &device.kind {
        DeviceKind::Cpu => DevicePick::Cpu,
        DeviceKind::Card(id) => DevicePick::Card(id.clone()),
    }
}

/// A card's failed check for the note band.
fn failed(item: &DeviceItem) -> Option<FailedCheck> {
    match item {
        DeviceItem::Card {
            name,
            check: CardCheck::Done(verdict),
            ..
        } if !matches!(verdict, Verdict::Passed { .. }) => Some(FailedCheck {
            name: name.clone(),
            verdict: *verdict,
        }),
        _ => None,
    }
}

/// The device picked, Automatic with the device that runs `ready` now, as the home list and Compare name it.
#[must_use]
pub(crate) fn label(facts: &Facts<'_>, ready: Option<&Ready>) -> DeviceLabel {
    match facts.settings.device() {
        DevicePick::Auto => {
            let card = ready.and_then(|r| r.card.as_ref()).map(|c| &c.id);
            ran_on(facts, true, card)
        }
        DevicePick::Cpu => DeviceLabel::Cpu,
        DevicePick::Card(id) => DeviceLabel::Card(card_name(facts, &id)),
    }
}

/// Where a run went, as Compare's results name it: Automatic with the device that served when `auto`, else the card
/// that served or the processor.
#[must_use]
pub(crate) fn ran_on(facts: &Facts<'_>, auto: bool, card: Option<&CardId>) -> DeviceLabel {
    let name = card.map(|id| card_name(facts, id));
    match (auto, name) {
        (true, name) => DeviceLabel::Auto(name.map_or(DeviceName::Cpu, DeviceName::Card)),
        (false, Some(name)) => DeviceLabel::Card(name),
        (false, None) => DeviceLabel::Cpu,
    }
}

/// Card `id`'s name as the system gives it, else its id once it is gone.
fn card_name(facts: &Facts<'_>, id: &CardId) -> String {
    let card = (facts.devices.iter()).find(|d| d.kind == DeviceKind::Card(id.clone()));
    card.map_or_else(|| id.as_str().to_owned(), |d| d.name.clone())
}

//! The Device page's acts: Use writes `device`, and a card not checked for the model in use is checked; Check runs a
//! card's check in the background, whose warm-up and verdict come back here (ADR-0044, edges 26 and 27).

use kx_module_api::ports::{
    BarHint, CardId, CardUse, CheckAsk, CheckDone, Device, DeviceKind, DevicePick, PanelButton,
    SpeechError, Verdict,
};

use super::{Check, Do, Key, Panel};
use crate::counts::Checked;
use crate::device::verdict;
use crate::flow::{Ready, Supply};
use crate::models::{code_in, table};
use crate::settings::DEVICE;
use crate::view::Facts;

impl Panel {
    /// A Device page button.
    pub(super) fn device_button(&mut self, button: PanelButton, facts: &Facts<'_>) -> Vec<Do> {
        let Some(Key::Device(pick)) = self.ticked().cloned() else {
            return Vec::new();
        };
        match (button, &pick) {
            (PanelButton::Use, _) => {
                let mut out = vec![Do::Set(DEVICE, toml::Value::String(pick.name().to_owned()))];
                if let DevicePick::Card(card) = &pick {
                    out.extend(self.check_unchecked(card, facts));
                }
                out
            }
            (PanelButton::Check, DevicePick::Card(card)) => self.check(card, facts),
            _ => Vec::new(),
        }
    }

    /// The check a card lacks for the model the language in front uses now (edge 26): the picked card's when it has
    /// none under its driver, and with Automatic a card checked before under another driver. Call it after the
    /// devices or the folder are read; `follow` and the folder's reads call it too.
    pub fn recheck(&mut self, facts: &Facts<'_>) -> Vec<Do> {
        let Some(print) = self.used_print(facts).filter(|_| self.checking.is_none()) else {
            return Vec::new();
        };
        let pick = facts.settings.device();
        let before =
            |id: &CardId| (facts.checks.rows().iter()).any(|c| c.card == *id && c.model == print);
        let wanted = |id: &CardId| match &pick {
            DevicePick::Card(want) => want == id,
            DevicePick::Auto => before(id),
            DevicePick::Cpu => false,
        };
        let lacks = |d: &Device| match &d.kind {
            DeviceKind::Card(id) => {
                let unchecked = facts.checks.verdict(id, &d.driver, &print).is_none();
                (unchecked && wanted(id)).then(|| id.clone())
            }
            DeviceKind::Cpu => None,
        };
        let card = facts.devices.iter().find_map(lacks);
        card.map_or_else(Vec::new, |id| self.check(&id, facts))
    }

    /// A card picked with no check for the model in use is checked now.
    fn check_unchecked(&mut self, card: &CardId, facts: &Facts<'_>) -> Vec<Do> {
        let device = facts
            .devices
            .iter()
            .find(|d| d.kind == DeviceKind::Card(card.clone()));
        let print = self.used_print(facts);
        let checked = device
            .zip(print)
            .is_some_and(|(d, p)| facts.checks.verdict(card, &d.driver, &p).is_some());
        if checked || self.checking.is_some() {
            return Vec::new();
        }
        self.check(card, facts)
    }

    /// Checks `card` for the model used for the language in front, on the first clip that model knows.
    fn check(&mut self, card: &CardId, facts: &Facts<'_>) -> Vec<Do> {
        let device = facts
            .devices
            .iter()
            .find(|d| d.kind == DeviceKind::Card(card.clone()));
        let ready = self.stock(facts).supply(facts.iso()).ok();
        // A check goes by the file's fingerprint, so a file not read yet is not checked.
        let ready = ready.filter(|r| self.folder.known(&r.print));
        let (Some(device), Some(ready)) = (device, ready) else {
            return Vec::new();
        };
        let Some((need_mb, clip, lang)) = self.clip_of(&ready) else {
            return Vec::new();
        };
        let mut model = ready.model;
        model.lang = lang;
        let settings = facts.settings;
        let use_ = CardUse {
            id: card.clone(),
            need_mb,
            margin_mb: settings.card_margin_mb,
        };
        let ask = CheckAsk {
            job: settings.job(),
            model,
            card: use_,
            threads: settings.threads,
            clip: clip.to_owned(),
        };
        self.checking = Some((card.clone(), 0));
        let check = Check {
            card: card.clone(),
            driver: device.driver.clone(),
            print: ready.print,
            clip: clip.to_owned(),
            ask,
        };
        vec![Do::Check(Box::new(check))]
    }

    /// The card memory `ready` needs, the clip it is checked on and that clip's language as the model names it: a
    /// row's own, else an other file's size and the first clip its facts know.
    fn clip_of(&self, ready: &Ready) -> Option<(u32, &'static str, String)> {
        let table = table().ok()?;
        if let Some(row) = table.printed(&ready.print) {
            let clip = table.clip_for(row)?;
            return Some((row.card_mb, clip, row.code(clip)?.to_owned()));
        }
        let known = self.folder.facts(&ready.print)?;
        let clip = table.clips().iter().find(|c| code_in(known, c).is_some())?;
        Some((
            ready.model.cpu_mb,
            clip.as_str(),
            code_in(known, clip)?.to_owned(),
        ))
    }

    /// A card's warm-up moved on.
    pub fn warming(&mut self, card: &CardId, percent: u8) {
        if let Some((_, now)) = self.checking.as_mut().filter(|(c, _)| c == card) {
            *now = percent;
        }
    }

    /// A card's check ended: its verdict is kept, or its error shows a note and nothing is kept.
    pub fn checked(
        &mut self,
        check: &Check,
        done: Result<CheckDone, SpeechError>,
        facts: &Facts<'_>,
    ) -> Vec<Do> {
        if self
            .checking
            .as_ref()
            .is_some_and(|(c, _)| *c == check.card)
        {
            self.checking = None;
        }
        // A card gone keeps no verdict: only a check that met the card says why it is not used.
        let verdict = match done {
            Ok(done) => verdict(&done, &check.clip, &facts.settings.card_rules()),
            Err(SpeechError::CardRoom) => Verdict::NoRoom,
            Err(SpeechError::CardFailed) => Verdict::Failed,
            Err(_) => {
                self.note = Some(BarHint::Fault);
                return Vec::new();
            }
        };
        let checked = Checked {
            card: check.card.clone(),
            driver: check.driver.clone(),
            model: check.print,
            verdict,
            day: facts.now.date,
        };
        vec![Do::Checked(checked)]
    }
}

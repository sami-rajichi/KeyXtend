//! The Device page's acts: Use writes `device`, and a card not checked for the model in use is checked; Check runs a
//! card's check in the background, whose warm-up and verdict come back here (ADR-0044, edges 26 and 27).

use kx_module_api::ports::{
    BarHint, CardId, CardUse, CheckAsk, CheckDone, Device, DeviceKind, DevicePick, Fingerprint,
    PanelButton, SpeechError, Verdict,
};

use super::{Check, Do, Key, Panel};
use crate::counts::{Checked, Checks};
use crate::device::{CardRules, told, verdict};
use crate::flow::{Ready, Supply};
use crate::models::table;
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

    /// The check a card lacks under its driver for the model the language in front uses now (edge 26). While a check,
    /// a dictation or Compare runs, it waits for `dequeue`. The actor calls it after the devices or a layout; `follow`
    /// and the folder's reads call it too.
    pub fn recheck(&mut self, facts: &Facts<'_>) -> Vec<Do> {
        self.queued = self.checking.is_some() || facts.dictating || self.compare.busy();
        let Some(print) = self.used_print(facts).filter(|_| !self.queued) else {
            return Vec::new();
        };
        let pick = facts.settings.device();
        let lacks = |d: &Device| match &d.kind {
            DeviceKind::Card(id) => {
                let unchecked = facts.checks.verdict(id, &d.driver, &print).is_none();
                (unchecked && wanted(&pick, d, id, &print, facts.checks)).then(|| id.clone())
            }
            DeviceKind::Cpu => None,
        };
        let card = facts.devices.iter().find_map(lacks);
        card.map_or_else(Vec::new, |id| self.check(&id, facts))
    }

    /// The check `recheck` held back, once no check, dictation or Compare runs; the actor calls it after each input.
    pub fn dequeue(&mut self, facts: &Facts<'_>) -> Vec<Do> {
        if !self.queued {
            return Vec::new();
        }
        self.recheck(facts)
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

    /// Checks `card` for the model used for the language in front, on that language's clip when the model knows it.
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
        let Some((need_mb, clip, lang)) = self.clip_of(&ready, facts) else {
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
            clip: clip.clone(),
        };
        self.checking = Some((card.clone(), 0));
        let check = Check {
            card: card.clone(),
            driver: device.driver.clone(),
            print: ready.print,
            clip,
            ask,
        };
        vec![Do::Check(Box::new(check))]
    }

    /// The card memory `ready` needs, a row's own else an other file's size, with the clip its check reads for the
    /// language in front and that clip's language as the model names it (ADR-0052).
    fn clip_of(&self, ready: &Ready, facts: &Facts<'_>) -> Option<(u32, String, String)> {
        let (clip, code) = self.stock(facts).clip(&ready.print, facts.iso())?;
        let row = table().ok().and_then(|t| t.printed(&ready.print));
        let need_mb = row.map_or(ready.model.cpu_mb, |r| r.card_mb);
        Some((need_mb, clip.to_owned(), code))
    }

    /// A card's warm-up moved on.
    pub fn warming(&mut self, card: &CardId, percent: u8) {
        if let Some((_, now)) = self.checking.as_mut().filter(|(c, _)| c == card) {
            *now = percent;
        }
    }

    /// A card's check ended: its verdict is kept and another card the model lacks is queued, or its error shows a
    /// note and nothing is kept, so a fault never loops.
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
        let Some(verdict) = verdict_of(done, &check.clip, facts.settings.card_rules()) else {
            self.note = Some(BarHint::Fault);
            return Vec::new();
        };
        let checked = Checked {
            card: check.card.clone(),
            driver: check.driver.clone(),
            model: check.print,
            verdict,
            day: facts.now.date,
        };
        self.queued = true;
        vec![Do::Checked(checked)]
    }
}

/// The verdict a check of `clip` that ended with `done` gives under `rules`, logged. A card gone keeps none: only a
/// check that met the card says why it is not used.
fn verdict_of(
    done: Result<CheckDone, SpeechError>,
    clip: &str,
    rules: CardRules,
) -> Option<Verdict> {
    match done {
        Ok(done) => {
            let verdict = verdict(&done, clip, &rules);
            told::checked(verdict, &done);
            Some(verdict)
        }
        Err(error) => {
            told::failed(error);
            match error {
                SpeechError::CardRoom => Some(Verdict::NoRoom),
                SpeechError::CardFailed => Some(Verdict::Failed),
                _ => None,
            }
        }
    }
}

/// Whether `pick` wants card `id`, device `d`, checked for file `print`: the picked card; with Automatic, a card
/// checked for it under another driver, or one that passed another model's check under its present driver.
fn wanted(
    pick: &DevicePick,
    d: &Device,
    id: &CardId,
    print: &Fingerprint,
    checks: &Checks,
) -> bool {
    let mut rows = checks.rows().iter().filter(|c| c.card == *id);
    let passed = |c: &Checked| matches!(c.verdict, Verdict::Passed { .. });
    match pick {
        DevicePick::Card(want) => want == id,
        DevicePick::Auto => rows.any(|c| c.model == *print || (c.driver == d.driver && passed(c))),
        DevicePick::Cpu => false,
    }
}

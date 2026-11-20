//! The flow's `Supply`: the model a language gets from the models table, `picks`, the folder and `device`'s rule over
//! the devices and their checks. A pick's file that is gone takes the default; a model too big for this PC never runs.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    CardUse, Device, DeviceKind, DevicePick, Fingerprint, Memory, SpeechModel,
};

use super::{File, Folder, Print};
use crate::VoiceSettings;
use crate::counts::Checks;
use crate::device::{Seen, Serve, Why, clip, serve};
use crate::flow::{Lack, Ready, Supply};
use crate::models::{Other, Pick, Row, code_in, table};

/// What serves a language: the settings, the folder, the devices as last read and the cards' checks.
#[derive(Copy, Clone, Debug)]
pub struct Stock<'a> {
    /// `[voice]`: the picks, the device, `job_mb` and `card_margin_mb`.
    pub settings: &'a VoiceSettings,
    /// The models folder and what it holds.
    pub folder: &'a Folder,
    /// The processor and the cards; empty before they are read, when every model fits and no card serves.
    pub devices: &'a [Device],
    /// Each card's check by driver and file.
    pub checks: &'a Checks,
    /// The check clips' languages the speech port ships, in order.
    pub clips: &'a [String],
}

impl Supply for Stock<'_> {
    fn supply(&self, iso: &str) -> Result<Ready, Lack> {
        let table = table().map_err(|_| Lack::Unknown)?;
        match table.used(iso, &self.settings.picks).ok_or(Lack::Unknown)? {
            Pick::Row(row) => self.row(row, iso),
            Pick::File(name) => match self.folder.file(name) {
                Some(file) => self.file(file, iso),
                None => self.row(table.default_for(iso).ok_or(Lack::Unknown)?, iso),
            },
        }
    }
}

impl<'a> Stock<'a> {
    /// The row Download fetches for `iso`: the picked row, else the default.
    #[must_use]
    pub fn wanted(&self, iso: &str) -> Option<&'a Row> {
        let table = table().ok()?;
        match table.used(iso, &self.settings.picks)? {
            Pick::Row(row) => Some(row),
            Pick::File(_) => table.default_for(iso),
        }
    }

    /// What `row` serves for `iso`: its file in the folder on the device the rule picks.
    ///
    /// # Errors
    /// `Stranger` when it does not know `iso`, `NoFit` past this PC's memory, `Missing` without its file.
    pub fn row(&self, row: &Row, iso: &str) -> Result<Ready, Lack> {
        let code = row.code(iso).ok_or(Lack::Stranger)?;
        let too_big = self
            .memory()
            .and_then(|m| row.not_here(&m, self.settings.job_mb));
        if too_big.is_some() {
            return Err(Lack::NoFit);
        }
        let missing = Lack::Missing { bytes: row.bytes };
        let file = self.folder.of(row).ok_or(missing)?;
        let dir = self.folder.dir().ok_or(missing)?;
        let model = SpeechModel {
            file: Redacted::new(dir.join(&file.name)),
            lang: code.to_owned(),
            runner: row.runner.clone(),
            cpu_mb: row.cpu_mb,
        };
        // A file served by its name and size before its fingerprint is read stays on the processor, as each card's
        // check goes by the fingerprint.
        let read = file.print == Print::Known(row.print);
        let (card, why) = if read {
            self.card(row.card_mb, &row.print)
        } else {
            (None, self.held(Why::Unread))
        };
        Ok(Ready {
            model,
            card,
            why,
            name: Redacted::new(row.id.clone()),
            print: row.print,
            bytes: row.bytes,
        })
    }

    /// What other file `file` serves for `iso`, by its own facts, its size counting as its memory.
    ///
    /// # Errors
    /// `Checking` while its fingerprint or facts are read, `Stranger` when it cannot be read or its facts do not name
    /// `iso`, `NoFit` past this PC's memory.
    pub fn file(&self, file: &File, iso: &str) -> Result<Ready, Lack> {
        let print = match file.print {
            Print::Known(print) => print,
            Print::Pending => return Err(Lack::Checking),
            Print::Failed => return Err(Lack::Stranger),
        };
        if let Some(row) = table().ok().and_then(|t| t.printed(&print)) {
            return self.row(row, iso);
        }
        let lack = if self.folder.wants_facts(&print) {
            Lack::Checking
        } else {
            Lack::Stranger
        };
        let facts = self.folder.facts(&print).ok_or(lack)?;
        let dir = self.folder.dir().ok_or(Lack::Stranger)?;
        let other = Other {
            name: file.name.clone(),
            bytes: file.bytes,
        };
        let need = other.need_mb().ok_or(Lack::NoFit)?;
        let cap = other.cap_mb(self.settings.job_mb).ok_or(Lack::NoFit)?;
        if self.memory().is_some_and(|m| cap > m.total_mb) {
            return Err(Lack::NoFit);
        }
        let model = other.model(dir, facts, iso).ok_or(Lack::Stranger)?;
        let (card, why) = self.card(need, &print);
        Ok(Ready {
            model,
            card,
            why,
            name: Redacted::new(file.name.clone()),
            print,
            bytes: file.bytes,
        })
    }

    /// The clip a card's check of file `print` reads with `iso` in front, with the model's code for its language
    /// (ADR-0052): by its row, else by its facts; `None` when the model knows no shipped clip.
    #[must_use]
    pub fn clip(&self, print: &Fingerprint, iso: &str) -> Option<(&'a str, String)> {
        let owned = |(clip, code): (&'a str, &str)| (clip, code.to_owned());
        if let Some(row) = table().ok()?.printed(print) {
            return clip(iso, self.clips, |c| row.code(c)).map(owned);
        }
        let facts = self.folder.facts(print)?;
        clip(iso, self.clips, |c| code_in(facts, c)).map(owned)
    }

    /// The card that runs a model needing `need_mb` on it, by `device`'s rule and each card's check of file `print`;
    /// else why the processor serves, when a card is picked.
    fn card(&self, need_mb: u32, print: &Fingerprint) -> (Option<CardUse>, Option<Why>) {
        let check = |d: &Device| match &d.kind {
            DeviceKind::Card(id) => self.checks.verdict(id, &d.driver, print),
            DeviceKind::Cpu => None,
        };
        let seen: Vec<Seen<'_>> = (self.devices.iter())
            .map(|device| Seen {
                device,
                check: check(device),
            })
            .collect();
        let margin = self.settings.card_margin_mb;
        match serve(&self.settings.device(), &seen, need_mb, margin) {
            Serve::Card(card) => (Some(card), None),
            Serve::Cpu(why) => (None, self.held(why)),
        }
    }

    /// `why` the processor serves, kept only while a card is picked.
    fn held(&self, why: Why) -> Option<Why> {
        matches!(self.settings.device(), DevicePick::Card(_)).then_some(why)
    }

    /// The processor's memory, once the devices were read.
    fn memory(&self) -> Option<Memory> {
        let cpu = self.devices.iter().find(|d| d.kind == DeviceKind::Cpu);
        cpu.map(|d| d.memory)
    }
}

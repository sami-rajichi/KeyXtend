//! Each staged edge on the copy, through the product's own speech host over the staged worker: 28 lists the devices,
//! 29 and 39 run the card's self-test on the shipped clip while the worker's modules are listed. Only numbers come out.
#![cfg(windows)]

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use kx_mod_voice::VoiceSettings;
use kx_mod_voice::models::Row;
use kx_mod_voice::score::Wrong;
use kx_module_api::ports::{CardId, CheckAsk, CheckDone, Device, DeviceKind, Speech};
use kx_platform::speech::{Assets, WorkerSpeech};
use kx_platform_windows::worker::WORKER_EXE;
use kx_platform_windows::{QpcClock, WinHardware, WinSpawner};
use serde_json::{Value, json};

use super::copy::Edge;
use crate::g22cfg::{Clip, is_part};
use crate::modlist::{self, Sampler, Seen};

/// What the edges read: the copy, the product's settings, the clip with its language and default model, how often
/// the modules are listed and how long a worker let go may take to end.
pub(super) struct Host<'a> {
    /// The copy's folder, its full path.
    pub(super) copy: &'a Path,
    pub(super) settings: &'a VoiceSettings,
    pub(super) clip: &'a Clip,
    pub(super) lang: &'a str,
    pub(super) row: &'a Row,
    pub(super) poll: Duration,
    /// The longest wait for a worker the host let go to end, in ms.
    pub(super) end_ms: u64,
    /// The plainest processor part, and what every processor part's name starts with.
    pub(super) floor: &'a str,
    pub(super) prefix: &'a str,
}

impl Host<'_> {
    /// Waits for every worker of ours to end, as the product's host lets each go without waiting; true once none runs.
    pub(super) fn let_go(&self) -> bool {
        let poll_ms = u64::try_from(self.poll.as_millis()).unwrap_or(u64::MAX);
        modlist::ended(WORKER_EXE, self.end_ms, poll_ms)
    }
}

/// Why a check has no reading when the device list's worker outlived its wait.
const LIST_LEFT: &str = "the worker that listed the devices did not end within the wait";

/// Edge 28's reading: the devices the worker offered.
#[derive(Debug, Clone, Copy)]
pub(super) struct NoCard {
    pub(super) devices: usize,
    pub(super) cpu: bool,
    pub(super) cards: usize,
}

impl NoCard {
    /// The reading of device list `list`.
    pub(super) fn of(list: &[Device]) -> Self {
        let cards = list
            .iter()
            .filter(|d| matches!(d.kind, DeviceKind::Card(_)))
            .count();
        let cpu = list.iter().any(|d| d.kind == DeviceKind::Cpu);
        Self {
            devices: list.len(),
            cpu,
            cards,
        }
    }

    /// The worker started and offered the processor and no card.
    pub(super) fn pass(&self) -> bool {
        self.cpu && self.cards == 0
    }

    pub(super) fn json(&self) -> Value {
        json!({ "edge": Edge::NoCard.number(), "devices": self.devices, "cpu": self.cpu, "cards": self.cards, "pass": self.pass() })
    }
}

/// Edge 29's reading: the words wrong on the processor and the card, and whether the list saw the floor part loaded.
#[derive(Debug, Clone, Copy)]
pub(super) struct Floor {
    pub(super) wrong: Option<Wrong>,
    pub(super) card_wrong: Option<Wrong>,
    pub(super) loaded: bool,
}

impl Floor {
    /// The processor read the clip within `diff` words wrong; the card's text and the module list are readings.
    pub(super) fn pass(&self, diff: u8) -> bool {
        self.wrong.is_some_and(|w| w.words <= u16::from(diff))
    }

    pub(super) fn json(&self, diff: u8) -> Value {
        let words = |w: Option<Wrong>| w.map(|w| w.words);
        json!({
            "edge": Edge::Floor.number(), "wrong": words(self.wrong), "of": self.wrong.map(|w| w.total),
            "card_wrong": words(self.card_wrong), "floor_loaded": self.loaded, "pass": self.pass(diff),
        })
    }
}

/// Edge 39's reading: the snapshots, the copy's processor parts seen loaded and the modules from the planted folder.
#[derive(Debug, Clone, Copy)]
pub(super) struct Plant {
    pub(super) snaps: u32,
    pub(super) failed: u32,
    pub(super) copy_parts: usize,
    pub(super) planted: usize,
}

impl Plant {
    /// The list saw the worker load the copy's parts and nothing from the planted folder.
    pub(super) fn pass(&self) -> bool {
        self.snaps > 0 && self.copy_parts > 0 && self.planted == 0
    }

    pub(super) fn json(&self) -> Value {
        json!({
            "edge": Edge::Plant.number(), "snaps": self.snaps, "failed": self.failed, "copy_parts": self.copy_parts,
            "planted": self.planted, "pass": self.pass(),
        })
    }
}

/// The first card in `list`.
pub(super) fn first_card(list: &[Device]) -> Option<CardId> {
    list.iter().find_map(|d| match &d.kind {
        DeviceKind::Card(id) => Some(id.clone()),
        DeviceKind::Cpu => None,
    })
}

/// Edge 28: the copy's worker lists its devices.
pub(super) fn no_card(h: &Host) -> Result<NoCard, String> {
    let list = speech(h.copy)?
        .devices(&h.settings.job())
        .map_err(|e| e.to_string())?;
    Ok(NoCard::of(&list))
}

/// Edge 29: the self-test of the default model in `models` on the copy with only the floor part.
pub(super) fn floor(h: &Host, models: &Path) -> Result<Floor, String> {
    let (done, seen) = checked(h, models)?;
    Ok(Floor {
        wrong: h.clip.wrong(done.cpu_text.expose(), h.lang),
        card_wrong: h.clip.wrong(done.card_text.expose(), h.lang),
        loaded: seen.has(&h.copy.join(h.floor)),
    })
}

/// Edge 39: the self-test of the model hard-linked into the planted folder `planted`.
pub(super) fn plant(h: &Host, planted: &Path) -> Result<Plant, String> {
    let (_, seen) = checked(h, planted)?;
    Ok(Plant {
        snaps: seen.snaps,
        failed: seen.failed,
        copy_parts: seen.named_under(h.copy, |n| is_part(n, h.prefix)),
        planted: seen.under(planted),
    })
}

/// The product's speech host over the copy's worker and voice files.
fn speech(copy: &Path) -> Result<WorkerSpeech<WinSpawner>, String> {
    let clock = Arc::new(QpcClock::new().map_err(|e| e.to_string())?);
    let hardware = Arc::new(WinHardware::new());
    Ok(WorkerSpeech::new(
        WinSpawner::beside(copy),
        clock,
        Assets::beside(copy),
        hardware,
    ))
}

/// The card's self-test of the default model from folder `models` on the first card offered, its modules listed.
fn checked(h: &Host, models: &Path) -> Result<(CheckDone, Seen), String> {
    let speech = speech(h.copy)?;
    let s = h.settings;
    let devices = speech.devices(&s.job()).map_err(|e| e.to_string())?;
    let card =
        first_card(&devices).ok_or("the copy offers no card, and the self-test needs one")?;
    let model = h
        .row
        .model(models, h.lang)
        .ok_or("the default model does not know the clip")?;
    let ask = CheckAsk {
        job: s.job(),
        model,
        card: h.row.card(card, s.card_margin_mb),
        threads: s.threads,
        clip: h.lang.to_owned(),
    };
    // Only the check's own worker may be listed, so the device list's goes first.
    if !h.let_go() {
        return Err(LIST_LEFT.into());
    }
    let sampler = Sampler::start(WORKER_EXE, h.poll);
    let done = speech.check(ask, Box::new(|_| {}));
    let seen = sampler.finish();
    Ok((done.map_err(|e| e.to_string())?, seen))
}

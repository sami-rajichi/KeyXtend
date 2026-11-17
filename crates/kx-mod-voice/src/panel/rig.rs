//! The panel's tests' rig: the facts the actor would hold, owned, five rows to a page as `[ui] page_rows` is by
//! default, English in front, the folder's drive and the processor.

use std::collections::BTreeSet;
use std::path::PathBuf;

use kx_module_api::ports::{
    BarPlace, CardId, Device, DeviceKind, Drive, Fingerprint, FolderScan, LayoutChanged, LayoutId,
    LocalTime, MB, Memory, ModelFile, VoiceAct, VoiceBar, VoiceClick, VoicePage, VoicePanel,
};
use kx_module_api::{Redacted, Zeroizing};

use super::{Do, Panel};
use crate::VoiceSettings;
use crate::cloud::Cloud;
use crate::counts::{Checks, Scores};
use crate::flow::Kept;
use crate::history::{History, Rules, Stamp};
use crate::models::{Row, table};
use crate::services::{self, Services};
use crate::view::Facts;

/// Rows a page shows, as `[ui] page_rows` is by default.
pub(crate) const ROWS: u8 = 5;
/// The models folder the tests use.
pub(crate) const DIR: &str = r"D:\KeyXtend\models";
/// The wall time now: 2026-11-14 12:00 UTC.
const NOW_MS: i64 = 1_794_657_600_000;

/// What the actor would hold.
pub(crate) struct Rig {
    pub(crate) panel: Panel,
    pub(crate) settings: VoiceSettings,
    pub(crate) front: LayoutChanged,
    pub(crate) drives: Vec<Drive>,
    pub(crate) devices: Vec<Device>,
    /// The check clips' languages the speech port ships, in order.
    pub(crate) clips: Vec<String>,
    pub(crate) checks: Checks,
    pub(crate) scores: Scores,
    pub(crate) history: History,
    pub(crate) again: bool,
    pub(crate) dictating: bool,
    /// The services table the cloud rules read; `None` as if it did not load.
    pub(crate) services: Option<Services>,
    /// The services with a key.
    pub(crate) keyed: BTreeSet<String>,
    /// The run is hidden, so cloud is off.
    pub(crate) hidden: bool,
    /// The round of the last view, counted as the flow counts it: once per change.
    pub(crate) round: u32,
    drawn: Option<VoicePanel>,
}

impl Rig {
    /// English in front, no folder chosen, a roomy drive D: and a processor of 32 GB, the panel open.
    pub(crate) fn new() -> Self {
        let settings = VoiceSettings::from_defaults().unwrap();
        let mut rig = Self {
            panel: Panel::new(&settings),
            settings,
            front: LayoutChanged {
                id: LayoutId::new(1),
                iso: "en".into(),
                name: "English".into(),
                rtl: false,
            },
            drives: vec![drive(r"C:\", 40_000 * MB), drive(r"D:\", 900_000 * MB)],
            devices: vec![cpu(32_768)],
            clips: ["en", "fr", "ar"].map(String::from).to_vec(),
            checks: Checks::default(),
            scores: Scores::default(),
            history: History::default(),
            again: false,
            dictating: false,
            services: services::table().ok().cloned(),
            keyed: BTreeSet::new(),
            hidden: false,
            round: 0,
            drawn: None,
        };
        rig.panel.open();
        rig.click(&VoiceClick::Rows(ROWS));
        rig
    }

    /// The rig with the folder at `DIR` chosen, not read yet.
    pub(crate) fn foldered() -> Self {
        let mut rig = Self::new();
        rig.settings.models_dir = DIR.into();
        rig.with(Panel::follow);
        rig
    }

    /// Calls `f` with the panel and the facts.
    pub(crate) fn with<R>(&mut self, f: impl FnOnce(&mut Panel, &Facts<'_>) -> R) -> R {
        let owned = self.facts_owned();
        f(&mut self.panel, &owned.as_facts())
    }

    /// A click on the view the ui would have drawn now.
    pub(crate) fn click(&mut self, click: &VoiceClick) -> Vec<Do> {
        self.view();
        let round = self.round;
        self.click_at(round, click)
    }

    /// A click on the view of `round`, as a slow finger may send it after the view changed.
    pub(crate) fn click_at(&mut self, round: u32, click: &VoiceClick) -> Vec<Do> {
        let act = VoiceAct {
            round,
            click: click.clone(),
        };
        self.with(|panel, facts| panel.click(&act, facts))
    }

    /// The view now; a changed one is told as the flow tells it, with a new round.
    pub(crate) fn view(&mut self) -> Option<VoicePanel> {
        let view = self.with(Panel::view);
        if view != self.drawn {
            self.round += 1;
            self.drawn.clone_from(&view);
            let page = VoicePage {
                round: self.round,
                bar: VoiceBar::Hidden,
                panel: view.clone(),
                place: BarPlace::Above,
                sounds: true,
                cloud: None,
            };
            self.panel.told(&page);
        }
        view
    }

    /// The folder as scanned, holding `files`, with each fingerprint still to read.
    pub(crate) fn scan(&mut self, files: Vec<ModelFile>) -> Vec<Do> {
        let scan = FolderScan {
            folders: Vec::new(),
            files,
        };
        self.with(|p, f| p.scanned(Ok(scan), f))
    }

    /// File `name` of `bytes` in the chosen folder read with fingerprint `print`.
    pub(crate) fn print(&mut self, name: &str, bytes: u64, print: Fingerprint) -> Vec<Do> {
        let file = self.panel.folder().dir().unwrap().join(name);
        self.with(|p, f| p.printed(&file, bytes, Ok(print), f))
    }

    /// The facts copied out, so the panel can change while they are lent.
    fn facts_owned(&self) -> Owned {
        Owned {
            settings: self.settings.clone(),
            front: self.front.clone(),
            drives: self.drives.clone(),
            devices: self.devices.clone(),
            clips: self.clips.clone(),
            checks: self.checks.clone(),
            scores: self.scores.clone(),
            history: self.history.clone(),
            again: self.again,
            dictating: self.dictating,
            services: self.services.clone(),
            keyed: self.keyed.clone(),
            hidden: self.hidden,
        }
    }

    /// The folder holds `rows`' files, their fingerprints read.
    pub(crate) fn holds(&mut self, rows: &[&Row]) {
        self.scan(rows.iter().map(|r| model_file(&r.file, r.bytes)).collect());
        for row in rows {
            self.print(&row.file, row.bytes, row.print);
        }
    }

    /// History holds `n` texts, `texte 1` the oldest.
    pub(crate) fn texts(&mut self, n: u32) {
        let rules = Rules {
            texts: true,
            max: 500,
            voice: false,
            voice_bytes: 0,
        };
        for take in 1..=n {
            let kept = Kept {
                take,
                text: Redacted::new(Zeroizing::new(format!("texte {take}"))),
                lang: "fr".into(),
                model: Redacted::new("parakeet-v3-q8".into()),
            };
            let at = Stamp {
                unix_ms: NOW_MS - i64::from(n - take) * 60_000,
                offset_min: 0,
            };
            self.history.keep(kept, at, &rules);
        }
    }
}

/// The facts owned.
struct Owned {
    settings: VoiceSettings,
    front: LayoutChanged,
    drives: Vec<Drive>,
    devices: Vec<Device>,
    clips: Vec<String>,
    checks: Checks,
    scores: Scores,
    history: History,
    again: bool,
    dictating: bool,
    services: Option<Services>,
    keyed: BTreeSet<String>,
    hidden: bool,
}

impl Owned {
    fn as_facts(&self) -> Facts<'_> {
        Facts {
            settings: &self.settings,
            front: &self.front,
            drives: &self.drives,
            devices: &self.devices,
            clips: &self.clips,
            checks: &self.checks,
            scores: &self.scores,
            history: &self.history,
            now: LocalTime::at(NOW_MS, 0).unwrap(),
            again: self.again,
            dictating: self.dictating,
            cloud: self.services.as_ref().map(|t| Cloud::new(t, self.hidden)),
            keyed: &self.keyed,
        }
    }
}

/// The row of `id`.
pub(crate) fn row(id: &str) -> &'static Row {
    table().unwrap().get(id).unwrap()
}

/// A fixed drive at `root` with `free` bytes free.
pub(crate) fn drive(root: &str, free: u64) -> Drive {
    Drive {
        root: Redacted::new(PathBuf::from(root)),
        name: Redacted::new(root.trim_end_matches('\\').to_owned()),
        free,
    }
}

/// The processor with 6 real cores and `total_mb` of memory.
pub(crate) fn cpu(total_mb: u32) -> Device {
    Device {
        kind: DeviceKind::Cpu,
        name: "CPU".into(),
        memory: Memory {
            total_mb,
            free_mb: total_mb / 2,
        },
        driver: String::new(),
        cores: 6,
        built_in: false,
    }
}

/// A card `Vulkan0` with `free_mb` free under driver `1.0`.
pub(crate) fn card(free_mb: u32) -> Device {
    Device {
        kind: DeviceKind::Card(CardId::new("Vulkan0").unwrap()),
        name: "Card".into(),
        memory: Memory {
            total_mb: 8_192,
            free_mb,
        },
        driver: "1.0".into(),
        cores: 0,
        built_in: false,
    }
}

/// A file the folder scan finds.
pub(crate) fn model_file(name: &str, bytes: u64) -> ModelFile {
    ModelFile {
        name: Redacted::new(name.to_owned()),
        bytes,
    }
}

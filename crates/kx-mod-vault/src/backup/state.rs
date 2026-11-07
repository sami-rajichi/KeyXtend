//! The backup's runtime on the actor's thread: what it keeps and its blob, the job queue and the places, the round of
//! each job it waits for, the write's timer, and the backup pages while they show. A job's answer from an older round
//! is dropped, and only error kinds are logged.

use std::path::PathBuf;
use std::time::Duration;

use kx_module_api::ports::{FillNote, LocalTime, NoteCounts, Place, Spot};
use kx_module_api::{Mono, Redacted, SettingsHandle};

use super::bring::{Bring, Source};
use super::flow::{Flow, Then};
use super::jobs::{Answer, Ask, Done, Job, Post};
use super::keepass::Import;
use super::{BackupStore, Kept};
use crate::book::Notebook;
use crate::entry::Stamp;
use crate::settings::VaultSettings;

/// The backup's part of the settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Config {
    /// The wait after the last change before a write (`box_delay_ms`).
    pub(crate) delay: Duration,
    /// The wait between looks at the waiting places (`place_check_s`).
    pub(crate) look_every: Duration,
    /// The fewest characters of a new password (`box_min_chars`).
    pub(crate) min_chars: usize,
    /// The most characters a typed line holds (`field_chars`).
    pub(crate) field_chars: usize,
    /// The scrypt work factor of a new padlock (`box_work`).
    pub(crate) work: u8,
    /// The largest box in bytes (`box_mb`).
    pub(crate) cap: u64,
    /// The wait before each memory check, the last one repeating (`check_days`).
    pub(crate) steps: Vec<Duration>,
    /// Wrong answers a memory check takes (`check_tries`).
    pub(crate) tries: u8,
    /// The wait after a check's Later (`later_h`).
    pub(crate) later: Duration,
}

impl Config {
    /// The backup's part of `settings`.
    pub(crate) fn of(settings: &VaultSettings) -> Self {
        Self {
            delay: settings.box_delay(),
            look_every: settings.place_check(),
            min_chars: settings.box_min_chars,
            field_chars: settings.field_chars,
            work: settings.box_work,
            cap: settings.box_cap(),
            steps: settings.check_waits(),
            tries: settings.check_tries,
            later: settings.later(),
        }
    }
}

/// The services the backup uses: the job queue, which reads the places too, and the module's own settings.
pub(crate) struct Links {
    pub(crate) jobs: Box<dyn Post>,
    pub(crate) handle: SettingsHandle,
}

/// A write in flight: its round, the folders it went to, and whether it went to every place.
#[derive(Debug)]
pub(super) struct Writing {
    pub(super) round: u32,
    pub(super) folders: Vec<Redacted<PathBuf>>,
    pub(super) full: bool,
}

/// What the interactive job waited for is for.
#[derive(Debug)]
pub(super) enum Purpose {
    /// A look at `Place`, picked for `Then`.
    Look(Place, Then),
    /// A padlock, for setup's place or a change.
    Padlock(Option<Place>, Then),
    /// A list for the page that shows, which takes it.
    Listing,
    /// The boxes found, for Bring back's page or for the offer that shows by itself.
    Find { offer: bool },
    /// A picked box's place, looked at before its password.
    Ready(Source),
    /// A box opened to bring its entries in.
    Open(Source),
    /// A memory check's answer.
    Check,
    /// A `KeePass` file read and opened.
    Keepass,
}

/// What the actor does after a backup step or answer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Flowed {
    /// Nothing changed.
    Stay,
    /// The backup pages, or the footer, changed.
    Show,
    /// The backup pages were left for the list.
    Leave,
    /// The boxes found offer themselves, but only over the list of an empty notebook.
    Offer,
    /// A box gave its entries, which the actor brings in, then `settle`s.
    Brought,
    /// A `KeePass` file gave its entries, which the actor brings in, then `settle_import`s.
    Keepass,
}

/// What a backup step or answer reads: the notebook, the times, and the rows a page shows.
pub(crate) struct At<'a> {
    pub(crate) book: &'a Notebook,
    pub(crate) now: Mono,
    pub(crate) wall: LocalTime,
    pub(crate) rows: usize,
}

/// The backup's runtime.
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact kept between messages: a box too large, the checks on, a banner up, an unsaved blob."
)]
pub(crate) struct Backup {
    pub(super) kept: Kept,
    store: BackupStore,
    pub(super) links: Links,
    pub(super) cfg: Config,
    round: u32,
    /// The interactive job waited for: a list, a look or a padlock.
    pub(super) asked: Option<(u32, Purpose)>,
    /// The write in flight.
    pub(super) writing: Option<Writing>,
    /// When the delayed write goes.
    pub(super) due: Option<Mono>,
    /// When the waiting places are looked at next.
    pub(super) look_at: Mono,
    /// The last seal passed `box_mb`, so looks stop until a change or a full write; unsaved, so a restart may try once.
    pub(super) too_large: bool,
    /// Memory checks are on.
    pub(super) checks: bool,
    /// The backup pages while they show.
    pub(super) flow: Option<Flow>,
    /// Where the shown list starts, so a row click names what the user saw.
    pub(super) start: usize,
    pub(super) note: Option<FillNote>,
    /// The numbers the `Imported` note names.
    pub(super) counts: NoteCounts,
    /// A memory check's banner asks now.
    pub(super) raised: bool,
    /// Bring back's runtime.
    pub(super) bring: Bring,
    /// Import's runtime.
    pub(super) import: Import,
    unsaved: bool,
}

impl Backup {
    /// The backup over `kept`, read by `store`, which `begin` starts.
    pub(crate) fn new(cfg: Config, kept: Kept, store: BackupStore, links: Links) -> Self {
        Self {
            kept,
            store,
            links,
            cfg,
            round: 0,
            asked: None,
            writing: None,
            due: None,
            look_at: Mono::from_us(0),
            too_large: false,
            checks: true,
            flow: None,
            start: 0,
            note: None,
            counts: NoteCounts::default(),
            raised: false,
            bring: Bring::default(),
            import: Import::default(),
            unsaved: false,
        }
    }

    /// Starts at `now` with memory checks `checks`; a notebook loaded `damaged` holds the boxes a padlock made, and a
    /// write due at the last stop comes soon.
    pub(crate) fn begin(&mut self, now: Mono, checks: bool, damaged: bool) {
        self.checks = checks;
        if damaged && self.kept.padlock.is_some() && !self.kept.held {
            self.kept.held = true;
            self.save();
        }
        self.look_at = now.saturating_add(self.cfg.look_every);
        if self.kept.behind {
            self.due = Some(now.saturating_add(self.cfg.delay));
        }
    }

    /// Takes a job's answer; one from an older round is dropped.
    pub(crate) fn done(&mut self, done: Done, at: &At<'_>) -> Flowed {
        match done.answer {
            Answer::Wrote(wrote) => {
                if !self.wrote(done.round, wrote, Stamp::of(&at.wall), at.now) {
                    return Flowed::Stay;
                }
                Flowed::Show
            }
            Answer::Look(look) => self.looked(done.round, look, at),
            Answer::Suggested(places) => self.suggested(done.round, places),
            Answer::Folders(listed) => self.listed(done.round, listed),
            Answer::Padlock(made) => self.padlocked(done.round, made, at),
            Answer::Found(found) => self.found(done.round, found),
            Answer::State(state) => self.readied(done.round, state),
            Answer::Opened(opened) => self.opened(done.round, opened),
            Answer::Checked(checked) => self.checked(done.round, checked, at),
            Answer::Keepass(read) => self.kdbx_read(done.round, read),
        }
    }

    /// Queues `job` under a new round, which it gives.
    pub(super) fn post(&mut self, job: Job) -> u32 {
        self.round = self.round.wrapping_add(1);
        let round = self.round;
        self.links.jobs.post(Ask { round, job });
        round
    }

    /// The purpose of the interactive job of `round`, which is then waited for no more; `None` for another round.
    pub(super) fn answered(&mut self, round: u32) -> Option<Purpose> {
        let (_, purpose) = self.asked.take_if(|(r, _)| *r == round)?;
        Some(purpose)
    }

    /// Saves the blob; a failure is logged once by its kind, and the next save, change, retry or stop tries again.
    pub(super) fn save(&mut self) {
        match self.store.save(&self.kept) {
            Ok(()) => self.unsaved = false,
            Err(error) => {
                if !std::mem::replace(&mut self.unsaved, true) {
                    tracing::warn!(%error, "the backup state was not saved");
                }
            }
        }
    }

    /// Saves the blob again when its last save failed.
    pub(crate) fn retry(&mut self) {
        if self.unsaved {
            self.save();
        }
    }

    /// True while the blob waits for a save that works.
    pub(crate) fn unsaved(&self) -> bool {
        self.unsaved
    }

    /// The first place the backup waits for, which the footer names.
    pub(crate) fn waiting(&self) -> Option<Spot> {
        let site = self.kept.sites.iter().find(|s| s.waiting)?;
        Some(spot(&site.place))
    }

    /// The backup pages were left: their typed password is wiped and the job waited for no longer counts.
    pub(crate) fn leave(&mut self) {
        (self.flow, self.asked, self.note) = (None, None, None);
        self.bring.offer = false;
    }

    /// Stops: a write due or cut short stays `behind`, saved when it was marked; a failed save is tried again, and the
    /// job thread ends after its job, running none queued.
    pub(crate) fn stop(&mut self) {
        self.leave();
        self.retry();
        self.links.jobs.stop();
    }
}

/// `place` as a page names it.
pub(super) fn spot(place: &Place) -> Spot {
    Spot {
        kind: place.kind,
        name: place.name.clone(),
    }
}

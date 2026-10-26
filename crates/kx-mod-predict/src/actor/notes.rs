//! What the actor publishes: the lane with a new round after each change, `PressesWanted` when it flips or is asked,
//! both again at stop, and each notice once per run (a missing pack once per language).

use super::Actor;
use super::store::Blob;
use crate::ID;
use crate::lane::{Chips, Sources};
use crate::pack::Pack;
use crate::score::{Learned, NoLearned, knows};
use kx_module_api::ports::{FieldKind, Lane, PressesWanted, Suggested};
use kx_module_api::{Event, Notice, NoticeArg, args, keys};
use std::collections::BTreeSet;

/// The notices told this run.
#[derive(Debug, Default)]
pub(super) struct Notes {
    /// The keys told once per run.
    told: BTreeSet<&'static str>,
    /// The languages whose missing pack was told.
    missing: BTreeSet<String>,
}

impl Actor {
    /// Publishes the lane as it is now, with a new round, then whether presses are wanted.
    pub(super) fn tell(&mut self, seed: Option<&Pack<'_>>) {
        self.round = self.round.wrapping_add(1);
        let lane = self.lane(seed);
        let list = (self.menu.list).map(|l| self.listed().page(l.page, usize::from(l.rows)));
        let suggested = Suggested {
            round: self.round,
            lane,
            forgetting: self.menu.forgetting,
            learn: self.settings.learn,
            on: self.settings.on,
            list,
        };
        self.publish(&suggested);
        self.last = Some(suggested);
        self.presses();
    }

    /// Publishes the last lane again, round and all.
    pub(super) fn again(&self) {
        if let Some(last) = &self.last {
            self.publish(last);
        }
    }

    /// The lane now, for the box the watch sees, keeping its chips' picks and what the tracker showed for them.
    fn lane(&mut self, seed: Option<&Pack<'_>>) -> Lane {
        self.chips = Chips::default();
        self.show = None;
        if !self.settings.on {
            return Lane::Off;
        }
        self.moved();
        if self.kind == FieldKind::Password {
            return Lane::Locked;
        }
        if self.mode.shortcut || self.mode.dead || self.pending {
            return Lane::Empty;
        }
        let show = self.tracker.show(&|w: &str| knows(seed, w));
        let view = self.learner.as_ref().map(|l| l.view(seed));
        let learned: &dyn Learned = view.as_ref().map_or(&NoLearned, |v| v);
        let limit = usize::from(self.settings.chips);
        let src = Sources {
            iso: &self.code,
            seed,
            learned,
            mode: self.mode,
            score: &self.score_cfg,
            emails: &self.emails,
            email: &self.email_cfg,
            limit,
            confirms: self.confirms(),
            forgetting: self.menu.forgetting,
        };
        let mut chips = Chips::of(&show, &src);
        chips.with_menu(self.menu, self.rtl, limit);
        let lane = if chips.chips.is_empty() {
            Lane::Empty
        } else {
            Lane::Chips(chips.chips.clone())
        };
        (self.chips, self.show) = (chips, Some(show));
        lane
    }

    /// Tells the mouse module to report presses while the tracker holds a word or its context.
    pub(super) fn presses(&mut self) {
        let want = self.following() && self.tracker.holds();
        if want != self.pressing {
            self.pressing = want;
            self.publish(&PressesWanted { on: want });
        }
    }

    /// Tells whether presses are wanted again, for a mouse module that starts later.
    pub(super) fn presses_again(&self) {
        self.publish(&PressesWanted { on: self.pressing });
    }

    /// Saves what changed, tells the lane empty and presses unwanted, and logs the counts.
    pub(super) fn quit(&mut self) {
        for blob in [Blob::Learned, Blob::Emails, Blob::Left] {
            self.save(blob);
        }
        (self.chips, self.show) = (Chips::default(), None);
        self.round = self.round.wrapping_add(1);
        let lane = Suggested {
            round: self.round,
            lane: Lane::Empty,
            forgetting: false,
            learn: self.settings.learn,
            on: self.settings.on,
            list: None,
        };
        self.publish(&lane);
        self.pressing = false;
        self.publish(&PressesWanted { on: false });
        self.stats.log();
    }

    /// Tells the user `key` once per run.
    pub(super) fn say_once(&mut self, key: &'static str) {
        if self.notes.told.insert(key) {
            self.publish(&notice(key, Vec::new()));
        }
    }

    /// Tells once per language and run, by the language's name, that the language in front has no pack though the build
    /// shipped one.
    pub(super) fn say_missing(&mut self) {
        let shipped = self.ports.packs.pack_of(&self.iso).is_some();
        if shipped && self.notes.missing.insert(self.iso.clone()) {
            let lang = (args::LANG, NoticeArg::Text(self.lang_name.clone()));
            self.publish(&notice(keys::WORDS_MISSING, vec![lang]));
        }
    }

    /// Publishes `event`; a refusal is the bus's gate on a capability the module holds, so it never happens.
    pub(super) fn publish<E: Event>(&self, event: &E) {
        let _ = self.ports.bus.publish(event);
    }
}

/// The module's notice `key`, naming the module, with `more` arguments.
pub(crate) fn notice(key: &'static str, more: Vec<(&'static str, NoticeArg)>) -> Notice {
    let mut args = vec![(args::MODULE, NoticeArg::Text(ID.to_string()))];
    args.extend(more);
    Notice {
        key,
        module: Some(ID),
        args,
    }
}

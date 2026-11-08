//! Translate: the one consent question, the one read at a click after a yes, and the page opened off the actor.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use kx_module_api::ports::{Link, SelChip, SelKind};
use kx_module_api::{Mono, Notice, NoticeArg, NoticeAsk, Redacted, Secret, Zeroizing, args, keys};
use toml::Value;

use super::{Actor, Asking, Job, Msg};
use crate::manifest::ID;
use crate::settings::{SelSettings, TRANSLATE_OK_KEY};
use crate::strip::{Input, Pick};

/// The boxes whose selection may be read.
const READ_KINDS: [SelKind; 3] = [SelKind::Editable, SelKind::ReadOnly, SelKind::Terminal];

/// What the click's one read gave.
enum Fetch {
    /// The site's page for the text.
    Page(Link),
    /// The selection holds no text to translate.
    Blank,
    /// The box changed or the read failed.
    Failed,
}

impl Actor {
    /// Translate on the strip of `round`: at once after the user's yes, else the one question, which a click while it
    /// waits only points at the latest strip.
    pub(super) fn translate(&mut self, round: u32, pick: Pick) {
        if self.settings.translate_ok {
            return self.go(round, pick);
        }
        let now = self.ports.clock.now();
        if let Some(asking) = self
            .asking
            .as_mut()
            .filter(|a| !late(a, now, &self.settings))
        {
            (asking.round, asking.pick) = (round, pick);
            return;
        }
        let Some(site) = self.site else {
            return;
        };
        let mut notice = Notice {
            ask: Some(&NoticeAsk::YES_LATER),
            ..Notice::about(keys::TRANSLATE_ASK, ID)
        };
        let name = NoticeArg::Text(site.name.clone());
        notice.args.push((args::SITE, name));
        self.publish(&notice);
        self.asking = Some(Asking {
            notice,
            at: now,
            round,
            pick,
        });
    }

    /// The user answered `notice`: only the question's own Yes within `consent_s` is saved, and it opens the page only
    /// while the strip asked about still shows, as a moved or new selection gets a new round. A late Yes is told.
    pub(super) fn answered(&mut self, notice: &Notice, yes: bool) {
        let Some(asking) = self.asking.take_if(|a| a.notice == *notice) else {
            return;
        };
        if !yes {
            return;
        }
        if late(&asking, self.ports.clock.now(), &self.settings) {
            return self.say(keys::TRANSLATE_FAILED);
        }
        let saved = self
            .ports
            .handle
            .set(TRANSLATE_OK_KEY, Value::Boolean(true));
        if let Err(error) = saved {
            // The yes still holds for this run.
            tracing::warn!(kind = error.kind(), "the translate consent was not saved");
        }
        self.settings.translate_ok = true;
        // The strip asked about has already gone otherwise, so nothing is left to hide.
        if self.strip.pick(asking.round, SelChip::Translate).is_some() {
            self.go(asking.round, asking.pick);
        }
    }

    /// The open for the strip of `round` ended: its yes ticks Translate while that open is in flight, and its no tells
    /// the user in any case.
    pub(super) fn opened(&mut self, round: u32, ok: bool) {
        let chip = SelChip::Translate;
        let busy = self.land(|b| b.round == round && b.chip == chip);
        if !ok {
            self.fail(round);
        } else if busy.is_some() {
            self.step(Input::Acted { round, chip });
        }
    }

    /// Reads the selection of `pick` once and opens the site's page for it; a blank one ends the strip quietly.
    fn go(&mut self, round: u32, pick: Pick) {
        match self.fetch(&pick) {
            Fetch::Page(link) => self.open(round, link),
            Fetch::Blank => self.refuse(round),
            Fetch::Failed => self.fail(round),
        }
    }

    /// Checks the box of `pick` again right before its one read, capped one past `translate_chars` so a cut word
    /// shows; the text is wiped once its link is made.
    fn fetch(&mut self, pick: &Pick) -> Fetch {
        let Some(site) = self.site else {
            return Fetch::Failed;
        };
        if !(self.same(pick, &READ_KINDS) && self.still(pick)) {
            return Fetch::Failed;
        }
        let chars = self.settings.translate_chars;
        let cap = chars.min(site.limit).saturating_add(1);
        let Some(run) = self.ports.run.as_mut() else {
            return Fetch::Failed;
        };
        match run.read(pick.box_id, cap) {
            Ok(read) => {
                // Wrapped while the link is made, so no format shows it, and wiped after.
                let held: Secret = Redacted::new(Zeroizing::new(read.into_inner()));
                let text = held.expose().as_str();
                // A read that hit its cap may end inside a word, even once cleaning made it shorter.
                let capped = text.chars().count() >= usize::from(cap);
                let link = if capped {
                    site.cut_link(text, &self.code, chars)
                } else {
                    site.link(text, &self.code, chars)
                };
                link.map_or(Fetch::Blank, Fetch::Page)
            }
            Err(error) => {
                tracing::warn!(%error, "the selection could not be read");
                Fetch::Failed
            }
        }
    }

    /// Opens `link` off the actor, as a shell can hold the call; its end comes back as `Msg::Opened`.
    fn open(&mut self, round: u32, link: Link) {
        self.fly(round, SelChip::Translate);
        let (web, back) = (Arc::clone(&self.ports.web), self.ports.back.clone());
        let job: Job = Box::new(move || {
            let opened = catch_unwind(AssertUnwindSafe(|| web.open(&link)));
            let ok = match opened {
                Ok(Ok(())) => true,
                Ok(Err(error)) => {
                    tracing::warn!(%error, "the translate page did not open");
                    false
                }
                Err(_) => {
                    tracing::warn!("the browser link panicked");
                    false
                }
            };
            // Only a stopped actor misses this, and then no strip waits for it.
            let _ = back.send(Msg::Opened { round, ok });
        });
        if !(self.ports.spawn)(job) {
            self.opened(round, false);
        }
    }

    /// Hides the strip of `round` and tells the user the page did not open.
    fn fail(&mut self, round: u32) {
        self.refuse(round);
        self.say(keys::TRANSLATE_FAILED);
    }
}

/// True once `asking` waited past `consent_s` at `now`.
fn late(asking: &Asking, now: Mono, settings: &SelSettings) -> bool {
    now.saturating_since(asking.at) > settings.consent()
}

//! What Snip's windows draw, told once per change: each view with a new round, a click taken only while the windows
//! it saw keep their face, and the line the keyboard aims at, which no countdown or recording has, so Esc never ends
//! one.

use std::mem::{Discriminant, discriminant};

use kx_module_api::Bus;
use kx_module_api::ports::{
    CloseAsk, Countdown, PanelLine, Recording, Rect, SnipFit, SnipLine, SnipNote, SnipPage,
    SnipSheet, SnipStage, SnipTab, SnipView, Taken,
};

use super::Actor;
use crate::edit::Canvas;
use crate::flow::{Held, Rec, Reel, Stage, Strip};
use crate::part;
use crate::strip::{self, Size};

/// The windows' kind, whether the question shows, and the panel's page: a click made on another face is dropped.
type Face = (Discriminant<SnipStage>, bool, Option<SnipTab>);

/// What was told last.
#[derive(Debug, Default)]
pub(super) struct Told {
    page: Option<SnipPage>,
    /// The last view's round.
    round: u32,
    /// The round since which the windows have kept their face.
    since: u32,
    /// The round told with the last step aside, whose answer counts from then on.
    asked: u32,
    face: Option<Face>,
    /// The line as the keyboard was told.
    line: Option<PanelLine>,
    /// The keyboard was told a line, so only a change is told again.
    said: bool,
}

impl Told {
    /// Tells `page` as a new round when it changed.
    pub(super) fn show(&mut self, mut page: SnipPage, bus: &Bus) {
        page.round = self.round;
        if self.page.as_ref() != Some(&page) {
            self.tell(page, bus);
        }
    }

    /// Tells `page` as a new round even unchanged, just before a step aside, whose answer counts from that round.
    pub(super) fn ask(&mut self, page: SnipPage, bus: &Bus) {
        self.tell(page, bus);
        self.asked = self.round;
    }

    /// Tells `page` as a new round; windows of another face start a new span for clicks.
    fn tell(&mut self, mut page: SnipPage, bus: &Bus) {
        self.round = self.round.wrapping_add(1);
        page.round = self.round;
        let tab = page.panel.as_ref().map(|sheet| match sheet {
            SnipSheet::Capture(_) => SnipTab::Capture,
            SnipSheet::Video(_) => SnipTab::Video,
        });
        let face = Some((discriminant(&page.stage), page.ask.is_some(), tab));
        if self.face != face {
            self.face = face;
            self.since = self.round;
        }
        // The module holds `Snip`, which the view needs, or its start would have failed.
        let _ = bus.publish(&SnipView::new(page.clone()));
        self.page = Some(page);
    }

    /// Tells the keyboard `line` when it changed.
    pub(super) fn tell_line(&mut self, line: Option<PanelLine>, bus: &Bus) {
        if !self.said || self.line != line {
            let _ = bus.publish(&SnipLine(line.clone()));
            (self.line, self.said) = (line, true);
        }
    }

    /// The keyboard asked, so the line is told again at the next look, even unchanged.
    pub(super) fn again(&mut self) {
        self.said = false;
    }

    /// True when `round` lies between the windows' last change of face and now, wrapping.
    pub(super) fn fresh(&self, round: u32) -> bool {
        self.within(round, self.since)
    }

    /// True when the ui's answer to a step aside saw the view told with it, or a newer one.
    pub(super) fn answers(&self, round: u32) -> bool {
        self.within(round, self.asked)
    }

    /// True when `round` lies between `mark` and now, wrapping.
    fn within(&self, round: u32, mark: u32) -> bool {
        self.round.wrapping_sub(round) <= self.round.wrapping_sub(mark)
    }

    /// The view told last.
    pub(super) fn page(&self) -> Option<&SnipPage> {
        self.page.as_ref()
    }

    /// A line shows, so a key the keyboard sends is the snip's.
    pub(super) fn aimed(&self) -> bool {
        self.line.is_some()
    }
}

/// The theme's sizes before the ui tells them: no bar height, no gap, no editor sizes, no slop and no handle.
pub(super) fn unfit() -> SnipFit {
    SnipFit {
        strip: 0,
        bar: 0,
        gap: 0,
        thick: Vec::new(),
        text: Vec::new(),
        slop: 0,
        handle: 0,
    }
}

impl Actor {
    /// The view now.
    pub(super) fn page(&self) -> SnipPage {
        let frozen = self.flow.frozen().map(|f| f.screens.clone());
        SnipPage {
            round: 0,
            frozen: frozen.unwrap_or_default(),
            stage: self.stage(),
            ask: self.flow.asking().then(|| CloseAsk {
                save_dim: self.save_dim(),
            }),
            note: self.note.map(|(note, _)| note),
            panel: self.panel.map(|tab| self.sheet(tab)),
            dialog: self.dialog(),
        }
    }

    /// A system Save or folder dialog sits in front, so Snip's windows leave the top and Esc is the dialog's.
    pub(super) fn dialog(&self) -> bool {
        self.keeping.saving || self.folders > 0
    }

    /// The capture's windows, as the flow's stage says.
    fn stage(&self) -> SnipStage {
        match self.flow.stage() {
            Stage::Idle | Stage::Freezing => SnipStage::Idle,
            Stage::Picking(picking) => self
                .picking(*picking)
                .map_or(SnipStage::Idle, SnipStage::Pick),
            Stage::Strip(strip) => SnipStage::Strip(self.taken(strip)),
            Stage::Count(count) => SnipStage::Count(Countdown {
                part: count.film.part,
                left_s: count.left_s,
            }),
            Stage::Record(rec) => SnipStage::Record(self.recording(rec)),
            Stage::Edit(edit) => self
                .editor(*edit)
                .map_or(SnipStage::Idle, |e| SnipStage::Edit(Box::new(e))),
        }
    }

    /// The part taken with its strip under, above or at the bottom of its screen, keyless while a handle is held; a
    /// recorded video's strip keeps to the live screen's work area and shows its speed and its length at that speed.
    fn taken(&self, strip: &Strip) -> Taken {
        let home = strip.home;
        let size = Size::scaled(self.fit.strip, self.fit.gap, home.dpi);
        let reel = match strip.held {
            Held::Video(reel) => Some(reel),
            Held::Picture(_) | Held::Ready(_) => None,
        };
        let screen = reel.map_or(home.rect, |_| self.work(home.rect));
        let keys = if strip.gripped() {
            Vec::new()
        } else {
            self.strip_keys(&strip.held)
        };
        Taken {
            part: strip.part,
            screen,
            spot: strip::place(strip.part, screen, size),
            keys,
            length_ms: reel.map(Reel::length_ms),
            speed: reel.map(|reel| reel.speed),
            done: strip.done,
            waits: self.flow.waits(),
            frame: strip.frame,
        }
    }

    /// The red edge and the record bar above the part or at the top of its screen's work area.
    fn recording(&self, rec: &Rec) -> Recording {
        let (part, home) = (rec.film.part, rec.film.home);
        let size = Size::scaled(self.fit.bar, self.fit.gap, home.dpi);
        let screen = self.work(home.rect);
        Recording {
            part,
            screen,
            spot: strip::bar(part, screen, size),
            elapsed_ms: rec.ms,
            live: rec.live,
            keys: self.rec_keys(rec.film.sounds),
        }
    }

    /// The work area told for the live screen `rect`, else the screen itself.
    fn work(&self, rect: Rect) -> Rect {
        let mut told = self.seen.work.iter().copied();
        told.find(|area| part::meets(*area, rect)).unwrap_or(rect)
    }

    /// The line the keyboard aims at: the editor's text while typed, else none while a video counts down or records,
    /// so an Esc meant for the recorded app never reaches the snip; else the windows' own.
    pub(super) fn line(&self) -> Option<PanelLine> {
        let typing = self.canvas.as_ref().is_some_and(Canvas::typing);
        let windows = match self.flow.stage() {
            Stage::Idle | Stage::Count(_) | Stage::Record(_) => self.panel.is_some(),
            Stage::Freezing | Stage::Picking(_) | Stage::Strip(_) | Stage::Edit(_) => true,
        };
        if typing && !self.flow.asking() {
            Some(PanelLine::Private)
        } else {
            windows.then_some(PanelLine::None)
        }
    }

    /// On the strip of a video to record, the drive's room is read once: under `min_free_mb`, Record dims with a note.
    pub(super) fn check_room(&mut self) {
        let ready = matches!(
            self.flow.stage(),
            Stage::Strip(Strip {
                held: Held::Ready(_),
                ..
            })
        );
        if ready && !self.seen.ready {
            self.seen.low = self.low();
            if self.seen.low {
                self.show_note(SnipNote::DiskLow);
            }
        }
        self.seen.ready = ready;
    }
}

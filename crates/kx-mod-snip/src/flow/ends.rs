//! A snip's start and every end: the key lights, the keyboard steps aside and, once it has, the screens freeze; at the
//! end the videos not copied go, the keyboard comes back and the key goes dark (edge 17). Close asks first when nothing
//! was kept.

use std::time::Duration;

use kx_module_api::ports::{
    AskKey, BarKey, FreezeError, Frozen, SnipAside, SnipKeyboard, SnipLit, SnipNote,
};

use super::{After, Call, Flow, Held, Out, Picking, Stage};

impl Flow {
    /// The Snip key: it starts a snip, ends one picking or counting down, stops a recording, and closes the strip or
    /// the editor, which asks first when nothing was kept; the editor's then starts a new snip (edge 22).
    pub(super) fn key(&mut self, out: &mut Vec<Out>) {
        if !self.free() {
            return;
        }
        match self.stage {
            Stage::Idle => self.start(false, out),
            Stage::Freezing | Stage::Picking(_) | Stage::Count(_) => self.end(out),
            Stage::Strip(_) => self.close(After::End, out),
            Stage::Record(_) => self.stop(out),
            Stage::Edit(_) => self.close(After::Start, out),
        }
    }

    /// Our keyboard's Esc, and the hand's over frozen screens: at the question Don't save while frozen screens show, so
    /// a second Esc always ends the snip (ADR-0059), else Back; a held handle drops first (edge 4); elsewhere as the
    /// Snip key, but it never starts a snip.
    pub(super) fn esc(&mut self, out: &mut Vec<Out>) {
        if self.busy.is_some() {
            return;
        }
        if self.ask.is_some() {
            let way = if self.frozen.is_some() {
                AskKey::Drop
            } else {
                AskKey::Back
            };
            return self.answer(way, out);
        }
        if self.let_go() {
            return;
        }
        match self.stage {
            Stage::Idle => {}
            Stage::Freezing | Stage::Picking(_) | Stage::Count(_) => self.end(out),
            Stage::Strip(_) | Stage::Edit(_) => self.close(After::End, out),
            Stage::Record(_) => self.stop(out),
        }
    }

    /// The key lights and the keyboard steps aside as `keyboard` says; the screens freeze at once when it stays, else
    /// once the ui answers or `aside_ms` passes. When Snip's `windows` showed, as the editor, a stay waits too.
    pub(super) fn start(&mut self, windows: bool, out: &mut Vec<Out>) {
        self.lit = true;
        out.push(Out::Lit(SnipLit::On));
        self.stage = Stage::Freezing;
        self.fresh();
        let aside = match self.settings.keyboard {
            SnipKeyboard::Bubble => SnipAside::Bubble,
            SnipKeyboard::Hidden => SnipAside::Hidden,
            SnipKeyboard::Shown if windows => SnipAside::Stay,
            SnipKeyboard::Shown => return out.push(Out::Freeze),
        };
        self.aside = aside != SnipAside::Stay;
        out.push(Out::Aside(aside));
        let wait = Duration::from_millis(u64::from(self.settings.aside_ms));
        self.stepping = Some(self.now().saturating_add(wait));
    }

    /// The keyboard stepped aside, or the wait for it ended: the screens freeze, once.
    pub(super) fn stepped(&mut self, out: &mut Vec<Out>) {
        if self.stepping.take().is_some() && matches!(self.stage, Stage::Freezing) {
            out.push(Out::Freeze);
        }
    }

    /// The freeze's answer: Region is ready, or the snip ends with a note saying why. One that comes while the freeze
    /// waits for the keyboard is an older snip's.
    pub(super) fn froze(&mut self, answer: Result<Frozen, FreezeError>, out: &mut Vec<Out>) {
        if !matches!(self.stage, Stage::Freezing) || self.stepping.is_some() {
            return;
        }
        match answer {
            Ok(frozen) => {
                self.frozen = Some(frozen);
                let way = BarKey::Region;
                self.stage = Stage::Picking(Picking { way, corner: None });
            }
            Err(error) => {
                let note = match error {
                    FreezeError::TooBig => SnipNote::TooBig,
                    FreezeError::Refused(_) => SnipNote::FreezeFailed,
                };
                out.push(Out::Note(note));
                self.end(out);
            }
        }
    }

    /// Closes the strip or the editor: at once when nothing is held or it was kept, else the question asks first. A
    /// held handle drops, so none is held under the question.
    pub(super) fn close(&mut self, after: After, out: &mut Vec<Out>) {
        self.let_go();
        let holds = match &self.stage {
            Stage::Strip(strip) => !matches!(strip.held, Held::Ready(_)),
            Stage::Edit(_) => true,
            _ => false,
        };
        if holds && !self.kept() {
            self.ask = Some(after);
        } else {
            self.finish(after, out);
        }
    }

    /// Ends the snip, then starts a new one when `after` says so, which waits for the ui to hide the editor first.
    pub(super) fn finish(&mut self, after: After, out: &mut Vec<Out>) {
        self.end(out);
        if after == After::Start {
            self.start(true, out);
        }
    }

    /// Ends the snip, cancelling its recording first; the host deletes that video.
    pub(super) fn end(&mut self, out: &mut Vec<Out>) {
        if let Stage::Record(rec) = &self.stage {
            let video = rec.video;
            out.push(Out::Run {
                video,
                call: Call::Cancel,
            });
        }
        self.wrap(out);
    }

    /// Ends the snip whose run is over: the videos not copied go, the keyboard comes back and the key goes dark.
    /// Settings that came during the snip apply from now on.
    pub(super) fn wrap(&mut self, out: &mut Vec<Out>) {
        if matches!(self.stage, Stage::Idle) {
            return;
        }
        self.spare_copy();
        self.files.drop_all(out);
        self.release(out);
        out.push(Out::End);
        self.stage = Stage::Idle;
        self.frozen = None;
        self.ask = None;
        self.busy = None;
        (self.due, self.stepping) = (None, None);
        if let Some(next) = self.next.take() {
            self.settings = *next;
        }
    }

    /// The key goes dark and the keyboard comes back, each if not yet.
    pub(super) fn release(&mut self, out: &mut Vec<Out>) {
        if std::mem::take(&mut self.lit) {
            out.push(Out::Lit(SnipLit::Off));
        }
        if std::mem::take(&mut self.aside) {
            out.push(Out::Aside(SnipAside::Back));
        }
    }
}

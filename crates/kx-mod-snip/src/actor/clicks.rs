//! The bus's inputs: the Snip key, each click on Snip's windows, routed to the flow, the editor's model or the panel,
//! the keyboard's ask, and its keys, which reach the snip only while one of its lines shows, so an Esc meant for
//! another app never stops a recording; the hand's Esc acts only over frozen screens.

use kx_module_api::ports::{EditKey, PanelKey, Point, SnipAct, SnipClick, SnipFit};

use super::Actor;
use crate::edit::{Canvas, Did};
use crate::flow::{In, Stage};

impl Actor {
    /// The Snip key: the text being typed ends first, as a click elsewhere does; one that starts a snip is timed.
    pub(super) fn key(&mut self) {
        if matches!(self.flow.stage(), Stage::Idle) {
            self.keyed = Some(self.now());
        }
        self.end_text();
        self.step(In::Key);
    }

    /// A click: the theme's sizes, the work areas, the ui's answers, the editor's place, the tray's Open and the hand's
    /// Esc act on any view; any other only on the windows' face it saw, and never on a dimmed key.
    pub(super) fn act(&mut self, act: SnipAct) {
        match act.click {
            SnipClick::Fit(fit) => self.refit(fit),
            SnipClick::Screens(screens) => {
                self.seen.work = screens.iter().map(|s| s.rect).collect();
            }
            SnipClick::Aside if self.told.answers(act.round) => self.step(In::Aside),
            SnipClick::Aside => {}
            SnipClick::Bare => {
                self.seen.bare = true;
                self.step(In::Bare);
            }
            SnipClick::Placed { rect, full, .. } => self.placed(rect, full),
            SnipClick::Open => self.open_panel(),
            SnipClick::Esc => self.hand_esc(),
            click if self.told.fresh(act.round) && !self.dimmed(&click) => self.click(click),
            _ => {}
        }
    }

    /// A fresh click on a key that acts.
    fn click(&mut self, click: SnipClick) {
        match click {
            SnipClick::Bar(key) => self.step(In::Bar(key)),
            SnipClick::Spot(at) => self.step(In::Spot(at)),
            SnipClick::Handle(handle) if self.canvas.is_some() => self.draw(|c| c.handle(handle)),
            SnipClick::Handle(handle) => self.step(In::Handle(handle)),
            SnipClick::Strip(key) => self.step(In::Strip(key)),
            SnipClick::Record(key) => self.step(In::Record(key)),
            SnipClick::Ask(key) => self.step(In::Ask(key)),
            SnipClick::Edit(key) => self.edit_key(key),
            SnipClick::Crop(key) => self.draw(|c| c.crop(key)),
            SnipClick::Flat { keep, shot } => {
                self.end_text();
                self.step(In::Flat { keep, shot });
            }
            SnipClick::Tool(tool) => self.draw(|c| c.tool(tool)),
            SnipClick::Ink(ink) => self.draw(|c| c.ink(ink)),
            SnipClick::Size { up } => self.draw(|c| c.size(up)),
            SnipClick::Press { at, scale } => self.draw(|c| c.press(at, scale)),
            SnipClick::Stroke(path) => self.stroke(&path),
            SnipClick::Caret(units) => self.draw(|c| c.caret(*units.expose())),
            panel => self.panel_click(&panel),
        }
    }

    /// A pen's or highlighter's path, of which the editor keeps the first `stroke_max` points.
    fn stroke(&mut self, path: &[Point]) {
        let most = usize::try_from(self.held.stroke_max).unwrap_or(usize::MAX);
        self.draw(|c| c.stroke(&path[..path.len().min(most)]));
    }

    /// Undo and Redo are the editor model's; Close is the flow's, after the text being typed ends.
    fn edit_key(&mut self, key: EditKey) {
        match key {
            EditKey::Undo => self.draw(Canvas::undo),
            EditKey::Redo => self.draw(Canvas::redo),
            EditKey::Close => {
                self.end_text();
                self.step(In::Edit(key));
            }
        }
    }

    /// The keyboard asked: the key's light is told now and the line after this message, changed or not.
    pub(super) fn asked(&mut self) {
        self.publish(&self.flow.lit());
        self.told.again();
    }

    /// A keyboard key, only while a line of the snip shows: the editor's text takes it first, then Esc closes the
    /// panel, lets go of Crop's held handle or drops its waiting frame, or goes to the flow.
    pub(super) fn line_key(&mut self, key: &PanelKey) {
        if !self.told.aimed() {
            return;
        }
        let asking = self.flow.asking();
        if let Some(did) = self
            .canvas
            .as_mut()
            .filter(|_| !asking)
            .and_then(|c| c.line(key))
        {
            return self.drawn(did);
        }
        if *key != PanelKey::Esc || self.panel.take().is_some() {
            return;
        }
        let dropped = self.canvas.as_mut().filter(|_| !asking).map(Canvas::esc);
        if dropped != Some(Did::Show) {
            self.step(In::Esc);
        }
    }

    /// The hand's Esc (ADR-0059), on any view: while the screens freeze or show frozen and no system dialog sits in
    /// front, it closes the panel or goes to the flow as our keyboard's Esc does; else that Esc is the app's.
    fn hand_esc(&mut self) {
        let frozen = matches!(self.flow.stage(), Stage::Freezing) || self.flow.frozen().is_some();
        if !frozen || self.dialog() {
            return;
        }
        if self.panel.take().is_none() {
            self.step(In::Esc);
        }
    }

    /// Hands a picture click to the editor's model, never while the close question shows.
    fn draw(&mut self, edit: impl FnOnce(&mut Canvas) -> Did) {
        if self.flow.asking() {
            return;
        }
        if let Some(did) = self.canvas.as_mut().map(edit) {
            self.drawn(did);
        }
    }

    /// The flow hears each change of the picture.
    fn drawn(&mut self, did: Did) {
        if did == Did::Change {
            self.step(In::Changed);
        }
    }

    /// Ends the text being typed, so a Copy, Save, Close or the Snip key keeps it; the flow hears it first.
    fn end_text(&mut self) {
        if let Some(did) = self.canvas.as_mut().map(Canvas::end) {
            self.drawn(did);
        }
    }

    /// The theme's sizes, which the strip, the frame's handles and the editor's model follow.
    fn refit(&mut self, fit: SnipFit) {
        if let Some(canvas) = &mut self.canvas {
            canvas.fit(fit.clone());
        }
        self.step(In::HitPx(fit.handle));
        self.fit = fit;
    }
}

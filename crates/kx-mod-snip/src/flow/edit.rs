//! The picture editor and what changes a snip: a change makes it unkept and takes its tick, the flat picture's Copy or
//! Save keeps the editor open, and Close asks first when nothing was kept.

use kx_module_api::ports::{EditKey, Keep, Shot};

use super::{After, Flow, Out, Stage, What};

impl Flow {
    /// An editor key: Close; Undo and Redo are `edit::Canvas`'s.
    pub(super) fn edit_key(&mut self, key: EditKey, out: &mut Vec<Out>) {
        let editing = matches!(self.stage, Stage::Edit(_));
        if editing && self.free() && key == EditKey::Close {
            self.close(After::End, out);
        }
    }

    /// The editor's picture changed; a late word after the editor closed changes nothing.
    pub(super) fn drew(&mut self) {
        if matches!(self.stage, Stage::Edit(_)) {
            self.change();
        }
    }

    /// The editor's picture or a recorded video's speed changed, so the snip is unkept and its tick goes.
    pub(super) fn change(&mut self) {
        if let Some(tick) = self.lasting() {
            *tick = None;
            self.changes = self.changes.wrapping_add(1);
            self.due = None;
        }
    }

    /// The flat picture for the editor's Copy or Save, or the question's Save, which closes the question.
    pub(super) fn flat(&mut self, keep: Keep, shot: Shot, out: &mut Vec<Out>) {
        if !matches!(self.stage, Stage::Edit(_)) || self.busy.is_some() {
            return;
        }
        let after = match (self.ask, keep) {
            (None, _) => None,
            (Some(after), Keep::Save) => Some(after),
            (Some(_), Keep::Copy) => return,
        };
        self.ask = None;
        self.ask_keep(keep, What::Picture(shot), after, out);
    }
}

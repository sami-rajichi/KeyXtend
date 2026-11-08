//! A keycap's click: Copy and Cut check the box again, Paste looks again, then the keyboard sends the edit.

use kx_module_api::keys;
use kx_module_api::ports::{ChipAct, Edit, EditAsk, EditDone, SelChip, SelKind};

use super::{Actor, Busy};
use crate::strip::{Input, Pick};

/// The boxes a paste may go into.
const PASTE_KINDS: [SelKind; 2] = [SelKind::Editable, SelKind::Terminal];

impl Actor {
    /// The click of `act`, when its strip is live, shows that keycap and has no act in flight.
    pub(super) fn act(&mut self, act: ChipAct) {
        if self.busy.is_some_and(|b| b.round == act.round) {
            return;
        }
        let Some(pick) = self.strip.pick(act.round, act.chip) else {
            return;
        };
        self.swallow = false;
        let round = act.round;
        match pick.chip {
            SelChip::Copy if self.still(&pick) => self.edit(round, &pick, Edit::Copy),
            SelChip::Cut if self.still(&pick) => self.edit(round, &pick, Edit::Cut),
            SelChip::Paste if self.same(&pick, &PASTE_KINDS) => {
                self.edit(round, &pick, Edit::Paste);
            }
            SelChip::Translate => self.translate(round, pick),
            SelChip::Copy | SelChip::Cut | SelChip::Paste => self.refuse(round),
        }
    }

    /// The keyboard's answer to an edit: its yes ticks the keycap of the edit in flight, and its no, for any strip,
    /// hides that strip and tells the user.
    pub(super) fn done(&mut self, done: EditDone) {
        let ours = |b: &mut Busy| b.round == done.round && b.chip != SelChip::Translate;
        let busy = self.busy.take_if(ours);
        if !done.ok {
            self.refuse(done.round);
            self.say(keys::KEYS_NOT_SENT);
        } else if let Some(Busy { round, chip, .. }) = busy {
            // The keyboard tells one break for the edit's keys, which is no key of the user's.
            self.swallow = true;
            self.step(Input::Acted { round, chip });
        }
    }

    /// True while the box of `pick` still holds a selection, checked right before any key or read.
    pub(super) fn still(&mut self, pick: &Pick) -> bool {
        (self.ports.run.as_mut()).is_some_and(|run| run.still(pick.box_id))
    }

    /// True when a fresh look finds the box of `pick` focused, of the same kind, and one of `kinds`.
    pub(super) fn same(&mut self, pick: &Pick, kinds: &[SelKind]) -> bool {
        let Some(run) = self.ports.run.as_mut() else {
            return false;
        };
        run.look().is_ok_and(|spot| {
            let kind = spot.kind;
            spot.box_id == pick.box_id && kind == pick.kind && kinds.contains(&kind)
        })
    }

    /// Asks the keyboard for `edit` on the box of `pick`, from the strip of `round`.
    fn edit(&mut self, round: u32, pick: &Pick, edit: Edit) {
        // The keyboard may answer from inside its handler, so the act is in flight before the ask.
        self.fly(round, pick.chip);
        let terminal = pick.kind == SelKind::Terminal;
        self.publish(&EditAsk {
            round,
            edit,
            terminal,
        });
    }

    /// Puts `chip`'s act on the strip of `round` in flight, late once `edit_ms` has passed.
    pub(super) fn fly(&mut self, round: u32, chip: SelChip) {
        let until = self.ports.clock.now().saturating_add(self.settings.edit());
        self.busy = Some(Busy { round, chip, until });
    }

    /// An act that went unanswered past `edit_ms`: its strip hides, if it still shows, and the user hears it did not go.
    pub(super) fn expire(&mut self, late: Busy) {
        self.refuse(late.round);
        self.say(if late.chip == SelChip::Translate {
            keys::TRANSLATE_FAILED
        } else {
            keys::KEYS_NOT_SENT
        });
    }
}

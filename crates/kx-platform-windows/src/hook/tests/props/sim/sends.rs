//! Our sends as Windows lands them: the taken part of each batch, a gesture that slips in right after it, and our
//! hooks going in or out.

use super::super::super::fake::Call;
use super::super::{Ev, Fate};
use super::Sim;

impl Sim {
    /// Takes the host's OS calls: the taken part of each batch lands as Windows puts it, a slipped gesture lands after
    /// it, and the hooks go in or out.
    pub(super) fn collect(&mut self) {
        for call in self.rig.world.take() {
            match call {
                Call::Send(acts) => {
                    let (takes, len) = (self.rig.world.takes.get(), acts.len());
                    let took = takes.map_or(len, |n| n.min(len));
                    let fate = if took < len { Fate::Cut } else { Fate::Kept };
                    for (i, act) in acts.iter().enumerate() {
                        self.mark(*act, if i < took { fate } else { Fate::Lost });
                    }
                    let ours: Vec<_> = self.rig.world.ids.borrow_mut().drain(..took).collect();
                    let sent = ours.into_iter().map(|(act, id)| Ev {
                        act,
                        ours: true,
                        id,
                        unseen: false,
                    });
                    self.land(sent.collect());
                    if let Some(g) = self.slip.take() {
                        self.gesture(g);
                    }
                }
                Call::Hook => self.hooked = true,
                Call::Unhook => self.hooked = false,
                Call::Arm(_) | Call::Wake => {}
            }
        }
        self.grew |= self.rig.host.room() != self.room;
    }

    /// Puts our taken `sent` where Windows does. `SendInput` returns only once its input passed, so with our hooks out
    /// what waited ahead of it passes first, by our hooks; else it reaches the apps when heard inside, or joins the stream.
    fn land(&mut self, sent: Vec<Ev>) {
        if !self.hooked && !sent.is_empty() {
            self.apps.extend(self.stream.drain(..));
            self.apps.extend(sent);
        } else if self.rig.world.inside.get() {
            self.apps.extend(sent);
        } else {
            self.stream.extend(sent);
        }
    }
}

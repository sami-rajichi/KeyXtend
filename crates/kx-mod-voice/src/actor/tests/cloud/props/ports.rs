//! The fakes' answers: the service's texts, faults and cuts, the clipboard history's pastes and takes, the keyboard's
//! typing and the clock, with the act that answers the oldest thing still out.

use kx_module_api::ports::{ClipTake, ClipText, CloudFault, TextDone, TextPasted};
use kx_platform_fake::{CloudAnswer, CloudLog, CloudState};

use super::super::{key, secret};
use super::act::Act;
use super::{Seen, said};
use crate::actor::Msg;
use crate::actor::tests::taps::Heard;

impl Seen {
    /// The act that answers the oldest thing out: a stopped run, a sending run, a paste, a text, then a take.
    pub(super) fn due(&self) -> Option<Act> {
        if self.stopped().is_some() {
            Some(Act::Done)
        } else if self.pick(0).is_some() {
            Some(Act::Answer(0, None))
        } else if !self.unpasted().is_empty() {
            Some(Act::Pasted(0, true))
        } else if self.typed < self.rig.asks().len() {
            Some(Act::Typed(true))
        } else {
            self.untaken().map(|_| Act::Taken)
        }
    }

    /// The sending run at `at`, wrapped.
    fn pick(&self, at: usize) -> Option<usize> {
        let runs = self.rig.fake.cloud.calls().runs;
        let out = |(at, r): (usize, &CloudLog)| (r.state == CloudState::Sending).then_some(at);
        let places: Vec<usize> = runs.iter().enumerate().filter_map(out).collect();
        places.get(at % places.len().max(1)).copied()
    }

    /// The service answers the sending run at `at`, wrapped: its next text, else `fault`.
    pub(super) fn answer(&mut self, at: usize, fault: Option<CloudFault>) {
        let Some(run) = self.pick(at) else {
            return;
        };
        let answer = fault.map_or_else(|| self.token(run), CloudAnswer::Fault);
        let _ = self.rig.fake.cloud.answer(run, vec![answer]);
        self.rig.pump();
    }

    /// A text no other holds, kept as run `run`'s: a `w`, its number and an `x`, so none sits inside another.
    fn token(&mut self, run: usize) -> CloudAnswer {
        let token = format!("w{}x", self.tokens.len());
        let answer = CloudAnswer::text(&token);
        self.tokens.push((token, run));
        answer
    }

    /// The connection to the sending run at `at`, wrapped, drops.
    pub(super) fn cut(&mut self, at: usize) {
        if let Some(run) = self.pick(at) {
            let _ = self.rig.fake.cloud.cut(run, CloudFault::Network);
            self.rig.pump();
        }
    }

    /// The paste rounds not answered yet, oldest first.
    fn unpasted(&self) -> Vec<u32> {
        let rounds = self.rig.pastes().into_iter().map(|p| p.round);
        rounds.filter(|r| !self.pasted.contains(r)).collect()
    }

    /// The clipboard history answers the open paste at `at`, wrapped; one pasted before its text was typed put its
    /// tokens in.
    pub(super) fn paste_answer(&mut self, at: usize, ok: bool) {
        let open = self.unpasted();
        let Some(&round) = open.get(at % open.len().max(1)) else {
            return;
        };
        self.pasted.insert(round);
        let (pastes, asks) = (self.rig.pastes(), self.rig.asks());
        let paste = pastes.iter().find(|p| p.round == round);
        let text = paste.map(|p| said(&p.text));
        let typed = asks.iter().any(|a| Some(said(&a.text)) == text);
        if let Some(text) = text.filter(|_| ok && !typed) {
            let tokens = self.tokens.iter();
            let held = tokens.filter(|(t, _)| text.contains(t.as_str()));
            self.first.extend(held.map(|(t, _)| t.clone()));
        }
        self.rig.send(Msg::Pasted(TextPasted { round, ok }));
    }

    /// The keyboard answers the oldest text not answered; a refused erase may leave a text waiting for good.
    pub(super) fn type_answer(&mut self, ok: bool) {
        let Some(ask) = self.rig.asks().get(self.typed).cloned() else {
            return;
        };
        self.typed += 1;
        self.lost |= !ok && ask.erase > 0;
        let chars = ask.text.expose().chars().count();
        let count = if ok { u32::try_from(chars).unwrap() } else { 0 };
        let round = ask.round;
        self.rig.send(Msg::Typed(TextDone { round, count, ok }));
    }

    /// The newest key page's take not answered.
    fn untaken(&self) -> Option<ClipTake> {
        self.rig.heard().into_iter().rev().find_map(|h| match h {
            Heard::Take(take) if !self.taken.contains(&take.round) => Some(take),
            _ => None,
        })
    }

    /// The clipboard history answers the newest open take with the key.
    pub(super) fn take_answer(&mut self) {
        let Some(take) = self.untaken() else {
            return;
        };
        self.taken.insert(take.round);
        let text = ClipText::Text(secret(&key()));
        self.rig.send(Msg::Taken(take.answer(text)));
    }

    /// The clock moves on by the key page's wait when `take`, else by Paste at once's.
    pub(super) fn wait(&mut self, take: bool) {
        let s = self.rig.actor.settings();
        let by = if take { s.take_wait() } else { s.paste_wait() };
        self.rig.advance(by);
    }
}

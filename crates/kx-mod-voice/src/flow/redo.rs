//! A recording written again: Cancel keeps it for Transcribe anyway, and Try again writes it with the model, or sends
//! it to the service, the language gets now.

use kx_module_api::ports::{BarButton, BarHint};

use super::net::Engine;
use super::{Flow, Out, Phase, RunCall, State, Supply};

impl Flow {
    /// Cancel: nothing is typed, and the recording stays for Transcribe anyway. A cloud dictation's run hands its
    /// speech over and keeps it, and its upload is dropped.
    pub(super) fn cancel(&mut self, out: &mut Vec<Out>) {
        self.lit = false;
        let listening = self.phase() == Phase::Listening;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        if take.engine.net().is_some() {
            take.halt(out);
            if listening {
                out.push(take.call(RunCall::Stop));
            }
        } else if take.open {
            out.push(take.call(RunCall::Cancel));
        }
        take.pieces.clear();
        let iso = take.lang.iso().to_owned();
        self.show_note(BarHint::Cancelled, Some(BarButton::Anyway), iso);
    }

    /// Transcribe anyway: the cancelled recording is written with the ask it was last written with, or sent again.
    pub(super) fn anyway(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        let cancelled = matches!(
            &self.state,
            State::Note {
                hint: BarHint::Cancelled,
                ..
            }
        );
        if !cancelled || !self.anyway_ok() {
            return;
        }
        let now = self.now();
        let Some(take) = self.take.as_mut() else {
            return;
        };
        take.redo();
        if let (Some(wire), Some(_)) = (take.wire.as_mut(), take.engine.net()) {
            wire.since = now;
            return self.upload(supply, out);
        }
        out.push(take.call(RunCall::Again(take.ask.clone())));
        self.state = State::Transcribing;
    }

    /// Try again: the held recording is written again with the model and device the language gets now. The Text bar
    /// waits in the take, to come back should the new run write nothing.
    pub(super) fn again(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        if !self.can_again() {
            return;
        }
        let Some(take) = self.take.as_mut() else {
            return;
        };
        if take.engine.net().is_some() {
            return self.retry(supply, out);
        }
        match supply.supply(take.lang.iso()) {
            Ok(ready) => {
                let ask = (self.settings).ask(Some(ready.model.clone()), ready.card.clone());
                take.engine = Engine::Local(ready);
                take.ask = ask.clone();
                take.redo();
                out.push(take.call(RunCall::Again(ask)));
                take.before = Some(std::mem::replace(&mut self.state, State::Transcribing));
            }
            Err(lack) => {
                let lang = take.lang.clone();
                self.lack(lack, &lang);
            }
        }
    }
}

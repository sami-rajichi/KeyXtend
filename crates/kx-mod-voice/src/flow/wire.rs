//! A cloud dictation (ADR-0047): a listen-only run from Start, its speech sent at Stop to the service the bar shows,
//! the replies typed once, and a note naming why when none come; the recording stays for Try again and "Use the local
//! model". Nothing moves to another service or model by itself, and once the local model wrote a take it stays here.

use kx_module_api::ports::{
    BarButton, BarHint, CloudNote, CloudWhy, LocalUse, SpeechAsk, SpeechDone,
};

use super::acts::local_ask;
use super::net::{self, Engine, Net, Sent, Upload};
use super::notes::lost;
use super::take::Lang;
use super::{Count, Flow, Out, Phase, RunCall, State, Supply};

impl Flow {
    /// Starts a cloud dictation, listen-only and cut within the service's row; a password box takes the local model,
    /// or the bar says none is ready (edge 10). A pick with no key or newer terms says so and never listens.
    pub(super) fn start_cloud(
        &mut self,
        lang: Lang,
        net: Net,
        supply: &dyn Supply,
        out: &mut Vec<Out>,
    ) {
        if self.password() {
            return match supply.supply(lang.iso()) {
                Ok(ready) => self.open(lang, ready, out),
                Err(_) => self.show_cloud(CloudWhy::NoLocal, lang.iso().to_owned()),
            };
        }
        if let Some(why) = net::blocked(&net, Some(&net)) {
            return self.show_cloud(why, lang.iso().to_owned());
        }
        let ask = SpeechAsk {
            piece_s: net.piece_s,
            ..self.settings.ask(None, None)
        };
        self.open_with(lang, Engine::Cloud(net), ask, out);
    }

    /// The listen-only run handed its speech over: it goes to the service while the dictation runs, unless no speech
    /// was heard; a note shown since keeps it for the user's act. Once the local model took the take over, this end is
    /// only taken in: nothing is counted, closed or sent.
    pub(super) fn listened(&mut self, done: SpeechDone, supply: &dyn Supply, out: &mut Vec<Out>) {
        self.lit = false;
        let busy = self.busy();
        let Some(take) = self.take.as_mut() else {
            return;
        };
        if take.engine.net().is_none() {
            take.wire = None;
            return;
        }
        if let Some(wire) = take.wire.as_mut() {
            wire.spoken = Some(done);
        }
        if !busy {
            return;
        }
        if !take.heard {
            return self.nothing(false, out);
        }
        self.upload(supply, out);
    }

    /// Sends the held speech to the take's service once its run handed it over: never from a password box, nor from a
    /// take that went to one (edge 19), nor to a service now off, without its key, or waiting on new terms.
    pub(super) fn upload(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        let password = self.password();
        let Some(take) = self.take.as_mut() else {
            return;
        };
        let (Some(wire), Some(net)) = (take.wire.as_mut(), take.engine.net()) else {
            return;
        };
        self.state = State::Transcribing;
        if wire.spoken.is_none() {
            return;
        }
        if password || take.masked {
            take.masked = true;
            return self.write_local(CloudWhy::NoLocal, supply, out);
        }
        if let Some(why) = net::blocked(net, supply.cloud(take.lang.iso()).as_ref()) {
            return self.cloud_note(why, out);
        }
        wire.round = wire.round.wrapping_add(1);
        wire.sending = true;
        out.push(Out::Send(Upload {
            take: take.id,
            round: wire.round,
            service: net.service.clone(),
            lang: take.lang.iso().to_owned(),
        }));
        take.pieces.clear();
    }

    /// Writes the held recording with the language's local model: "Use the local model", a password box, or Try
    /// again with no cloud pick left; with none ready, the bar shows `none`. The run no longer listens alone, so the
    /// take stays local for good; its side stays only until a hand-over still due comes, so that end is taken in.
    pub(super) fn write_local(&mut self, none: CloudWhy, supply: &dyn Supply, out: &mut Vec<Out>) {
        let Some(take) = self.take.as_mut() else {
            return;
        };
        let Ok(ready) = supply.supply(take.lang.iso()) else {
            return self.cloud_note(none, out);
        };
        let ask = local_ask(&self.settings, &ready);
        take.engine = Engine::Local(ready);
        take.wire = take.wire.take().filter(|w| w.spoken.is_none());
        take.ask = ask.clone();
        take.redo();
        out.push(take.call(RunCall::Again(ask)));
        self.state = State::Transcribing;
    }

    /// "Use the local model", only while the cloud note lights it.
    pub(super) fn use_local(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        let why = match &self.state {
            State::Note {
                hint:
                    BarHint::Cloud {
                        why,
                        local: LocalUse::Ready,
                    },
                ..
            } => why.clone(),
            _ => return,
        };
        if self.anyway_ok() {
            self.write_local(why, supply, out);
        }
    }

    /// Try again on a cloud dictation: the held speech goes to the service the language picks now, or to its local
    /// model when it picks none, with the Off note when none is ready. A Text bar waits in the take, to come back
    /// should nothing be written.
    pub(super) fn retry(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        let now = self.now();
        let text = self.phase() == Phase::Text;
        // A take the local model wrote never goes back to a service.
        let Some(take) = self.take.as_mut().filter(|t| t.engine.net().is_some()) else {
            return;
        };
        if let Some(wire) = take.wire.as_mut() {
            wire.since = now;
        }
        take.redo();
        if text {
            take.before = Some(std::mem::replace(&mut self.state, State::Transcribing));
        }
        match supply.cloud(take.lang.iso()) {
            Some(net) => {
                take.engine = Engine::Cloud(net);
                self.upload(supply, out);
            }
            None => self.write_local(CloudWhy::Off, supply, out),
        }
    }

    /// What dictation `id`'s upload of `round` tells; an older or dropped upload's notes are not heard.
    pub(super) fn sent(&mut self, id: u32, round: u32, sent: Sent, out: &mut Vec<Out>) {
        let live = |w: &net::Wire| w.sending && w.round == round;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        let Some(wire) = take.wire.as_mut().filter(|w| take.id == id && live(w)) else {
            return;
        };
        // A piece keeps the upload running; anything else ends it.
        wire.sending = matches!(sent, Sent::Note(CloudNote::Piece { .. }));
        match sent {
            Sent::Note(CloudNote::Piece { index, text }) => {
                take.pieces.insert(index, text);
            }
            Sent::Note(CloudNote::Done) => self.replied(out),
            Sent::Note(CloudNote::Fault(fault)) => {
                let why = net::why(fault, &take.lang.shown);
                self.cloud_note(why, out);
            }
            Sent::Silent => self.nothing(false, out),
            Sent::Speech(error) if lost(error) => self.let_go(out),
            Sent::Speech(_) => self.cloud_note(CloudWhy::Fault, out),
        }
    }

    /// Every piece came back: counted under the pick, then typed once; with no text, Try again's bar comes back, else
    /// the bar says the service sent none.
    fn replied(&mut self, out: &mut Vec<Out>) {
        let now = self.now();
        let hidden = self.hidden;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        let spoken = take
            .wire
            .as_ref()
            .and_then(|w| w.spoken.map(|s| (s, w.since)));
        if let (false, Some((spoken, since))) = (hidden, spoken) {
            let wait = now.saturating_since(since).as_millis();
            let done = SpeechDone {
                pieces: u16::try_from(take.pieces.len()).unwrap_or(u16::MAX),
                wait_ms: u32::try_from(wait).unwrap_or(u32::MAX),
                ..spoken
            };
            let model = take.engine.name();
            out.push(Out::Count(Count {
                model,
                card: None,
                done,
            }));
        }
        let before = take.before.take();
        let text = take.joined();
        if !text.is_empty() {
            return self.deliver(text, out);
        }
        match before {
            Some(before) => {
                self.state = before;
                self.shown_at = now;
            }
            None => self.cloud_note(CloudWhy::Empty, out),
        }
    }

    /// The run let the recording go before it reached the service: the bar says so, with nothing to retry.
    fn let_go(&mut self, out: &mut Vec<Out>) {
        let Some(take) = self.take.as_ref() else {
            return;
        };
        let iso = take.lang.iso().to_owned();
        self.end_run(false, out);
        self.take = None;
        self.show_note(BarHint::Fault, None, iso);
    }

    /// The service switched off, its pick moved or its key removed while a cloud dictation runs: nothing is sent after
    /// it, and the bar says why (edge 9).
    pub(super) fn recheck(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        let Some(take) = self.take.as_ref().filter(|_| self.busy()) else {
            return;
        };
        let Some(net) = take.engine.net() else {
            return;
        };
        let Some(why) = net::blocked(net, supply.cloud(take.lang.iso()).as_ref()) else {
            return;
        };
        match self.phase() {
            Phase::Preparing => {
                let iso = take.lang.iso().to_owned();
                self.end_run(true, out);
                self.take = None;
                self.show_cloud(why, iso);
            }
            Phase::Listening => {
                out.push(take.call(RunCall::Stop));
                self.cloud_note(why, out);
            }
            _ => self.cloud_note(why, out),
        }
    }

    /// A cloud note: any upload is dropped and the pieces it told are never typed (edge 6); the recording stays.
    pub(super) fn cloud_note(&mut self, why: CloudWhy, out: &mut Vec<Out>) {
        self.lit = false;
        let front = self.front.iso().to_owned();
        let iso = (self.take.as_ref()).map_or(front, |t| t.lang.iso().to_owned());
        if let Some(take) = self.take.as_mut() {
            take.halt(out);
            take.pieces.clear();
            take.before = None;
        }
        self.show_cloud(why, iso);
    }

    /// Shows the cloud note for `why` about language `iso`, with Try again unless no local model can serve a password
    /// box; `refresh` sets "Use the local model".
    fn show_cloud(&mut self, why: CloudWhy, iso: String) {
        let act = (why != CloudWhy::NoLocal).then_some(BarButton::Again);
        let local = LocalUse::Dimmed;
        self.show_note(BarHint::Cloud { why, local }, act, iso);
    }

    /// "Use the local model" on a cloud note follows the language's local model and the recording held (edge 20).
    pub(super) fn refresh(&mut self, supply: &dyn Supply) {
        let voiced = self.anyway_ok();
        if let State::Note {
            hint: BarHint::Cloud { local, .. },
            iso,
            ..
        } = &mut self.state
        {
            *local = net::local_use(&supply.supply(iso), voiced);
        }
    }
}

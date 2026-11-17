//! The user's acts: the key, the bar's buttons, the gear and the pointer; and starting, stopping and closing a
//! dictation. Cancel, Transcribe anyway and Try again are in `redo`.

use kx_module_api::ports::{
    BarButton, BarHint, LocalUse, PanelButton, SpeechAsk, VoiceAct, VoiceClick, VoiceText,
};

use super::net::Engine;
use super::take::{Lang, Take};
use super::{Flow, Lack, Out, Phase, Ready, RunCall, State, Supply};

impl Flow {
    /// The Dictation key: Start, Stop while listening, or, as Cancel, an end with nothing sent and no bar while
    /// preparing; a second click within `debounce_ms` counts as none.
    pub(super) fn key(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        let now = self.now();
        let quick =
            (self.key_at).is_some_and(|at| now.saturating_since(at) < self.settings.debounce());
        if quick || !self.settings.on {
            return;
        }
        self.key_at = Some(now);
        if self.lent_click(BarButton::Stop, out) {
            return;
        }
        match self.phase() {
            Phase::Preparing => self.close(out),
            Phase::Listening => self.stop(true, out),
            Phase::Transcribing => {}
            Phase::Idle | Phase::Text | Phase::Note => self.start(supply, out),
        }
    }

    /// A click on the bar or the panel; a click on a bar that changed face since is dropped.
    pub(super) fn act(&mut self, act: &VoiceAct, supply: &dyn Supply, out: &mut Vec<Out>) {
        let fresh = self.fresh(act.round);
        match &act.click {
            VoiceClick::Over(over) => self.hover(*over),
            VoiceClick::Bar(button) if fresh => self.button(*button, supply, out),
            VoiceClick::Button(PanelButton::Again) if fresh => self.again(supply, out),
            VoiceClick::Local if fresh => self.use_local(supply, out),
            _ => {}
        }
    }

    /// A bar button, where the bar shows it.
    fn button(&mut self, button: BarButton, supply: &dyn Supply, out: &mut Vec<Out>) {
        use BarButton as B;
        if self.lent_click(button, out) {
            return;
        }
        match (button, self.phase()) {
            (B::Cancel, Phase::Preparing) | (B::Close, Phase::Text | Phase::Note) => {
                self.close(out);
            }
            (B::Cancel, Phase::Listening | Phase::Transcribing) => self.cancel(out),
            (B::Stop, Phase::Listening) => self.stop(true, out),
            (B::Copy, Phase::Text) => self.copy(out),
            (B::Undo, Phase::Text) => self.undo(out),
            (B::Anyway, Phase::Note) => self.anyway(supply, out),
            (B::Again, Phase::Note) if self.offers(B::Again) && self.anyway_ok() => {
                self.retry(supply, out);
            }
            (B::Listen, Phase::Note) if self.offers(B::Listen) => self.start(supply, out),
            (B::Download, Phase::Note) => self.download(out),
            _ => {}
        }
    }

    /// Starts a dictation in the language shown, fixed from now on, after closing any bar; never while Compare holds
    /// the microphone's run.
    pub(super) fn start(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        if self.lent.is_some() {
            return;
        }
        self.close(out);
        let lang = self.front.clone();
        if lang.iso().is_empty() {
            return self.lack(Lack::Unknown, &lang);
        }
        if let Some(net) = supply.cloud(lang.iso()) {
            return self.start_cloud(lang, net, supply, out);
        }
        match supply.supply(lang.iso()) {
            Ok(ready) => self.open(lang, ready, out),
            Err(lack) => self.lack(lack, &lang),
        }
    }

    /// Opens a run for `ready`, which prepares until its microphone opens.
    pub(super) fn open(&mut self, lang: Lang, ready: Ready, out: &mut Vec<Out>) {
        let ask = (self.settings).ask(Some(ready.model.clone()), ready.card.clone());
        self.open_with(lang, Engine::Local(ready), ask, out);
    }

    /// Opens a run as `ask` says, for `engine`.
    pub(super) fn open_with(
        &mut self,
        lang: Lang,
        engine: Engine,
        ask: SpeechAsk,
        out: &mut Vec<Out>,
    ) {
        self.takes = self.takes.wrapping_add(1);
        out.push(Out::Open {
            take: self.takes,
            ask: ask.clone(),
        });
        self.take = Some(Take::new(self.takes, lang, engine, ask));
        self.state = State::Preparing;
    }

    /// The note for a language no model serves now.
    pub(super) fn lack(&mut self, lack: Lack, lang: &Lang) {
        let shown = lang.shown.clone();
        let (hint, act) = match lack {
            Lack::Missing { bytes } => (
                BarHint::NoModel { lang: shown, bytes },
                Some(BarButton::Download),
            ),
            Lack::Stranger => (BarHint::Language(shown), None),
            Lack::Checking => (BarHint::Checking, None),
            Lack::Unknown => (BarHint::NoLanguage(shown), None),
            Lack::NoFit => (BarHint::NoFit, None),
        };
        self.show_note(hint, act, lang.iso().to_owned());
    }

    /// Shows a note about language `iso`; `done_ms` starts.
    pub(super) fn show_note(&mut self, hint: BarHint, act: Option<BarButton>, iso: String) {
        self.state = State::Note { hint, act, iso };
        self.shown_at = self.now();
    }

    /// Listening ends, by Stop when `ask`, else by the job itself. With no speech heard nothing is written, so no
    /// model makes words up over silence.
    pub(super) fn stop(&mut self, ask: bool, out: &mut Vec<Out>) {
        self.lit = false;
        let now = self.now();
        let Some(take) = self.take.as_mut() else {
            return;
        };
        if !take.heard {
            return self.nothing(true, out);
        }
        if let Some(wire) = take.wire.as_mut() {
            wire.since = now;
        }
        if ask {
            out.push(take.call(RunCall::Stop));
        }
        self.state = State::Transcribing;
    }

    /// Nothing was heard: the run ends, cancelled first when `cancel`, and Listen again is offered.
    pub(super) fn nothing(&mut self, cancel: bool, out: &mut Vec<Out>) {
        let iso = (self.take.as_ref())
            .map_or_else(|| self.front.iso().to_owned(), |t| t.lang.iso().to_owned());
        self.end_run(cancel, out);
        self.take = None;
        self.show_note(BarHint::Nothing, Some(BarButton::Listen), iso);
    }

    /// The gear or the tray opens the panel: a dictation, or Compare's recording, ends first as the bar's Cancel ends
    /// it, with no bar while it prepares, as nothing was recorded (mock-up `end()`).
    pub(super) fn opening(&mut self, out: &mut Vec<Out>) {
        if self.lent_click(BarButton::Cancel, out) {
            return;
        }
        match self.phase() {
            Phase::Preparing => self.close(out),
            Phase::Listening | Phase::Transcribing => self.cancel(out),
            Phase::Idle | Phase::Text | Phase::Note => {}
        }
    }

    /// Copy: the text shown, never a password box's.
    fn copy(&self, out: &mut Vec<Out>) {
        let shown = matches!(
            &self.state,
            State::Text {
                text: VoiceText::Shown { .. },
                ..
            }
        );
        let text = self.take.as_ref().and_then(|t| t.text.clone());
        if let (true, Some(text)) = (shown, text) {
            out.push(Out::Copy(text));
        }
    }

    /// Download: the model of the note's language, also from a cloud note whose local model is missing (edge 20).
    fn download(&self, out: &mut Vec<Out>) {
        let fetch = matches!(
            &self.state,
            State::Note {
                hint: BarHint::Cloud {
                    local: LocalUse::Download { .. },
                    ..
                },
                ..
            }
        );
        let offered = fetch || self.offers(BarButton::Download);
        if let (true, State::Note { iso, .. }) = (offered, &self.state) {
            out.push(Out::Download(iso.clone()));
        }
    }

    /// The note shown names `button` as its action.
    fn offers(&self, button: BarButton) -> bool {
        matches!(&self.state, State::Note { act: Some(a), .. } if *a == button)
    }

    /// The pointer over the bar keeps it; leaving starts `done_ms` again.
    fn hover(&mut self, over: bool) {
        if self.over && !over {
            self.shown_at = self.now();
        }
        self.over = over;
    }

    /// Closes the bar: the run is cancelled if busy, then closed, which wipes the recording.
    pub(super) fn close(&mut self, out: &mut Vec<Out>) {
        self.end_run(true, out);
        self.take = None;
        self.state = State::Idle;
    }

    /// Ends the take's run: cancelled first when `cancel` and busy, then closed; the light goes off.
    pub(super) fn end_run(&mut self, cancel: bool, out: &mut Vec<Out>) {
        self.lit = false;
        let busy = self.busy();
        let Some(take) = self.take.as_mut() else {
            return;
        };
        take.held = false;
        take.halt(out);
        if !take.open {
            return;
        }
        take.open = false;
        if cancel && busy {
            out.push(take.call(RunCall::Cancel));
        }
        out.push(take.call(RunCall::Close));
    }
}

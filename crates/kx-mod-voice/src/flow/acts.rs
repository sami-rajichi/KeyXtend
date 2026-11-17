//! The user's acts: the key, the bar's buttons, the gear, the pointer and Try again; and starting, stopping,
//! cancelling and closing a dictation.

use kx_module_api::ports::{BarButton, BarHint, PanelButton, VoiceAct, VoiceClick, VoiceText};

use super::take::{Lang, Take};
use super::{Flow, Lack, Out, Phase, Ready, RunCall, State, Supply};

impl Flow {
    /// The Dictation key: Start, Stop while listening, or a cancel with nothing sent while preparing; a second click
    /// within `debounce_ms` counts as none.
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
            (B::Cancel, Phase::Preparing | Phase::Listening | Phase::Transcribing) => {
                self.cancel(out);
            }
            (B::Stop, Phase::Listening) => self.stop(true, out),
            (B::Copy, Phase::Text) => self.copy(out),
            (B::Undo, Phase::Text) => self.undo(out),
            (B::Anyway, Phase::Note) => self.anyway(out),
            (B::Listen, Phase::Note) if self.offers(B::Listen) => self.start(supply, out),
            (B::Download, Phase::Note) => self.download(out),
            (B::Close, Phase::Text | Phase::Note) => self.close(out),
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
        let supplied = if lang.iso().is_empty() {
            Err(Lack::Unknown)
        } else {
            supply.supply(lang.iso())
        };
        match supplied {
            Ok(ready) => self.open(lang, ready, out),
            Err(lack) => self.lack(lack, &lang),
        }
    }

    /// Opens a run for `ready`, which prepares until its microphone opens.
    fn open(&mut self, lang: Lang, ready: Ready, out: &mut Vec<Out>) {
        self.takes = self.takes.wrapping_add(1);
        let ask = (self.settings).ask(Some(ready.model.clone()), ready.card.clone());
        out.push(Out::Open {
            take: self.takes,
            ask: ask.clone(),
        });
        self.take = Some(Take::new(self.takes, lang, ready, ask));
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
        let Some(take) = &self.take else {
            return;
        };
        if !take.heard {
            return self.nothing(true, out);
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

    /// The gear or the tray opens the panel: a dictation that prepares, listens or writes, or Compare's recording, ends
    /// first as the bar's Cancel ends it.
    pub(super) fn opening(&mut self, out: &mut Vec<Out>) {
        if !self.lent_click(BarButton::Cancel, out) && self.busy() {
            self.cancel(out);
        }
    }

    /// Cancel: nothing is typed, and the recording stays for Transcribe anyway.
    fn cancel(&mut self, out: &mut Vec<Out>) {
        self.lit = false;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        if take.open {
            out.push(take.call(RunCall::Cancel));
        }
        take.pieces.clear();
        let iso = take.lang.iso().to_owned();
        self.show_note(BarHint::Cancelled, Some(BarButton::Anyway), iso);
    }

    /// Transcribe anyway: the cancelled recording is written with the ask it was last written with.
    fn anyway(&mut self, out: &mut Vec<Out>) {
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
        let Some(take) = self.take.as_mut() else {
            return;
        };
        take.redo();
        out.push(take.call(RunCall::Again(take.ask.clone())));
        self.state = State::Transcribing;
    }

    /// Try again: the held recording is written again with the model and device the language gets now. The Text bar
    /// waits in the take, to come back should the new run write nothing.
    fn again(&mut self, supply: &dyn Supply, out: &mut Vec<Out>) {
        if !self.can_again() {
            return;
        }
        let Some(take) = self.take.as_mut() else {
            return;
        };
        match supply.supply(take.lang.iso()) {
            Ok(ready) => {
                let ask = (self.settings).ask(Some(ready.model.clone()), ready.card.clone());
                take.ready = ready;
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

    /// Download: the model of the note's language.
    fn download(&self, out: &mut Vec<Out>) {
        if let (true, State::Note { iso, .. }) = (self.offers(BarButton::Download), &self.state) {
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

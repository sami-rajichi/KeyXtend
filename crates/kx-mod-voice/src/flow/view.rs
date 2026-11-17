//! The bar and the light as the flow tells them: once per change, each view with a new round, and a click taken only
//! while the bar it saw keeps its face.

use std::mem::{Discriminant, discriminant};

use kx_module_api::ports::{
    BarButton, BarHint, BarNote, BarText, CloudWhy, Listening, ServiceName, VoiceBar, VoiceLit,
    VoicePage, VoiceView,
};

use super::{Flow, Out, State};

/// A bar's kind, its take and a note's line; a click made on another face is dropped.
type Face = (Discriminant<VoiceBar>, Option<u32>, Option<BarHint>);

/// What was told last.
#[derive(Debug, Default)]
pub(super) struct Told {
    page: Option<VoicePage>,
    lit: Option<VoiceLit>,
    /// The last view's round.
    round: u32,
    /// The round since which the bar has kept its face.
    since: u32,
    /// The bar's face, as last told.
    face: Option<Face>,
}

impl Told {
    /// Tells `page` as a new round; a bar of another face starts a new span for clicks.
    fn show(&mut self, mut page: VoicePage, take: Option<u32>, out: &mut Vec<Out>) {
        self.round = self.round.wrapping_add(1);
        page.round = self.round;
        let line = match &page.bar {
            VoiceBar::Note(note) => Some(note.hint.clone()),
            _ => None,
        };
        let face = Some((discriminant(&page.bar), take, line));
        if self.face != face {
            self.face = face;
            self.since = self.round;
        }
        out.push(Out::View(VoiceView::new(page.clone())));
        self.page = Some(page);
    }

    /// True when `round` lies between the bar's last change of face and now, wrapping.
    fn fresh(&self, round: u32) -> bool {
        self.round.wrapping_sub(round) <= self.round.wrapping_sub(self.since)
    }
}

impl Flow {
    /// Tells the view and the light when they changed.
    pub(super) fn tell(&mut self, out: &mut Vec<Out>) {
        let page = VoicePage {
            round: self.told.round,
            bar: self.bar(),
            panel: self.panel.clone(),
            place: self.settings.bar_place(),
            sounds: self.settings.sounds,
            cloud: self.mark(),
        };
        if !self.told.page.as_ref().is_some_and(|p| same(p, &page)) {
            let take = self.take.as_ref().map(|t| t.id);
            self.told.show(page, take, out);
        }
        let light = self.light();
        if self.told.lit != Some(light) {
            self.told.lit = Some(light);
            out.push(Out::Lit(light));
        }
    }

    /// The key's light: red while the microphone is open, amber while a dictation or Compare waits for it, else off.
    fn light(&self) -> VoiceLit {
        if self.lit {
            VoiceLit::Rec
        } else if matches!(self.state, State::Preparing) || self.lent_prep() {
            VoiceLit::Prep
        } else {
            VoiceLit::Off
        }
    }

    /// Tells the last view and the light again, for a late starter.
    pub(super) fn retell(&mut self, out: &mut Vec<Out>) {
        if let Some(page) = &self.told.page {
            out.push(Out::View(VoiceView::new(page.clone())));
        }
        self.told.lit = None;
    }

    /// A click on the view of `round` may act.
    pub(super) fn fresh(&self, round: u32) -> bool {
        self.told.fresh(round)
    }

    /// The bar for the state, or Compare's Listening while it records.
    fn bar(&self) -> VoiceBar {
        if let Some(lent) = self.lent_bar() {
            return lent;
        }
        match &self.state {
            State::Idle | State::Preparing => VoiceBar::Hidden,
            State::Listening { since, nothing } => self.listening(*since, *nothing),
            State::Transcribing => VoiceBar::Transcribing,
            State::Text {
                text,
                hint,
                unclipped,
            } => VoiceBar::Text(BarText {
                text: text.clone(),
                hint: unclipped
                    .then_some(BarHint::CopyFailed)
                    .or_else(|| hint.clone()),
                undo: self.undo_ok(),
            }),
            State::Note { hint, act, .. } => VoiceBar::Note(BarNote {
                hint: hint.clone(),
                act: act.filter(|a| {
                    !matches!(a, BarButton::Anyway | BarButton::Again) || self.anyway_ok()
                }),
                mic: false,
            }),
        }
    }

    /// The cloud mark: the service a cloud dictation goes to, from Start to its text or its note.
    fn mark(&self) -> Option<ServiceName> {
        let net = self.take.as_ref()?.engine.net()?;
        let shows = match &self.state {
            State::Preparing | State::Listening { .. } | State::Transcribing => true,
            State::Note {
                hint: BarHint::Cloud { why, .. },
                ..
            } => *why != CloudWhy::NoLocal,
            State::Idle | State::Text { .. } | State::Note { .. } => false,
        };
        shows.then(|| net.name.clone())
    }

    /// The Listening bar, with the time listened so far.
    fn listening(&self, since: kx_module_api::Mono, nothing: bool) -> VoiceBar {
        let Some(take) = &self.take else {
            return VoiceBar::Hidden;
        };
        let elapsed = self.now().saturating_since(since).as_millis();
        VoiceBar::Listening(Listening {
            lang: take.lang.shown.clone(),
            elapsed_ms: u32::try_from(elapsed).unwrap_or(u32::MAX),
            nothing,
            wave_ms: self.settings.wave_ms,
        })
    }

    /// Undo insert can act: the keyboard typed the text and nothing else was typed since.
    pub(super) fn undo_ok(&self) -> bool {
        (self.take.as_ref()).is_some_and(|t| t.typed.is_some() && t.asking.is_none())
    }

    /// Transcribe anyway can act: speech was heard and the recording is still held.
    pub(super) fn anyway_ok(&self) -> bool {
        self.held() && self.take.as_ref().is_some_and(|t| t.heard)
    }
}

/// True when two views draw the same, ignoring their rounds and the time listened, which the ui counts on itself.
fn same(a: &VoicePage, b: &VoicePage) -> bool {
    let bars = match (&a.bar, &b.bar) {
        (VoiceBar::Listening(x), VoiceBar::Listening(y)) => {
            x.lang == y.lang && x.nothing == y.nothing && x.wave_ms == y.wave_ms
        }
        (x, y) => x == y,
    };
    bars && a.panel == b.panel && a.place == b.place && a.sounds == b.sounds && a.cloud == b.cloud
}

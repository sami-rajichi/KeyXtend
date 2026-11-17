//! Compare models borrowing the microphone: while its run opens the microphone the key glows amber with no bar, then
//! while it records the set text the bar shows Listening and the key glows red. The key, Stop and Cancel go back to
//! Compare, and no dictation starts until Compare lets go.

use kx_module_api::Mono;
use kx_module_api::ports::{BarButton, Listening, VoiceBar, VoiceLang};

use super::{Flow, Out};

/// What Compare models holds of the microphone's run, as `Compare::lend` tells it.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Lend {
    /// Nothing: dictation is free.
    #[default]
    Off,
    /// Its run opens the microphone: no bar yet, and the key glows amber.
    Prep,
    /// Its run records the set text: the bar shows Listening and the key glows red.
    Listen,
    /// Its models write the recording: no bar, and the key does nothing.
    Write,
}

/// The flow's side of a lend.
#[derive(Clone, Debug)]
pub(super) struct Lent {
    /// Where Compare's run stands; a click that ends its recording makes it `Write` until Compare tells the next.
    lend: Lend,
    /// When the recording began: when its microphone opened.
    since: Mono,
    /// The language in front when Compare took the run.
    lang: VoiceLang,
}

impl Flow {
    /// Compare takes the microphone's run, records, writes, or lets go; taking it ends any dictation and its bar first.
    pub(super) fn lend(&mut self, lend: Lend, out: &mut Vec<Out>) {
        if lend == Lend::Off {
            self.lent = None;
            self.lit = false;
            return;
        }
        if self.lent.is_none() {
            self.close(out);
        }
        let now = self.now();
        let (since, lang) = match &self.lent {
            Some(l) if l.lend == Lend::Listen => (l.since, l.lang.clone()),
            Some(l) => (now, l.lang.clone()),
            None => (now, self.front.shown.clone()),
        };
        self.lent = Some(Lent { lend, since, lang });
        self.lit = lend == Lend::Listen;
    }

    /// The key (as Stop), Stop, Cancel or the gear (as Cancel) while Compare holds the run; true when the lend took the
    /// click. While its microphone opens nothing was recorded, so Stop cancels; the light goes off at the click.
    pub(super) fn lent_click(&mut self, button: BarButton, out: &mut Vec<Out>) -> bool {
        let Some(lent) = self.lent.as_mut() else {
            return false;
        };
        let ends = matches!(button, BarButton::Stop | BarButton::Cancel);
        match lent.lend {
            Lend::Prep if ends => out.push(Out::Lent(BarButton::Cancel)),
            Lend::Listen if ends => out.push(Out::Lent(button)),
            _ => return true,
        }
        lent.lend = Lend::Write;
        self.lit = false;
        true
    }

    /// A lock, a sleep, Quit or the feature turned off cancels Compare's run.
    pub(super) fn end_lend(&mut self, out: &mut Vec<Out>) {
        if self.lent.take().is_some() {
            out.push(Out::Lent(BarButton::Cancel));
            self.lit = false;
        }
    }

    /// Compare's run opens the microphone, so the key glows amber.
    pub(super) fn lent_prep(&self) -> bool {
        self.lent.as_ref().is_some_and(|l| l.lend == Lend::Prep)
    }

    /// The Listening bar while Compare records.
    pub(super) fn lent_bar(&self) -> Option<VoiceBar> {
        let lent = self.lent.as_ref().filter(|l| l.lend == Lend::Listen)?;
        let elapsed = self.now().saturating_since(lent.since).as_millis();
        Some(VoiceBar::Listening(Listening {
            lang: lent.lang.clone(),
            elapsed_ms: u32::try_from(elapsed).unwrap_or(u32::MAX),
            nothing: false,
            wave_ms: self.settings.wave_ms,
        }))
    }
}

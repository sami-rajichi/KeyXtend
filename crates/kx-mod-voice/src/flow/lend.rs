//! Compare models borrowing the microphone: while its run records the set text, the bar shows Listening and the key
//! glows, and the key, Stop and Cancel go back to Compare; no dictation starts until Compare lets go.

use kx_module_api::Mono;
use kx_module_api::ports::{BarButton, Listening, VoiceBar, VoiceLang};

use super::{Flow, Out};

/// What Compare models holds of the microphone's run, as `Compare::lend` tells it.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Lend {
    /// Nothing: dictation is free.
    #[default]
    Off,
    /// Its run records the set text: the bar shows Listening and the key glows.
    Listen,
    /// Its models write the recording: no bar, and the key does nothing.
    Write,
}

/// The flow's side of a lend.
#[derive(Clone, Debug)]
pub(super) struct Lent {
    /// Compare still records.
    listen: bool,
    /// When the recording began.
    since: Mono,
    /// The language in front at that time.
    lang: VoiceLang,
}

impl Flow {
    /// Compare takes the microphone's run, writes, or lets go; taking it ends any dictation and its bar first.
    pub(super) fn lend(&mut self, lend: Lend, out: &mut Vec<Out>) {
        if lend == Lend::Off {
            self.lent = None;
            self.lit = false;
            return;
        }
        if self.lent.is_none() {
            self.close(out);
        }
        let since = self.lent.as_ref().map_or_else(|| self.now(), |l| l.since);
        let lang =
            (self.lent.as_ref()).map_or_else(|| self.front.shown.clone(), |l| l.lang.clone());
        let listen = lend == Lend::Listen;
        self.lent = Some(Lent {
            listen,
            since,
            lang,
        });
        self.lit = listen;
    }

    /// The key (as Stop), Stop or Cancel while Compare holds the run; true when the lend took the click. The light goes
    /// off at the click, as for a dictation's Stop.
    pub(super) fn lent_click(&mut self, button: BarButton, out: &mut Vec<Out>) -> bool {
        let Some(lent) = self.lent.as_mut() else {
            return false;
        };
        if lent.listen && matches!(button, BarButton::Stop | BarButton::Cancel) {
            out.push(Out::Lent(button));
            lent.listen = false;
            self.lit = false;
        }
        true
    }

    /// A lock, a sleep, Quit or the feature turned off cancels Compare's run.
    pub(super) fn end_lend(&mut self, out: &mut Vec<Out>) {
        if self.lent.take().is_some() {
            out.push(Out::Lent(BarButton::Cancel));
            self.lit = false;
        }
    }

    /// The Listening bar while Compare records.
    pub(super) fn lent_bar(&self) -> Option<VoiceBar> {
        let lent = self.lent.as_ref().filter(|l| l.listen)?;
        let elapsed = self.now().saturating_since(lent.since).as_millis();
        Some(VoiceBar::Listening(Listening {
            lang: lent.lang.clone(),
            elapsed_ms: u32::try_from(elapsed).unwrap_or(u32::MAX),
            nothing: false,
            wave_ms: self.settings.wave_ms,
        }))
    }
}

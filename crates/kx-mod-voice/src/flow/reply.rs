//! What a cloud dictation's upload tells (ADR-0047): its pieces, kept until every one came and typed once; a fault's
//! note, which keeps the recording; and a run that let the recording go before it reached the service.

use kx_module_api::ports::{BarHint, CloudNote, CloudWhy, Renewal, SpeechDone};

use super::net::{self, Sent};
use super::notes::lost;
use super::{Count, Flow, Out};

impl Flow {
    /// What dictation `id`'s upload of `round` tells, a used allowance with `renews`; an older or dropped upload's
    /// notes go unheard.
    pub(super) fn sent(
        &mut self,
        id: u32,
        round: u32,
        sent: Sent,
        renews: Option<Renewal>,
        out: &mut Vec<Out>,
    ) {
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
                let why = net::why(fault, &take.lang.shown, renews);
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
}

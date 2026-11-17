//! Texts for the keyboard: each goes to one typer thread of its own, as a `TextAsk` holds the publishing thread for the
//! whole text, so one text is in flight at a time. Each gets the typer's own round, which maps the keyboard's answer
//! back to the flow, or to Paste at once after its erase; History's texts get no answer there.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use kx_module_api::Bus;
use kx_module_api::ports::{TextAsk, TextDone};

use super::Actor;
use crate::flow::In;

/// The typer thread's name, as a debugger shows it.
const TYPER: &str = "kx-voice-type";

/// Hands a text to the typer; false once its thread is gone.
pub(crate) type Typist = Box<dyn Fn(TextAsk) -> bool + Send>;

/// Who asked for a text.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Source {
    /// The flow, under its own round.
    Flow(u32),
    /// History's Insert.
    History,
    /// Paste at once's erase, before the text that replaces it is pasted.
    Erase,
}

/// The texts handed to the typer and not yet answered, oldest first, by the typer's round.
#[derive(Default)]
pub(super) struct Typing {
    last: u32,
    asked: VecDeque<(u32, Source)>,
}

impl Actor {
    /// Before a written text reaches the flow: when no box was told, the box in front is looked at again, as a focus
    /// change told out of order can leave none. The look reads no text, and what it finds reaches the flow first.
    pub(super) fn look(&mut self) {
        if !self.flow.no_box() {
            return;
        }
        let Some(run) = self.ports.field.as_mut() else {
            return;
        };
        match run.look() {
            Ok(Some(change)) => {
                tracing::info!(kind = ?change.kind, "the box in front was looked at again");
                self.step(In::Field(change));
            }
            Ok(None) => {}
            Err(error) => tracing::warn!(%error, "the box in front could not be looked at"),
        }
    }

    /// Hands `ask` to the typer under a round of its own; a typer that is gone answers nothing typed.
    pub(super) fn type_text(&mut self, source: Source, mut ask: TextAsk) {
        let round = self.typing.last.wrapping_add(1);
        self.typing.last = round;
        self.typing.asked.push_back((round, source));
        ask.round = round;
        tracing::debug!("a text went to the keyboard");
        if !(self.ports.typist)(ask) {
            self.typed(TextDone {
                round,
                count: 0,
                ok: false,
            });
        }
    }

    /// The keyboard's answer, to the flow under the flow's round; the texts asked before it get none now.
    pub(super) fn typed(&mut self, done: TextDone) {
        if !done.ok {
            tracing::warn!(
                count = done.count,
                "the keyboard did not type all of a dictated text"
            );
        }
        let asked = &mut self.typing.asked;
        let Some(at) = asked.iter().position(|(round, _)| *round == done.round) else {
            return;
        };
        let last = asked.drain(..=at).next_back();
        match last {
            Some((_, Source::Flow(round))) => self.step(In::Typed(TextDone { round, ..done })),
            Some((_, Source::Erase)) => self.erased(done.ok),
            _ => {}
        }
    }
}

/// The actor's end of the typer's queue; dropping it, as the actor stops, tells the typer to type nothing more.
pub(super) struct Hand {
    tx: Sender<TextAsk>,
    live: Arc<AtomicBool>,
}

impl Hand {
    /// Queues `ask` for the typer; false once its thread is gone.
    pub(super) fn send(&self, ask: TextAsk) -> bool {
        self.tx.send(ask).is_ok()
    }
}

impl Drop for Hand {
    fn drop(&mut self) {
        self.live.store(false, Ordering::SeqCst);
    }
}

/// A typer's queue: the actor's hand, the typer's end, and the flag the hand clears when it drops.
pub(super) fn queue() -> (Hand, Receiver<TextAsk>, Arc<AtomicBool>) {
    let (tx, texts) = mpsc::channel();
    let live = Arc::new(AtomicBool::new(true));
    let hand = Hand {
        tx,
        live: Arc::clone(&live),
    };
    (hand, texts, live)
}

/// Publishes each text of `texts` on `bus`, in order, while `live` holds; a text still queued at the stop is dropped.
pub(super) fn type_all(bus: &Bus, texts: &Receiver<TextAsk>, live: &AtomicBool) {
    for ask in texts {
        if !live.load(Ordering::SeqCst) {
            return;
        }
        // The module holds `Voice`, which this event needs, or its start would have failed.
        let _ = bus.publish(&ask);
    }
}

/// Starts the typer's thread over `bus`; `None` when it cannot start. Nothing joins it: it ends after the text in
/// flight once the actor drops the typist.
pub(crate) fn typer(bus: Bus) -> Option<Typist> {
    let (hand, texts, live) = queue();
    thread::Builder::new()
        .name(TYPER.to_owned())
        .spawn(move || type_all(&bus, &texts, &live))
        .ok()?;
    Some(Box::new(move |ask| hand.send(ask)))
}

//! One run: its pieces' sound taken in order into wiped WAV buffers sized once, each full piece sent on its own thread
//! within `at_once` and `per_min`, and the texts told in the pieces' order. A part out of order or past its piece is
//! `Gone`; a run whose every text is blank ends `Empty`, as the service heard nothing.

use super::send::{self, Sending};
use super::shape;
use super::wav;
use super::{Event, Kit, Said, Told};
use kx_ipc::{NetCall, NetFault, NetNote, NetRun};
use std::collections::{BTreeMap, VecDeque};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use zeroize::Zeroizing;

/// The span `per_min` counts requests in, in ms.
const MINUTE_MS: u64 = 60_000;
/// The name of each request's thread.
const SENDER: &str = "speech-net-send";

/// One run of the job.
pub struct Run {
    /// Tells this run's outcomes from an older run's of the same id.
    serial: u64,
    sending: Arc<Sending>,
    sizes: Vec<u32>,
    at_once: usize,
    per_min: usize,
    /// The piece being filled, as a WAV file.
    filling: Option<Zeroizing<Vec<u8>>>,
    /// The next piece to fill.
    next: usize,
    /// Full pieces not sent yet, by index.
    waiting: VecDeque<(u16, Zeroizing<Vec<u8>>)>,
    /// Requests sent and not answered.
    flying: usize,
    /// When each request started or will start, for `per_min`.
    starts: VecDeque<u64>,
    /// Texts come out of order, told in order.
    texts: BTreeMap<u16, Zeroizing<String>>,
    /// The next index to tell.
    told: u16,
    /// True once a text holds more than blanks.
    heard: bool,
}

impl Run {
    /// A run of `run`'s pieces over `call`.
    ///
    /// # Errors
    /// `Range` for a shape this job does not build yet.
    pub fn new(
        serial: u64,
        call: NetCall<'static>,
        run: NetRun<'static>,
    ) -> Result<Self, NetFault> {
        let shape = shape::of(call.shape.code()).ok_or(NetFault::Range)?;
        let sending = Sending {
            shape,
            call,
            model: run.model,
            lang: run.lang,
            tries: run.tries,
            dropped: Arc::new(AtomicBool::new(false)),
        };
        Ok(Self {
            serial,
            sending: Arc::new(sending),
            sizes: run.sizes,
            at_once: usize::from(run.limits.at_once),
            per_min: usize::from(run.limits.per_min),
            filling: None,
            next: 0,
            waiting: VecDeque::new(),
            flying: 0,
            starts: VecDeque::new(),
            texts: BTreeMap::new(),
            told: 0,
            heard: false,
        })
    }

    /// This run's serial.
    pub fn serial(&self) -> u64 {
        self.serial
    }

    /// Takes a part of piece `piece`; once full, the piece waits to be sent.
    ///
    /// # Errors
    /// `Gone` for a part of another piece than the next to fill, or past its size; `Gone` when memory runs out.
    pub fn sound(&mut self, piece: u16, bytes: &[u8]) -> Result<(), NetFault> {
        let index = usize::from(piece);
        let size = self.sizes.get(index).map(|&s| s as usize);
        let (Some(size), true) = (size, index == self.next) else {
            return Err(NetFault::Gone);
        };
        let wav = match &mut self.filling {
            Some(wav) => wav,
            empty => empty.insert(wav::start(size)?),
        };
        if wav.len() - wav::HEAD + bytes.len() > size {
            return Err(NetFault::Gone);
        }
        wav.extend_from_slice(bytes);
        if wav.len() - wav::HEAD == size {
            if let Some(full) = self.filling.take() {
                self.waiting.push_back((piece, full));
            }
            self.next += 1;
        }
        Ok(())
    }

    /// Sends the waiting pieces within `at_once` and `per_min`, each on its own thread telling `tx` as run `id`.
    ///
    /// # Errors
    /// `Gone` when a thread cannot start.
    pub fn launch(&mut self, id: u32, kit: &Kit, tx: &Sender<Event>) -> Result<(), NetFault> {
        while self.flying < self.at_once {
            let Some((index, wav)) = self.waiting.pop_front() else {
                return Ok(());
            };
            let delay = self.pace(kit.clock.now_ms());
            let (kit, tx, sending, serial) =
                (kit.clone(), tx.clone(), self.sending.clone(), self.serial);
            let work = move || {
                let said = panic::catch_unwind(AssertUnwindSafe(|| {
                    if delay > 0 {
                        kit.clock.wait(delay);
                    }
                    send::piece(&kit, &sending, &wav)
                }));
                let said = Said::Piece(index, said.unwrap_or(Err(NetFault::Gone)));
                let _ = tx.send(Event::Told(Told { id, serial, said }));
            };
            thread::Builder::new()
                .name(SENDER.to_owned())
                .spawn(work)
                .map_err(|_| NetFault::Gone)?;
            self.flying += 1;
        }
        Ok(())
    }

    /// The ms to wait before the next request, so no minute holds more than `per_min` starts.
    fn pace(&mut self, now: u64) -> u64 {
        while self
            .starts
            .front()
            .is_some_and(|&s| s.saturating_add(MINUTE_MS) <= now)
        {
            self.starts.pop_front();
        }
        let last = self.starts.back().copied().unwrap_or(now);
        let full = self.starts.len().checked_sub(self.per_min);
        let free = full
            .and_then(|at| self.starts.get(at))
            .map_or(now, |s| s + MINUTE_MS);
        let at = now.max(last).max(free);
        self.starts.push_back(at);
        at - now
    }

    /// Takes piece `index`'s outcome: the notes now due in order, and whether the run ended.
    pub fn told(
        &mut self,
        id: u32,
        index: u16,
        said: Result<Zeroizing<String>, NetFault>,
    ) -> (Vec<NetNote>, bool) {
        self.flying = self.flying.saturating_sub(1);
        let text = match said {
            Ok(text) => text,
            Err(fault) => return (vec![NetNote::Fault { id, fault }], true),
        };
        self.texts.insert(index, text);
        let mut notes = Vec::new();
        while let Some(text) = self.texts.remove(&self.told) {
            self.heard |= !text.trim().is_empty();
            let note = NetNote::Text {
                id,
                index: self.told,
                text,
            };
            if note.check().is_err() {
                notes.push(NetNote::Fault {
                    id,
                    fault: NetFault::Strange,
                });
                return (notes, true);
            }
            notes.push(note);
            self.told += 1;
        }
        if usize::from(self.told) < self.sizes.len() {
            return (notes, false);
        }
        let end = if self.heard {
            NetNote::Done { id }
        } else {
            NetNote::Fault {
                id,
                fault: NetFault::Empty,
            }
        };
        notes.push(end);
        (notes, true)
    }

    /// Stops the run: its threads send nothing more, and their outcomes are not told.
    pub fn drop_requests(&self) {
        self.sending.dropped.store(true, Ordering::SeqCst);
    }

    /// Full pieces not sent yet.
    #[cfg(test)]
    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }
}

//! The text: typed or pasted into the box in front, kept on the bar when nothing takes it, dots for a password box,
//! kept in History, and Undo insert while nothing else was typed.

use kx_module_api::ports::{BarHint, FieldChange, FieldKind, TextAsk, TextDone, VoiceText};
use kx_module_api::{Redacted, Zeroizing};

use super::take::{Asking, Typed, secret, shown, suffixed};
use super::{Flow, Kept, Out, RunCall, State};
use crate::settings::Insert;

impl Flow {
    /// The written text: typed when the box takes it, first or in place of a text Undo can still erase.
    pub(super) fn deliver(&mut self, text: Zeroizing<String>, out: &mut Vec<Out>) {
        let kind = self.kind();
        self.type_text(&text, kind, out);
        self.keep(&text, kind, out);
        let hint = self.hint(kind);
        self.show(text, kind, hint);
    }

    /// The pieces a broken run wrote, under `hint`: on the bar with Copy, never typed, and kept unless they would
    /// replace a whole earlier text in History.
    pub(super) fn salvage(&mut self, text: Zeroizing<String>, hint: BarHint, out: &mut Vec<Out>) {
        let kind = self.kind();
        if self.take.as_ref().is_some_and(|t| t.text.is_none()) {
            self.keep(&text, kind, out);
        }
        if let Some(take) = self.take.as_mut() {
            take.typed = None;
        }
        self.show(text, kind, Some(hint));
    }

    /// The box the text goes to: a password box once the take went to one, else the box in front.
    fn kind(&self) -> FieldKind {
        let masked = self.take.as_ref().is_some_and(|t| t.masked);
        if masked {
            FieldKind::Password
        } else {
            self.field.kind
        }
    }

    /// The Text bar for `text`, as dots in a password box; `done_ms` starts.
    fn show(&mut self, text: Zeroizing<String>, kind: FieldKind, hint: Option<BarHint>) {
        let masked = kind == FieldKind::Password;
        let shown = if masked {
            VoiceText::Masked
        } else {
            shown(&text)
        };
        if let Some(take) = self.take.as_mut() {
            take.masked |= masked;
            take.text = Some(Redacted::new(text));
        }
        self.state = State::Text {
            text: shown,
            hint,
            unclipped: false,
        };
        self.shown_at = self.now();
    }

    /// Asks for `text` to go in, erasing the text before while Undo can: pasted when `insert` says so and the box is
    /// no password box, else typed. Nothing when no box takes it, or when a later text can no longer replace the first.
    fn type_text(&mut self, text: &str, kind: FieldKind, out: &mut Vec<Out>) {
        let round = self.asks.wrapping_add(1);
        let box_id = self.field.box_id;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        let erase = take.typed.map(|t| t.count);
        if kind == FieldKind::None || (take.text.is_some() && erase.is_none()) {
            return;
        }
        self.asks = round;
        take.typed = None;
        take.asking = Some(Asking {
            round,
            box_id,
            undo: false,
        });
        let ask = TextAsk {
            round,
            erase: erase.unwrap_or_default(),
            text: suffixed(text, &self.settings.type_suffix),
            gap_ms: self.settings.type_gap_ms,
        };
        let paste = self.settings.way() == Insert::Paste && kind != FieldKind::Password;
        out.push(if paste {
            Out::Paste(ask)
        } else {
            Out::Type(ask)
        });
    }

    /// Keeps `text` in History and, with Keep recordings, its sound; never a password box's, never in a hidden run.
    /// With `history` off the sound is never asked for, so it stays in the worker.
    fn keep(&mut self, text: &str, kind: FieldKind, out: &mut Vec<Out>) {
        if self.hidden || kind == FieldKind::Password {
            return;
        }
        let voice = self.settings.keep_voice && self.settings.history;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        out.push(Out::Keep(Kept {
            take: take.id,
            text: secret(text),
            lang: take.lang.iso().to_owned(),
            model: take.ready.name.clone(),
        }));
        if voice && !take.kept && take.open && take.held {
            take.kept = true;
            out.push(take.call(RunCall::Keep));
        }
    }

    /// The line under the text: a password box, no box, the length limit, then a failed card.
    fn hint(&self, kind: FieldKind) -> Option<BarHint> {
        let take = self.take.as_ref();
        match kind {
            FieldKind::Password => Some(BarHint::Password),
            FieldKind::None => Some(BarHint::NoField),
            _ if take.is_some_and(|t| t.long) => Some(BarHint::TooLong),
            _ if take.is_some_and(|t| t.fell) => Some(BarHint::Fell),
            _ => None,
        }
    }

    /// Undo insert: erases exactly what the keyboard typed.
    pub(super) fn undo(&mut self, out: &mut Vec<Out>) {
        if !self.undo_ok() {
            return;
        }
        let round = self.asks.wrapping_add(1);
        let box_id = self.field.box_id;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        let Some(typed) = take.typed.take() else {
            return;
        };
        self.asks = round;
        take.asking = Some(Asking {
            round,
            box_id,
            undo: true,
        });
        out.push(Out::Type(TextAsk {
            round,
            erase: typed.count,
            text: Redacted::new(Zeroizing::new(String::new())),
            gap_ms: self.settings.type_gap_ms,
        }));
    }

    /// The keyboard's answer: a text typed in full is what Undo erases, while the box kept focus. One it could not
    /// type dims Undo and stays on the bar with Copy (edge 8), saying typing stopped when part of it went in.
    pub(super) fn typed(&mut self, done: TextDone) {
        let box_id = self.field.box_id;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        let Some(asking) = take.asking.filter(|a| a.round == done.round) else {
            return;
        };
        take.asking = None;
        if asking.undo {
            return;
        }
        if done.ok && done.count > 0 && asking.box_id == box_id {
            take.typed = Some(Typed { count: done.count });
        }
        let shown = matches!(
            self.state,
            State::Text {
                text: VoiceText::Shown { .. },
                ..
            }
        );
        if let (false, true, State::Text { hint, .. }) = (done.ok, shown, &mut self.state) {
            let part = done.count > 0;
            *hint = Some(if part {
                BarHint::Stopped
            } else {
                BarHint::NoField
            });
        }
    }

    /// The clipboard's answer to Copy: a refusal shows its line for a full `done_ms`; a copy that went through brings
    /// the line before back.
    pub(super) fn copied(&mut self, ok: bool) {
        let now = self.now();
        if let State::Text { unclipped, .. } = &mut self.state {
            *unclipped = !ok;
            if !ok {
                self.shown_at = now;
            }
        }
    }

    /// Another box took focus, so Undo can no longer act.
    pub(super) fn field(&mut self, change: FieldChange) {
        if change.box_id != self.field.box_id
            && let Some(take) = self.take.as_mut()
        {
            take.typed = None;
        }
        self.field = change;
    }

    /// A key or a click elsewhere: Undo can no longer act, unless it is the keyboard typing our own text.
    pub(super) fn elsewhere(&mut self) {
        if let Some(take) = self.take.as_mut().filter(|t| t.asking.is_none()) {
            take.typed = None;
        }
    }
}

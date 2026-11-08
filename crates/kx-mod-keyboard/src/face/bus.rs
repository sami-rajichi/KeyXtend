//! The face on the kernel's bus (spec §4.4, §5.1, §5.2, §6, §7, §8.1, §8.5): the side keys ask the mouse module to flip
//! a mode and the power module for its dialog, the mouse, scroll, power, clipboard and vault modules' states come back,
//! the latched set goes out for modifier + click, the typing mode for word prediction, the wish for presses for the
//! panels' aim, picked words come in, which a press outside our windows stops, and so do paste, edit and fill
//! requests. Handlers run on the sender's thread, so nothing is sent while the face's lock is held.

use super::Face;
use super::outbox::Outbox;
use crate::panel::Panel;
use kx_module_api::ports::{
    AssistAsk, AssistChanged, AssistToggle, ClipLine, ClipPanel, EditAsk, FillAsk, FillLine,
    FillLive, FillPanel, FillType, HistoryAsk, HistoryWanted, LatchesAsk, LatchesChanged,
    LatchesUsed, PanelPressesWanted, PasteAsk, PointerPressed, PowerAsk, PowerAsked, PowerChanged,
    PressesAsk, Scan, ScrollAsk, ScrollChanged, Toggle, TypeWord, TypingMode, TypingModeAsk,
};
use kx_module_api::{Event, EventError, ModuleCx};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex, PoisonError};

/// The latched set, typing mode and wish for presses the bus last heard, so it hears each change once and the newest
/// last, from any thread.
#[derive(Default)]
pub(super) struct Told {
    /// Requests to check them, one sending thread at a time.
    outbox: Outbox,
    /// `LatchesAsk` came, or the module started, so the set goes out even if unchanged.
    again: AtomicBool,
    /// The module started or `TypingModeAsk` came, so the mode goes out even if unchanged.
    fresh: AtomicBool,
    /// `PressesAsk` came, so the wish goes out even if unchanged.
    asked: AtomicBool,
    /// What was last sent, which only the sending thread touches.
    last: Mutex<Last>,
}

/// The latched set, typing mode and wish for presses last sent.
#[derive(Default)]
struct Last {
    held: Vec<Scan>,
    mode: TypingMode,
    pressing: bool,
}

/// What goes out even if unchanged: the latched set, the typing mode and the wish for presses.
type Flags = (bool, bool, bool);

impl Face {
    /// Sends the latched set, the typing mode and the wish for presses when they changed since last sent, the set
    /// always when `again`, through the outbox, so the bus hears the newest last and a handler may ask again.
    pub(super) fn tell(&self, again: bool) {
        let t = &self.told;
        if again {
            t.again.store(true, SeqCst);
        }
        let flags = || {
            (
                t.again.swap(false, SeqCst),
                t.fresh.swap(false, SeqCst),
                t.asked.swap(false, SeqCst),
            )
        };
        t.outbox.post(|| self.send(flags()));
    }

    /// At start, the latched set and the typing mode go out even when unchanged.
    fn tell_all(&self) {
        self.told.fresh.store(true, SeqCst);
        self.tell(true);
    }

    /// A prediction module asked: the typing mode and the line in effect go out even when unchanged, the latched set
    /// only if it changed.
    fn tell_mode(&self) {
        self.told.fresh.store(true, SeqCst);
        self.lock().line_again();
        self.pour();
        self.tell(false);
    }

    /// A mouse module asked: the wish for presses goes out even when unchanged, the rest only if changed.
    fn tell_presses(&self) {
        self.told.asked.store(true, SeqCst);
        self.tell(false);
    }

    /// Sends the latched set when it differs from the last sent or `again`, the mode when it differs or `fresh`, then
    /// the wish for presses when it differs or `asked`; only the sending thread calls it.
    fn send(&self, (again, fresh, asked): Flags) {
        let (held, mode, on, bus) = {
            let inner = self.lock();
            (inner.held(), inner.mode(), inner.pressing(), inner.bus())
        };
        let mut last = self
            .told
            .last
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let latches = again || last.held != held;
        let moved = fresh || last.mode != mode;
        let wish = asked || last.pressing != on;
        last.held.clone_from(&held);
        last.mode = mode;
        last.pressing = on;
        drop(last);
        if latches {
            // The module holds `InjectInput`, which this event needs, or its start would have failed.
            let _ = bus.publish(&LatchesChanged { held });
        }
        if moved {
            // It holds `ReadFocusedText` too, which the mode needs.
            let _ = bus.publish(&mode);
        }
        if wish {
            // This event needs no capability, so none is refused.
            let _ = bus.publish(&PanelPressesWanted { on });
        }
    }

    /// A click on a mouse-assist side key: asks the mouse module to flip `which` while it runs, else does nothing.
    pub(super) fn toggle(&self, which: Toggle) {
        let bus = {
            let inner = self.lock();
            inner.live().then(|| inner.bus())
        };
        if let Some(bus) = bus {
            // As in `tell`, the module holds the `InjectInput` this event needs.
            let _ = bus.publish(&AssistToggle { which });
        }
    }

    /// A click on the Power key: asks the power module for the system's dialog while it runs, else does nothing.
    pub(super) fn power(&self) {
        let bus = {
            let inner = self.lock();
            inner.powers().then(|| inner.bus())
        };
        if let Some(bus) = bus {
            // The power events need no capability, so none is refused.
            let _ = bus.publish(&PowerAsked);
        }
    }

    /// A mouse click used the latches: they let go as their mode says, and the bus hears it.
    fn used(&self) {
        self.lock().clicked();
        self.tell(false);
    }

    /// Forgets the mouse, scroll, power, clipboard and vault modules' states, as each may have changed unheard while the
    /// keyboard was down: their keys and the pad dim, both panels count as shut and the history chord as unwanted until
    /// told again; the next live peek also tells a pointer that left.
    fn forget(&self) {
        {
            let mut inner = self.lock();
            inner.heard(AssistChanged::OFF);
            inner.heard_scroll(ScrollChanged::OFF);
            inner.heard_power(PowerChanged::OFF);
            inner.panel_shut(Panel::Clip);
            inner.heard_fill(FillLive::OFF);
            inner.heard_history(false);
        }
        self.pour();
        self.peek_again();
    }
}

/// Hears the mouse, scroll, power, clipboard and vault modules' states, the latch events, the mode and presses asks,
/// picked words, pointer presses, panel lines, paste and edit requests and fills for `face` until the module stops,
/// then tells the latched set and typing mode, forgets the five modules' states and asks for them, as each may have
/// missed the others while it was down.
///
/// # Errors
/// `EventError::NotGranted` when the module lacks `InjectInput`, which the latch events need, `ReadFocusedText`, which
/// picked words and the mode ask need, `ReadClipboard`, which the panel's state needs, `WriteClipboard`, which paste
/// requests need, `Vault`, which Quick-fill's state and fills need, or `Selection`, which edit requests need.
pub(crate) fn listen(cx: &mut ModuleCx<'_>, face: &Arc<Face>) -> Result<(), EventError> {
    on(cx, face, |f, e: &AssistChanged| f.lock().heard(*e))?;
    on(cx, face, |f, _: &LatchesUsed| f.used())?;
    on(cx, face, |f, _: &LatchesAsk| f.tell(true))?;
    on(cx, face, |f, e: &ScrollChanged| f.scrolled(*e))?;
    on(cx, face, |f, e: &PowerChanged| f.lock().heard_power(*e))?;
    on(cx, face, |f, e: &TypeWord| f.type_word(e))?;
    on(cx, face, |f, e: &PointerPressed| f.pressed(*e))?;
    on(cx, face, |f, _: &TypingModeAsk| f.tell_mode())?;
    on(cx, face, |f, _: &PressesAsk| f.tell_presses())?;
    on(cx, face, |f, e: &ClipPanel| {
        f.heard_panel(Panel::Clip, e.open);
    })?;
    on(cx, face, |f, e: &ClipLine| {
        f.heard_line(Panel::Clip, &e.0);
    })?;
    on(cx, face, |f, e: &HistoryWanted| {
        f.lock().heard_history(e.on);
    })?;
    on(cx, face, |f, e: &PasteAsk| f.paste(e.round))?;
    on(cx, face, |f, e: &EditAsk| f.edit(*e))?;
    on(cx, face, |f, e: &FillLive| {
        f.lock().heard_fill(*e);
        f.pour();
        f.tell(false);
    })?;
    on(cx, face, |f, e: &FillPanel| {
        f.heard_panel(Panel::Fill, e.open);
    })?;
    on(cx, face, |f, e: &FillLine| {
        f.heard_line(Panel::Fill, &e.0);
    })?;
    on(cx, face, |f, e: &FillType| f.fill(e))?;
    face.tell_all();
    face.forget();
    cx.bus().publish(&AssistAsk)?;
    cx.bus().publish(&ScrollAsk)?;
    cx.bus().publish(&PowerAsk)?;
    cx.bus().publish(&HistoryAsk)?;
    cx.bus().publish(&FillAsk)
}

/// Calls `then` with the face for each `E` on the bus, while the face lives.
fn on<E: Event>(
    cx: &mut ModuleCx<'_>,
    face: &Arc<Face>,
    then: impl Fn(&Face, &E) + Send + Sync + 'static,
) -> Result<(), EventError> {
    let face = Arc::downgrade(face);
    cx.subscribe(move |e: &E| {
        if let Some(face) = face.upgrade() {
            then(&face, e);
        }
    })
}

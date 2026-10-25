//! Running a tap: the key's plan from the board or the shortcuts layer, the layout shown, Caps Lock and the app in
//! front, sent through the injector or the shortcuts; the latches and a dead key's wait follow what went out, and
//! what reached the app waits to be told.

mod checks;
mod held;
mod layer;
mod ports;
mod told;
mod word;

pub use ports::{Io, Ports};

use crate::latch::Latches;
use crate::plan::{Now, Plan, Route, Wait, plan};
use crate::settings::KeyboardSettings;
use crate::spec::{BoardSpec, LatchRole, Role, Tell};
use checks::{keyed, reached, stale, types};
use kx_module_api::ports::{InjectError, Piece, Scan, ShortcutError, Toggle};
use std::collections::{HashMap, HashSet};
use std::fmt;
use thiserror::Error;

/// Why a tap sent nothing, or not all of it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum TapError {
    /// The board has no such key, and the layer draws none now.
    #[error("the board has no such key")]
    UnknownKey,
    /// The keys or text did not all reach the app.
    #[error(transparent)]
    Inject(#[from] InjectError),
    /// The shortcut did not all reach the app.
    #[error(transparent)]
    Shortcut(#[from] ShortcutError),
}

/// What a tap did, for the face to follow up.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tapped {
    /// Keys or text went out, Caps Lock was tapped or a latch changed: the state needs a redraw.
    Done,
    /// The language key was clicked; the face steps or opens the list.
    Lang {
        /// The click landed on the key's back part.
        back: bool,
    },
    /// A mouse-assist side key: nothing was sent and the latches stay; the face asks the mouse module to flip this.
    Assist(Toggle),
    /// The Shortcuts key: nothing was sent and the latches stay; the face shows or hides the layer.
    Layer,
    /// The Power key: nothing was sent and the latches stay; the face asks the power module for the dialog.
    Power,
    /// The layer's Next key: nothing was sent and the latches stay; the face turns the page.
    Next,
    /// A key from a later version: nothing happened.
    Later,
    /// A character key that types nothing with the latched Shift or `AltGr`: nothing happened.
    Empty,
    /// The app in front has another layout than the one shown, and this key's result depends on it: nothing
    /// was sent. The face refreshes the layouts, which read the same layout source, and redraws.
    Stale,
}

/// The typing state: the latched modifiers and a dead key's wait, over the board's keys and `key_apps`.
pub struct Typing {
    keys: HashMap<String, (Role, Option<Scan>)>,
    /// The keys the board lets repeat while held.
    repeating: HashSet<String>,
    /// What the keys that tell more than a break tell, by id.
    tells: HashMap<String, Tell>,
    /// The key a picked word erases with.
    erase: Option<Scan>,
    apps: Vec<String>,
    /// The most characters a picked word may erase or type.
    cap: u16,
    latches: Latches,
    dead: bool,
    /// While a held key repeats: whether a key went out, so the latches let go when the burst ends.
    burst: Option<bool>,
    /// The pieces that reached the app and wait to be told, oldest first.
    pieces: Vec<Piece>,
    /// Our Caps Lock tap reached the app since the face last asked.
    caps_sent: bool,
}

/// Prints no field: the latches and a dead key's wait tell what the user is typing.
impl fmt::Debug for Typing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Typing").finish_non_exhaustive()
    }
}

impl Typing {
    /// Nothing latched and no dead key waiting, on the keys of `spec` and the `modifiers` and `key_apps` of `settings`.
    #[must_use]
    pub fn new(spec: &BoardSpec, settings: &KeyboardSettings) -> Self {
        let keys = spec.roles().into_iter();
        Self {
            keys: keys.map(|(id, role, sc)| (id, (role, sc))).collect(),
            repeating: spec.repeating(),
            tells: spec.tells(),
            erase: spec.erase_key(),
            apps: settings.key_apps.clone(),
            cap: settings.text_cap,
            latches: Latches::new(settings.modifiers),
            dead: false,
            burst: None,
            pieces: Vec::new(),
            caps_sent: false,
        }
    }

    /// The latched modifiers, which light their keys.
    #[must_use]
    pub fn latches(&self) -> &Latches {
        &self.latches
    }

    /// A mouse click used the latches (modifier + click): they let go as their mode says. True when any let go.
    pub fn clicked(&mut self) -> bool {
        self.latches.used()
    }

    /// True once after our Caps Lock tap reached the app, so the face flips its Caps view before the system's next read.
    pub fn took_caps(&mut self) -> bool {
        std::mem::take(&mut self.caps_sent)
    }

    /// Takes a new `modifiers` mode, `key_apps` list and `text_cap`, keeping the latched modifiers and a dead key's wait.
    pub fn update(&mut self, settings: &KeyboardSettings) {
        self.apps.clone_from(&settings.key_apps);
        self.cap = settings.text_cap;
        self.latches.set_mode(settings.modifiers);
    }

    /// True while a dead key waits for the next key.
    #[must_use]
    pub fn waiting(&self) -> bool {
        self.dead
    }

    /// A click on key `id`, a board key or a layer key drawn now, on the language key's back part when `back`: sends
    /// its plan through `io`. A character key sent as a scan code sends nothing while the app in front has another
    /// layout (`Stale`).
    ///
    /// # Errors
    /// `UnknownKey` for an id on neither, `Inject` when the keys or text did not all reach the app, or `Shortcut` when
    /// the shortcut did not. A send refused whole changes nothing; one taken in part moves the state as if it all went,
    /// never retried.
    pub fn tap(&mut self, io: &Io<'_>, id: &str, back: bool) -> Result<Tapped, TapError> {
        let planned = match self.keys.get(id) {
            Some(&(role, sc)) => {
                let app = || io.injector.target_app();
                let planned = plan(role, sc, &self.now(io, &self.apps, &app));
                if keyed(role, &planned) && stale(io) {
                    return Ok(Tapped::Stale);
                }
                planned
            }
            None => layer::planned(io, id).ok_or(TapError::UnknownKey)?,
        };
        match planned {
            Plan::Send { route, wait } => {
                let piece = told::piece(&route, wait, self.tells.get(id));
                self.send(io, &route, wait, piece)?;
            }
            Plan::Caps(key) => self.caps(io, key)?,
            Plan::Latch(r, key) => self.latch(io, r, key)?,
            Plan::Shortcut(id) => self.shortcut(io, &id)?,
            Plan::Lang => return Ok(Tapped::Lang { back }),
            Plan::Assist(which) => return Ok(Tapped::Assist(which)),
            Plan::Layer => return Ok(Tapped::Layer),
            Plan::Power => return Ok(Tapped::Power),
            Plan::Next => return Ok(Tapped::Next),
            Plan::Later => return Ok(Tapped::Later),
            Plan::Empty => return Ok(Tapped::Empty),
        }
        Ok(Tapped::Done)
    }

    /// What a plan reads now: the layout shown, the latches, Caps Lock and a dead key's wait, with `apps` and the
    /// app in front from `app`.
    fn now<'a>(
        &'a self,
        io: &'a Io<'_>,
        apps: &'a [String],
        app: &'a dyn Fn() -> Option<String>,
    ) -> Now<'a> {
        Now {
            keys: &io.table.keys,
            latches: &self.latches,
            caps: io.source.caps_on(),
            dead: self.dead,
            apps,
            app,
        }
    }

    /// Sends `route`; once any of it reached the app, the latches are used, the dead-key wait follows `wait` and
    /// `piece` waits to be told.
    fn send(
        &mut self,
        io: &Io<'_>,
        route: &Route,
        wait: Wait,
        piece: Piece,
    ) -> Result<(), TapError> {
        let went = match route {
            Route::Tap(key) => io.injector.tap(*key),
            Route::Chord(held, key) => io.injector.chord(held, *key),
            Route::Text(text, by) => io.injector.text(text, *by),
        };
        if reached(went) {
            self.dead = match wait {
                Wait::Start => true,
                Wait::End => false,
                Wait::Ask => !types(io, route),
            };
            self.used();
            self.pieces.push(piece);
        }
        Ok(went?)
    }

    /// Taps Caps Lock `key`; once it reached the app, the latches stay on, used, and the face hears it was sent.
    fn caps(&mut self, io: &Io<'_>, key: Scan) -> Result<(), TapError> {
        let went = io.injector.tap(key);
        if reached(went) {
            self.latches.spend();
            self.caps_sent = true;
        }
        Ok(went?)
    }

    /// A click on the latch key sending `key`: latches `r` or lets it go. A lone latch let go unused first sends
    /// `key` alone, which uses the other latches as any key does; refused whole, it stays latched.
    fn latch(&mut self, io: &Io<'_>, r: LatchRole, key: Scan) -> Result<(), TapError> {
        let lone = r.lone && self.latches.unused(r.latch);
        let went = if lone { io.injector.tap(key) } else { Ok(()) };
        if reached(went) {
            self.latches.toggle(r.latch, key);
            if lone {
                self.used();
            }
        }
        Ok(went?)
    }

    /// A key went out: the latches let go as their mode says, or when the burst ends while a held key repeats.
    fn used(&mut self) {
        match &mut self.burst {
            Some(went) => {
                *went = true;
                self.latches.spend();
            }
            None => {
                self.latches.used();
            }
        }
    }
}

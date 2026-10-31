//! Running a tap: the key's plan from the board or the shortcuts layer, the layout shown, Caps Lock and the app in
//! front, sent through the injector or the shortcuts; the latches and a dead key's wait follow what went out, and
//! what reached the app waits to be told.

mod checks;
mod clip;
mod held;
mod layer;
mod ports;
mod tapped;
mod told;
mod word;

pub use clip::ToPanel;
pub use ports::{Io, Pause, Ports, Sleep};
pub use tapped::{TapError, Tapped};
pub use word::{Part, Word, drive};

use crate::latch::Latches;
use crate::plan::{Now, Plan, Route, Wait, plan};
use crate::settings::KeyboardSettings;
use crate::spec::{BoardSpec, LatchRole, Role, Tell};
use checks::{keyed, reached, stale, types};
use kx_module_api::ports::{Piece, Scan};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Duration;

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
    /// The pause after each Backspace of a picked word.
    gap: Duration,
    latches: Latches,
    dead: bool,
    /// While a held key repeats: whether a key went out, so the latches let go when the burst ends.
    burst: Option<bool>,
    /// The pieces that reached the app and wait to be told, oldest first.
    pieces: Vec<Piece>,
    /// Our Caps Lock tap reached the app since the face last asked.
    caps_sent: bool,
    /// How many board keys reached the app, so a picked word stops when one goes out during it.
    sent: u64,
    /// The clipboard module wants the platform's history chord, as the bus last told.
    history: bool,
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
            gap: gap(settings),
            latches: Latches::new(settings.modifiers),
            dead: false,
            burst: None,
            pieces: Vec::new(),
            caps_sent: false,
            sent: 0,
            history: false,
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

    /// Takes a new `modifiers` mode, `key_apps` list, `text_cap` and `erase_gap_ms`, keeping the latched modifiers and
    /// a dead key's wait.
    pub fn update(&mut self, settings: &KeyboardSettings) {
        self.apps.clone_from(&settings.key_apps);
        self.cap = settings.text_cap;
        self.gap = gap(settings);
        self.latches.set_mode(settings.modifiers);
    }

    /// The most characters a picked word may erase or type (`text_cap`).
    #[must_use]
    pub fn cap(&self) -> u16 {
        self.cap
    }

    /// True while a dead key waits for the next key. It outlasts a focus change: the keyboard hears none, and the OS
    /// may still hold the accent, so a picked word never types over it.
    #[must_use]
    pub fn waiting(&self) -> bool {
        self.dead
    }

    /// A click on key `id`, a board key or a layer key drawn now, on the language key's back part when `back`: sends
    /// its plan through `io`. A character key sent as a scan code sends nothing while the app in front has another
    /// layout (`Stale`), and the history chord sends nothing while the clipboard module wants it (`Clip`).
    ///
    /// # Errors
    /// `UnknownKey` for an id on neither, `Inject` when the keys or text did not all reach the app, or `Shortcut` when
    /// the shortcut did not. A send refused whole changes nothing; one taken in part moves the state as if it all went,
    /// never retried.
    pub fn tap(&mut self, io: &Io<'_>, id: &str, back: bool) -> Result<Tapped, TapError> {
        if self.chord(io, id) {
            return Ok(Tapped::Clip);
        }
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
            Plan::Clip => return Ok(Tapped::Clip),
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
            self.sent = self.sent.wrapping_add(1);
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

    /// A key went out: the latches let go as their mode says, or when the burst ends while a held key repeats, and a
    /// picked word going out stops.
    fn used(&mut self) {
        self.sent = self.sent.wrapping_add(1);
        self.spent();
    }

    /// A key or picked word that started under `then` went out: those latches let go as their mode says, or when the
    /// burst ends while a held key repeats; one latched since stays.
    fn used_of(&mut self, then: &Latches) {
        match &mut self.burst {
            Some(went) => {
                *went = true;
                self.latches.spend_of(then);
            }
            None => {
                self.latches.used_of(then);
            }
        }
    }
}

/// The pause after each Backspace of a picked word, from `erase_gap_ms`.
fn gap(settings: &KeyboardSettings) -> Duration {
    Duration::from_millis(settings.erase_gap_ms)
}

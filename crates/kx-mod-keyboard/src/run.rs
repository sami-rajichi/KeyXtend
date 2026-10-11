//! Running a tap: the key's plan from the board, the layout shown, Caps Lock and the app in front,
//! sent through the injector; the latches and a dead key's wait follow what went out.

mod held;

use crate::latch::Latches;
use crate::plan::{Now, Plan, Route, Wait, plan};
use crate::settings::KeyboardSettings;
use crate::spec::{BoardSpec, Role};
use kx_module_api::ports::{
    InjectError, InputInjector, InputInjectorKey, LayoutSource, LayoutSourceKey, LayoutTable,
    Layouts, LayoutsKey, Scan,
};
use kx_module_api::{Clock, ModuleCx, ServiceError};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

/// Why a tap sent nothing, or not all of it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum TapError {
    /// The board has no such key.
    #[error("the board has no such key")]
    UnknownKey,
    /// The keys or text did not all reach the app.
    #[error(transparent)]
    Inject(#[from] InjectError),
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
    /// A key from a later version: nothing happened.
    Later,
    /// A character key that types nothing with the latched Shift or `AltGr`: nothing happened.
    Empty,
    /// The app in front has another layout than the one shown, and this key's result depends on it: nothing
    /// was sent. The face refreshes the layouts, which read the same layout source, and redraws.
    Stale,
}

/// The ports one tap goes through, and the layout table the keyboard shows.
#[derive(Clone, Copy)]
pub struct Io<'a> {
    /// Sends keys and text to the app in front, and names that app.
    pub injector: &'a dyn InputInjector,
    /// Says whether Caps Lock is on and whether a key types anything.
    pub source: &'a dyn LayoutSource,
    /// What each key types in the layout shown.
    pub table: &'a LayoutTable,
}

/// The services typing needs, as the kernel hands them out, and the kernel's clock.
#[derive(Clone)]
pub struct Ports {
    /// The input injector.
    pub injector: Arc<dyn InputInjector>,
    /// The layout source.
    pub source: Arc<dyn LayoutSource>,
    /// The layouts service, whose current table the keyboard shows.
    pub layouts: Arc<dyn Layouts>,
    /// The kernel's clock, which times a held key's repeats.
    pub clock: Arc<dyn Clock>,
}

impl Ports {
    /// The three services and the clock from the kernel.
    ///
    /// # Errors
    /// The `ServiceError` of the first service the module cannot have.
    pub fn from_cx(cx: &ModuleCx<'_>) -> Result<Self, ServiceError> {
        Ok(Self {
            injector: cx.service::<InputInjectorKey>()?,
            source: cx.service::<LayoutSourceKey>()?,
            layouts: cx.service::<LayoutsKey>()?,
            clock: cx.clock(),
        })
    }

    /// The ports of one tap, with `table`, the current table of `layouts`.
    #[must_use]
    pub fn io<'a>(&'a self, table: &'a LayoutTable) -> Io<'a> {
        Io {
            injector: &*self.injector,
            source: &*self.source,
            table,
        }
    }
}

/// The typing state: the latched modifiers and a dead key's wait, over the board's keys and `key_apps`.
pub struct Typing {
    keys: HashMap<String, (Role, Option<Scan>)>,
    /// The keys the board lets repeat while held.
    repeating: HashSet<String>,
    apps: Vec<String>,
    latches: Latches,
    dead: bool,
    /// While a held key repeats: whether a key went out, so the latches let go when the burst ends.
    burst: Option<bool>,
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
            apps: settings.key_apps.clone(),
            latches: Latches::new(settings.modifiers),
            dead: false,
            burst: None,
        }
    }

    /// The latched modifiers, which light their keys.
    #[must_use]
    pub fn latches(&self) -> &Latches {
        &self.latches
    }

    /// Takes a new `modifiers` mode and `key_apps` list, keeping the latched modifiers and a dead key's wait.
    pub fn update(&mut self, settings: &KeyboardSettings) {
        self.apps.clone_from(&settings.key_apps);
        self.latches.set_mode(settings.modifiers);
    }

    /// True while a dead key waits for the next key.
    #[must_use]
    pub fn waiting(&self) -> bool {
        self.dead
    }

    /// A click on key `id`, on the language key's back part when `back`: sends its plan through `io`.
    /// A character key sent as a scan code sends nothing while the app in front has another layout (`Stale`).
    ///
    /// # Errors
    /// `UnknownKey` for an id not on the board, or `Inject` when the keys or text did not all reach the app.
    /// A send refused whole changes nothing; one taken in part moves the state as if it all went, never retried.
    pub fn tap(&mut self, io: &Io<'_>, id: &str, back: bool) -> Result<Tapped, TapError> {
        let &(role, sc) = self.keys.get(id).ok_or(TapError::UnknownKey)?;
        let now = Now {
            keys: &io.table.keys,
            latches: &self.latches,
            caps: io.source.caps_on(),
            dead: self.dead,
            apps: &self.apps,
            app: &|| io.injector.target_app(),
        };
        let planned = plan(role, sc, &now);
        if keyed(role, &planned) && stale(io) {
            return Ok(Tapped::Stale);
        }
        match planned {
            Plan::Send { route, wait } => self.send(io, &route, wait)?,
            Plan::Caps(key) => io.injector.tap(key)?,
            Plan::Latch(l, key) => self.latches.toggle(l, key),
            Plan::Lang => return Ok(Tapped::Lang { back }),
            Plan::Later => return Ok(Tapped::Later),
            Plan::Empty => return Ok(Tapped::Empty),
        }
        Ok(Tapped::Done)
    }

    /// Sends `route`; once any of it reached the app, the latches are used and the dead-key wait follows `wait`.
    fn send(&mut self, io: &Io<'_>, route: &Route, wait: Wait) -> Result<(), TapError> {
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
        }
        Ok(went?)
    }

    /// A key went out: the latches let go as their mode says, or when the burst ends while a held key repeats.
    fn used(&mut self) {
        match &mut self.burst {
            Some(went) => *went = true,
            None => self.latches.used(),
        }
    }
}

/// True when any of a send reached the app: all of it, or the part the system took before refusing the rest.
fn reached(went: Result<(), InjectError>) -> bool {
    match went {
        Ok(()) => true,
        Err(InjectError::Refused { sent, .. }) => sent > 0,
        Err(InjectError::Unmappable) => false,
    }
}

/// True when `plan` sends a character key as a scan code, so the app's own layout decides what it types.
fn keyed(role: Role, plan: &Plan) -> bool {
    let scan = |p: &Plan| {
        matches!(
            p,
            Plan::Send {
                route: Route::Tap(_) | Route::Chord(..),
                ..
            }
        )
    };
    role == Role::Char && scan(plan)
}

/// True when the app in front has another installed layout than the table shown, as `Follow` would show
/// after a refresh; with none in front, or one that is not installed, the table shown stands.
fn stale(io: &Io<'_>) -> bool {
    let shown = io.table.label.id;
    let other = io.source.foreground().filter(|&front| front != shown);
    other.is_some_and(|front| io.source.installed().contains(&front))
}

/// True when `route` typed anything in the layout shown, as the layout source reads it.
fn types(io: &Io<'_>, route: &Route) -> bool {
    let layout = io.table.label.id;
    match route {
        Route::Tap(key) => io.source.types_any(layout, *key, &[]),
        Route::Chord(held, key) => io.source.types_any(layout, *key, held),
        Route::Text(..) => true,
    }
}

//! Running a tap: the key's plan from the board, the layout shown, Caps Lock and the app in front,
//! sent through the injector; the latches and a dead key's wait follow what went out.

use crate::latch::Latches;
use crate::plan::{Now, Plan, Route, Wait, plan, text_route};
use crate::settings::KeyboardSettings;
use crate::spec::{BoardSpec, Role};
use kx_module_api::ports::{
    FaceError, InputInjector, InputInjectorKey, LayoutSource, LayoutSourceKey, LayoutTable,
    Layouts, LayoutsKey, Scan, TextRoute,
};
use kx_module_api::{ModuleCx, ServiceError};
use std::collections::HashMap;
use std::sync::Arc;

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

/// The services typing needs, as the kernel hands them out.
#[derive(Clone)]
pub struct Ports {
    /// The input injector.
    pub injector: Arc<dyn InputInjector>,
    /// The layout source.
    pub source: Arc<dyn LayoutSource>,
    /// The layouts service, whose current table the keyboard shows.
    pub layouts: Arc<dyn Layouts>,
}

impl Ports {
    /// The three services from the kernel.
    ///
    /// # Errors
    /// The `ServiceError` of the first service the module cannot have.
    pub fn from_cx(cx: &ModuleCx<'_>) -> Result<Self, ServiceError> {
        Ok(Self {
            injector: cx.service::<InputInjectorKey>()?,
            source: cx.service::<LayoutSourceKey>()?,
            layouts: cx.service::<LayoutsKey>()?,
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
#[derive(Debug)]
pub struct Typing {
    keys: HashMap<String, (Role, Option<Scan>)>,
    apps: Vec<String>,
    latches: Latches,
    dead: bool,
}

impl Typing {
    /// Nothing latched and no dead key waiting, on the keys of `spec` and the `modifiers` and `key_apps` of `settings`.
    #[must_use]
    pub fn new(spec: &BoardSpec, settings: &KeyboardSettings) -> Self {
        let keys = spec.roles().into_iter();
        Self {
            keys: keys.map(|(id, role, sc)| (id, (role, sc))).collect(),
            apps: settings.key_apps.clone(),
            latches: Latches::new(settings.modifiers),
            dead: false,
        }
    }

    /// The latched modifiers, which light their keys.
    #[must_use]
    pub fn latches(&self) -> &Latches {
        &self.latches
    }

    /// True while a dead key waits for the next key.
    #[must_use]
    pub fn waiting(&self) -> bool {
        self.dead
    }

    /// A click on key `id`, on the language key's back part when `back`: sends its plan through `io`.
    ///
    /// # Errors
    /// `UnknownKey` for an id not on the board, or `Inject` when the keys or text did not reach the app;
    /// then the latches and a dead key's wait stay as they were.
    pub fn tap(&mut self, io: &Io<'_>, id: &str, back: bool) -> Result<Tapped, FaceError> {
        let &(role, sc) = self.keys.get(id).ok_or(FaceError::UnknownKey)?;
        let now = Now {
            keys: &io.table.keys,
            latches: &self.latches,
            caps: io.source.caps_on(),
            dead: self.dead,
            text: self.text_route(io.injector),
        };
        match plan(role, sc, &now) {
            Plan::Send { route, wait } => self.send(io, &route, wait)?,
            Plan::Caps(key) => io.injector.tap(key)?,
            Plan::Latch(l, key) => self.latches.toggle(l, key),
            Plan::Lang => return Ok(Tapped::Lang { back }),
            Plan::Later => return Ok(Tapped::Later),
        }
        Ok(Tapped::Done)
    }

    /// How text reaches the app in front; the app is looked up only when `key_apps` names any.
    fn text_route(&self, injector: &dyn InputInjector) -> TextRoute {
        if self.apps.is_empty() {
            return TextRoute::Unicode;
        }
        text_route(&self.apps, injector.target_app().as_deref())
    }

    /// Sends `route`; only once it went out do the latches let go and the dead-key wait follow `wait`.
    fn send(&mut self, io: &Io<'_>, route: &Route, wait: Wait) -> Result<(), FaceError> {
        match route {
            Route::Tap(key) => io.injector.tap(*key)?,
            Route::Chord(held, key) => io.injector.chord(held, *key)?,
            Route::Text(text, by) => io.injector.text(text, *by)?,
        }
        self.dead = match wait {
            Wait::Start => true,
            Wait::End => false,
            Wait::Ask => !types(io, route),
        };
        self.latches.used();
        Ok(())
    }
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
